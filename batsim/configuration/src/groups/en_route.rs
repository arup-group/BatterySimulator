use rand::Rng;
use serde::Deserialize;

use crate::{filter::FilterableSpec, filters::Filters, group::ConfigGroup, utils};
use tracer::population::PersonAttributes;

pub type EnRouteGroup = ConfigGroup<EnRouteSpec>;

impl Default for EnRouteGroup {
    fn default() -> Self {
        EnRouteGroup::from(vec![EnRouteSpec::default()])
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct EnRouteSpec {
    pub name: Option<String>,
    pub charge_rate: f32,
    pub p: Option<f32>,
    pub filters: Option<Filters>,
}

impl EnRouteSpec {
    #[allow(dead_code)]
    pub fn new(name: Option<String>, rate: f32, p: Option<f32>, filters: Option<Filters>) -> Self {
        Self {
            name,
            charge_rate: rate,
            p,
            filters,
        }
    }
}

impl Default for EnRouteSpec {
    fn default() -> Self {
        EnRouteSpec {
            name: Some("default".to_string()),
            charge_rate: 10.0,
            p: None,
            filters: None,
        }
    }
}

impl FilterableSpec for EnRouteSpec {
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
    fn load_charge_enroute_group() {
        let str = "enroute_group:
  - name: test
    charge_rate: 10
    p: 0.5
    filters:
      - {key: car_type, values: [private, taxi]}";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_filters: Filters = Filters::from(vec![Filter {
            key: "car_type".to_string(),
            values: vec!["private".to_string(), "taxi".to_string()],
        }]);
        let expected = EnRouteGroup::from(EnRouteSpec {
            name: Some("test".to_string()),
            charge_rate: 10.0,
            p: Some(0.5),
            filters: Some(expected_filters),
        });
        assert_eq!(decoded.enroute_group, expected)
    }
}
