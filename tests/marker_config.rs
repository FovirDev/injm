use std::io::Write;
use std::{fs, path::Path, process::Command};

use predicates::prelude::predicate;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn write_file(dir: &Path, relative_path: &str, content: &str) {
    let path = dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn injm() -> assert_cmd::Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("injm");
    cmd.arg("--no-gitignore");
    cmd
}

fn inject_files(input: &Path, output: &Path) -> assert_cmd::Command {
    let mut cmd = injm();
    cmd.arg("inject")
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output);
    cmd
}

// ----- offset -----

#[test]
fn offset_preserves_wrapper_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\nhello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :offset=1\nwrapper one\nwrapper two\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id :offset=1\nwrapper one\nhello\nwrapper two\n// injm end\n"
    );
}

#[test]
fn offset_on_single_line_block_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\nhello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :offset=1\nonly line\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid range: (2, 2)"));
}

#[test]
fn offset_exceeds_block_lines_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\nhello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :offset=2\nonly line\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid range: (3, 1)"));
}

#[test]
fn offset_on_empty_block_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\nhello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :offset=1\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure();
}

// ----- trim -----

#[test]
fn trim_removes_leading_and_trailing_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n\nhello\n\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :trim=true\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(result, "// injm begin >id :trim=true\nhello\n// injm end\n");
}

#[test]
fn trim_false_keeps_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n\nhello\n\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :trim=false\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id :trim=false\n\nhello\n\n// injm end\n"
    );
}

#[test]
fn trim_default_keeps_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n\nhello\n\n// injm end\n",
    );
    write_file(temp.path(), "dest.rs", "// injm begin >id\n// injm end\n");

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(result, "// injm begin >id\n\nhello\n\n// injm end\n");
}

#[test]
fn trim_with_stdin() {
    let temp = TempDir::new().unwrap();
    let dest = temp.path().join("dest.rs");
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin :trim=true\n// injm end\n",
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_injm"))
        .arg("inject")
        .arg("--no-gitignore")
        .arg("--output")
        .arg(&dest)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"\nhello\n")
        .unwrap();
    assert!(child.wait().unwrap().success());

    let result = fs::read_to_string(&dest).unwrap();
    assert_eq!(result, "// injm begin :trim=true\nhello\n// injm end\n");
}

// ----- indent -----

#[test]
fn indent_rebases_to_minimum() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n        if true {\n            println!(\"Hello world\");\n        }\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :indent=4\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id :indent=4\n    if true {\n        println!(\"Hello world\");\n    }\n// injm end\n"
    );
}

#[test]
fn indent_none_keeps_original() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n        if true {\n            println!(\"Hello world\");\n        }\n// injm end\n",
    );
    write_file(temp.path(), "dest.rs", "// injm begin >id\n// injm end\n");

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id\n        if true {\n            println!(\"Hello world\");\n        }\n// injm end\n"
    );
}

#[test]
fn indent_zero_dedents_to_column_zero() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n        if true {\n            println!(\"Hello world\");\n        }\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :indent=0\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id :indent=0\nif true {\n    println!(\"Hello world\");\n}\n// injm end\n"
    );
}

#[test]
fn indent_increases_shallow_content() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\n  a\n    b\n  c\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :indent=4\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        "// injm begin >id :indent=4\n    a\n      b\n    c\n// injm end\n"
    );
}

// ----- combined -----

#[test]
fn offset_trim_indent_combined() {
    let temp = TempDir::new().unwrap();

    let input = r#"
    // injm begin <id

        if true {
            x();
        }
    // injm end
"#;

    let dest = r#"
    // injm begin >id :offset=1 :trim=true :indent=4
    wrapper

    // injm end
"#;

    write_file(temp.path(), "input.rs", input);
    write_file(temp.path(), "dest.rs", dest);

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();

    let expected = r#"
    // injm begin >id :offset=1 :trim=true :indent=4
    wrapper
    if true {
        x();
    }

    // injm end
"#;

    assert_eq!(result, expected);
}

#[test]
fn options_on_input_marker_do_not_break() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id :trim=true\nhello\n// injm end\n",
    );
    write_file(temp.path(), "dest.rs", "// injm begin >id\n// injm end\n");

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert!(result.contains("hello"));
}

// ----- errors -----

#[test]
fn invalid_option_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <id\nhello\n// injm end\n",
    );
    write_file(
        temp.path(),
        "dest.rs",
        "// injm begin >id :unknown=1\n// injm end\n",
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure();
}
