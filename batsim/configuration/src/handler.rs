use anyhow::Result;
use rand::Rng;
use serde::Serialize;

use crate::{
    config::Config,
    groups::{
        activity::ActivitySpec, battery::BatterySpec, en_route::EnRouteSpec, trigger::TriggerSpec,
    },
    BatsimConfigError,
};
use tracer::Person;

pub struct AgentConfig<'a> {
    pub pid: &'a str,
    pub battery: Option<&'a BatterySpec>,
    pub trigger: Option<&'a TriggerSpec>,
    pub en_route: Option<&'a EnRouteSpec>,
    pub activities: Vec<&'a ActivitySpec>,
}

#[derive(serde::Serialize)]
pub struct AgentConfigRecord<'a> {
    pid: &'a str,
    battery: &'a str,
    trigger: &'a str,
    en_route: &'a str,
    activities: String,
}

impl<'a> AgentConfig<'a> {
    pub fn build(
        config: &'a Config,
        pid: &'a str,
        person: &'a Person,
        rng: &mut impl Rng,
    ) -> AgentConfig<'a> {
        let attributes = &person.attributes;
        AgentConfig {
            pid,
            battery: config.battery_group.find(attributes, rng),
            trigger: config.trigger_group.find(attributes, rng),
            en_route: config.enroute_group.find(attributes, rng),
            activities: config.activity_group.filter(attributes, rng),
        }
    }
    /// Check that enroute charging is available if a battery is available
    pub fn validate(&self) -> Result<()> {
        if self.battery.is_some() & self.en_route.is_none() {
            anyhow::bail!(BatsimConfigError::AgentMissingEnRouteCharging(
                self.pid.to_string()
            ))
        } else if self.battery.is_some() & self.trigger.is_none() {
            anyhow::bail!(BatsimConfigError::AgentMissingTrigger(self.pid.to_string()))
        } else {
            Ok(())
        }
    }
    pub fn to_record(&self) -> AgentConfigRecord
    where
        AgentConfigRecord<'a>: Serialize,
    {
        let battery: &str = match self.battery {
            Some(spec) => spec.name.as_deref().unwrap(),
            None => "None",
        };
        let trigger: &str = match self.trigger {
            Some(spec) => spec.name.as_deref().unwrap(),
            None => "None",
        };
        let en_route: &str = match self.en_route {
            Some(spec) => spec.name.as_deref().unwrap(),
            None => "None",
        };

        let activities = self
            .activities
            .iter()
            .filter_map(|cnfg| cnfg.name.clone())
            .collect::<Vec<String>>()
            .join("+");

        AgentConfigRecord {
            pid: self.pid,
            battery,
            trigger,
            en_route,
            activities,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // todo!
}
