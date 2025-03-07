use anyhow::{Context, Result};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::borrow::Cow;
use std::io::BufRead;
use std::ops::DerefMut;
use xml;

pub struct MATSimEventsReader(Reader<Box<dyn BufRead>>);

impl MATSimEventsReader {
    pub fn from_xml(reader: Reader<Box<dyn BufRead>>) -> MATSimEventsReader {
        MATSimEventsReader(reader)
    }
}

impl std::ops::Deref for MATSimEventsReader {
    type Target = Reader<Box<dyn BufRead>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MATSimEventsReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum MATSimEvent<'a> {
    ActStart(ActStart<'a>),
    ActEnd(ActEnd<'a>),
    // VehicleEntersTraffic(VehicleEntersTraffic<'a>),
    VehicleLeavesTraffic(VehicleLeavesTraffic<'a>),
    EnteredLink(EnteredLink<'a>),
    LeftLink(LeftLink<'a>),
    Eof,
    Other,
}

impl<'a> MATSimEvent<'a> {
    /// Parse a raw event from an XML Reader.
    pub fn from_raw_event(event: &'a Event) -> Result<Self> {
        match event {
            Event::Empty(ref e) if e.name().into_inner() == b"event" => {
                let event_type = xml::get_attribute(b"type", e)?;
                match event_type.as_ref() {
                    b"actstart" => Ok(MATSimEvent::ActStart(ActStart::from_element(e)?)),
                    b"actend" => Ok(MATSimEvent::ActEnd(ActEnd::from_element(e)?)),
                    // b"vehicle enters traffic" => Ok(MATSimEvent::VehicleEntersTraffic(
                    //     VehicleEntersTraffic::from_element(e),
                    // )),
                    b"vehicle leaves traffic" => Ok(MATSimEvent::VehicleLeavesTraffic(
                        VehicleLeavesTraffic::from_element(e)?,
                    )),
                    b"entered link" => Ok(MATSimEvent::EnteredLink(EnteredLink::from_element(e)?)),
                    b"left link" => Ok(MATSimEvent::LeftLink(LeftLink::from_element(e)?)),
                    _ => Ok(MATSimEvent::Other),
                }
            }
            Event::Eof => Ok(MATSimEvent::Eof),
            _ => Ok(MATSimEvent::Other),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ActStart<'a> {
    pub time: u32,
    pub person: Cow<'a, [u8]>,
    pub act_type: Cow<'a, [u8]>,
    pub lid: Cow<'a, [u8]>,
}
impl<'a> ActStart<'a> {
    fn from_element(e: &'a BytesStart) -> Result<Self> {
        let mut attributes = e.attributes();
        attributes.with_checks(false);
        let mut time: u32 = 0;
        let mut person: Cow<[u8]> = Cow::default();
        let mut act_type: Cow<[u8]> = Cow::default();
        let mut lid: Cow<[u8]> = Cow::default();
        for attribute in attributes.flatten() {
            match attribute.key.into_inner() {
                b"time" => {
                    time = parse_matsim_time(&attribute).context("failed to parse time field")?;
                }
                b"person" => {
                    person = attribute.value;
                }
                b"actType" => {
                    act_type = attribute.value;
                }
                b"link" => {
                    lid = attribute.value;
                }
                _ => (),
            }
        }
        Ok(ActStart {
            time,
            person,
            act_type,
            lid,
        })
    }
}

fn parse_matsim_time(attribute: &Attribute) -> Result<u32> {
    let time = std::str::from_utf8(&attribute.value)?;
    let (time, _) = time.split_once('.').context("failed to split time")?;
    Ok(time.parse::<u32>()?)
}

#[derive(Debug, PartialEq)]
pub struct ActEnd<'a> {
    pub time: u32,
    pub person: Cow<'a, [u8]>,
    pub act_type: Cow<'a, [u8]>,
    pub lid: Cow<'a, [u8]>,
}
impl<'a> ActEnd<'a> {
    fn from_element(e: &'a BytesStart) -> Result<Self> {
        let mut attributes = e.attributes();
        attributes.with_checks(false);
        let mut time: u32 = 0;
        let mut person: Cow<[u8]> = Cow::default();
        let mut act_type: Cow<[u8]> = Cow::default();
        let mut lid: Cow<[u8]> = Cow::default();
        for attribute in attributes.flatten() {
            match attribute.key.into_inner() {
                b"time" => {
                    time = parse_matsim_time(&attribute).context("failed to parse time field")?;
                }
                b"person" => {
                    person = attribute.value;
                }
                b"actType" => {
                    act_type = attribute.value;
                }
                b"link" => {
                    lid = attribute.value;
                }
                _ => (),
            }
        }
        Ok(ActEnd {
            time,
            person,
            act_type,
            lid,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct VehicleLeavesTraffic<'a> {
    pub time: u32,
    pub vehicle: Cow<'a, [u8]>,
    pub person: Cow<'a, [u8]>,
    pub link: Cow<'a, [u8]>,
    pub mode: Cow<'a, [u8]>,
}
impl<'a> VehicleLeavesTraffic<'a> {
    fn from_element(e: &'a BytesStart) -> Result<Self> {
        let mut attributes = e.attributes();
        attributes.with_checks(false);
        let mut time: u32 = 0;
        let mut person: Cow<[u8]> = Cow::default();
        let mut vehicle: Cow<[u8]> = Cow::default();
        let mut link: Cow<[u8]> = Cow::default();
        let mut mode: Cow<[u8]> = Cow::default();
        for attribute in attributes.flatten() {
            match attribute.key.into_inner() {
                b"time" => {
                    time = parse_matsim_time(&attribute).context("failed to parse time field")?;
                }
                b"vehicle" => {
                    vehicle = attribute.value;
                }
                b"person" => {
                    person = attribute.value;
                }
                b"link" => {
                    link = attribute.value;
                }
                b"networkMode" => {
                    mode = attribute.value;
                }
                _ => (),
            }
        }
        Ok(VehicleLeavesTraffic {
            time,
            vehicle,
            person,
            link,
            mode,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct EnteredLink<'a> {
    pub time: u32,
    pub vehicle: Cow<'a, [u8]>,
    pub link: Cow<'a, [u8]>,
}
impl<'a> EnteredLink<'a> {
    fn from_element(e: &'a BytesStart) -> Result<Self> {
        let mut attributes = e.attributes();
        attributes.with_checks(false);
        let mut time: u32 = 0;
        let mut vehicle: Cow<[u8]> = Cow::default();
        let mut link: Cow<[u8]> = Cow::default();
        for attribute in attributes.flatten() {
            match attribute.key.into_inner() {
                b"time" => {
                    time = parse_matsim_time(&attribute).context("failed to parse time field")?;
                }
                b"vehicle" => {
                    vehicle = attribute.value;
                }
                b"link" => {
                    link = attribute.value;
                }
                _ => (),
            }
        }
        Ok(EnteredLink {
            time,
            vehicle,
            link,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct LeftLink<'a> {
    pub time: u32,
    pub vehicle: Cow<'a, [u8]>,
    pub link: Cow<'a, [u8]>,
}
impl<'a> LeftLink<'a> {
    fn from_element(e: &'a BytesStart) -> Result<Self> {
        let mut attributes = e.attributes();
        attributes.with_checks(false);
        let mut time: u32 = 0;
        let mut vehicle: Cow<[u8]> = Cow::default();
        let mut link: Cow<[u8]> = Cow::default();
        for attribute in attributes.flatten() {
            match attribute.key.into_inner() {
                b"time" => {
                    time = parse_matsim_time(&attribute).context("failed to parse time field")?;
                }
                b"vehicle" => {
                    vehicle = attribute.value;
                }
                b"link" => {
                    link = attribute.value;
                }
                _ => (),
            }
        }
        Ok(LeftLink {
            time,
            vehicle,
            link,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_start() {
        let content = "event time=\"0.0\" type=\"actstart\" person=\"0\" link=\"a\" x=/\"0.0\" y=\"0.0\" actType=\"home\"";
        let event = Event::Empty(BytesStart::from_content(content, 5));
        assert_eq!(
            MATSimEvent::from_raw_event(&event).unwrap(),
            MATSimEvent::ActStart(ActStart {
                time: 0,
                person: Cow::Borrowed(b"0"),
                act_type: Cow::Borrowed(b"home"),
                lid: Cow::Borrowed(b"a")
            })
        )
    }
    #[test]
    fn test_act_end() {
        let content = "event time=\"0.0\" type=\"actend\" person=\"0\" link=\"a\" x=/\"0.0\" y=\"0.0\" actType=\"home\"";
        let event = Event::Empty(BytesStart::from_content(content, 5));
        assert_eq!(
            MATSimEvent::from_raw_event(&event).unwrap(),
            MATSimEvent::ActEnd(ActEnd {
                time: 0,
                person: Cow::Borrowed(b"0"),
                act_type: Cow::Borrowed(b"home"),
                lid: Cow::Borrowed(b"a")
            })
        )
    }
    #[test]
    fn test_veh_leaves_traffic() {
        let content = "event time=\"0.0\" type=\"vehicle leaves traffic\" person=\"0\" link=\"a\" vehicle=\"0\" networkMode=\"car\" relativePosition=\"1.0\"";
        let event = Event::Empty(BytesStart::from_content(content, 5));
        assert_eq!(
            MATSimEvent::from_raw_event(&event).unwrap(),
            MATSimEvent::VehicleLeavesTraffic(VehicleLeavesTraffic {
                time: 0,
                person: Cow::Borrowed(b"0"),
                vehicle: Cow::Borrowed(b"0"),
                mode: Cow::Borrowed(b"car"),
                link: Cow::Borrowed(b"a")
            })
        )
    }
    #[test]
    fn test_veh_enter_link() {
        let content = "event time=\"0.0\" type=\"entered link\" link=\"a\" vehicle=\"0\"";
        let event = Event::Empty(BytesStart::from_content(content, 5));
        assert_eq!(
            MATSimEvent::from_raw_event(&event).unwrap(),
            MATSimEvent::EnteredLink(EnteredLink {
                time: 0,
                vehicle: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"a")
            })
        )
    }
    #[test]
    fn test_veh_leaves_link() {
        let content = "event time=\"0.0\" type=\"left link\" link=\"a\" vehicle=\"0\"";
        let event = Event::Empty(BytesStart::from_content(content, 5));
        assert_eq!(
            MATSimEvent::from_raw_event(&event).unwrap(),
            MATSimEvent::LeftLink(LeftLink {
                time: 0,
                vehicle: Cow::Borrowed(b"0"),
                link: Cow::Borrowed(b"a")
            })
        )
    }
}
