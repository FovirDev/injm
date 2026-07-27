use std::path::PathBuf;

#[derive(Debug, Default)]
pub(crate) struct PatternParserOption {
    pub no_gitignore: bool,
    pub cwd: PathBuf,
}
