use crate::peekset::PeekSet;
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::io::BufRead;
use std::ops::Deref;
use std::str::from_utf8;
use xml;

type PeekAttributes = BTreeMap<String, PeekSet<String>>;

pub fn peek_attributes(
    reader: &mut Reader<Box<dyn BufRead>>,
    max: usize,
) -> Result<PeekAttributes> {
    let mut attributes = PeekAttributes::new();
    let mut buf = Vec::new();
    let mut parser = PeekAttributesParser::new(max);

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,
            Ok(event) => parser.process(event, &mut attributes),
        }
        buf.clear();
    }
    Ok(attributes)
}

#[derive(Clone, Debug, PartialEq)]
enum PeekAttributesParserState {
    Person,
    Plan,
    Attributes,
    Attribute { key: String },
}

/// MATSim xml attributes parser
pub struct PeekAttributesParser {
    /// Starting state of state machine
    state: PeekAttributesParserState,
    max: usize,
}
impl PeekAttributesParser {
    /// Return an AttributeParser with AttributesParserState::Population starting state
    pub fn new(max: usize) -> PeekAttributesParser {
        PeekAttributesParser {
            state: PeekAttributesParserState::Person,
            max,
        }
    }

    fn process(&mut self, event: Event, attributes: &mut PeekAttributes) {
        self.state = match &self.state {
            PeekAttributesParserState::Person => self.process_from_person_state(event),
            PeekAttributesParserState::Plan => self.process_from_plan_state(event),
            PeekAttributesParserState::Attributes => self.process_from_attributes_state(event),
            PeekAttributesParserState::Attribute { key } => {
                self.process_from_attribute_state(event, key, attributes)
            }
        }
    }

    fn process_from_person_state(&self, event: Event) -> PeekAttributesParserState {
        match event {
            Event::Start(event) if event.name().into_inner() == b"attributes" => {
                PeekAttributesParserState::Attributes
            }
            Event::Start(event) if event.name().into_inner() == b"plan" => {
                PeekAttributesParserState::Plan
            }
            _ => PeekAttributesParserState::Person,
        }
    }

    fn process_from_plan_state(&self, event: Event) -> PeekAttributesParserState {
        match event {
            Event::End(event) if event.name().into_inner() == b"plan" => {
                PeekAttributesParserState::Person
            }
            _ => PeekAttributesParserState::Plan,
        }
    }

    fn process_from_attributes_state(&self, event: Event) -> PeekAttributesParserState {
        match event {
            Event::Start(event) if event.name().into_inner() == b"attribute" => {
                let key: String = from_utf8(xml::get_attribute(b"name", &event).unwrap().deref())
                    .unwrap()
                    .to_string();
                PeekAttributesParserState::Attribute { key }
            }
            Event::End(event) if event.name().into_inner() == b"attributes" => {
                PeekAttributesParserState::Person
            }
            _ => PeekAttributesParserState::Attributes,
        }
    }

    fn process_from_attribute_state(
        &self,
        event: Event,
        key: &str,
        attributes: &mut PeekAttributes,
    ) -> PeekAttributesParserState {
        match event {
            // If we see some text we grab it as the attribute value
            Event::Text(event) => {
                let value = event.unescape().unwrap().into_owned();
                attributes
                    .entry(key.to_owned())
                    .or_insert(PeekSet::new(self.max))
                    .insert(value);
                PeekAttributesParserState::Attributes
            }
            _ => PeekAttributesParserState::Attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::{
        events::{BytesEnd, BytesStart, BytesText},
        Reader,
    };

    use super::*;

    #[test]
    fn test_parser_initial_state() {
        let parser = PeekAttributesParser::new(10);
        assert_eq!(parser.state, PeekAttributesParserState::Person)
    }

    #[test]
    fn test_expected_transitions() {
        let mut attributes = PeekAttributes::new();

        let mut parser = PeekAttributesParser::new(10);
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("population")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Person);

        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("person")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Person);

        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("attributes")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Attributes);

        let xml = r#"<attribute name="a" class="java.lang.String">"#;
        let mut reader = Reader::from_str(xml);
        let event = reader.trim_text(true).read_event().unwrap();

        parser.process(event, &mut attributes);
        assert_eq!(
            parser.state,
            PeekAttributesParserState::Attribute {
                key: "a".to_string()
            }
        );

        parser.process(
            quick_xml::events::Event::Text(BytesText::new("A")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Attributes);
        assert_eq!(
            attributes,
            BTreeMap::from([("a".to_string(), PeekSet::from_iter(["A".to_string()]))])
        );

        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("attribute")),
            &mut attributes,
        );

        let xml = r#"<attribute name="b" class="java.lang.String">"#;
        let mut reader = Reader::from_str(xml);
        let event = reader.trim_text(true).read_event().unwrap();

        parser.process(event, &mut attributes);
        assert_eq!(
            parser.state,
            PeekAttributesParserState::Attribute {
                key: "b".to_string()
            }
        );

        parser.process(
            quick_xml::events::Event::Text(BytesText::new("B")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Attributes);
        assert_eq!(
            attributes,
            BTreeMap::from([
                ("a".to_string(), PeekSet::from_iter(["A".to_string()])),
                ("b".to_string(), PeekSet::from_iter(["B".to_string()])),
            ])
        );

        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("attributes")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Person);

        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("plan")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Plan);

        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("plan")),
            &mut attributes,
        );
        assert_eq!(parser.state, PeekAttributesParserState::Person);
    }

    #[test]
    fn test_set_builds_correctly() {
        let mut attributes =
            PeekAttributes::from([("a".to_string(), PeekSet::from_iter(["A".to_string()]))]);
        let mut parser = PeekAttributesParser::new(10);
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("person")),
            &mut attributes,
        );
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("attributes")),
            &mut attributes,
        );
        let xml = r#"<attribute name="a" class="java.lang.String">"#;
        let event = Reader::from_str(xml).trim_text(true).read_event().unwrap();
        parser.process(event, &mut attributes);
        parser.process(
            quick_xml::events::Event::Text(BytesText::new("B")),
            &mut attributes,
        );
        parser.process(
            quick_xml::events::Event::End(BytesEnd::new("attributes")),
            &mut attributes,
        );
        assert_eq!(
            attributes,
            BTreeMap::from([(
                "a".to_string(),
                PeekSet::from_iter(["A".to_string(), "B".to_string()])
            )])
        );
    }
    #[test]
    fn test_set_avoids_trip_attributes() {
        let mut attributes = PeekAttributes::new();
        let mut parser = PeekAttributesParser::new(10);
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("person")),
            &mut attributes,
        );
        parser.process(
            quick_xml::events::Event::Start(BytesStart::new("plan")),
            &mut attributes,
        );
        let xml = r#"<attribute name="a" class="java.lang.String">"#;
        let event = Reader::from_str(xml).trim_text(true).read_event().unwrap();
        parser.process(event, &mut attributes);
        parser.process(
            quick_xml::events::Event::Text(BytesText::new("A")),
            &mut attributes,
        );
        assert_eq!(attributes, BTreeMap::new());
    }
}
