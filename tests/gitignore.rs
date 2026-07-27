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

// ----- check with .gitignore -----

#[test]
fn check_gitignore_skips_input_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "skip.rs\n");
    write_file(
        temp.path(),
        "keep.rs",
        "// injm begin <shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip.rs",
        "// injm begin <shared\nstale\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >shared\nok\n// injm end\n",
    );

    // skip.rs is gitignored → no duplicate input id error
    injm()
        .arg("check")
        .arg(temp.path().join("keep.rs"))
        .arg(temp.path().join("skip.rs"))
        .arg(temp.path().join("out.rs"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_gitignore_skips_output_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "skip.rs\n");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "synced.rs",
        "// injm begin >shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("synced.rs"))
        .arg(temp.path().join("skip.rs"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_gitignore_with_negation() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "*.rs\n!important.rs\n");
    write_file(
        temp.path(),
        "common.rs",
        "// injm begin <shared\ncommon\n// injm end\n",
    );
    write_file(
        temp.path(),
        "important.rs",
        "// injm begin <shared\nimportant\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.txt",
        "// injm begin >shared\nimportant\n// injm end\n",
    );

    // *.rs ignores common.rs; !important.rs re-includes it
    injm()
        .arg("check")
        .arg(temp.path().join("common.rs"))
        .arg(temp.path().join("important.rs"))
        .arg(temp.path().join("output.txt"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_no_gitignore_flag_includes_ignored_files() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "input_dup.rs\n");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <shared\nfrom input\n// injm end\n",
    );
    write_file(
        temp.path(),
        "input_dup.rs",
        "// injm begin <shared\nfrom duplicate\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >shared\nfrom input\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("input_dup.rs"))
        .arg(temp.path().join("out.rs"))
        .arg("--no-gitignore")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicated input id"));
}

#[test]
fn check_no_gitignore_file_is_fine() {
    let temp = TempDir::new().unwrap();

    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >shared\nok\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("out.rs"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ----- inject with .gitignore -----

#[test]
fn inject_gitignore_skips_input_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "skip.rs\n");
    write_file(
        temp.path(),
        "keep.rs",
        "// injm begin <msg\nfrom keep\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip.rs",
        "// injm begin <msg\nfrom skip\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");

    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("keep.rs"))
        .arg(temp.path().join("skip.rs"))
        .arg("--output")
        .arg(temp.path().join("out.rs"))
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(content.contains("from keep"));
    assert!(!content.contains("from skip"));
}

#[test]
fn inject_gitignore_skips_output_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "out_b.rs\n");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nnew_value\n// injm end\n",
    );
    write_file(temp.path(), "out_a.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "out_b.rs",
        "// injm begin >msg\noriginal_value\n// injm end\n",
    );

    let output = injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("input.rs"))
        .arg("--output")
        .arg(temp.path().join("out_a.rs"))
        .arg(temp.path().join("out_b.rs"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "inject failed: {:?}",
        std::str::from_utf8(&output.stderr),
    );

    assert!(
        fs::read_to_string(temp.path().join("out_a.rs"))
            .unwrap()
            .contains("new_value")
    );
    // out_b.rs was gitignored — original content preserved
    assert!(
        !fs::read_to_string(temp.path().join("out_b.rs"))
            .unwrap()
            .contains("new_value")
    );
    assert!(
        fs::read_to_string(temp.path().join("out_b.rs"))
            .unwrap()
            .contains("original_value")
    );
}

#[test]
fn inject_gitignore_skips_output_file_single() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "out_b.rs\n");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nnew_value\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out_b.rs",
        "// injm begin >msg\noriginal_value\n// injm end\n",
    );

    let output = injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("input.rs"))
        .arg("--output")
        .arg(temp.path().join("out_b.rs"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "inject failed: {:?}",
        std::str::from_utf8(&output.stderr),
    );

    // out_b.rs is gitignored — should NOT be modified
    let content = fs::read_to_string(temp.path().join("out_b.rs")).unwrap();
    assert!(
        content.contains("original_value"),
        "should preserve original"
    );
    assert!(
        !content.contains("new_value"),
        "should NOT have new content"
    );
}

#[test]
fn inject_no_gitignore_flag_includes_ignored_output() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "out_ignored.rs\n");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nnew content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out_ignored.rs",
        "// injm begin >msg\nold content\n// injm end\n",
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("input.rs"))
        .arg("--output")
        .arg(temp.path().join("out_ignored.rs"))
        .arg("--no-gitignore")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(
        fs::read_to_string(temp.path().join("out_ignored.rs"))
            .unwrap()
            .contains("new content")
    );
}

// ----- list with .gitignore -----

#[test]
fn list_gitignore_ignores_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "ignored.rs\n");
    write_file(
        temp.path(),
        "visible.rs",
        "// injm begin <visible\ncontent\n// injm end\n",
    );
    write_file(
        temp.path(),
        "ignored.rs",
        "// injm begin <hidden\ncontent\n// injm end\n",
    );

    injm()
        .arg("list")
        .arg(temp.path().join("visible.rs"))
        .arg(temp.path().join("ignored.rs"))
        .arg("--format")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("visible"))
        .stdout(predicate::str::contains("hidden").not());
}

#[test]
fn list_no_gitignore_flag_includes_ignored_file() {
    let temp = TempDir::new().unwrap();

    write_file(temp.path(), ".gitignore", "ignored.rs\n");
    write_file(
        temp.path(),
        "visible.rs",
        "// injm begin <visible\ncontent\n// injm end\n",
    );
    write_file(
        temp.path(),
        "ignored.rs",
        "// injm begin <hidden\ncontent\n// injm end\n",
    );

    injm()
        .arg("list")
        .arg(temp.path().join("visible.rs"))
        .arg(temp.path().join("ignored.rs"))
        .arg("--format")
        .arg("json")
        .arg("--no-gitignore")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("visible"))
        .stdout(predicate::str::contains("hidden"));
}
