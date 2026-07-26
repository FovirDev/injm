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
    let mut includes = pattern_set(includes, false)?;
    let exclude_files = pattern_set(excludes, true)?;

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

fn pattern_set(patterns: &[String], ignore_no_match_error: bool) -> Result<HashSet<PathBuf>> {
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

        if no_pattern_match && !ignore_no_match_error {
            return Err(ParserError::NoPatternMatch { pattern });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_set_empty() {
        let result = pattern_set(&[], false).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pattern_set_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.rs");
        std::fs::write(&path, "fn main() {}").unwrap();

        let result = pattern_set(&[path.to_string_lossy().to_string()], false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&path));
    }

    #[test]
    fn pattern_set_multiple_files() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "").unwrap();
        std::fs::write(&b, "").unwrap();

        let result = pattern_set(
            &[
                a.to_string_lossy().to_string(),
                b.to_string_lossy().to_string(),
            ],
            false,
        )
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_glob() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "").unwrap();
        std::fs::write(dir.path().join("b.rs"), "").unwrap();
        std::fs::write(dir.path().join("c.txt"), "").unwrap();

        let glob = dir.path().join("*.rs").to_string_lossy().to_string();
        let result = pattern_set(&[glob], false).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_directory_traverses_recursively() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("root.rs"), "").unwrap();
        std::fs::write(dir.path().join("sub/nested.rs"), "").unwrap();

        let result = pattern_set(&[dir.path().to_string_lossy().to_string()], false).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_directory_with_trailing_slash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.rs"), "").unwrap();

        let path_str = format!("{}/", dir.path().display());
        let result = pattern_set(&[path_str], false).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn pattern_set_no_match_errors() {
        let dir = tempfile::tempdir().unwrap();
        let pattern = dir
            .path()
            .join("*.nonexistent")
            .to_string_lossy()
            .to_string();
        let err = pattern_set(std::slice::from_ref(&pattern), false).unwrap_err();
        assert!(matches!(&err, ParserError::NoPatternMatch { pattern: p } if *p == pattern));
    }

    #[test]
    fn pattern_set_no_match_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let pattern = dir
            .path()
            .join("*.nonexistent")
            .to_string_lossy()
            .to_string();
        let result = pattern_set(&[pattern], true).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pattern_set_deduplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.rs");
        std::fs::write(&path, "").unwrap();

        let ps = path.to_string_lossy().to_string();
        let result = pattern_set(&[ps.clone(), ps], false).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn pattern_set_skips_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("f.rs"), "").unwrap();
        std::fs::write(dir.path().join("subdir/g.rs"), "").unwrap();

        let glob = dir.path().join("**/*").to_string_lossy().to_string();
        let result = pattern_set(&[glob], false).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|p| p.is_file()));
    }

    #[test]
    fn pattern_set_multiple_patterns() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("a.rs"), "").unwrap();
        std::fs::write(dir.path().join("src/b.rs"), "").unwrap();

        let p1 = dir.path().join("a.rs").to_string_lossy().to_string();
        let p2 = dir.path().join("src").to_string_lossy().to_string();
        let result = pattern_set(&[p1, p2], false).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_patterns_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rs");
        std::fs::write(&path, "fn main() {}").unwrap();

        let result = parse_patterns(&[path.to_string_lossy().to_string()], &[]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_patterns_multiple_includes() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "fn a() {}").unwrap();
        std::fs::write(&b, "fn b() {}").unwrap();

        let result = parse_patterns(
            &[
                a.to_string_lossy().to_string(),
                b.to_string_lossy().to_string(),
            ],
            &[],
        )
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_patterns_exclude_removes_matched() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("keep.rs");
        let b = dir.path().join("skip.rs");
        std::fs::write(&a, "fn keep() {}").unwrap();
        std::fs::write(&b, "fn skip() {}").unwrap();

        let result = parse_patterns(
            &[
                a.to_string_lossy().to_string(),
                b.to_string_lossy().to_string(),
            ],
            &[b.to_string_lossy().to_string()],
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path.file_stem().unwrap(), "keep");
    }

    #[test]
    fn parse_patterns_exclude_all_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.rs");
        std::fs::write(&path, "fn f() {}").unwrap();

        let ps = path.to_string_lossy().to_string();
        let result = parse_patterns(std::slice::from_ref(&ps.clone()), &[ps]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_patterns_exclude_with_glob() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "fn a() {}").unwrap();
        std::fs::write(&b, "fn b() {}").unwrap();

        let result = parse_patterns(
            &[
                a.to_string_lossy().to_string(),
                b.to_string_lossy().to_string(),
            ],
            &[dir.path().join("*.rs").to_string_lossy().to_string()],
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_patterns_include_no_match_errors() {
        let dir = tempfile::tempdir().unwrap();
        let pattern = dir
            .path()
            .join("*.nonexistent")
            .to_string_lossy()
            .to_string();
        let err = parse_patterns(&[pattern], &[]).unwrap_err();
        assert!(matches!(&err, ParserError::NoPatternMatch { .. }));
    }

    #[test]
    fn parse_patterns_exclude_no_match_ok() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("f.rs");
        std::fs::write(&f, "").unwrap();

        let bad_exclude = dir
            .path()
            .join("*.nonexistent")
            .to_string_lossy()
            .to_string();
        // exclude with no match is not an error
        let result = parse_patterns(&[f.to_string_lossy().to_string()], &[bad_exclude]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_patterns_exclude_no_match_empty_includes_errors() {
        // include still errors on no match, even when exclude also has no match
        let dir = tempfile::tempdir().unwrap();
        let bad_include = dir
            .path()
            .join("*.nonexistent")
            .to_string_lossy()
            .to_string();
        let bad_exclude = dir
            .path()
            .join("*.also_nonexistent")
            .to_string_lossy()
            .to_string();
        let err = parse_patterns(&[bad_include], &[bad_exclude]).unwrap_err();
        assert!(matches!(&err, ParserError::NoPatternMatch { .. }));
    }

    #[test]
    fn parse_patterns_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.rs");
        let err = parse_patterns(&[missing.to_string_lossy().to_string()], &[]).unwrap_err();
        // glob matches nothing → NoPatternMatch before we reach validate_file
        assert!(matches!(&err, ParserError::NoPatternMatch { .. }));
    }

    #[test]
    fn parse_patterns_binary_file() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin.rs");
        std::fs::write(&bin, [0x00, 0x01, 0x02]).unwrap();

        let err = parse_patterns(&[bin.to_string_lossy().to_string()], &[]).unwrap_err();
        assert!(matches!(&err, ParserError::Checker(_)));
    }

    #[test]
    fn parse_patterns_marker_errors_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("bad.rs");
        std::fs::write(&f, "// injm end\n").unwrap();

        let err = parse_patterns(&[f.to_string_lossy().to_string()], &[]).unwrap_err();
        assert!(matches!(&err, ParserError::EndWithoutBegin { .. }));
    }

    #[test]
    fn parse_patterns_file_without_markers() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("clean.rs");
        std::fs::write(&f, "fn main() {}\n").unwrap();

        let result = parse_patterns(&[f.to_string_lossy().to_string()], &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].blocks.is_empty());
    }

    #[test]
    fn parse_patterns_empty_exclude_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("f.rs");
        std::fs::write(&f, "fn f() {}\n").unwrap();

        let result = parse_patterns(
            &[f.to_string_lossy().to_string()],
            &[], // empty exclude
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_patterns_include_with_glob() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
        std::fs::write(dir.path().join("b.rs"), "fn b() {}").unwrap();

        let result = parse_patterns(
            &[dir.path().join("*.rs").to_string_lossy().to_string()],
            &[],
        )
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_patterns_include_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
        std::fs::write(dir.path().join("nested/b.rs"), "fn b() {}").unwrap();

        let result = parse_patterns(&[dir.path().to_string_lossy().to_string()], &[]).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_patterns_preserves_content_and_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.rs");
        std::fs::write(&path, "fn f() {}\n").unwrap();

        let result = parse_patterns(&[path.to_string_lossy().to_string()], &[]).unwrap();
        assert_eq!(result[0].path, path);
        assert_eq!(result[0].content, "fn f() {}\n");
    }

    #[test]
    fn parse_patterns_preserves_marker_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.rs");
        std::fs::write(&path, "// injm begin <x\ncontent\n// injm end\n").unwrap();

        let result = parse_patterns(&[path.to_string_lossy().to_string()], &[]).unwrap();
        assert_eq!(result[0].blocks.len(), 1);
    }
}
