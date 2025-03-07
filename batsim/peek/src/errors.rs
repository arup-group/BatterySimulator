use thiserror::Error;

#[derive(Error, Debug)]
pub enum BatsimError {
    #[error("config scale is invalid")]
    InvalidScale,

    #[error("file missing extension")]
    NoFileExtension,

    #[error("unknown extension")]
    UnknownFileExtension,

    #[error("en-route charging not made available for pid: '{0}'")]
    AgentMissingEnRouteCharging(String),

    #[error("charge 'trigger' not made available for pid: '{0}'")]
    AgentMissingTrigger(String),
}
