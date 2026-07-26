use std::{fs, io::Write, path::Path, process};

use predicates::prelude::*;
use tempfile::TempDir;

fn write_file(dir: &Path, relative_path: &str, content: &str) {
    let path = dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn injm() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("injm")
}

// ----- check with --exclude -----

#[test]
fn check_exclude_skips_input_file() {
    let temp = TempDir::new().unwrap();
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

    injm()
        .arg("check")
        .arg(temp.path().join("keep.rs"))
        .arg(temp.path().join("skip.rs"))
        .arg(temp.path().join("out.rs"))
        .arg("--exclude")
        .arg(temp.path().join("skip.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_skips_output_file() {
    let temp = TempDir::new().unwrap();
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
        "stale.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("synced.rs"))
        .arg(temp.path().join("stale.rs"))
        .arg("--exclude")
        .arg(temp.path().join("stale.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_multiple_flags() {
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
    write_file(
        temp.path(),
        "skip1.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );
    write_file(
        temp.path(),
        "skip2.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("out.rs"))
        .arg(temp.path().join("skip1.rs"))
        .arg(temp.path().join("skip2.rs"))
        .arg("--exclude")
        .arg(temp.path().join("skip1.rs"))
        .arg("--exclude")
        .arg(temp.path().join("skip2.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_glob_pattern() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >shared\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "tmp/stale.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("output.rs"))
        .arg(temp.path().join("tmp/stale.rs"))
        .arg("--exclude")
        .arg(temp.path().join("tmp/*.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_no_match_ok() {
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
        .arg("--exclude")
        .arg(temp.path().join("*.nonexistent"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_short_flag() {
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
    write_file(
        temp.path(),
        "skip.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("out.rs"))
        .arg(temp.path().join("skip.rs"))
        .arg("-e")
        .arg(temp.path().join("skip.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_exclude_merged_with_config() {
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
    write_file(
        temp.path(),
        "cfg_skip.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );
    write_file(
        temp.path(),
        "cli_skip.rs",
        "// injm begin >shared\nstale\n// injm end\n",
    );
    // config excludes cfg_skip.rs; CLI excludes cli_skip.rs
    write_file(
        temp.path(),
        "injm.toml",
        &format!(
            r#"input = ["{root}/input.rs"]
output = ["{root}/out.rs", "{root}/cfg_skip.rs", "{root}/cli_skip.rs"]
exclude = ["{root}/cfg_skip.rs"]
"#,
            root = temp.path().display().to_string().replace('\\', "\\\\")
        ),
    );

    injm()
        .arg("check")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .arg("--exclude")
        .arg(temp.path().join("cli_skip.rs"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ----- inject with --exclude -----

#[test]
fn inject_exclude_skips_output_files() {
    let temp = TempDir::new().unwrap();
    let out_a = temp.path().join("out/a.rs");
    let out_b = temp.path().join("out/b.rs");
    write_file(temp.path(), "out/a.rs", "// injm begin\n// injm end\n");
    write_file(temp.path(), "out/b.rs", "// injm begin\n// injm end\n");

    let mut cmd = process::Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("inject")
        .arg("--output")
        .arg(&out_a)
        .arg(&out_b)
        .arg("--exclude")
        .arg(&out_b)
        .stdin(process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new content")
        .unwrap();
    assert!(child.wait().unwrap().success());

    assert!(fs::read_to_string(&out_a).unwrap().contains("new content"));
    // out_b was excluded — should keep original empty content between markers
    assert!(!fs::read_to_string(&out_b).unwrap().contains("new content"));
}

#[test]
fn inject_exclude_skips_input_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "src/keep.rs",
        "// injm begin <msg\nfrom keep\n// injm end\n",
    );
    write_file(
        temp.path(),
        "src/skip.rs",
        "// injm begin <msg\nfrom skip\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");

    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("src/keep.rs"))
        .arg(temp.path().join("src/skip.rs"))
        .arg("--output")
        .arg(temp.path().join("out.rs"))
        .arg("--exclude")
        .arg(temp.path().join("src/skip.rs"))
        .assert()
        .success();

    let result = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(result.contains("from keep"));
    assert!(!result.contains("from skip"));
}

#[test]
fn inject_exclude_with_config_merged() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nok\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out/keep.rs",
        "// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out/skip_cfg.rs",
        "// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &format!(
            r#"output = ["{root}/out/skip_cfg.rs"]
exclude = ["{root}/out/skip_cfg.rs"]
"#,
            root = temp.path().display().to_string().replace('\\', "\\\\")
        ),
    );

    // --output is required by clap; config adds more on top
    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("input.rs"))
        .arg("--output")
        .arg(temp.path().join("out/keep.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();

    assert!(
        fs::read_to_string(temp.path().join("out/keep.rs"))
            .unwrap()
            .contains("ok")
    );
    assert!(
        !fs::read_to_string(temp.path().join("out/skip_cfg.rs"))
            .unwrap()
            .contains("ok")
    );
}

// ----- list with --exclude -----

#[test]
fn list_exclude_removes_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "visible.rs",
        "// injm begin <visible\ncontent\n// injm end\n",
    );
    write_file(
        temp.path(),
        "hidden.rs",
        "// injm begin <hidden\ncontent\n// injm end\n",
    );

    injm()
        .arg("list")
        .arg(temp.path().join("visible.rs"))
        .arg(temp.path().join("hidden.rs"))
        .arg("--exclude")
        .arg(temp.path().join("hidden.rs"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("visible"))
        .stdout(predicate::str::contains("hidden").not());
}
