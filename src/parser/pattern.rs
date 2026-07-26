use super::{ParserError, Result};
use crate::{
    parser::{detector::detect, marker::extract_marker_blocks},
    types::ParsedFile,
    validator::validate_file,
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub fn parse_patterns(includes: &[String], excludes: &[String]) -> Result<Vec<ParsedFile>> {
    let mut files: Vec<ParsedFile> = Vec::new();
    let mut includes = pattern_set(includes)?;
    let exclude_files = pattern_set(excludes)?;

    includes.retain(|path| !exclude_files.contains(path));

    for path in includes {
        files.push(parse_file(&path)?);
    }

    Ok(files)
}

fn parse_file(path: &Path) -> Result<ParsedFile> {
    validate_file(path)?;
    let lang = detect(path)?;
    let content = fs::read_to_string(path)?;
    let blocks = extract_marker_blocks(&content, path, lang)?;
    Ok(ParsedFile {
        content,
        blocks,
        path: path.to_path_buf(),
    })
}

fn pattern_set(patterns: &[String]) -> Result<HashSet<PathBuf>> {
    let mut result: HashSet<PathBuf> = HashSet::new();
    let mut no_pattern_match;

    for pattern in patterns {
        let pattern = if std::path::Path::new(pattern).is_dir() {
            format!("{}/**/*", pattern.trim_end_matches('/'))
        } else {
            pattern.to_string()
        };

        no_pattern_match = true;
        for entry in glob::glob(&pattern)? {
            no_pattern_match = false;
            let path = entry?;
            if path.is_dir() {
                continue;
            }
            result.insert(path);
        }

        if no_pattern_match {
            return Err(ParserError::NoPatternMatch { pattern });
        }
    }

    Ok(result)
}
