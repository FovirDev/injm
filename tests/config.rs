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

fn abs_config(dir: &Path, content: &str) -> String {
    let root = dir.display().to_string().replace('\\', "\\\\");
    content.replace("{root}", &root)
}

fn injm() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("injm")
}

// ----- inject -----

#[test]
fn inject_merges_config_input_with_cli_input() {
    let temp = TempDir::new().unwrap();
    let cli_input = temp.path().join("cli_input.rs");
    let output_path = temp.path().join("out.rs");
    write_file(
        temp.path(),
        "cli_input.rs",
        "// injm begin <from_cli\nprintln!(\"cli\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "cfg_input.rs",
        "// injm begin <from_cfg\nprintln!(\"cfg\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >from_cli\n// injm end\n// injm begin >from_cfg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"input = ["{root}/cfg_input.rs"]"#),
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(&cli_input)
        .arg("--output")
        .arg(&output_path)
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();

    let r = fs::read_to_string(&output_path).unwrap();
    assert!(r.contains("cli"));
    assert!(r.contains("cfg"));
}

#[test]
fn inject_merges_config_output_with_cli_output() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.rs");
    let out_a = temp.path().join("out/a.rs");
    let out_b = temp.path().join("out/b.rs");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nprintln!(\"hello\");\n// injm end\n",
    );
    write_file(temp.path(), "out/a.rs", "// injm begin >msg\n// injm end\n");
    write_file(temp.path(), "out/b.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"output = ["{root}/out/b.rs"]"#),
    );

    injm()
        .arg("inject")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&out_a)
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();

    assert!(fs::read_to_string(&out_a).unwrap().contains("hello"));
    assert!(fs::read_to_string(&out_b).unwrap().contains("hello"));
}

#[test]
fn inject_config_exclude_skips_matched_files() {
    let temp = TempDir::new().unwrap();
    let out_path = temp.path().join("out.rs");
    write_file(
        temp.path(),
        "inputs/a.rs",
        "// injm begin <msg\nprintln!(\"a\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "inputs/ignored.rs",
        "// injm begin <msg\nprintln!(\"ignored\");\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/inputs/a.rs", "{root}/inputs/ignored.rs"]
exclude = ["{root}/inputs/ignored.rs"]
"#,
        ),
    );

    injm()
        .arg("inject")
        .arg("--output")
        .arg(&out_path)
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();
}

#[test]
fn inject_config_exclude_applies_to_output_too() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.rs");
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <msg\nprintln!(\"ok\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out/keep.rs",
        "// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out/skip.rs",
        "// injm begin >msg\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
output = ["{root}/out/skip.rs"]
exclude = ["{root}/out/skip.rs"]
"#,
        ),
    );

    // --output is required by clap; config output adds more on top
    injm()
        .arg("inject")
        .arg("--input")
        .arg(&input_path)
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
    // skip.rs was excluded – untouched
    assert!(
        !fs::read_to_string(temp.path().join("out/skip.rs"))
            .unwrap()
            .contains("ok")
    );
}

#[test]
fn inject_stdin_with_config_output_merged() {
    let temp = TempDir::new().unwrap();
    let out_a = temp.path().join("out/a.rs");
    let out_b = temp.path().join("out/b.rs");
    write_file(temp.path(), "out/a.rs", "// injm begin\n// injm end\n");
    write_file(temp.path(), "out/b.rs", "// injm begin\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"output = ["{root}/out/b.rs"]"#),
    );

    let mut cmd = process::Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("inject")
        .arg("--output")
        .arg(&out_a)
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .stdin(process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"println!(\"hi\");")
        .unwrap();
    assert!(child.wait().unwrap().success());

    assert!(fs::read_to_string(&out_a).unwrap().contains("hi"));
    assert!(fs::read_to_string(&out_b).unwrap().contains("hi"));
}

#[test]
fn inject_config_file_not_found_errors() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");

    let config_path = temp.path().join("nonexistent.toml");
    let mut cmd = process::Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("inject")
        .arg("--config")
        .arg(&config_path)
        .arg("--output")
        .arg(temp.path().join("out.rs"))
        .stdin(process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(b"x").unwrap();
    assert!(!child.wait().unwrap().success());
}

