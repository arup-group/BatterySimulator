use anyhow::{Context, Result};

use configuration::{charge_plan::ActivityChargingPlanner, config::Config, handler::AgentConfig};
use simulate::record::{AgentSimulationRecord, EventsRecord};
use tracer::Person;

use super::run;

pub struct OptimiseHandler<'a> {
    pub config: &'a Config,
}

impl OptimiseHandler<'_> {
    pub fn new(config: &Config) -> OptimiseHandler {
        OptimiseHandler { config }
    }

    pub fn optimise<'a>(
        &'a self,
        config: &'a Config,
        pid: &'a String,
        person: &'a Person,
        agent_config: AgentConfig<'a>,
    ) -> Result<AgentSimulationRecord> {
        match agent_config.battery {
            // run simulations
            Some(_battery_spec) => {
                let _trigger_spec = agent_config.trigger.context(format!(
                    "agent {pid}, no trigger spec provided, agent has a battery so requires a 'trigger' specification"
                ))?;
                let _en_route_spec = agent_config.en_route.context(format!(
                    "agent {pid}, no en-route charging spec provided, agent has a battery so requires an 'en-route' charging specification"
                ))?;
                let activity_charging_planner =
                    ActivityChargingPlanner::new(agent_config.activities.clone());
                self.config.patience.context("no patience provided")?;
                self.config
                    .precision
                    .context("no closing precision provided")?;
                let mut result = run::run_simulations(
                    pid,
                    person,
                    &agent_config,
                    activity_charging_planner,
                    config,
                )
                .context(format!("failed find result for '{}'", pid))?;
                result.finalise(config);
                Ok(result)
            }
            None => Ok(AgentSimulationRecord::empty(pid)), // return empty record
        }
    }
}
