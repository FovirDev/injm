use std::{fs, path::Path};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn write_file(dir: &Path, relative: &str, content: &str) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn injm() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("injm")
}

// ----- list -----

#[test]
fn list_markdown_file() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "readme.md",
        "\
# Title

<!-- injm begin <greet -->
hello world
<!-- injm end -->
",
    );

    injm()
        .arg("list")
        .arg(dir.path().join("readme.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("greet"));
}

#[test]
fn list_markdown_multiple_blocks() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "doc.md",
        "\
<!-- injm begin <input_id -->
input content
<!-- injm end -->

<!-- injm begin >output_id -->
output content
<!-- injm end -->
",
    );

    injm()
        .arg("list")
        .arg(dir.path().join("doc.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("input_id"))
        .stdout(predicate::str::contains("output_id"));
}

// ----- check -----

#[test]
fn check_markdown_synchronized() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "<!-- injm begin <shared -->\nok\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >shared -->\nok\n<!-- injm end -->\n",
    );

    injm()
        .arg("check")
        .arg(dir.path().join("input.md"))
        .arg(dir.path().join("output.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_markdown_out_of_sync() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "<!-- injm begin <shared -->\nexpected\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >shared -->\nactual\n<!-- injm end -->\n",
    );

    injm()
        .arg("check")
        .arg(dir.path().join("input.md"))
        .arg(dir.path().join("output.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("out of sync"));
}

#[test]
fn check_markdown_no_markers() {
    let dir = TempDir::new().unwrap();

    write_file(dir.path(), "plain.md", "# Just a heading\n\nSome text.\n");

    injm()
        .arg("check")
        .arg(dir.path().join("plain.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_markdown_diff() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "<!-- injm begin <shared -->\nexpected\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >shared -->\nactual\n<!-- injm end -->\n",
    );

    injm()
        .arg("check")
        .arg(dir.path().join("input.md"))
        .arg(dir.path().join("output.md"))
        .arg("--diff")
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("-actual"))
        .stdout(predicate::str::contains("+expected"));
}

// ----- inject -----

#[test]
fn inject_markdown_file() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "<!-- injm begin <msg -->\nnew content\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >msg -->\n<!-- injm end -->\n",
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(dir.path().join("input.md"))
        .arg("--output")
        .arg(dir.path().join("output.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("output.md")).unwrap();
    assert!(
        content.contains("new content"),
        "output should contain injected content"
    );
}

#[test]
fn inject_markdown_dry_run() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "<!-- injm begin <msg -->\ndry content\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >msg -->\nold content\n<!-- injm end -->\n",
    );

    let output = injm()
        .arg("inject")
        .arg("--input")
        .arg(dir.path().join("input.md"))
        .arg("--output")
        .arg(dir.path().join("output.md"))
        .arg("--dry-run")
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("dry content"));

    // file should not be modified
    let content = fs::read_to_string(dir.path().join("output.md")).unwrap();
    assert!(
        !content.contains("dry content"),
        "dry run should not modify file"
    );
}

#[test]
fn inject_markdown_multiple_blocks() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "input.md",
        "\
<!-- injm begin <a -->
value a
<!-- injm end -->

<!-- injm begin <b -->
value b
<!-- injm end -->
",
    );
    write_file(
        dir.path(),
        "output.md",
        "\
<!-- injm begin >a -->
<!-- injm end -->

<!-- injm begin >b -->
<!-- injm end -->
",
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(dir.path().join("input.md"))
        .arg("--output")
        .arg(dir.path().join("output.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("output.md")).unwrap();
    assert!(content.contains("value a"));
    assert!(content.contains("value b"));
}

#[test]
fn inject_markdown_input_glob() {
    let dir = TempDir::new().unwrap();

    write_file(
        dir.path(),
        "inputs/a.md",
        "<!-- injm begin <msg -->\nglob content\n<!-- injm end -->\n",
    );
    write_file(
        dir.path(),
        "output.md",
        "<!-- injm begin >msg -->\n<!-- injm end -->\n",
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(dir.path().join("inputs/*.md"))
        .arg("--output")
        .arg(dir.path().join("output.md"))
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("output.md")).unwrap();
    assert!(content.contains("glob content"));
}
