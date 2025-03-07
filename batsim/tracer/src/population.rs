use anyhow::{Context, Result};
use itertools::Itertools;
use quick_xml::{events::Event, Reader};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    ops::Deref,
    str::from_utf8,
};
use xml;

use super::{Component, Trace};

pub type PersonAttributes = HashMap<String, String>;

///Person struct to hold agent info
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub attributes: PersonAttributes,
    pub trace: Trace,
}
impl Person {
    pub fn viable_combinations(&self, activities: Vec<&String>) -> Vec<Vec<Vec<usize>>> {
        charge_combinations(self.viable_charge_activities(activities))
    }
    // return viable charge activities
    fn viable_charge_activities(&self, activities: Vec<&String>) -> Vec<usize> {
        self.trace
            .plan
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match c {
                Component::ActivityType(ref activity) => {
                    match activities.contains(&&activity.act) {
                        true => Some(i),
                        false => None,
                    }
                }
                _ => None,
            })
            .collect()
    }
}

/// Given a vec of integers, return a vector of combination sizes, where each size holds vectors of combinations of that size
// The order of viable activities is reverse such that the last activity comes first in each combination.
// In the case of indifference between charging activities, later activities should be preferred.
fn charge_combinations(viable: Vec<usize>) -> Vec<Vec<Vec<usize>>> {
    let mut combinations = Vec::<Vec<Vec<usize>>>::default();
    for k in 0..(viable.len() + 1) {
        combinations.push(
            viable
                .clone()
                .into_iter()
                .rev()
                .combinations(k)
                .collect_vec(),
        );
    }
    if combinations.len() > 1 {
        // in the case of charge activities do not bother checking empty case
        combinations.remove(0);
    }
    combinations
}

/// Population struct used to hold map of all agent attributes
/// "people" are held as a BTree to preserve order and allow reproducibility
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Population {
    pub people: BTreeMap<String, Person>,
}

impl Population {
    /// Return a population of attributes loaded from a MATSim plans file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to MATSim plans file
    ///
    pub fn from_xml(reader: &mut Reader<Box<dyn BufRead>>) -> Result<Population> {
        let mut people = BTreeMap::<String, Person>::new();
        let mut buf = Vec::new();
        let mut parser = AttributesParser::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,
                Ok(event) => parser.process(event, &mut people),
            }
            buf.clear();
        }
        Ok(Population { people })
    }
    pub fn is_empty(&self) -> bool {
        self.people.is_empty()
    }
    pub fn len(&self) -> usize {
        self.people.len()
    }
    pub fn serialise(&self, out_file: File, json: bool) -> Result<()> {
        let writer = BufWriter::new(out_file);
        if json {
            // human readable json
            serde_json::to_writer(writer, self)
                .context("failed to serialise json format traces")?;
        } else {
            bincode::serialize_into(writer, self)
                .context("failed to serialise binary format traces")?;
        }
        Ok(())
    }
    pub fn deserialise(reader: BufReader<File>, json: bool) -> Result<Self> {
        if json {
            serde_json::from_reader(reader)
                .context("unable to json deserialise traces (check files are json)")
        } else {
            bincode::deserialize_from(reader)
                .context("unable to deserialise binary traces (check files are binary)")
        }
    }
}

impl<'h> IntoIterator for &'h Population {
    type Item = <&'h BTreeMap<String, Person> as IntoIterator>::Item;
    type IntoIter = <&'h BTreeMap<String, Person> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.people.iter()
    }
}

/// MATSim xml attributes parser
pub struct AttributesParser {
    /// Starting state of state machine
    state: AttributesParserState,
}

#[derive(Clone, Debug, PartialEq)]
enum AttributesParserState {
    Population,
    Person { pid: String },
    Attributes { pid: String },
    Attribute { pid: String, name: String },
}

impl AttributesParser {
    /// Return an AttributeParser with AttributesParserState::Population starting state
    fn new() -> AttributesParser {
        AttributesParser {
            state: AttributesParserState::Population,
        }
    }

