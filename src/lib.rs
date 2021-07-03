pub mod config;

use config::{ConfigError, DsConfig};

pub fn read_config(file: &Option<String>) -> Result<DsConfig, ConfigError> {
    DsConfig::read(file)
}
