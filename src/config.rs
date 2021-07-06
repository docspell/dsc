use crate::opts::Format;
use serde::{Deserialize, Serialize};
use std::default;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct DsConfig {
    pub docspell_url: String,
    pub default_format: Format,
    pub admin_secret: Option<String>,
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
    #[snafu(display("The config file could not be serialized"))]
    WriteFile {
        source: toml::ser::Error,
        path: PathBuf,
    },
    #[snafu(display("The config directory could not be found"))]
    NoConfigDir,
}

impl default::Default for DsConfig {
    fn default() -> Self {
        Self {
            docspell_url: "http://localhost:7880".into(),
            default_format: Format::Json,
            admin_secret: None,
        }
    }
}

impl DsConfig {
    pub fn read(file: &Option<String>) -> Result<DsConfig, ConfigError> {
        if let Some(cfg_file) = &file {
            let given_path = Path::new(&cfg_file);
            let cfg = given_path
                .canonicalize()
                .map_err(|e| ConfigError::ReadFile {
                    source: e,
                    path: given_path.to_path_buf(),
                })?;
            log::debug!("Load config from: {:}", cfg.display());
            load_from(&cfg)
        } else {
            let mut dir = config_dir()?;
            dir.push("dsc");
            dir.push("config.toml");
            if dir.exists() {
                log::debug!("Load config from: {:}", dir.display());
                load_from(&dir)
            } else {
                log::debug!("No config file present; using default config");
                Ok(DsConfig::default())
            }
        }
    }

    pub fn write_default_file() -> Result<PathBuf, ConfigError> {
        DsConfig::default().write_default()
    }

    pub fn write_default(&self) -> Result<PathBuf, ConfigError> {
        let mut dir = config_dir()?;
        dir.push("dsc");
        dir.push("config.toml");
        if dir.exists() {
            log::info!("The default config file already exists. Not writing it!");
            Err(ConfigError::CreateDefault {
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "The config file already exists!",
                ),
                path: dir,
            })
        } else {
            log::debug!("Writing config file: {:}", dir.display());
            write_to(self, &dir)?;
            Ok(dir)
        }
    }
}

fn load_from(file: &Path) -> Result<DsConfig, ConfigError> {
    let cnt = std::fs::read_to_string(file).map_err(|e| ConfigError::ReadFile {
        source: e,
        path: file.to_path_buf(),
    });
    cnt.and_then(|c| {
        toml::from_str(&c).map_err(|e| ConfigError::ParseFile {
            source: e,
            path: file.to_path_buf(),
        })
    })
}

fn config_dir() -> Result<PathBuf, ConfigError> {
    match dirs::config_dir() {
        Some(dir) => Ok(dir),
        None => Err(ConfigError::NoConfigDir),
    }
}

fn write_to(cfg: &DsConfig, file: &Path) -> Result<(), ConfigError> {
    if !file.exists() {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| ConfigError::CreateDefault {
                source: e,
                path: file.to_path_buf(),
            })?;
        }
    }
    let cnt = toml::to_string(cfg).map_err(|e| ConfigError::WriteFile {
        source: e,
        path: file.to_path_buf(),
    });

    cnt.and_then(|c| {
        std::fs::write(&file, &c).map_err(|e| ConfigError::CreateDefault {
            source: e,
            path: file.to_path_buf(),
        })
    })
}