    /// Process an xml event, record required information and progress state
    /// Please note that there is no "plan" state. The parser uses the first attributes
    /// it finds after entering the "Person" state, it therefore expects that person
    /// attributes will come before the plan.
    /// In the case where there are no person attributes, the parser will record the
    /// first attributes it finds - which may be leg attributes. // todo
    ///
    /// # Arguments
    ///
    /// * `event` - quick_xml.events.Event
    /// * `people` - BTreeMap used to record person attributes
    ///
    fn process(&mut self, event: Event, people: &mut BTreeMap<String, Person>) {
        self.state = match &self.state {
            // Starting from population level, we use the recorder to record a new person when encountered
            AttributesParserState::Population => Self::process_population_state(event, people),

            // Starting from person state
            AttributesParserState::Person { pid } => Self::process_person_state(event, pid),

            // Starting from attributes state
            AttributesParserState::Attributes { pid } => Self::process_attributes_state(event, pid),

            // Starting from attribute state
            AttributesParserState::Attribute { pid, name: key } => {
                Self::process_attribute_state(event, pid, key, people)
            }
        }
    }

    fn process_population_state(
        event: Event,
        people: &mut BTreeMap<String, Person>,
    ) -> AttributesParserState {
        match event {
            // person event encountered, get the "id" attribute and move to person state
            Event::Start(event) if event.name().into_inner() == b"person" => {
                let pid = from_utf8(xml::get_attribute(b"id", &event).unwrap().deref())
                    .unwrap()
                    .to_string();
                people.insert(pid.to_string(), Person::default());
                AttributesParserState::Person { pid }
            }

            // anything else stay put
            _ => AttributesParserState::Population,
        }
    }

    fn process_person_state(event: Event, pid: &String) -> AttributesParserState {
        match event {
            // end of person, return to previous
            Event::End(event) if event.name().into_inner() == b"person" => {
                AttributesParserState::Population
            }

            // move to attributes, keep id
            Event::Start(event) if event.name().into_inner() == b"attributes" => {
                AttributesParserState::Attributes {
                    pid: pid.to_string(),
                }
            }

            // otherwise remain in place (for example for plans info)
            _ => AttributesParserState::Person {
                pid: pid.to_string(),
            },
        }
    }

    fn process_attributes_state(event: Event, pid: &String) -> AttributesParserState {
        match event {
            // end of attributes, return to first state (population)
            Event::End(event) if event.name().into_inner() == b"attributes" => {
                AttributesParserState::Population
            }

            // record attribute
            Event::Start(event) if event.name().into_inner() == b"attribute" => {
                let name: String = from_utf8(xml::get_attribute(b"name", &event).unwrap().deref())
                    .unwrap()
                    .to_string();
                AttributesParserState::Attribute {
                    pid: pid.to_string(),
                    name,
                }
            }

            _ => AttributesParserState::Attributes {
                pid: pid.to_string(),
            },
        }
    }

    fn process_attribute_state(
        event: Event,
        pid: &String,
        key: &String,
        people: &mut BTreeMap<String, Person>,
    ) -> AttributesParserState {
        match event {
            // If we see some text we grab it as the attribute value
            Event::Text(event) => {
                let value = event.unescape().unwrap().into_owned();
                people
                    .get_mut(pid)
                    .unwrap()
                    .attributes
                    .insert(key.to_string(), value);
                AttributesParserState::Attributes {
                    pid: pid.to_string(),
                }
            }
            // Else we return to attributes
            _ => AttributesParserState::Attributes {
                pid: pid.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Activity, Link};
    use super::*;
    use quick_xml::{
        events::{BytesEnd, BytesStart, BytesText},
        reader::Reader,
    };
    use tempfile::tempdir;

    #[test]
    fn test_parser_initial_state() {
        let parser = AttributesParser::new();
        assert_eq!(parser.state, AttributesParserState::Population)
    }

    #[test]
    fn test_parser_expected_transitions_from_population() {
        let mut people = BTreeMap::<String, Person>::new();

        // Test transition from population state given another population start event
        let mut parser = AttributesParser::new();
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("population")),
            &mut people,
        );
        assert_eq!(parser.state, AttributesParserState::Population);

        // Test transition from population state given person start event
        let mut parser = AttributesParser::new();
        let xml = r#"<person id = "x">"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let person_event = reader.read_event().unwrap();
        parser.process(person_event, &mut people);
        assert_eq!(
            parser.state,
            AttributesParserState::Person {
                pid: "x".to_string()
            }
        );
        assert_eq!(
            people.get(&"x".to_string()).unwrap().attributes,
            HashMap::<String, String>::new()
        );
    }

