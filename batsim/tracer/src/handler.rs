use crate::{MATSimEvent, Network, Node, Population};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::from_utf8;

use super::MATSimEventsReader;

pub struct TraceHandler<'a> {
    network: Option<&'a Network>,
    activity_starts: HashMap<String, (u32, Option<String>, Option<String>)>,
    link_entries: HashMap<String, u32>,
}

impl<'a> Default for TraceHandler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TraceHandler<'a> {
    pub fn new() -> TraceHandler<'a> {
        TraceHandler {
            network: None,
            activity_starts: HashMap::new(),
            link_entries: HashMap::new(),
        }
    }

    pub fn add_network(&mut self, nw: &'a Network) {
        self.network = Some(nw);
    }

    pub fn add_traces(
        &mut self,
        population: &mut Population,
        events: &'a mut MATSimEventsReader,
    ) -> Result<()> {
        let network = self.network.context("network not added to handler")?;
        let mut buf = Vec::new();
        loop {
            match events.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(raw_event) => {
                    let event = MATSimEvent::from_raw_event(&raw_event)?;
                    self.process(population, &event, network)?;
                }
                Err(e) => panic!("Error at position {}: {:?}", events.buffer_position(), e),
            }
            buf.clear();
        }
        self.finalise(population, network);
        self.clean(population);
        self.wrap(population);
        Ok(())
    }

    pub fn process(
        &mut self,
        population: &mut Population,
        event: &MATSimEvent,
        network: &Network,
    ) -> Result<()> {
        match event {
            MATSimEvent::ActStart(e) => {
                let pid = from_utf8(&e.person)?.to_string();
                if population.people.contains_key(&pid) {
                    let act_type = from_utf8(&e.act_type)?.to_string();
                    let lid: String = from_utf8(&e.lid)?.to_string();
                    self.activity_starts
                        .insert(pid, (e.time, Some(act_type), Some(lid)));
                }
                Ok(())
            }
            MATSimEvent::ActEnd(e) => {
                let pid = from_utf8(&e.person).unwrap().to_string();
                if let Some(person) = population.people.get_mut(&pid) {
                    let (start_time, act_type, lid) =
                        self.activity_starts.remove(&pid).unwrap_or((0, None, None));
                    let end_time = e.time;
                    let act_type = match act_type {
                        Some(act) => act,
                        None => from_utf8(&e.act_type)?.to_string(),
                    };
                    let lid = match lid {
                        Some(lid) => lid,
                        None => from_utf8(&e.lid)?.to_string(),
                    };
                    let (_, node) = network
                        .links
                        .get(&lid)
                        .context(format!("failed to find link '{}' in network", &lid))?;
                    person.trace.add(Component::ActivityType(Activity {
                        start_time,
                        end_time,
                        act: act_type,
                        node: *node,
                    }))
                }
                Ok(())
            }
            MATSimEvent::EnteredLink(e) => {
                let pid = from_utf8(&e.vehicle).unwrap().to_string();
                if population.people.contains_key(&pid) {
                    self.link_entries.insert(pid, e.time);
                }
                Ok(())
            }
            MATSimEvent::LeftLink(e) => {
                let pid = from_utf8(&e.vehicle).unwrap().to_string();
                if let Some(person) = population.people.get_mut(&pid) {
                    if let Some(start_time) = self.link_entries.remove(&pid) {
                        let end_time = e.time;
                        let lid = from_utf8(&e.link)?.to_string();
                        let (distance, node) = network
                            .links
                            .get(&lid)
                            .context(format!("failed to find link '{}' in network", &lid))?;
                        person.trace.add(Component::LinkType(Link {
                            start_time,
                            end_time,
                            lid,
                            distance: *distance,
                            node: *node,
                        }))
                    }
                }
                Ok(())
            }

            MATSimEvent::VehicleLeavesTraffic(e) => {
                let pid = from_utf8(&e.vehicle).unwrap().to_string();
                if let Some(person) = population.people.get_mut(&pid) {
                    if let Some(start_time) = self.link_entries.remove(&pid) {
                        let end_time = e.time;
                        let lid = from_utf8(&e.link)?.to_string();
                        let (distance, node) = network
                            .links
                            .get(&lid)
                            .context(format!("failed to find link '{}' in network", &lid))?;
                        person.trace.add(Component::LinkType(Link {
                            start_time,
                            end_time,
                            lid,
                            distance: *distance * 0.5,
                            node: *node,
                        }))
                    }
                }
                Ok(())
            }
            MATSimEvent::Eof => Ok(()),
            MATSimEvent::Other => Ok(()),
        }
    }
    /// Add final activity assuming end time at 24 hours (this could result in negative durations)
    // todo consider cropping or some other method
    pub fn finalise(&self, population: &mut Population, network: &Network) {
        for (pid, (start_time, act_type, lid)) in &self.activity_starts {
            if let Some(person) = population.people.get_mut(pid) {
                let act_type = match act_type {
                    Some(act) => act.to_owned(),
                    None => "home".to_string(),
                };
                let lid = match lid {
                    Some(lid) => lid,
                    None => panic!("Failed to find activity link when finalising activity: person {}, {} at {}", pid, act_type, start_time),
                };
                let (_, node) = network.links.get(lid).unwrap();
                person.trace.add(Component::ActivityType(Activity {
                    start_time: *start_time,
                    end_time: 24 * 60 * 60,
                    act: act_type,
                    node: *node,
                }))
            }
        }
    }
    /// Remove plans with no links
    pub fn clean(&self, population: &mut Population) {
        population
            .people
            .retain(|_, person| person.trace.contains_link());
    }
    /// Wrap activities
    pub fn wrap(&self, population: &mut Population) {
        for (pid, person) in population.people.iter_mut() {
            if person.trace.is_wrappable() {
                match person.trace.wrap() {
                    Ok(_) => (),
                    Err(e) => println!("WrapError {} at person: {}", e, pid),
                }
            }
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    pub plan: Vec<Component>,
}
impl Trace {
    fn add(&mut self, component: Component) {
        self.plan.push(component);
    }
    fn len(&self) -> usize {
        self.plan.len()
    }
    fn first(&self) -> Option<&Component> {
        match self.plan.len() {
            0 => None,
            _usize => Some(&self.plan[0]),
        }
    }
    fn last(&self) -> Option<&Component> {
        match self.plan.len() {
            0 => None,
            n => Some(&self.plan[n - 1]),
        }
    }
    fn last_mut(&mut self) -> Option<&mut Component> {
        match self.plan.len() {
            0 => None,
            n => Some(&mut self.plan[n - 1]),
        }
    }
    fn is_wrappable(&self) -> bool {
        if self.len() <= 1 {
            return false;
        }
        let first_act = match self.first() {
            Some(Component::ActivityType(activity)) => &activity.act,
            _ => return false,
        };
        let last_act = match self.last() {
            Some(Component::ActivityType(activity)) => &activity.act,
            _ => return false,
        };
        first_act == last_act
    }
    fn wrap(&mut self) -> WrapResult<()> {
        let start_duration: u32 = match self.first() {
            Some(Component::ActivityType(activity)) => activity.duration(),
            Some(Component::LinkType(_)) => return Err(WrapError),
            None => panic!("No start activity returned"),
        };
        match self.last_mut() {
            Some(Component::ActivityType(activity)) => activity.end_time += start_duration,
            Some(Component::LinkType(_)) => return Err(WrapError),
            None => panic!("No end activity returned"),
        }
        self.plan.remove(0);
        Ok(())
    }
    fn contains_link(&self) -> bool {
        for component in &self.plan {
            if let Component::LinkType(_) = component {
                return true;
            }
        }
        false
    }
}

type WrapResult<T> = std::result::Result<T, WrapError>;
#[derive(Debug, Clone)]
struct WrapError;
impl fmt::Display for WrapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to wrap")
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Component {
    ActivityType(Activity),
    LinkType(Link),
}
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Activity {
    pub start_time: u32,
    pub end_time: u32,
    pub act: String,
    pub node: Node,
}
impl Activity {
    pub fn duration(&self) -> u32 {
        self.end_time - self.start_time
    }
}
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub start_time: u32,
    pub end_time: u32,
    pub lid: String,
    pub distance: f32,
    pub node: Node,
}
impl Link {
    pub fn duration(&self) -> u32 {
        self.end_time - self.start_time
    }
    pub fn speed(&self) -> f32 {
        self.distance / self.duration() as f32
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::BTreeMap};

