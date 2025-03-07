use crate::{filter::FilterableSpec, filters::Filters, group::ConfigGroup, utils};
use rand::Rng;
use serde::Deserialize;
use tracer::population::PersonAttributes;

pub type TriggerGroup = ConfigGroup<TriggerSpec>;

impl Default for TriggerGroup {
    fn default() -> Self {
        TriggerGroup::from(vec![TriggerSpec::default()])
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TriggerSpec {
    pub name: Option<String>,
    pub trigger: f32, // todo - ensure this is between 0 and 1 inclusive
    pub p: Option<f32>,
    pub filters: Option<Filters>,
}

impl Default for TriggerSpec {
    fn default() -> Self {
        TriggerSpec {
            name: Some("default".to_string()),
            trigger: 0.2,
            p: None,
            filters: None,
        }
    }
}

impl FilterableSpec for TriggerSpec {
    fn matches(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> bool {
        match self.filters {
            None => utils::sample_p(self.p, rng),
            Some(ref filters) => filters.filter(attributes) & utils::sample_p(self.p, rng),
        }
    }
}

impl TriggerSpec {
    pub fn empty() -> Self {
        TriggerSpec {
            name: Some("empty".to_string()),
            trigger: 0.0,
            p: None,
            filters: None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, filter::Filter};
    #[test]
    fn load_trigger_group() {
        let str = "trigger_group:
  - name: default
    trigger: 0.2
  - name: brave
    trigger: 0.1
    p: 0.5
    filters:
      - {key: car_type, values: [private, taxi]}";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_filter: Filters = Filters::from(vec![Filter {
            key: "car_type".to_string(),
            values: vec!["private".to_string(), "taxi".to_string()],
        }]);
        let expected = TriggerGroup::from(vec![
            TriggerSpec {
                name: Some("default".to_string()),
                trigger: 0.2,
                p: None,
                filters: None,
            },
            TriggerSpec {
                name: Some("brave".to_string()),
                trigger: 0.1,
                p: Some(0.5),
                filters: Some(expected_filter),
            },
        ]);
        assert_eq!(decoded.trigger_group, expected)
    }
}
