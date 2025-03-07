use rand::Rng;
use serde::Deserialize;
use std::ops::{Deref, DerefMut};

use crate::filter::FilterableSpec;
use tracer::population::PersonAttributes;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)] // transparent so derializing uses the internal Vec
pub struct ConfigGroup<T>(Vec<T>);

impl<T: FilterableSpec> ConfigGroup<T> {
    pub fn find(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> Option<&T> {
        self.iter().rev().find(|cnfg| cnfg.matches(attributes, rng))
    }

    pub fn filter(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> Vec<&T> {
        self.iter()
            .filter(|cnfg| cnfg.matches(attributes, rng))
            .collect()
    }
}

impl<T> From<Vec<T>> for ConfigGroup<T> {
    fn from(specs: Vec<T>) -> ConfigGroup<T> {
        Self(specs)
    }
}

impl<T> From<T> for ConfigGroup<T> {
    fn from(spec: T) -> ConfigGroup<T> {
        Self(vec![spec])
    }
}

impl<T> Deref for ConfigGroup<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ConfigGroup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{filter::Filter, filters::Filters, utils};
    use rand::{rngs::SmallRng, SeedableRng};

    type TestGroup = ConfigGroup<TestSpec>;

    #[derive(Deserialize, Debug, PartialEq, Clone)]
    struct TestSpec {
        pub name: Option<String>,
        pub p: Option<f32>,
        pub filters: Option<Filters>,
    }

    impl Default for TestSpec {
        fn default() -> Self {
            TestSpec {
                name: Some("default".to_string()),
                p: None,
                filters: None,
            }
        }
    }

    impl FilterableSpec for TestSpec {
        fn matches(&self, attributes: &PersonAttributes, rng: &mut impl Rng) -> bool {
            match &self.filters {
                None => utils::sample_p(self.p, rng),
                Some(filters) => filters.filter(attributes) && utils::sample_p(self.p, rng),
            }
        }
    }
    fn test_config_group() -> TestGroup {
        TestGroup::from(vec![
            TestSpec::default(),
            TestSpec {
                name: Some("A".to_string()),
                p: None,
                filters: Some(Filters::from(vec![
                    Filter {
                        key: "A".to_string(),
                        values: vec!["A1".to_string(), "A2".to_string()],
                    },
                    Filter {
                        key: "B".to_string(),
                        values: vec!["B1".to_string(), "B2".to_string()],
                    },
                ])),
            },
        ])
    }
    fn test_config_group_with_p() -> TestGroup {
        TestGroup::from(vec![
            TestSpec::default(),
            TestSpec {
                name: Some("A".to_string()),
                p: Some(0.5),
                filters: Some(Filters::from(vec![
                    Filter {
                        key: "A".to_string(),
                        values: vec!["A1".to_string(), "A2".to_string()],
                    },
                    Filter {
                        key: "B".to_string(),
                        values: vec!["B1".to_string(), "B2".to_string()],
                    },
                ])),
            },
        ])
    }

    fn person_empty() -> PersonAttributes {
        PersonAttributes::new()
    }
    fn person_a() -> PersonAttributes {
        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A1".to_string());
        attributes
    }
    fn person_b() -> PersonAttributes {
        let mut attributes = PersonAttributes::new();
        attributes.insert("A".to_string(), "A2".to_string());
        attributes.insert("B".to_string(), "B2".to_string());
        attributes
    }
    fn person_c() -> PersonAttributes {
        let mut attributes = PersonAttributes::new();
        attributes.insert("C".to_string(), "C1".to_string());
        attributes.insert("A".to_string(), "A1".to_string());
        attributes
    }

    #[test]
    fn test_find() {
        let mut rng = SmallRng::from_entropy();
        assert_eq!(
            test_config_group()
                .find(&person_empty(), &mut rng)
                .unwrap()
                .name,
            Some("default".to_string())
        );
        assert_eq!(
            test_config_group()
                .find(&person_a(), &mut rng)
                .unwrap()
                .name,
            Some("default".to_string())
        );
        assert_eq!(
            test_config_group()
                .find(&person_b(), &mut rng)
                .unwrap()
                .name,
            Some("A".to_string())
        );
        assert_eq!(
            test_config_group()
                .find(&person_c(), &mut rng)
                .unwrap()
                .name,
            Some("default".to_string())
        );
    }

    #[test]
    fn test_filter() {
        let mut rng = SmallRng::seed_from_u64(1234);
        assert_eq!(
            test_config_group()
                .filter(&person_empty(), &mut rng)
                .iter()
                .map(|c| c.name.as_ref().unwrap())
                .collect::<Vec<&String>>(),
            vec!["default"]
        );
        assert_eq!(
            test_config_group()
                .filter(&person_a(), &mut rng)
                .iter()
                .map(|c| c.name.as_ref().unwrap())
                .collect::<Vec<&String>>(),
            vec!["default"]
        );
        assert_eq!(
            test_config_group()
                .filter(&person_b(), &mut rng)
                .iter()
                .map(|c| c.name.as_ref().unwrap())
                .collect::<Vec<&String>>(),
            vec!["default", "A"]
        );
        assert_eq!(
            test_config_group()
                .filter(&person_c(), &mut rng)
                .iter()
                .map(|c| c.name.as_ref().unwrap())
                .collect::<Vec<&String>>(),
            vec!["default"]
        );
    }

    #[test]
    fn test_find_with_probs() {
        for _ in 0..10 {
            // failed to mock a generator so hacking seeds...
            let mut rng = SmallRng::seed_from_u64(8); // 0.45353144, 0.32385755, 0.16778237
            assert_eq!(
                test_config_group_with_p()
                    .find(&person_b(), &mut rng)
                    .unwrap()
                    .name,
                Some("A".to_string())
            );
        }
        for _ in 0..10 {
            let mut rng = SmallRng::seed_from_u64(5); // 0.732753, 0.7052366, 0.71241844
            assert_eq!(
                test_config_group_with_p()
                    .find(&person_b(), &mut rng)
                    .unwrap()
                    .name,
                Some("default".to_string())
            );
        }
    }

    #[test]
    fn test_filter_with_probs() {
        for _ in 0..10 {
            // failed to mock a generator so hacking seeds...
            let mut rng = SmallRng::seed_from_u64(8); // 0.45353144, 0.32385755, 0.16778237
            assert_eq!(
                test_config_group_with_p()
                    .filter(&person_b(), &mut rng)
                    .iter()
                    .map(|c| c.name.as_ref().unwrap())
                    .collect::<Vec<&String>>(),
                vec!["default", "A"]
            );
        }
        for _ in 0..10 {
            let mut rng = SmallRng::seed_from_u64(5); // 0.732753, 0.7052366, 0.71241844
            assert_eq!(
                test_config_group_with_p()
                    .filter(&person_b(), &mut rng)
                    .iter()
                    .map(|c| c.name.as_ref().unwrap())
                    .collect::<Vec<&String>>(),
                vec!["default"]
            );
        }
    }
}
