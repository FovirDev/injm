use super::{ParserError, Result};
use std::{collections::HashMap, fs, path::Path, sync::LazyLock};

static FILENAME_LANGUAGE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (".envrc", "bash"),
        (".gitattributes", "gitattributes"),
        (".gitignore", "gitignore"),
        (".prettierrc", "json"),
        ("Cargo.lock", "toml"),
        ("flake.lock", "json"),
        ("justfile", "just"),
    ])
});

pub(crate) fn detect(path: &Path) -> Result<&'static str> {
    // Detect language from file path or extension.
    if let Some(path_str) = path.to_str()
        && let Some(lang) = tree_sitter_language_pack::detect_language(path_str)
    {
        return Ok(lang);
    }

    // Detect from shebang line.
    if let Ok(content) = fs::read_to_string(path)
        && let Some(lang) = tree_sitter_language_pack::detect_language_from_content(&content)
    {
        return Ok(lang);
    }

    if let Some(filename) = path.file_name().and_then(|name| name.to_str())
        && let Some(&lang) = FILENAME_LANGUAGE_MAP.get(filename)
    {
        return Ok(lang);
    }

    Err(ParserError::UnsupportedFileType {
        path: path.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(detect(Path::new("main.rs")).unwrap(), "rust");
        assert_eq!(detect(Path::new("main.py")).unwrap(), "python");
        assert_eq!(detect(Path::new("main.js")).unwrap(), "javascript");
        assert_eq!(detect(Path::new("main.go")).unwrap(), "go");
        assert_eq!(detect(Path::new("main.md")).unwrap(), "markdown");
    }

    #[test]
    fn test_detect_unknown_extension() {
        assert!(detect(Path::new("main.xyz")).is_err());
        assert!(detect(Path::new("noextension")).is_err());
    }

    #[test]
    fn test_detect_from_shebang() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env python3").unwrap();
        writeln!(f, "print('hello')").unwrap();
        assert_eq!(detect(f.path()).unwrap(), "python");

        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        assert_eq!(detect(f.path()).unwrap(), "bash");
    }
}
