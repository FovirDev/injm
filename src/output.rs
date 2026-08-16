use std::path::Path;

use clap::ValueEnum;
use serde::Serialize;
use similar::TextDiff;
use tabled::{Table, Tabled};

use anyhow::Result;

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

pub fn print<T>(rows: &[T], format: OutputFormat) -> Result<()>
where
    T: Serialize + Tabled,
{
    match format {
        OutputFormat::Table => {
            println!("{}", Table::new(rows));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
        }
    }

    Ok(())
}

pub fn print_diff(path: &Path, original: &str, replaced: &str) {
    let old_path = format!("a/{}", path.display());
    let new_path = format!("b/{}", path.display());

    println!("{}", unified_diff(original, replaced, &old_path, &new_path));
}

pub fn print_block_diff(path: &Path, lines: &str, id: &str, actual: &str, expected: &str) {
    let old_path = format!("{}:{}:{}:actual", path.display(), lines, id,);
    let new_path = format!("{}:{}:{}:expected", path.display(), lines, id,);

    print!("{}", unified_diff(actual, expected, &old_path, &new_path));
}

fn unified_diff(old: &str, new: &str, old_path: &str, new_path: &str) -> String {
    TextDiff::from_lines(old, new)
        .unified_diff()
        .header(old_path, new_path)
        .to_string()
}
