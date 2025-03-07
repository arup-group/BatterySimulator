use rand::Rng;
use serde::Deserialize;

use crate::{filter::FilterableSpec, filters::Filters, group::ConfigGroup, utils};
use tracer::population::PersonAttributes;

pub type ActivityGroup = ConfigGroup<ActivitySpec>;

impl Default for ActivityGroup {
    fn default() -> Self {
        ActivityGroup::from(vec![])
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ActivitySpec {
    pub name: Option<String>,
    pub activities: Vec<String>,
    pub charge_rate: f32,
    pub p: Option<f32>,
    pub filters: Option<Filters>,
}

impl ActivitySpec {
    #[allow(dead_code)]
    pub fn new(
        name: Option<String>,
        activities: Vec<String>,
        charge_rate: f32,
        p: Option<f32>,
        filters: Option<Filters>,
    ) -> Self {
        Self {
            name,
            activities,
            charge_rate,
            p,
            filters,
        }
    }
}

impl ActivitySpec {
    pub fn spec(&self) -> Self {
        ActivitySpec {
            name: self.name.clone(),
            activities: self.activities.clone(),
            charge_rate: self.charge_rate,
            p: None,
            filters: None,
        }
    }
}

impl Default for ActivitySpec {
    fn default() -> Self {
        ActivitySpec {
            name: Some("default".to_string()),
            activities: vec!["home".to_string()],
            charge_rate: 3.0,
            p: None,
            filters: None,
        }
    }
}

impl FilterableSpec for ActivitySpec {
    fn matches(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> bool {
        match self.filters {
            None => utils::sample_p(self.p, rng),
            Some(ref filters) => filters.filter(attributes) & utils::sample_p(self.p, rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, filter::Filter};

    #[test]
    fn load_charge_activity_group() {
        let str = "activity_group:
  - name: test
    activities: [home]
    charge_rate: 2
    p: 0.5
    filters:
      - {key: house_type, values: [terraced]}";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_filters: Vec<Filter> = vec![Filter {
            key: "house_type".to_string(),
            values: vec!["terraced".to_string()],
        }];
        let expected_charge_activities = ActivityGroup::from(ActivitySpec {
            name: Some("test".to_string()),
            activities: vec!["home".to_string()],
            charge_rate: 2.0,
            p: Some(0.5),
            filters: Some(Filters::from(expected_filters)),
        });
        assert_eq!(decoded.activity_group, expected_charge_activities)
    }

    #[test]
    fn load_charge_multi_activity_group() {
        let str = "activity_group:
  - name: test_a
    activities: [home]
    charge_rate: 2
  - name: test_b
    activities: [shop, work]
    charge_rate: 3
    p: 0.5
    filters:
      - {key: occupation, values: [a, b]}";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_filters: Vec<Filter> = vec![Filter {
            key: "occupation".to_string(),
            values: vec!["a".to_string(), "b".to_string()],
        }];
        let expected_charge_activities = ActivityGroup::from(vec![
            ActivitySpec {
                name: Some("test_a".to_string()),
                activities: vec!["home".to_string()],
                charge_rate: 2.0,
                p: None,
                filters: None,
            },
            ActivitySpec {
                name: Some("test_b".to_string()),
                activities: vec!["shop".to_string(), "work".to_string()],
                charge_rate: 3.0,
                p: Some(0.5),
                filters: Some(Filters::from(expected_filters)),
            },
        ]);
        assert_eq!(decoded.activity_group, expected_charge_activities)
    }
}
