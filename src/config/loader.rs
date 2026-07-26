use std::{fs, path::PathBuf};

use crate::config::{ConfigError, GlobalConfig, Result};

pub(crate) fn load_config(config_path: Option<PathBuf>) -> Result<GlobalConfig> {
    let Some(path) = config_path else {
        return Ok(GlobalConfig::default());
    };

    if !fs::exists(&path)? {
        return Err(ConfigError::ConfigFileNotExists { path });
    }

    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}
