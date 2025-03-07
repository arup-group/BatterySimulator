use crate::groups::activity::ActivitySpec;
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Clone)]
pub struct ActivityChargingPlanner<'a> {
    pub specs: Vec<&'a ActivitySpec>,
}

impl<'a> ActivityChargingPlanner<'a> {
    pub fn new(configs: Vec<&'a ActivitySpec>) -> Self {
        ActivityChargingPlanner { specs: configs }
    }
    pub fn get(&self, key: &String) -> Option<ActivitySpec> {
        self.specs.iter().rev().find_map(|cnfg| {
            if cnfg.activities.contains(key) {
                Some(cnfg.spec())
            } else {
                None
            }
        })
    }
    pub fn activities(&self) -> Vec<&'a String> {
        // todo - what if more than one
        self.specs
            .iter()
            .flat_map(|cnfg| &cnfg.activities)
            .collect::<Vec<&'a String>>()
    }
}

impl<'a> From<Vec<&'a ActivitySpec>> for ActivityChargingPlanner<'a> {
    fn from(specs: Vec<&'a ActivitySpec>) -> ActivityChargingPlanner<'a> {
        ActivityChargingPlanner { specs }
    }
}

impl<'a> Deref for ActivityChargingPlanner<'a> {
    type Target = Vec<&'a ActivitySpec>;
    fn deref(&self) -> &Self::Target {
        &self.specs
    }
}

impl<'a> DerefMut for ActivityChargingPlanner<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.specs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::activity::ActivitySpec;

    #[test]
    fn test_from_specs() {
        let spec_a = ActivitySpec {
            name: Some("A".to_string()),
            activities: vec!["a".to_string()],
            ..Default::default()
        };
        let spec_b = ActivitySpec {
            name: Some("B".to_string()),
            activities: vec!["b1".to_string(), "b2".to_string()],
            ..Default::default()
        };
        let spec_c = ActivitySpec {
            name: None,
            activities: vec!["b1".to_string()],
            ..Default::default()
        };
        let specs = vec![&spec_a, &spec_b, &spec_c];
        let planner = ActivityChargingPlanner::from(specs);
        assert_eq!(planner.activities(), vec!["a", "b1", "b2", "b1"])
    }

    #[test]
    fn test_get() {
        let spec_a = ActivitySpec {
            name: Some("A".to_string()),
            activities: vec!["a".to_string()],
            ..Default::default()
        };
        let spec_b = ActivitySpec {
            name: Some("B".to_string()),
            activities: vec!["b1".to_string(), "b2".to_string()],
            ..Default::default()
        };
        let spec_c = ActivitySpec {
            name: None,
            activities: vec!["b1".to_string()],
            ..Default::default()
        };
        let specs = vec![&spec_a, &spec_b, &spec_c];
        let planner = ActivityChargingPlanner::from(specs);
        assert_eq!(planner.get(&"a".to_string()), Some(spec_a.spec()));
        assert_eq!(planner.get(&"b2".to_string()), Some(spec_b.spec()));
        assert_eq!(planner.get(&"b1".to_string()), Some(spec_c.spec()));
        assert_eq!(planner.get(&"c".to_string()), None);
    }
}
