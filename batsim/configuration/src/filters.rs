use serde::Deserialize;
use tracer::population::PersonAttributes;

use crate::filter::Filter;

/// Convenience struct for dealing with filters
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(transparent)] // transparent so deserializing uses the internal Vec
pub struct Filters(Vec<Filter>);

impl Filters {
    pub fn filter(&self, attributes: &PersonAttributes) -> bool {
        self.iter()
            .all(|filter| filter.match_attributes(attributes))
    }
}

impl From<Vec<Filter>> for Filters {
    fn from(filters: Vec<Filter>) -> Filters {
        Self(filters)
    }
}

impl From<Filter> for Filters {
    fn from(filter: Filter) -> Filters {
        Self(vec![filter])
    }
}

impl std::ops::Deref for Filters {
    type Target = Vec<Filter>;
    fn deref(&self) -> &Self::Target {
        &self.0
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
            key: "B".to_string(),
            values: vec!["B1".to_string(), "B2".to_string()],
        }
    }

    #[test]
    fn test_apply_filters() {
        let filters: Filters = Filters::from(vec![filter_a(), filter_b()]);
        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A1".to_string());
        attributes.insert("B".to_string(), "B3".to_string());
        assert!(!filters.filter(&attributes));

        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A3".to_string());
        attributes.insert("B".to_string(), "B1".to_string());
        assert!(!filters.filter(&attributes));

        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A1".to_string());
        attributes.insert("B".to_string(), "B1".to_string());
        assert!(filters.filter(&attributes))
    }
}