    #[test]
    fn test_parser_expected_transitions_from_person() {
        let mut people = BTreeMap::<String, Person>::new();

        // Test transition from person state given person end event
        let mut parser = AttributesParser {
            state: AttributesParserState::Person {
                pid: "x".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("person")),
            &mut people,
        );
        assert_eq!(parser.state, AttributesParserState::Population);

        // Test transition from person state given attributes start event
        let mut parser = AttributesParser {
            state: AttributesParserState::Person {
                pid: "x".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("attributes")),
            &mut people,
        );
        assert_eq!(
            parser.state,
            AttributesParserState::Attributes {
                pid: "x".to_string()
            }
        );

        // Test transition from population state given other event
        let mut parser = AttributesParser {
            state: AttributesParserState::Person {
                pid: "x".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("plan")),
            &mut people,
        );
        assert_eq!(
            parser.state,
            AttributesParserState::Person {
                pid: "x".to_string()
            }
        );
    }

    #[test]
    fn test_parser_expected_transitions_from_attributes() {
        let mut people = BTreeMap::<String, Person>::new();

        // Test transition from attributes state given attributes end event
        let mut parser = AttributesParser {
            state: AttributesParserState::Attributes {
                pid: "x".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("attributes")),
            &mut people,
        );
        assert_eq!(parser.state, AttributesParserState::Population);

        // Test transition from attributes state given attribute start event
        let mut parser = AttributesParser {
            state: AttributesParserState::Attributes {
                pid: "x".to_string(),
            },
        };
        let xml = r#"<attribute name = "y">"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let attribute_event = reader.read_event().unwrap();
        parser.process(attribute_event, &mut people);
        assert_eq!(
            parser.state,
            AttributesParserState::Attribute {
                pid: "x".to_string(),
                name: "y".to_string(),
            }
        );

        // Test transition from attributes state given other event
        let mut parser = AttributesParser {
            state: AttributesParserState::Attributes {
                pid: "x".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::Comment(BytesText::new("<!--Test comment-->")),
            &mut people,
        );
        assert_eq!(
            parser.state,
            AttributesParserState::Attributes {
                pid: "x".to_string()
            }
        );
    }

    #[test]
    fn test_parser_expected_transitions_from_attribute() {
        let mut people = BTreeMap::<String, Person>::new();

        // Test transition from attribute state given non text event
        let mut parser = AttributesParser {
            state: AttributesParserState::Attribute {
                pid: "x".to_string(),
                name: "y".to_string(),
            },
        };
        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("attribute")),
            &mut people,
        );
        assert_eq!(
            parser.state,
            AttributesParserState::Attributes {
                pid: "x".to_string()
            }
        );

