use crate::cli::{GlobalArgs, ListArgs};
use crate::cmd::merged_with_fallback;
use crate::config::load_config;
use crate::output::print;
use crate::parser::{PatternParserOption, parse_patterns};
use crate::types::{BlockRole, SourceSpan};
use anyhow::Result;
use core::fmt;
use serde::Serialize;
use std::path::Path;
use tabled::Tabled;

#[derive(Serialize, Tabled)]
pub struct MarkerInfo {
    #[tabled(rename = "File")]
    pub file: String,

    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Type")]
    pub marker_type: MarkerType,

    #[tabled(rename = "Lines")]
    pub lines: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    Input,
    Output,
}

impl fmt::Display for MarkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkerType::Input => write!(f, "input"),
            MarkerType::Output => write!(f, "output"),
        }
    }
}

pub fn run(args: ListArgs, global_args: GlobalArgs) -> Result<()> {
    let cfg = load_config(global_args.config)?;

    // If the input is empty, then fallback to current directory (`.`)
    let input = merged_with_fallback(args.files, cfg.input, cfg.output);
    let excludes: Vec<String> = cfg.exclude.into_iter().chain(global_args.exclude).collect();

    // Get all input and output blocks.
    let mut rows = Vec::new();

    let files = parse_patterns(
        &input,
        &excludes,
        &PatternParserOption {
            no_gitignore: global_args.no_gitignore,
            cwd: std::env::current_dir()?,
        },
    )?;
    for file in &files {
        for block in &file.blocks {
            match &block.role {
                BlockRole::Input { ids, .. } => {
                    for id in ids {
                        push_row(&mut rows, &file.path, MarkerType::Input, id, &block.span);
                    }
                }
                BlockRole::Output { id } => {
                    if let Some(id) = id {
                        push_row(&mut rows, &file.path, MarkerType::Output, id, &block.span);
                    }
                }
            }
        }
    }

    // Display input and output blocks, including
    // Path, ID, input/output.
    print(&rows, args.format)?;

    Ok(())
}

fn push_row(
    rows: &mut Vec<MarkerInfo>,
    file: &Path,
    marker_type: MarkerType,
    id: &str,
    span: &SourceSpan,
) {
    rows.push(MarkerInfo {
        file: file.display().to_string(),
        marker_type,
        id: id.to_string(),
        lines: span.display_lines(),
    });
}
