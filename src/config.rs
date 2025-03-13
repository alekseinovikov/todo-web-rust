use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) logger: LoggerSettings,
    pub(crate) graceful_shutdown_timeout_seconds: u64,
    pub(crate) port: String,
    pub(crate) metrics_port: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoggerSettings {
    pub(crate) level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            logger: LoggerSettings {
                level: "info".to_string(),
            },
            graceful_shutdown_timeout_seconds: 10,
            port: "8080".to_string(),
            metrics_port: "9090".to_string(),
        }
    }
}

pub(crate) fn load_config() -> Result<Settings, ConfigError> {
    let config = Config::builder()
        // defaults
        .set_default("logger.level", "info")?
        .set_default("graceful_shutdown_timeout_seconds", 10)?
        .set_default("port", "8080")?
        .set_default("metrics_port", "9090")?
        // source file
        .add_source(File::with_name("config").required(false))
        .add_source(Environment::default().separator("_"))
        .build()?;
    config.try_deserialize()
}
