use rand::Rng;
use serde::Deserialize;
use tracer::population::PersonAttributes;

pub trait FilterableSpec {
    fn matches(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> bool;
}

/// Filter struct, holds a key and vec of valid values all as Strings
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Filter {
    pub key: String,
    pub values: Vec<String>,
}

impl Filter {
    pub fn match_attributes(&self, attributes: &PersonAttributes) -> bool {
        match attributes.get(&self.key) {
            None => false,
            Some(attribute) => self.values.contains(attribute),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn filter_a() -> Filter {
        Filter {
            key: "A".to_string(),
            values: vec!["A1".to_string(), "A2".to_string()],
        }
    }

    fn filter_b() -> Filter {
        Filter {
            key: "A".to_string(),
            values: vec!["A3".to_string(), "A4".to_string()],
        }
    }

    fn filter_c() -> Filter {
        Filter {
            key: "B".to_string(),
            values: vec!["B1".to_string(), "B2".to_string()],
        }
    }

    #[test]
    fn test_match_attributes() {
        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A3".to_string());
        assert!(!filter_a().match_attributes(&attributes));
        assert!(filter_b().match_attributes(&attributes));
        assert!(!filter_c().match_attributes(&attributes));
    }
}