    use crate::{
        events::{ActEnd, ActStart, EnteredLink, LeftLink, VehicleLeavesTraffic},
        Person,
    };

    use super::*;

    fn network() -> Network {
        Network {
            links: HashMap::from_iter([
                ("a".to_string(), (1.0, (0.0, 0.0))),
                ("b".to_string(), (1.0, (0.0, 0.0))),
            ]),
        }
    }
    fn population() -> Population {
        Population {
            people: BTreeMap::from_iter([(
                "0".to_string(),
                Person {
                    attributes: HashMap::from_iter([("a".to_string(), "a".to_string())]),
                    trace: Trace::default(),
                },
            )]),
        }
    }

    #[test]
    fn test_parse_activity_end() {
        let mut handler = TraceHandler::new();
        let network = network();
        let mut population = population();
        let event = MATSimEvent::ActEnd(ActEnd {
            time: 1,
            person: Cow::Borrowed(b"0"),
            act_type: Cow::Borrowed(b"home"),
            lid: Cow::Borrowed(b"a"),
        });
        _ = handler.process(&mut population, &event, &network);
        let plan = &population.people["0"].trace.plan;
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            Component::ActivityType(Activity {
                start_time: 0,
                end_time: 1,
                act: "home".to_string(),
                node: (0.0, 0.0)
            })
        )
    }

    #[test]
    fn test_parse_activity_start() {
        let mut handler = TraceHandler::new();
        let network = network();
        let mut population = population();
        let event = MATSimEvent::ActStart(ActStart {
            time: 1,
            person: Cow::Borrowed(b"0"),
            act_type: Cow::Borrowed(b"home"),
            lid: Cow::Borrowed(b"a"),
        });

        _ = handler.process(&mut population, &event, &network);
        handler.finalise(&mut population, &network);
        let plan = &population.people["0"].trace.plan;
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            Component::ActivityType(Activity {
                start_time: 1,
                end_time: 24 * 60 * 60,
                act: "home".to_string(),
                node: (0.0, 0.0)
            })
        )
    }

    #[test]
    fn test_parse_plan_wrap() {
        let mut handler = TraceHandler::new();
        let network = network();
        let mut population = population();
        _ = handler.process(
            &mut population,
            &MATSimEvent::ActEnd(ActEnd {
                time: 1,
                person: Cow::Borrowed(b"0"),
                act_type: Cow::Borrowed(b"home"),
                lid: Cow::Borrowed(b"a"),
            }),
            &network,
        );
        _ = handler.process(
            &mut population,
            &MATSimEvent::EnteredLink(EnteredLink {
                time: 1,
                vehicle: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"a"),
            }),
            &network,
        );
        _ = handler.process(
            &mut population,
            &MATSimEvent::LeftLink(LeftLink {
                time: 2,
                vehicle: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"a"),
            }),
            &network,
        );
        _ = handler.process(
            &mut population,
            &MATSimEvent::EnteredLink(EnteredLink {
                time: 2,
                vehicle: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"b"),
            }),
            &network,
        );
        _ = handler.process(
            &mut population,
            &MATSimEvent::VehicleLeavesTraffic(VehicleLeavesTraffic {
                time: 3,
                vehicle: Cow::Borrowed(b"0"),
                person: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"b"),
                mode: Cow::Borrowed(b"car"),
            }),
            &network,
        );
        _ = handler.process(
            &mut population,
            &MATSimEvent::ActStart(ActStart {
                time: 3,
                person: Cow::Borrowed(b"0"),
                act_type: Cow::Borrowed(b"home"),
                lid: Cow::Borrowed(b"a"),
            }),
            &network,
        );
        handler.finalise(&mut population, &network);
        handler.clean(&mut population);
        handler.wrap(&mut population);
        let plan = &population.people["0"].trace.plan;
        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan[0],
            Component::LinkType(Link {
                start_time: 1,
                end_time: 2,
                lid: "a".to_string(),
                distance: 1.0,
                node: (0.0, 0.0)
            })
        );
        assert_eq!(
            plan[1],
            Component::LinkType(Link {
                start_time: 2,
                end_time: 3,
                lid: "b".to_string(),
                distance: 0.5,
                node: (0.0, 0.0)
            })
        );
        assert_eq!(
            plan[2],
            Component::ActivityType(Activity {
                start_time: 3,
                end_time: (24 * 60 * 60) + 1,
                act: "home".to_string(),
                node: (0.0, 0.0)
            })
        )
    }
}