        // Test transition from attribute state given text event
        people.insert("x".to_string(), Person::default());
        let mut parser = AttributesParser {
            state: AttributesParserState::Attribute {
                pid: "x".to_string(),
                name: "y".to_string(),
            },
        };
        let xml = r#"z"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let attribute_event = reader.read_event().unwrap();
        parser.process(attribute_event, &mut people);
        assert_eq!(
            parser.state,
            AttributesParserState::Attributes {
                pid: "x".to_string(),
            }
        );
        assert_eq!(
            people
                .get(&"x".to_string())
                .unwrap()
                .attributes
                .get(&"y".to_string())
                .unwrap(),
            &"z".to_string()
        )
    }

    #[test]
    fn valid_plan_combinations() {
        let person = Person {
            attributes: HashMap::default(),
            trace: Trace {
                plan: vec![
                    Component::ActivityType(Activity {
                        act: "home".to_string(),
                        ..Activity::default()
                    }),
                    Component::LinkType(Link { ..Link::default() }),
                    Component::ActivityType(Activity {
                        act: "work".to_string(),
                        ..Activity::default()
                    }),
                    Component::LinkType(Link { ..Link::default() }),
                    Component::ActivityType(Activity {
                        act: "home".to_string(),
                        ..Activity::default()
                    }),
                ],
            },
        };
        assert_eq!(
            person.viable_charge_activities(vec![&"none".to_string()]),
            Vec::<usize>::new()
        );
        assert_eq!(
            person.viable_combinations(vec![&"none".to_string()]),
            vec![vec![Vec::<usize>::new()]]
        );
        assert_eq!(
            person.viable_charge_activities(vec![&"home".to_string()]),
            vec![0, 4]
        );
        assert_eq!(
            person.viable_combinations(vec![&"home".to_string()]),
            vec![vec![vec![4], vec![0]], vec![vec![4, 0]]]
        );
        assert_eq!(
            person.viable_charge_activities(vec![&"home".to_string(), &"work".to_string()]),
            vec![0, 2, 4]
        );
        assert_eq!(
            person.viable_combinations(vec![&"home".to_string(), &"work".to_string()]),
            vec![
                vec![vec![4], vec![2], vec![0]],
                vec![vec![4, 2], vec![4, 0], vec![2, 0]],
                vec![vec![4, 2, 0]],
            ]
        );
    }
    #[test]
    fn valid_empty_plan_combinations() {
        let person = Person::default();
        assert_eq!(
            person.viable_charge_activities(vec![&"home".to_string()]),
            Vec::<usize>::default()
        );
        assert_eq!(
            person.viable_combinations(vec![&"home".to_string()]),
            vec![vec![Vec::<usize>::default()]]
        )
    }

    fn test_pop() -> Population {
        let people: BTreeMap<String, Person> = BTreeMap::from([(
            "1".to_string(),
            Person {
                attributes: HashMap::from([("age".to_string(), "high".to_string())]),
                trace: Trace {
                    plan: vec![
                        Component::LinkType(Link {
                            start_time: 1,
                            end_time: 2,
                            lid: "a".to_string(),
                            distance: 1.0,
                            node: (0.0, 0.0),
                        }),
                        Component::LinkType(Link {
                            start_time: 2,
                            end_time: 3,
                            lid: "b".to_string(),
                            distance: 0.5,
                            node: (0.0, 0.0),
                        }),
                        Component::ActivityType(Activity {
                            start_time: 3,
                            end_time: (24 * 60 * 60) + 1,
                            act: "home".to_string(),
                            node: (0.0, 0.0),
                        }),
                    ],
                },
            },
        )]);
        Population { people }
    }
    #[test]
    fn test_serialise_deserialise_consistency_binary() {
        let population = test_pop();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tmp.txt");
        let out_file = File::create(&file_path).unwrap();
        population.serialise(out_file, false).unwrap();

        let in_file = File::open(&file_path).unwrap();
        let reader = BufReader::new(in_file);
        let new_pop = Population::deserialise(reader, false).unwrap();

        assert_eq!(population, new_pop)
    }
    #[test]
    fn test_serialise_deserialise_consistency_json() {
        let population = test_pop();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tmp.txt");
        let out_file = File::create(&file_path).unwrap();
        population.serialise(out_file, true).unwrap();

        let in_file = File::open(&file_path).unwrap();
        let reader = BufReader::new(in_file);
        let new_pop = Population::deserialise(reader, true).unwrap();

        assert_eq!(population, new_pop)
    }
}
