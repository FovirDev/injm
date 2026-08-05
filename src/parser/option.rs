use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct PatternParserOption {
    pub no_gitignore: bool,
    pub cwd: PathBuf,
}
