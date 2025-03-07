use crate::{filter::FilterableSpec, filters::Filters, group::ConfigGroup, utils};
use rand::Rng;
use serde::Deserialize;
use tracer::population::PersonAttributes;

pub type BatteryGroup = ConfigGroup<BatterySpec>;

impl Default for BatteryGroup {
    fn default() -> Self {
        BatteryGroup::from(vec![BatterySpec::default()])
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct BatterySpec {
    pub name: Option<String>,
    pub capacity: f32,
    pub initial: f32,
    pub consumption_rate: f32,
    pub p: Option<f32>,
    pub filters: Option<Filters>,
}

impl Default for BatterySpec {
    //todo change from KWs to KWhr
    fn default() -> Self {
        BatterySpec {
            name: Some("default".to_string()),
            capacity: 100.0,
            initial: 100.0,
            consumption_rate: 0.15,
            p: None,
            filters: None,
        }
    }
}

impl FilterableSpec for BatterySpec {
    fn matches(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> bool {
        match self.filters {
            None => utils::sample_p(self.p, rng),
            Some(ref filters) => filters.filter(attributes) & utils::sample_p(self.p, rng),
        }
    }
}

impl BatterySpec {
    #[allow(dead_code)]
    pub fn unit() -> Self {
        BatterySpec {
            name: Some("unit".to_string()),
            capacity: 1.0 / 3600.0,
            initial: 1.0 / 3600.0,
            consumption_rate: 1.0 / 3.6,
            p: None,
            filters: None,
        }
    }
}

/// Battery Spec builder pattern for help with tests
// todo move to tests
#[derive(Default)]
#[allow(dead_code)]
pub struct BatterySpecBuilder {
    battery: BatterySpec,
}
#[allow(dead_code)]
impl BatterySpecBuilder {
    pub fn new() -> BatterySpecBuilder {
        BatterySpecBuilder::default()
    }
    pub fn name(mut self, name: String) -> BatterySpecBuilder {
        self.battery.name = Some(name);
        self
    }
    /// Capacity in kWh
    pub fn capacity(mut self, capacity: f32) -> BatterySpecBuilder {
        self.battery.capacity = capacity;
        self
    }
    /// Initial state in kWh
    pub fn initial(mut self, initial: f32) -> BatterySpecBuilder {
        self.battery.initial = initial;
        self
    }
    /// Give full initial state (based on capacity)
    pub fn full(mut self) -> BatterySpecBuilder {
        self.battery.initial = self.battery.capacity;
        self
    }
    /// Consumption rate in kWh/km
    pub fn consumption_rate(mut self, consumption_rate: f32) -> BatterySpecBuilder {
        self.battery.consumption_rate = consumption_rate;
        self
    }
    pub fn build(self) -> BatterySpec {
        self.battery
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, filter::Filter};

    use super::*;

    #[test]
    fn test_build_default() {
        BatterySpec::default();
    }

    #[test]
    fn test_build_unit() {
        BatterySpec::unit();
    }

    #[test]
    fn test_builder_default() {
        assert_eq!(BatterySpecBuilder::new().build(), BatterySpec::default())
    }

    #[test]
    fn test_builder_full() {
        assert_eq!(
            BatterySpecBuilder::new()
                .name("test".to_string())
                .capacity(1.)
                .consumption_rate(1.)
                .full()
                .build(),
            BatterySpec {
                name: Some("test".to_string()),
                capacity: 1.,
                initial: 1.,
                consumption_rate: 1.0,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_builder_initial() {
        assert_eq!(
            BatterySpecBuilder::new()
                .name("test".to_string())
                .capacity(10.)
                .consumption_rate(1.)
                .initial(5.)
                .build(),
            BatterySpec {
                name: Some("test".to_string()),
                capacity: 10.,
                initial: 5.,
                consumption_rate: 1.,
                ..Default::default()
            }
        )
    }

    #[test]
    fn load_battery_group() {
        let str = "name: test
battery_group:
  - name: test
    capacity: 100
    trigger: 2
    initial: 10
    consumption_rate: 1";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_batteries = BatteryGroup::from(BatterySpec {
            name: Some("test".to_string()),
            capacity: 100.0,
            initial: 10.0,
            consumption_rate: 1.0,
            ..Default::default()
        });
        assert_eq!(
            decoded,
            Config {
                name: Some("test".to_string()),
                scale: Some(1.0),
                patience: Some(100),
                seed: None,
                battery_group: expected_batteries,
                ..Default::default()
            }
        )
    }

    #[test]
    fn load_battery_filter_group() {
        let str = "name: test
battery_group:
  - name: test
    capacity: 100
    initial: 10
    consumption_rate: 1
    filters:
      - {key: a, values: [A, B]}
      - {key: b, values: [C]}";
        let decoded: Config = Config::from_yaml(str).unwrap();
        let expected_filters: Filters = Filters::from(vec![
            Filter {
                key: "a".to_string(),
                values: vec!["A".to_string(), "B".to_string()],
            },
            Filter {
                key: "b".to_string(),
                values: vec!["C".to_string()],
            },
        ]);
        assert_eq!(decoded.battery_group[0].filters, Some(expected_filters))
    }
}
