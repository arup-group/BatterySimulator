use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

use crate::groups::{
    activity::ActivityGroup, battery::BatteryGroup, en_route::EnRouteGroup, trigger::TriggerGroup,
};
use crate::BatsimConfigError;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    /// Optional name for configuration
    pub name: Option<String>,

    /// Scale all outputs, for example to correct for a sample
    #[serde(default = "default_scale")]
    pub scale: Option<f32>,

    #[serde(default = "default_precision")]
    pub precision: Option<f32>,

    #[serde(default = "default_patience")]
    pub patience: Option<usize>,

    pub seed: Option<u64>,

    #[serde(default)]
    pub battery_group: BatteryGroup,

    #[serde(default)]
    pub trigger_group: TriggerGroup,

    #[serde(default)]
    pub enroute_group: EnRouteGroup,

    #[serde(default)]
    pub activity_group: ActivityGroup,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: None,
            scale: Some(1.0),
            patience: Some(100),
            precision: Some(1.0),
            seed: None,
            battery_group: BatteryGroup::default(),
            trigger_group: TriggerGroup::default(),
            enroute_group: EnRouteGroup::default(),
            activity_group: ActivityGroup::default(),
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let s = fs::read_to_string(path)
            .context(format!("failed to read config from '{}'", path.display()))?;
        Self::from_yaml(&s)
    }

    pub fn valid(&self) -> Result<()> {
        if self.scale.unwrap() < 0.0 {
            bail!(BatsimConfigError::InvalidScale)
        }
        Ok(())
    }

    pub fn from_yaml(s: &str) -> Result<Self> {
        serde_yaml::from_str(s).context("Failed to parse .yaml config")
    }
}

fn default_scale() -> Option<f32> {
    Some(1.0)
}

fn default_patience() -> Option<usize> {
    Some(100)
}

fn default_precision() -> Option<f32> {
    Some(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn load_default_config() {
        let str = "";
        let decoded = Config::from_yaml(str).unwrap();
        assert_eq!(decoded, Config::default());
    }

    #[test]
    fn load_example_yaml_config() {
        let path = PathBuf::from_str("configs/sim_config.yaml").unwrap();
        let _ = Config::load(&path);
    }
}
