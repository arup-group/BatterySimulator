use anyhow::{Context, Result};
use quick_xml::{events::Event, Reader};
use std::{collections::HashMap, io::BufRead, str};
use thiserror::Error;

pub type Node = (f32, f32);

/// TracerError enumerates all possible errors.
#[derive(Error, Debug)]
pub enum TracerError {
    /// Represents a failure to read from input.
    #[error("failed to read xml element")]
    NetworkXMLError(quick_xml::Error),
}

/// A network stuct containing map of all link lengths (generally assumed in m)
pub struct Network {
    pub links: HashMap<String, (f32, Node)>, // len, x, y
}

impl Network {
    /// Return a network with link lengths extracted from a MATSim network file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to MATSim network xml
    ///
    pub fn from_xml(reader: &mut Reader<Box<dyn BufRead>>) -> Result<Self> {
        let mut links = HashMap::<String, (f32, Node)>::new();
        let mut nodes: HashMap<String, Node> = HashMap::<String, Node>::new();
        let mut buf = Vec::new();

        // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
        loop {
            // NOTE: this is the generic case when we don't know about the input BufRead.
            // when the input is a &str or a &[u8], we don't actually need to use another
            // buffer, we could directly call `reader.read_event()`
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(TracerError::NetworkXMLError(e)).context(format!(
                        "error reading network xml at position {}",
                        reader.buffer_position()
                    ))
                }
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,

                // match for nodes
                Ok(Event::Start(ref e)) if e.name().into_inner() == b"node" => {
                    let mut nid = String::new();
                    let mut x: f32 = f32::NAN;
                    let mut y: f32 = f32::NAN;

                    let mut attributes = e.attributes();
                    attributes.with_checks(false);

                    for attribute in attributes.flatten() {
                        // todo flatten is unpacking and moving ownership?
                        match attribute.key.local_name().as_ref() {
                            b"id" => {
                                nid = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading node 'id' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string();
                            }
                            b"x" => {
                                x = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading node 'x' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string()
                                    .parse::<f32>()
                                    .context(format!(
                                        "error parsing node 'x' as float at position {}",
                                        reader.buffer_position()
                                    ))?;
                            }
                            b"y" => {
                                y = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading node 'y' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string()
                                    .parse::<f32>()
                                    .context(format!(
                                        "error parsing node 'y' as float at position {}",
                                        reader.buffer_position()
                                    ))?;
                            }
                            _ => (),
                        }
                    }
                    // add to map
                    nodes.insert(nid, (x, y));
                }

                Ok(Event::Start(ref e)) if e.name().into_inner() == b"link" => {
                    let mut lid = String::new();
                    let mut length: f32 = f32::NAN;
                    let mut to = String::new();

                    let mut attributes = e.attributes();
                    attributes.with_checks(false);

                    for attribute in attributes.flatten() {
                        // flatten is unpacking and moving ownership???
                        match attribute.key.local_name().as_ref() {
                            b"id" => {
                                lid = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading link 'id' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string();
                            }
                            b"length" => {
                                length = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading link 'length' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string()
                                    .parse::<f32>()
                                    .context(format!(
                                        "error parsing link 'length' as float at position {}",
                                        reader.buffer_position()
                                    ))?;
                            }
                            b"to" => {
                                to = str::from_utf8(attribute.value.as_ref())
                                    .context(format!(
                                        "error reading link 'to' as str at position {}",
                                        reader.buffer_position()
                                    ))?
                                    .to_string();
                            }
                            _ => (),
                        }
                    }
                    // add to map
                    let node: Node = *nodes
                        .get(&to)
                        .context(format!("error finding node id '{}'", to))?;
                    links.insert(lid, (length, node));
                }
                _ => (),
            }
            buf.clear();
        }
        Ok(Self { links })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use xml;

    use super::*;

    #[test]
    fn network_builds_from_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/output_network.xml");
        let mut network_reader = xml::reader(&path).unwrap();
        let network = Network::from_xml(&mut network_reader);
        let expected_links = HashMap::from([
            (std::string::String::from("1-2"), (1000.0, (100.0, 0.0))),
            (std::string::String::from("1-5"), (20000.0, (0.0, 10000.0))),
            (std::string::String::from("2-1"), (1000.0, (0.0, 0.0))),
            (std::string::String::from("2-3"), (20000.0, (10000.0, 0.0))),
            (std::string::String::from("3-2"), (20000.0, (100.0, 0.0))),
            (std::string::String::from("3-4"), (1000.0, (10100.0, 0.0))),
            (std::string::String::from("4-3"), (1000.0, (10000.0, 0.0))),
            (std::string::String::from("5-1"), (20000.0, (0.0, 0.0))),
        ]);
        assert_eq!(network.unwrap().links, expected_links);
    }
}
