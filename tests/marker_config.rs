use std::io::Write;
use std::{fs, path::Path, process::Command};

use predicates::boolean::PredicateBooleanExt;
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

fn check_cmd() -> assert_cmd::Command {
    let mut cmd = injm();
    cmd.arg("check");
    cmd
}

// ----- offset -----

#[test]
fn offset_preserves_wrapper_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :offset=1
wrapper one
wrapper two
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :offset=1
wrapper one
hello
wrapper two
// injm end
"#
    );
}

#[test]
fn offset_on_single_line_block_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :offset=1
only line
// injm end
"#,
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
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :offset=2
only line
// injm end
"#,
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
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :offset=1
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure();
}

#[test]
fn offset_larger_than_end_marker_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :offset=10
only line
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("panicked").not());
}

// ----- trim -----

#[test]
fn trim_removes_leading_and_trailing_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id

hello

// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :trim=true
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :trim=true
hello
// injm end
"#
    );
}

#[test]
fn trim_false_keeps_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id

hello

// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :trim=false
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :trim=false

hello

// injm end
"#
    );
}

#[test]
fn trim_default_keeps_blank_lines() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id

hello

// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id

hello

// injm end
"#
    );
}

#[test]
fn trim_with_stdin() {
    let temp = TempDir::new().unwrap();
    let dest = temp.path().join("dest.rs");
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin :trim=true
// injm end
"#,
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
        .write_all(
            br#"
hello
"#,
        )
        .unwrap();
    assert!(child.wait().unwrap().success());

    let result = fs::read_to_string(&dest).unwrap();
    assert_eq!(
        result,
        r#"// injm begin :trim=true
hello
// injm end
"#
    );
}

// ----- indent -----

#[test]
fn indent_rebases_to_minimum() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :indent=4
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :indent=4
    if true {
        println!("Hello world");
    }
// injm end
"#
    );
}

#[test]
fn indent_none_keeps_original() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id
        if true {
            println!("Hello world");
        }
// injm end
"#
    );
}

#[test]
fn indent_zero_dedents_to_column_zero() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :indent=0
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :indent=0
if true {
    println!("Hello world");
}
// injm end
"#
    );
}

#[test]
fn indent_increases_shallow_content() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <id
  a
    b
  c
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :indent=4
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("dest.rs")).unwrap();
    assert_eq!(
        result,
        r#"// injm begin >id :indent=4
    a
      b
    c
// injm end
"#
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
        r#"// injm begin <id :trim=true
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id
// injm end
"#,
    );

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
        r#"// injm begin <id
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :unknown=1
// injm end
"#,
    );

    inject_files(&temp.path().join("input.rs"), &temp.path().join("dest.rs"))
        .assert()
        .failure();
}

// ----- list -----

#[test]
fn list_with_config_options_runs() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :trim=true :offset=1 :indent=4
wrapper
// injm end
"#,
    );

    let mut cmd = injm();
    cmd.arg("list").arg(temp.path().join("dest.rs"));
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("id"));
}

#[test]
fn list_json_with_config_options_runs() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >id :trim=true
hello
// injm end
"#,
    );

    let mut cmd = injm();
    cmd.arg("list")
        .arg("--format")
        .arg("json")
        .arg(temp.path().join("dest.rs"));
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"id\": \"id\""));
}

// ----- check -----

#[test]
fn check_trim_considers_config() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg

hello

// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :trim=true
hello
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .success()
        .stdout(predicates::str::contains("synchronized"));
}

#[test]
fn check_trim_out_of_sync() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg

hello

// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :trim=true

hello

// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("out of sync"));
}

#[test]
fn check_offset_considers_config() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :offset=1
wrapper one
hello
wrapper two
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .success()
        .stdout(predicates::str::contains("synchronized"));
}

#[test]
fn check_offset_out_of_sync() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :offset=1
wrapper one
hello
world
wrapper two
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("out of sync"));
}

#[test]
fn check_offset_invalid_range() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :offset=1
hello
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid range"));
}

#[test]
fn check_offset_larger_than_end_marker_errors() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
hello
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :offset=10
only line
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn check_indent_considers_config() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :indent=4
    if true {
        println!("Hello world");
    }
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .success()
        .stdout(predicates::str::contains("synchronized"));
}

#[test]
fn check_indent_out_of_sync() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :indent=4
        if true {
            println!("Hello world");
        }
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("out of sync"));
}

#[test]
fn check_combined_considers_config() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <msg

        if true {
            x();
        }
// injm end
"#,
    );
    write_file(
        temp.path(),
        "dest.rs",
        r#"// injm begin >msg :offset=1 :trim=true :indent=4
wrapper one
    if true {
        x();
    }
wrapper two
// injm end
"#,
    );

    check_cmd()
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("dest.rs"))
        .assert()
        .success()
        .stdout(predicates::str::contains("synchronized"));
}
