//! Module for reading the configuration file.

use crate::cli::opts::Format;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::default;
use std::path::{Path, PathBuf};

/// Defines the contents of the configuration file.
#[derive(Serialize, Deserialize, Debug)]
pub struct DsConfig {
    pub docspell_url: String,
    pub default_format: Format,
    pub admin_secret: Option<String>,
    pub default_source_id: Option<String>,
    pub pass_entry: Option<String>,
    pub default_account: Option<String>,
    pub pdf_viewer: Vec<String>,
    pub proxy: Option<String>,
    pub proxy_user: Option<String>,
    pub proxy_password: Option<String>,
    pub extra_certificate: Option<PathBuf>,
    pub accept_invalid_certificates: Option<bool>,
}

/// Error states when reading and writing the config file.
#[derive(Debug, Snafu)]
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
            default_format: Format::Tabular,
            admin_secret: None,
            default_source_id: None,
            pass_entry: None,
            default_account: None,
            pdf_viewer: vec!["zathura".into(), "{}".into()],
            proxy: None,
            proxy_user: None,
            proxy_password: None,
            extra_certificate: None,
            accept_invalid_certificates: None,
        }
    }
}

impl DsConfig {
    /// Reads the configuration file.
    ///
    /// If the argument provides a config file, this is read. If not,
    /// the env variable `DSC_CONFIG` is used to lookup the
    /// configuration file. If this env variable is not set, the
    /// default location is used (which is ~/.config/dsc/config.toml`
    /// on linuxes).
    pub fn read(file: Option<&PathBuf>) -> Result<DsConfig, ConfigError> {
        if let Some(cfg_file) = &file {
            log::debug!(
                "Looking for {} in {}",
                cfg_file.to_path_buf().display(),
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown directory".into())
            );
            let given_path = cfg_file.as_path().canonicalize().context(ReadFileSnafu {
                path: cfg_file.as_path().to_path_buf(),
            })?;
            log::debug!("Load config from: {:}", given_path.display());
            load_from(&given_path)
        } else {
            match std::env::var(DSC_CONFIG).ok() {
                Some(cfg_file) => {
                    log::debug!("Loading config file given by env variable");
                    Self::read(Some(&PathBuf::from(cfg_file)))
                }
                None => {
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
        }
    }

    /// Write the default configuration to the default config file.
    /// The file must not yet exist.
    pub fn write_default_file() -> Result<PathBuf, ConfigError> {
        DsConfig::default().write_default()
    }

    /// Write this configuration to the default location. If the
    /// already file exists, a error is returned.
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

const DSC_CONFIG: &str = "DSC_CONFIG";
