use std::{fs, path::Path};

use predicates::prelude::*;
use tempfile::TempDir;

fn write_file(dir: &Path, relative_path: &str, content: &str) {
    let path = dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn abs_config(dir: &Path, content: &str) -> String {
    let root = dir.display().to_string().replace('\\', "\\\\");
    content.replace("{root}", &root)
}

fn injm() -> assert_cmd::Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("injm");
    cmd.arg("--no-gitignore");
    cmd
}

#[test]
fn no_config_file_errors() {
    let temp = TempDir::new().unwrap();
    injm()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("config file is missing"));
}

#[test]
fn nonexistent_config_path_errors() {
    let temp = TempDir::new().unwrap();
    injm()
        .arg("--config")
        .arg(temp.path().join("does-not-exist.toml"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("config file does not exist"));
}

#[test]
fn bad_toml_errors() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "bad.toml", "[[[invalid toml");
    injm()
        .arg("--config")
        .arg(temp.path().join("bad.toml"))
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("TOML parse error")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("expected")),
        );
}

#[test]
fn empty_input_errors() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "injm.toml", "output = [\"out.rs\"]\n");
    injm()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("input source is empty"));
}

#[test]
fn empty_input_with_explicit_config_errors() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "empty.toml", "output = [\"out.rs\"]\n");
    injm()
        .arg("--config")
        .arg(temp.path().join("empty.toml"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("input source is empty"));
}

#[test]
fn auto_discovered_config_success() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("out.rs");
    write_file(
        temp.path(),
        "src/lib.rs",
        "// injm begin <greet\n    println!(\"hi\");\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >greet\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/src/lib.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(&output).unwrap();
    assert!(result.contains("println!(\"hi\");"));
    assert!(!result.contains("injm begin >greet\n// injm end"));
}

#[test]
fn explicit_config_success() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("out.rs");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 42;\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "myconfig.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm()
        .arg("--config")
        .arg(temp.path().join("myconfig.toml"))
        .assert()
        .success();

    let result = fs::read_to_string(&output).unwrap();
    assert!(result.contains("let x = 42;"));
}

#[test]
fn multiple_inputs_all_injected() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "a.rs",
        "// injm begin <a\n// a content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "b.rs",
        "// injm begin <b\n// b content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >a\n// injm end\n// injm begin >b\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/a.rs", "{root}/b.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(result.contains("a content"));
    assert!(result.contains("b content"));
}

#[test]
fn glob_input_resolved() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "src/greet.rs",
        "// injm begin <greet\n// hello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "src/bye.rs",
        "// injm begin <bye\n// goodbye\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >greet\n// injm end\n// injm begin >bye\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/src/*.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(result.contains("hello"));
    assert!(result.contains("goodbye"));
}

#[test]
fn config_exclude_skips_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "keep.rs",
        "// injm begin <msg\n// kept\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip.rs",
        "// some random non-marker code\nfn skip() {}\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/keep.rs", "{root}/skip.rs"]
output = ["{root}/out.rs"]
exclude = ["{root}/skip.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(result.contains("kept"));
}

#[test]
fn cli_exclude_skips_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "keep.rs",
        "// injm begin <msg\n// kept\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip.rs",
        "// some random non-marker code\nfn skip() {}\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/keep.rs", "{root}/skip.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm()
        .arg("--exclude")
        .arg(temp.path().join("skip.rs"))
        .current_dir(temp.path())
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(result.contains("kept"));
}

#[test]
fn missing_input_id_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <missing_id\n// content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >other_id\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing").or(predicate::str::contains("not found")));
}

#[test]
fn no_matching_output_blocks_still_succeeds() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <unused\n// orphaned content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// just a normal file without markers\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert_eq!(result, "// just a normal file without markers\n");
}

#[test]
fn output_file_created_if_not_exists() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("generated.rs");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/generated.rs"]
"#,
        ),
    );
    write_file(
        temp.path(),
        "generated.rs",
        "// injm begin >msg\n// injm end\n",
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(&output).unwrap();
    assert!(result.contains("let x = 1;"));
}

#[test]
fn same_path_in_input_and_output_works() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "file.rs",
        "// injm begin <msg\n// original\n// injm end\n// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/file.rs"]
output = ["{root}/file.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("file.rs")).unwrap();
    assert!(result.contains("original"));
}

#[test]
fn nested_directories_resolved() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "src/greetings/hi.rs",
        "// injm begin <msg\n// hello from nested\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out/lib.rs",
        "// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/src/greetings/hi.rs"]
output = ["{root}/out/lib.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out/lib.rs")).unwrap();
    assert!(result.contains("hello from nested"));
}

#[test]
fn config_with_only_input_does_not_fail_on_root() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <a\n// content\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// no output markers at all\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();
}

#[test]
fn markdown_syntax_via_root() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "doc.md",
        "<!-- injm begin <msg -->\n*hello markdown*\n<!-- injm end -->\n",
    );
    write_file(
        temp.path(),
        "out.md",
        "<!-- injm begin >msg -->\n<!-- injm end -->\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/doc.md"]
output = ["{root}/out.md"]
"#,
        ),
    );

    injm().current_dir(temp.path()).assert().success();

    let result = fs::read_to_string(temp.path().join("out.md")).unwrap();
    assert!(result.contains("hello markdown"));
}

#[test]
fn dry_run_prints_stdout_does_not_modify() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    let original = fs::read_to_string(temp.path().join("out.rs")).unwrap();

    injm()
        .arg("--dry-run")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 1;"));

    let after = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert_eq!(original, after);
}

#[test]
fn diff_shows_diff_does_not_modify() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    let original = fs::read_to_string(temp.path().join("out.rs")).unwrap();

    injm()
        .arg("--dry-run")
        .arg("--diff")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::starts_with("--- ").and(predicate::str::contains("+    let x = 1")),
        );

    let after = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert_eq!(original, after);
}

#[test]
fn dry_run_without_diff_does_not_print_diff_prefix() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    injm()
        .arg("--dry-run")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("// "));
}

#[test]
fn dry_run_with_multiple_outputs_shows_path_headers() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(temp.path(), "a.rs", "// injm begin >msg\n// injm end\n");
    write_file(temp.path(), "b.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/a.rs", "{root}/b.rs"]
"#,
        ),
    );

    let out_a = fs::read_to_string(temp.path().join("a.rs")).unwrap();
    let out_b = fs::read_to_string(temp.path().join("b.rs")).unwrap();

    injm()
        .arg("--dry-run")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("==== ")
                .and(predicate::str::contains("a.rs"))
                .and(predicate::str::contains("b.rs"))
                .and(predicate::str::contains("let x = 1")),
        );

    assert_eq!(fs::read_to_string(temp.path().join("a.rs")).unwrap(), out_a);
    assert_eq!(fs::read_to_string(temp.path().join("b.rs")).unwrap(), out_b);
}

#[test]
fn diff_without_flag_does_not_print_diff() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\n    let x = 1;\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs"]
"#,
        ),
    );

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();

    // normal run (no --dry-run, no --diff) writes to file, nothing on stdout
    injm()
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let after = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert_ne!(result, after);
    assert!(after.contains("let x = 1"));
}