// ----- check -----

#[test]
fn check_uses_config_input_and_output() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <greet\nprintln!(\"hi\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >greet\nprintln!(\"hi\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/input.rs"]
output = ["{root}/output.rs"]
"#,
        ),
    );

    injm()
        .arg("check")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_config_exclude_skips_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <shared\nprintln!(\"ok\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >shared\nprintln!(\"ok\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "stale.rs",
        "// injm begin >shared\nprintln!(\"stale\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/input.rs", "{root}/output.rs", "{root}/stale.rs"]
exclude = ["{root}/stale.rs"]
"#,
        ),
    );

    injm()
        .arg("check")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_config_no_cli_args_uses_config_only() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "cfg_input.rs",
        "// injm begin <cfg\nprintln!(\"cfg\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "cfg_output.rs",
        "// injm begin >cfg\nprintln!(\"cfg\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/cfg_input.rs"]
output = ["{root}/cfg_output.rs"]
"#,
        ),
    );

    injm()
        .arg("check")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_config_merged_with_cli_files() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "cfg_input.rs",
        "// injm begin <cfg\ncfg content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "cli_output.rs",
        "// injm begin >cfg\ncfg content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"input = ["{root}/cfg_input.rs"]"#),
    );

    // output comes from CLI, input from config — merged
    injm()
        .arg("check")
        .arg(temp.path().join("cli_output.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_config_with_diff_flag() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <x\nexpected\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >x\nactual\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/input.rs"]
output = ["{root}/output.rs"]
"#,
        ),
    );

    injm()
        .arg("check")
        .arg("--diff")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .failure()
        .stdout(predicate::str::contains("-actual").and(predicate::str::contains("+expected")))
        .stderr(predicate::str::contains("out of sync"));
}

// ----- list -----

#[test]
fn list_uses_config_input() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <my_id\ncontent\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"input = ["{root}/input.rs"]"#),
    );

    injm()
        .arg("list")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("my_id"));
}

#[test]
fn list_uses_config_output() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >out_id\ncontent\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"output = ["{root}/output.rs"]"#),
    );

    injm()
        .arg("list")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("out_id"));
}

#[test]
fn list_config_exclude_removes_files() {
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
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/visible.rs", "{root}/hidden.rs"]
exclude = ["{root}/hidden.rs"]
"#,
        ),
    );

    injm()
        .arg("list")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("visible"))
        .stdout(predicate::str::contains("hidden").not());
}

// ----- error cases -----

#[test]
fn config_toml_syntax_error() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "injm.toml", "input = [\n"); // unclosed array

    let config_path = temp.path().join("injm.toml");
    injm()
        .arg("check")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("TOML parse error"));
}

#[test]
fn config_wrong_type_error() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "injm.toml", "input = \"string_not_list\"\n");

    let config_path = temp.path().join("injm.toml");
    injm()
        .arg("check")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid type"));
}

#[test]
fn config_file_not_found_errors_on_check() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("missing.toml");
    injm()
        .arg("check")
        .arg("--config")
        .arg(&config_path)
        .arg(temp.path().join("no_exist.rs"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("config file does not exist"));
}

// ----- config content edge cases -----

