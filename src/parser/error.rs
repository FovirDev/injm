use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    Process(#[from] tree_sitter_language_pack::Error),

    #[error(transparent)]
    Checker(#[from] crate::validator::ValidatorError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Glob(#[from] glob::GlobError),

    #[error(transparent)]
    Pattern(#[from] glob::PatternError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("unsupported file type: {}",path.display())]
    UnsupportedFileType { path: PathBuf },

    #[error("found nested `injm begin` without `injm end` at line {line} of {}", path.display())]
    NestedMarker { line: usize, path: PathBuf },

    #[error("found `injm end` without `injm begin` at line {line} of {}", path.display())]
    EndWithoutBegin { line: usize, path: PathBuf },

    #[error("found `injm begin` without `injm end` at line {line} of {}", path.display())]
    BeginWithoutEnd { line: usize, path: PathBuf },

    #[error("found both input and output ID: {comment}")]
    BothInputOutputMarker { comment: String },

    #[error("multiple output IDs detected: {comment}")]
    MultipleOutputMarker { comment: String },

    #[error("no files matched pattern `{pattern}`")]
    NoPatternMatch { pattern: String },

    #[error("create gitignore failed: {err}")]
    GitIgnoreError { err: ignore::Error },

    #[error("failed to parse: {content}")]
    ParseFailed { content: String },

    #[error("invalid option format: {opt}, expect `:opts...`")]
    InvalidOptionNameFormat { opt: String },

    #[error("option name is missing: {opt}")]
    MissingOptionName { opt: String },

    #[error("invalid integer option format: {opt}, expect `:opts=42`")]
    InvalidIntOptionFormat { opt: String },

    #[error("invalid option: {opt}")]
    InvalidOption { opt: String },
}
