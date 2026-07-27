use std::process::Command;

fn injm_bin_list() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("list").arg("--no-gitignore");
    cmd
}

#[test]
fn test_list_single_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(
        &path,
        "// injm begin <input_id\ncontent\n// injm end\n// injm begin >output_id\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("input_id"));
    assert!(stdout.contains("output_id"));
    assert!(stdout.contains("input "));
    assert!(stdout.contains("output"));
}

#[test]
fn test_list_json_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin <my_id\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg("--format")
        .arg("json")
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""id": "my_id""#));
    assert!(stdout.contains(r#""marker_type": "input""#));
    assert!(stdout.contains(&format!(r#""file": "{}""#, path.to_string_lossy())));
}

#[test]
fn test_list_no_markers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "fn main() {}\n").unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_list_short_flag() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin <my_id\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg("-f")
        .arg("json")
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""id": "my_id""#));
}

#[test]
fn test_list_binary_file_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, [0x00, 0x01, 0x02, 0x03]).unwrap();

    let status = injm_bin_list().arg(&path).status().unwrap();
    assert!(!status.success());
}

#[test]
fn test_list_file_not_exist_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let status = injm_bin_list()
        .arg(dir.path().join("not_exist.rs"))
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn test_list_anonymous_block_produces_no_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg(&path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_list_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.rs");
    std::fs::write(&path, "").unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_list_no_match_returns_error() {
    let dir = tempfile::tempdir().unwrap();

    let status = injm_bin_list()
        .arg(dir.path().join("*.rs"))
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_list_multiple_patterns() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.rs"),
        "// injm begin <id_a\ncontent\n// injm end\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.rs"),
        "// injm begin <id_b\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(dir.path().join("a.rs"))
        .arg(dir.path().join("b.rs"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("id_a"));
    assert!(stdout.contains("id_b"));
}

#[test]
fn test_list_directory_recursive() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    std::fs::write(
        dir.path().join("a.rs"),
        "// injm begin <id_a\ncontent\n// injm end\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("sub/b.rs"),
        "// injm begin <id_b\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(dir.path().to_string_lossy().as_ref())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("id_a"));
    assert!(stdout.contains("id_b"));
}
