pub mod charge_plan;
pub mod config;
pub mod filter;
pub mod filters;
pub mod group;
pub mod groups;
pub mod handler;
pub mod sampler;
pub mod utils;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BatsimConfigError {
    #[error("config scale is invalid")]
    InvalidScale,

    #[error("en-route charging not made available for pid: '{0}'")]
    AgentMissingEnRouteCharging(String),

    #[error("charge 'trigger' not made available for pid: '{0}'")]
    AgentMissingTrigger(String),
}
