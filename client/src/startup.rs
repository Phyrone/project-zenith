use clap::Parser;
use fast_log::error::LogError;
use fast_log::{init as init_fast_logger, Config as LoggerConfig, Logger};
use log::LevelFilter;
use serde::Serialize;

#[derive(Debug, Clone, Parser, Eq, PartialEq, Hash, Serialize)]
#[clap(version)]
pub struct StartupParams {
    #[clap(short, long, default_value = "warn")]
    #[cfg_attr(not(debug_assertions), clap(default_value = "warn"))]
    pub log_level: LevelFilter,
}

pub fn init_logger(level_filter: LevelFilter) -> Result<&'static Logger, LogError> {
    let config = LoggerConfig::new().level(level_filter).console();
    init_fast_logger(config)
}

#[derive(Debug, Clone)]
pub enum ClientStartupError {
    LoggerInit,
}

impl std::fmt::Display for ClientStartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientStartupError::LoggerInit => write!(f, "Logger init error"),
        }
    }
}
impl std::error::Error for ClientStartupError {}
