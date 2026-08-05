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
        "indent" => config.indent = Some(extract_int(opt)?),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> MarkerConfig {
        MarkerConfig::default()
    }

    #[test]
    fn test_default_config_when_no_options() {
        let config = extract_config("injm begin <hello").unwrap();
        assert_eq!(config, default_config());
    }

    #[test]
    fn test_offset_option() {
        let config = extract_config("injm begin <hello :offset=1").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                offset: 1,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_offset_zero_is_default() {
        let config = extract_config("injm begin <hello :offset=0").unwrap();
        assert_eq!(config, default_config());
    }

    #[test]
    fn test_offset_multiple_digits() {
        let config = extract_config("injm begin <hello :offset=12").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                offset: 12,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_trim_option() {
        let config = extract_config("injm begin <hello :trim=true").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: true,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_trim_as_flag_without_value() {
        let config = extract_config("injm begin <hello :trim").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: true,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_indent_option() {
        let config = extract_config("injm begin <hello :indent=4").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                indent: Some(4),
                ..default_config()
            }
        );
    }

    #[test]
    fn test_multiple_options() {
        let config = extract_config("injm begin <hello :trim=true :offset=2 :indent=4").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                offset: 2,
                trim: true,
                indent: Some(4),
            }
        );
    }

    #[test]
    fn test_options_in_any_order() {
        let config = extract_config("injm begin <hello :indent=4 :offset=2").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                offset: 2,
                indent: Some(4),
                ..default_config()
            }
        );
    }

    #[test]
    fn test_options_with_output_marker() {
        let config = extract_config("injm begin >id :trim=true").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: true,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_option_names_are_case_insensitive() {
        let config = extract_config("injm begin <hello :TRIM=true").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: true,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_multiple_same_option_last_wins() {
        let config = extract_config("injm begin <hello :offset=1 :offset=3").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                offset: 3,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_id_is_not_parsed_as_option() {
        let config = extract_config("injm begin <hello").unwrap();
        assert_eq!(config, default_config());
    }

    #[test]
    fn test_unknown_option_errors() {
        let err = extract_config("injm begin <hello :unknown=1").unwrap_err();
        assert!(matches!(err, ParserError::InvalidOption { .. }));
    }

    #[test]
    fn test_missing_option_name_errors() {
        let err = extract_config("injm begin <hello :").unwrap_err();
        assert!(matches!(err, ParserError::MissingOptionName { .. }));
    }

    #[test]
    fn test_int_option_without_equals_errors() {
        let err = extract_config("injm begin <hello :offset").unwrap_err();
        assert!(matches!(err, ParserError::InvalidIntOptionFormat { .. }));
    }

    #[test]
    fn test_int_option_with_non_integer_errors() {
        let err = extract_config("injm begin <hello :offset=abc").unwrap_err();
        assert!(matches!(err, ParserError::ParseIntError(_)));
    }

    #[test]
    fn test_int_option_with_trailing_text_errors() {
        let err = extract_config("injm begin <hello :offset=1x").unwrap_err();
        assert!(matches!(err, ParserError::ParseIntError(_)));
    }

    #[test]
    fn test_indent_without_value_errors() {
        let err = extract_config("injm begin <hello :indent").unwrap_err();
        assert!(matches!(err, ParserError::InvalidIntOptionFormat { .. }));
    }

    #[test]
    fn test_unknown_option_does_not_apply_previous() {
        let err = extract_config("injm begin <hello :offset=1 :unknown=2").unwrap_err();
        assert!(matches!(err, ParserError::InvalidOption { .. }));
    }

    #[test]
    fn test_trim_false() {
        let config = extract_config("injm begin <hello :trim=false").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: false,
                ..default_config()
            }
        );
    }

    #[test]
    fn test_trim_value_is_case_insensitive() {
        let config = extract_config("injm begin <hello :trim=TRUE").unwrap();
        assert_eq!(
            config,
            MarkerConfig {
                trim: true,
                ..default_config()
            }
        );

        let config = extract_config("injm begin <hello :trim=False").unwrap();
        assert_eq!(config, default_config());
    }

    #[test]
    fn test_trim_value_with_internal_whitespace_is_split() {
        let err = extract_config("injm begin <hello :trim= true").unwrap_err();
        assert!(matches!(err, ParserError::ParseBoolError(_)));
    }

    #[test]
    fn test_trim_invalid_value_errors() {
        for bad in [":trim=1", ":trim=yes", ":trim=on", ":trim=abc", ":trim="] {
            let opt = format!("injm begin <hello {bad}");
            let err = extract_config(&opt).unwrap_err();
            assert!(matches!(err, ParserError::ParseBoolError(_)), "{bad}");
        }
    }

    #[test]
    fn test_trim_repeated_last_wins() {
        let config = extract_config("injm begin <hello :trim=true :trim=false").unwrap();
        assert_eq!(config, default_config());
    }
}
