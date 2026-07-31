use super::Result;
use crate::{parser::ParserError, types::MarkerConfig};

pub(super) fn extract_config(comment: &str) -> Result<MarkerConfig> {
    let mut config = MarkerConfig::default();
    for opt in comment.split_whitespace().filter(|c| c.starts_with(":")) {
        extract_value(opt, &mut config)?;
    }

    Ok(config)
}

fn extract_value(opt: &str, config: &mut MarkerConfig) -> Result<()> {
    let name = extract_name(opt)?;

    match name.as_str() {
        "trim" => config.trim = extract_bool(opt, true)?,
        "offset" => config.offset = extract_int(opt)?,
        "indent" => config.indentation = Some(extract_int(opt)?),
        _ => {
            return Err(ParserError::InvalidOption {
                opt: opt.to_string(),
            });
        }
    }

    Ok(())
}

fn extract_int(opt: &str) -> Result<usize> {
    let (_, value) = opt
        .split_once("=")
        .ok_or(ParserError::InvalidIntOptionFormat {
            opt: opt.to_string(),
        })?;

    Ok(value.parse::<usize>()?)
}

fn extract_bool(opt: &str, flag_value: bool) -> Result<bool> {
    let Some(value) = opt
        .split_once('=')
        .map(|(_, value)| value.trim().to_lowercase())
    else {
        return Ok(flag_value);
    };

    Ok(value.parse::<bool>()?)
}

fn extract_name(opt: &str) -> Result<String> {
    let rest = opt
        .strip_prefix(":")
        .ok_or(ParserError::InvalidOptionNameFormat {
            opt: opt.to_string(),
        })?;

    if rest.is_empty() {
        return Err(ParserError::MissingOptionName {
            opt: opt.to_string(),
        });
    }

    let end = rest
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(rest.len());

    let name = &rest[..end].to_lowercase();

    Ok(name.to_string())
}
