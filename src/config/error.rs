use std::path::PathBuf;

use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error("config file does not exist: {}", path.display())]
    ConfigFileNotExists { path: PathBuf },
}
