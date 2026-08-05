use glob::Pattern;
use ignore::gitignore::Gitignore;

use super::{ParserError, Result};
use crate::{
    parser::{PatternParserOption, detector::detect, marker::extract_marker_blocks},
    types::ParsedFile,
    validator::validate_file,
};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

const IGNORED_DIRS: &[&str] = &[".git"];
const IGNORED_FILES: &[&str] = &["LICENSE"];

pub fn parse_patterns(
    includes: &[String],
    excludes: &[String],
    opts: &PatternParserOption,
) -> Result<Vec<ParsedFile>> {
    let mut files: Vec<ParsedFile> = Vec::new();
    let includes = pattern_set(includes, excludes, opts)?;

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

fn pattern_set(
    patterns: &[String],
    excludes: &[String],
    opts: &PatternParserOption,
) -> Result<HashSet<PathBuf>> {
    let mut result: HashSet<PathBuf> = HashSet::new();
    let exclude_patterns: Vec<Pattern> = excludes
        .iter()
        .map(|e| Pattern::new(e))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let gitignore = if opts.no_gitignore {
        None
    } else {
        load_gitignore(&opts.cwd)?
    };

    for pattern in patterns {
        let input_path = Path::new(pattern);
        let expanded_pattern = if input_path.is_dir() {
            format!("{}/**/*", pattern.trim_end_matches(['/', '\\']))
        } else {
            pattern.clone()
        };

        let entries: Vec<PathBuf> = if input_path.is_dir() {
            expand_directory(input_path)?
        } else if let Some(root) = recursive_glob_root(pattern) {
            if root.is_dir() {
                expand_directory(root)?
            } else {
                Vec::new()
            }
        } else {
            glob::glob(&expanded_pattern)?.collect::<std::result::Result<Vec<_>, _>>()?
        };

        let no_pattern_match = entries.is_empty();
        'outer: for path in entries {
            if let Some(ref g) = gitignore
                && g.matched_path_or_any_parents(&path, path.is_dir())
                    .is_ignore()
            {
                continue;
            }

            if path.is_dir() || is_ignored_file(&path) {
                continue;
            }

            for exclude in &exclude_patterns {
                if exclude.matches_path(&path) {
                    continue 'outer;
                }
            }

            result.insert(path);
        }

        if no_pattern_match {
            return Err(ParserError::NoPatternMatch {
                pattern: expanded_pattern,
            });
        }
    }

    Ok(result)
}

fn load_gitignore(cwd: &Path) -> Result<Option<Gitignore>> {
    let path = cwd.join(".gitignore");
    if path.exists() {
        let (ignore, err) = Gitignore::new(path);
        if let Some(err) = err {
            return Err(ParserError::GitIgnoreError { err });
        }

        Ok(Some(ignore))
    } else {
        Ok(None)
    }
}

fn expand_directory(root: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    collect_directory_entries(root, &mut entries)?;
    Ok(entries)
}

fn collect_directory_entries(dir: &Path, entries: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if is_ignored_dir(&path) {
                continue;
            }

            entries.push(path.clone());
            collect_directory_entries(&path, entries)?;
        } else {
            entries.push(path);
        }
    }

    Ok(())
}

fn is_ignored_dir(path: &Path) -> bool {
    path.file_name().is_some_and(|file_name| {
        IGNORED_DIRS
            .iter()
            .any(|&ignored| file_name == OsStr::new(ignored))
    })
}

fn is_ignored_file(path: &Path) -> bool {
    path.file_name().is_some_and(|file_name| {
        IGNORED_FILES
            .iter()
            .any(|&ignored| file_name == OsStr::new(ignored))
    })
}

