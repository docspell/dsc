use configr::toml;
use configr::ConfigError as ConfigrError;
use configr::{Config, Configr};
use serde::{Deserialize, Serialize};
use std::default;
use std::path::{Path, PathBuf};

#[derive(Configr, Serialize, Deserialize)]
pub struct DsConfig {
    pub docspell_url: String,
}

#[derive(Debug, snafu::Snafu)]
pub enum ConfigError {
    #[snafu(display("Unable to read config file {}: {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Unable to create default config file {}: {}", path.display(), source))]
    CreateDefault {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Unable to parse file {}: {}", path.display(), source))]
    ParseFile {
        source: toml::de::Error,
        path: PathBuf,
    },
    #[snafu(display("The config directory could not be found"))]
    NoConfigDir,
}

impl default::Default for DsConfig {
    fn default() -> Self {
        Self {
            docspell_url: "https://localhost:7880".into(),
        }
    }
}

impl DsConfig {
    pub fn read(file: &Option<String>) -> Result<DsConfig, ConfigError> {
        let result = if let Some(cfg_file) = &file {
            DsConfig::load_specific(Path::new(&cfg_file))
        } else {
            let mut dir = config_dir()?;
            DsConfig::load_custom("dsc", &mut dir)
        };
        result.map_err(make_error)
    }
}

fn config_dir() -> Result<PathBuf, ConfigError> {
    match dirs::config_dir() {
        Some(dir) => Ok(dir),
        None => Err(ConfigError::NoConfigDir),
    }
}

fn make_error(cr: ConfigrError) -> ConfigError {
    match cr {
        ConfigrError::ReadConfig { source, path } => ConfigError::ReadFile { source, path },
        ConfigrError::CreateFs { source, path } => ConfigError::CreateDefault { source, path },
        ConfigrError::Deserialize {
            source,
            path,
            toml: _,
        } => ConfigError::ParseFile { source, path },
        ConfigrError::ConfigDir => ConfigError::NoConfigDir,
    }
}