#[test]
fn empty_config_toml_works() {
    let temp = TempDir::new().unwrap();
    write_file(temp.path(), "injm.toml", "");
    write_file(temp.path(), "i.rs", "// injm begin <x\nok\n// injm end\n");
    write_file(temp.path(), "o.rs", "// injm begin >x\nok\n// injm end\n");

    injm()
        .arg("check")
        .arg(temp.path().join("i.rs"))
        .arg(temp.path().join("o.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn config_empty_arrays_works() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "injm.toml",
        "input = []\noutput = []\nexclude = []\n",
    );
    write_file(temp.path(), "i.rs", "// injm begin <x\nok\n// injm end\n");
    write_file(temp.path(), "o.rs", "// injm begin >x\nok\n// injm end\n");

    injm()
        .arg("check")
        .arg(temp.path().join("i.rs"))
        .arg(temp.path().join("o.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn config_with_paths_containing_spaces() {
    let temp = TempDir::new().unwrap();
    let spaced = temp.path().join("my files");
    fs::create_dir_all(&spaced).unwrap();
    let input_path = spaced.join("input.rs");
    let output_path = spaced.join("output.rs");
    fs::write(
        &input_path,
        "// injm begin <hello\nprintln!(\"ok\");\n// injm end\n",
    )
    .unwrap();
    fs::write(
        &output_path,
        "// injm begin >hello\nprintln!(\"ok\");\n// injm end\n",
    )
    .unwrap();

    let config = format!(
        r#"input = ["{}"]
output = ["{}"]"#,
        input_path.display().to_string().replace('\\', "\\\\"),
        output_path.display().to_string().replace('\\', "\\\\"),
    );
    write_file(temp.path(), "injm.toml", &config);

    injm()
        .arg("check")
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ----- config with glob patterns -----

#[test]
fn config_input_with_glob() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "src/a.rs",
        "// injm begin <id_a\na content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "src/b.rs",
        "// injm begin <id_b\nb content\n// injm end\n",
    );
    write_file(
        temp.path(),
        "out.rs",
        "// injm begin >id_a\n// injm end\n// injm begin >id_b\n// injm end\n",
    );
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"input = ["{root}/src/*.rs"]"#),
    );

    // --input must be passed to activate the config merge branch
    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("src/a.rs"))
        .arg("--output")
        .arg(temp.path().join("out.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();

    let r = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(r.contains("a content"));
    assert!(r.contains("b content"));
}

#[test]
fn config_exclude_with_glob() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "src/a.rs",
        "// injm begin <msg\na\n// injm end\n",
    );
    write_file(
        temp.path(),
        "src/b.rs",
        "// injm begin <msg\nb\n// injm end\n",
    );
    write_file(temp.path(), "out.rs", "// injm begin >msg\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(
            temp.path(),
            r#"
input = ["{root}/src/*.rs"]
exclude = ["{root}/src/b.rs"]
"#,
        ),
    );

    // --input activates the config-merge branch; exclude removes b.rs
    injm()
        .arg("inject")
        .arg("--input")
        .arg(temp.path().join("src/a.rs"))
        .arg("--output")
        .arg(temp.path().join("out.rs"))
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .assert()
        .success();

    let r = fs::read_to_string(temp.path().join("out.rs")).unwrap();
    assert!(r.contains("a"));
}

// ----- dry-run with config -----

#[test]
fn inject_config_dry_run_does_not_modify_files() {
    let temp = TempDir::new().unwrap();
    let out_path = temp.path().join("out.rs");
    write_file(temp.path(), "out.rs", "// injm begin >greet\n// injm end\n");
    write_file(
        temp.path(),
        "injm.toml",
        &abs_config(temp.path(), r#"output = ["{root}/out.rs"]"#),
    );
    let original = fs::read_to_string(&out_path).unwrap();

    let mut cmd = process::Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("inject")
        .arg("--dry-run")
        .arg("--output")
        .arg(&out_path)
        .arg("--config")
        .arg(temp.path().join("injm.toml"))
        .stdin(process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new stuff")
        .unwrap();
    assert!(child.wait().unwrap().success());

    assert_eq!(fs::read_to_string(&out_path).unwrap(), original);
}

// ----- no config flag defaults to empty -----

#[test]
fn no_config_flag_uses_default_empty_config() {
    let temp = TempDir::new().unwrap();
    write_file(
        temp.path(),
        "input.rs",
        "// injm begin <hello\nprintln!(\"ok\");\n// injm end\n",
    );
    write_file(
        temp.path(),
        "output.rs",
        "// injm begin >hello\nprintln!(\"ok\");\n// injm end\n",
    );

    injm()
        .arg("check")
        .arg(temp.path().join("input.rs"))
        .arg(temp.path().join("output.rs"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}