fn recursive_glob_root(pattern: &str) -> Option<&Path> {
    pattern
        .strip_suffix("/**/*")
        .or_else(|| pattern.strip_suffix(r"\**\*"))
        .map(Path::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts_with_cwd(cwd: &Path) -> PatternParserOption {
        PatternParserOption {
            no_gitignore: false,
            cwd: cwd.to_path_buf(),
        }
    }

    #[test]
    fn pattern_set_empty() {
        let result =
            pattern_set(&[], &[], &opts_with_cwd(&std::env::current_dir().unwrap())).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pattern_set_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.rs");
        std::fs::write(&path, "fn main() {}").unwrap();

        let result = pattern_set(
            &[path.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
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
            &[],
            &opts_with_cwd(dir.path()),
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
        let result = pattern_set(&[glob], &[], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_directory_traverses_recursively() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("root.rs"), "").unwrap();
        std::fs::write(dir.path().join("sub/nested.rs"), "").unwrap();

        let result = pattern_set(
            &[dir.path().to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_directory_with_trailing_slash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.rs"), "").unwrap();

        let path_str = format!("{}/", dir.path().display());
        let result = pattern_set(&[path_str], &[], &opts_with_cwd(dir.path())).unwrap();
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
        let err = pattern_set(
            std::slice::from_ref(&pattern),
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap_err();
        assert!(matches!(&err, ParserError::NoPatternMatch { pattern: p } if *p == pattern));
    }

    #[test]
    fn pattern_set_deduplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.rs");
        std::fs::write(&path, "").unwrap();

        let ps = path.to_string_lossy().to_string();
        let result = pattern_set(&[ps.clone(), ps], &[], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn pattern_set_skips_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("f.rs"), "").unwrap();
        std::fs::write(dir.path().join("subdir/g.rs"), "").unwrap();

        let glob = dir.path().join("**/*").to_string_lossy().to_string();
        let result = pattern_set(&[glob], &[], &opts_with_cwd(dir.path())).unwrap();
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
        let result = pattern_set(&[p1, p2], &[], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn pattern_set_exclude_matches_by_path() {
        let dir = tempfile::tempdir().unwrap();
        let kept = dir.path().join("keep.rs");
        let skipped = dir.path().join("skip.rs");
        std::fs::write(&kept, "").unwrap();
        std::fs::write(&skipped, "").unwrap();

        // exclude pattern checks full path via glob::Pattern::matches_path
        let exclude_glob = dir.path().join("skip.rs").to_string_lossy().to_string();
        let include_glob = dir.path().join("*.rs").to_string_lossy().to_string();
        let result =
            pattern_set(&[include_glob], &[exclude_glob], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&kept));
    }

    #[test]
    fn pattern_set_exclude_with_wildcard() {
        let dir = tempfile::tempdir().unwrap();
        let kept = dir.path().join("keep.md");
        let skipped = dir.path().join("skip.rs");
        std::fs::write(&kept, "").unwrap();
        std::fs::write(&skipped, "").unwrap();

        let exclude_glob = dir.path().join("*.rs").to_string_lossy().to_string();
        let include_glob = dir.path().join("*.*").to_string_lossy().to_string();
        let result =
            pattern_set(&[include_glob], &[exclude_glob], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&kept));
    }

    #[test]
    fn pattern_set_exclude_no_match_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("f.rs");
        std::fs::write(&f, "").unwrap();

        let result = pattern_set(
            &[f.to_string_lossy().to_string()],
            &["*.nonexistent".to_string()],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn pattern_set_exclude_multiple_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let keep_a = dir.path().join("a.rs");
        let skip_b = dir.path().join("b.rs");
        let skip_c = dir.path().join("c.rs");
        std::fs::write(&keep_a, "").unwrap();
        std::fs::write(&skip_b, "").unwrap();
        std::fs::write(&skip_c, "").unwrap();

        let include = dir.path().join("*.rs").to_string_lossy().to_string();
        let x1 = dir.path().join("b.rs").to_string_lossy().to_string();
        let x2 = dir.path().join("c.rs").to_string_lossy().to_string();
        let result = pattern_set(&[include], &[x1, x2], &opts_with_cwd(dir.path())).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&keep_a));
    }

    #[test]
    fn pattern_set_skips_git_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git/objects")).unwrap();
        std::fs::write(dir.path().join(".git/config"), "[core]\n").unwrap();
        std::fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(dir.path().join("keep.rs"), "").unwrap();
        std::fs::write(dir.path().join("LICENSE"), "").unwrap();

        let glob = dir.path().join("**/*").to_string_lossy().to_string();
        let result = pattern_set(&[glob], &[], &opts_with_cwd(dir.path())).unwrap();
        dbg!(&result);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&dir.path().join("keep.rs")));
    }

    #[test]
    fn pattern_set_gitignore_filters_path() {
        let dir = tempfile::tempdir().unwrap();
        let kept = dir.path().join("keep.rs");
        let skipped = dir.path().join("skip.rs");
        std::fs::write(&kept, "").unwrap();
        std::fs::write(&skipped, "").unwrap();
        std::fs::write(dir.path().join(".gitignore"), "skip.rs\n").unwrap();

        let g = load_gitignore(dir.path()).unwrap().unwrap();
        assert!(g.matched(&skipped, false).is_ignore());
        assert!(!g.matched(&kept, false).is_ignore());
    }

    #[test]
    fn parse_patterns_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rs");
        std::fs::write(&path, "fn main() {}").unwrap();

        let result = parse_patterns(
            &[path.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
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
            &opts_with_cwd(dir.path()),
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
            &opts_with_cwd(dir.path()),
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
        let result = parse_patterns(
            std::slice::from_ref(&ps.clone()),
            &[ps],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
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
            &opts_with_cwd(dir.path()),
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
        let err = parse_patterns(&[pattern], &[], &opts_with_cwd(dir.path())).unwrap_err();
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
        let result = parse_patterns(
            &[f.to_string_lossy().to_string()],
            &[bad_exclude],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
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
        let err =
            parse_patterns(&[bad_include], &[bad_exclude], &opts_with_cwd(dir.path())).unwrap_err();
        assert!(matches!(&err, ParserError::NoPatternMatch { .. }));
    }

    #[test]
    fn parse_patterns_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.rs");
        let err = parse_patterns(
            &[missing.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap_err();
        // glob matches nothing → NoPatternMatch before we reach validate_file
        assert!(matches!(&err, ParserError::NoPatternMatch { .. }));
    }

    #[test]
    fn parse_patterns_binary_file() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin.rs");
        std::fs::write(&bin, [0x00, 0x01, 0x02]).unwrap();

        let err = parse_patterns(
            &[bin.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap_err();
        assert!(matches!(&err, ParserError::Checker(_)));
    }

    #[test]
    fn parse_patterns_marker_errors_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("bad.rs");
        std::fs::write(&f, "// injm end\n").unwrap();

        let err = parse_patterns(
            &[f.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap_err();
        assert!(matches!(&err, ParserError::EndWithoutBegin { .. }));
    }

    #[test]
    fn parse_patterns_file_without_markers() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("clean.rs");
        std::fs::write(&f, "fn main() {}\n").unwrap();

        let result = parse_patterns(
            &[f.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
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
            &opts_with_cwd(dir.path()),
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
            &opts_with_cwd(dir.path()),
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

        let result = parse_patterns(
            &[dir.path().to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_patterns_preserves_content_and_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.rs");
        std::fs::write(&path, "fn f() {}\n").unwrap();

        let result = parse_patterns(
            &[path.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
        assert_eq!(result[0].path, path);
        assert_eq!(result[0].content, "fn f() {}\n");
    }

    #[test]
    fn parse_patterns_preserves_marker_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.rs");
        std::fs::write(&path, "// injm begin <x\ncontent\n// injm end\n").unwrap();

        let result = parse_patterns(
            &[path.to_string_lossy().to_string()],
            &[],
            &opts_with_cwd(dir.path()),
        )
        .unwrap();
        assert_eq!(result[0].blocks.len(), 1);
    }
}
