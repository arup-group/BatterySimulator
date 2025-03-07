use anyhow::{Context, Result};
use flate2::bufread::GzDecoder;
use quick_xml::{events::BytesStart, Reader};
use std::{
    borrow::Cow,
    fmt::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BatsimXmlError {
    #[error("file missing extension")]
    NoFileExtension,

    #[error("unknown extension")]
    UnknownFileExtension,
}

pub fn reader(path: impl AsRef<Path>) -> Result<Reader<Box<dyn BufRead>>> {
    let path = path.as_ref();
    let file = File::open(path).context(format!("unable to open '{}'", path.display()))?;
    let reader = BufReader::new(file);
    let extension = path
        .extension()
        .ok_or(BatsimXmlError::NoFileExtension)
        .context(format!(
            "'{}' has no file extension, expecting either 'xml' or 'xml.gz'",
            path.display()
        ))?;

    if extension == "xml" {
        let reader: Box<dyn BufRead> = Box::new(reader);
        let xml_reader = Reader::from_reader(reader);
        Ok(xml_reader)
    } else if extension == "gz" {
        let gz_decoder = GzDecoder::new(reader);
        let reader = BufReader::new(gz_decoder);
        let reader: Box<dyn BufRead> = Box::new(reader);
        let xml_reader = Reader::from_reader(reader);
        Ok(xml_reader)
    } else {
        Err(BatsimXmlError::UnknownFileExtension).context(format!(
            "unknown file extension '{}', expecting either 'xml' or 'xml.gz'",
            path.display()
        ))
    }
}

/// Retrieve the value associated with a specific key on an XML element.
pub fn get_attribute<'b>(key: &[u8], event: &'b BytesStart) -> Result<Cow<'b, [u8]>, Error> {
    let mut attributes = event.attributes();
    attributes.with_checks(false);
    let value = attributes.find(|a| {
        if let Ok(a) = a {
            a.key == quick_xml::name::QName(key)
        } else {
            false
        }
    });
    if let Some(Ok(a)) = value {
        Ok(a.value)
    } else {
        panic!("Element did not have a '{}' key", unsafe {
            str::from_utf8_unchecked(key)
        },)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_attribute() {
        let xml = r#"tag key1='A' key2='B'"#;
        let person_event = BytesStart::from_content(xml, 3);
        assert_eq!(
            get_attribute(b"key1", &person_event).unwrap().into_owned(),
            b"A"
        );
        assert_eq!(
            get_attribute(b"key2", &person_event).unwrap().into_owned(),
            b"B"
        );
    }
    #[test]
    #[should_panic]
    fn test_get_attribute_should_panic() {
        let xml = r#"tag key1='A' key2='B'"#;
        let person_event = BytesStart::from_content(xml, 3);
        let _ = get_attribute(b"missing_key", &person_event);
    }
}
