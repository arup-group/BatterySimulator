use core::fmt;
use indicatif::HumanCount;
use std::collections::HashMap;

use crate::events::{ChargeType, Event};
use configuration::config::Config;

#[derive(Debug)]
pub struct SummaryHandler<'a> {
    config: &'a Config,
    // charge sum
    en_route_charge: f32,
    activity_charge_map: HashMap<&'a str, f32>,
    // events count
    en_route_events: f32,
    activity_events_map: HashMap<&'a str, f32>,
    // energy leak from unclosed plans
    leak: f32,
}

impl<'a> SummaryHandler<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            en_route_charge: 0.0,
            activity_charge_map: HashMap::new(),
            en_route_events: 0.0,
            activity_events_map: HashMap::new(),
            leak: 0.0,
        }
    }

    pub fn add(&mut self, event: &'a Event) {
        match event.charge_type {
            ChargeType::EnRoute => {
                self.en_route_charge += event.charge;
                self.en_route_events += 1.0;
            }
            ChargeType::Activity => {
                self.activity_charge_map
                    .entry(event.activity.unwrap())
                    .and_modify(|charge| *charge += event.charge)
                    .or_insert(event.charge);
                self.activity_events_map
                    .entry(event.activity.unwrap())
                    .and_modify(|count| *count += 1.0)
                    .or_insert(1.0);
            }
        }
    }

    pub fn add_leak(&mut self, leak: f32) {
        self.leak += leak
    }

    pub fn finalise(&mut self) {
        self.leak *= self.config.scale.unwrap();
        self.en_route_events *= self.config.scale.unwrap();
        self.activity_events_map = self
            .activity_events_map
            .iter()
            .map(|(k, v)| (*k, v * self.config.scale.unwrap()))
            .collect();
    }
}

impl fmt::Display for SummaryHandler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let activity_charge = self.activity_charge_map.values().sum::<f32>();
        let activity_events = self.activity_events_map.values().sum::<f32>();
        let total_charge: f32 = self.en_route_charge + activity_charge;
        let total_events: f32 = self.en_route_events + activity_events;
        write!(f, "\n\nTotal Charge: {}", HumanEnergyCount(total_charge))?;
        write!(f, "\nTotal Events: {}", HumanCount(total_events as u64))?;
        write!(f, "\nTotal Energy Leak: {}", HumanEnergyCount(self.leak))?;
        write!(f, "\n\n[En Route Charging]")?;
        write!(
            f,
            "\nTotal En-route Charge: {}",
            HumanEnergyCount(self.en_route_charge)
        )?;
        write!(
            f,
            "\nTotal En-route Charge Events: {}",
            HumanCount(self.en_route_events as u64)
        )?;
        write!(f, "\n\n[Activity Charging]")?;
        write!(
            f,
            "\nTotal Activity Charge: {}",
            HumanEnergyCount(activity_charge)
        )?;
        write!(
            f,
            "\nTotal Activity Charge Events: {}",
            HumanCount(activity_events as u64)
        )?;
        write!(f, "\n\n[Charging by activity]")?;
        for (k, v) in self.activity_charge_map.iter() {
            write!(
                f,
                "\n{}: {} from {} charge events",
                k,
                HumanEnergyCount(*v),
                HumanCount(*self.activity_events_map.get(k).unwrap() as u64)
            )?;
        }
        Ok(())
    }
}

//base unit of energy is the KWs
const ENERGY_UNITS: &[(f32, &str)] = &[
    (3_600_000_000_000.0, "tWh"),
    (3_600_000_000.0, "gWh"),
    (3_600_000.0, "mWh"),
    (3_600.0, "kWh"),
    (1.0, "kWs"),
];

/// Formats Kilo-Watt-seconds for human readability
#[derive(Debug)]
pub struct HumanEnergyCount(pub f32);

impl fmt::Display for HumanEnergyCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut idx = 0;
        for (i, &(cur, _)) in ENERGY_UNITS.iter().enumerate() {
            idx = i;
            match ENERGY_UNITS.get(i + 1) {
                Some(&next) if self.0 + next.0 / 2. >= cur + cur / 2. => break,
                _ => continue,
            }
        }

        let (unit, name) = ENERGY_UNITS[idx];
        let t = (self.0 / unit).round() as usize;

        write!(f, "{} {}", t, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_energy_count_fmt() {
        assert_eq!("1000 kWs", format!("{}", HumanEnergyCount(1000.0)));
        assert_eq!("1500 kWs", format!("{}", HumanEnergyCount(1500.0)));
        assert_eq!("2 kWh", format!("{}", HumanEnergyCount(7200.0)));
        assert_eq!("1000 kWh", format!("{}", HumanEnergyCount(3_600_000.0)));
        assert_eq!("2 mWh", format!("{}", HumanEnergyCount(5_400_000.0)));
    }
}
