use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn write_file(directory: &Path, relative_path: &str, content: &str) -> PathBuf {
    let path = directory.join(relative_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    fs::write(&path, content).unwrap();

    path
}

fn injm() -> Command {
    let mut cmd: Command = assert_cmd::cargo::cargo_bin_cmd!("injm");
    cmd.arg("--no-gitignore");
    cmd
}

#[test]
fn check_succeeds_when_single_block_is_synchronized() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
println!("hello");
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"fn main() {
    // injm begin >hello
println!("hello");
    // injm end
}
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_fails_when_single_block_is_out_of_sync() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
println!("expected");
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
println!("actual");
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("output block `hello` is out of sync")
                .and(predicate::str::contains(output.display().to_string()))
                .and(predicate::str::contains("1-3"))
                .and(predicate::str::contains(
                    "1 output block(s) are out of sync",
                )),
        );
}

#[test]
fn check_ignores_default_output_block_without_id() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin
expected default content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin
different default content
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_ignores_default_output_even_without_default_input() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <named
named content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin
default content without an input
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_fails_when_output_id_has_no_matching_input() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <first
first content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >missing
content
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing input id `missing`"));
}

#[test]
fn check_fails_when_input_id_is_duplicated_across_files() {
    let temp = TempDir::new().unwrap();

    let input_one = write_file(
        temp.path(),
        "inputs/one.rs",
        r#"// injm begin <hello
first definition
// injm end
"#,
    );

    let input_two = write_file(
        temp.path(),
        "inputs/two.rs",
        r#"// injm begin <hello
second definition
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
first definition
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input_one)
        .arg(&input_two)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicated input id `hello`"));
}

#[test]
fn check_fails_when_same_input_block_repeats_an_id() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello <hello
content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
content
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicated input id `hello`"));
}

#[test]
fn one_input_id_can_synchronize_multiple_output_blocks() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <shared
shared content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >shared
shared content
// injm end

fn between() {}

// injm begin >shared
shared content
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_reports_every_unsynchronized_output_block() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <first
expected first
// injm end

// injm begin <second
expected second
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >first
actual first
// injm end

// injm begin >second
actual second
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("output block `first` is out of sync")
                .and(predicate::str::contains(
                    "output block `second` is out of sync",
                ))
                .and(predicate::str::contains(
                    "2 output block(s) are out of sync",
                )),
        );
}

#[test]
fn check_reports_issues_from_multiple_output_files() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <shared
expected content
// injm end
"#,
    );

    let output_one = write_file(
        temp.path(),
        "outputs/one.rs",
        r#"// injm begin >shared
old content one
// injm end
"#,
    );

    let output_two = write_file(
        temp.path(),
        "outputs/two.rs",
        r#"// injm begin >shared
old content two
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output_one)
        .arg(&output_two)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains(output_one.display().to_string())
                .and(predicate::str::contains(output_two.display().to_string()))
                .and(predicate::str::contains(
                    "2 output block(s) are out of sync",
                )),
        );
}

#[test]
fn check_detects_leading_whitespace_difference() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
    println!("hello");
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
println!("hello");
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output block `hello` is out of sync",
        ));
}

#[test]
fn check_detects_blank_line_difference() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
first line

second line
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
first line
second line
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output block `hello` is out of sync",
        ));
}

#[test]
fn check_handles_multiline_synchronized_content() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <function
fn generated() {
    println!("first");
    println!("second");
}
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"mod generated {
    // injm begin >function
fn generated() {
    println!("first");
    println!("second");
}
    // injm end
}
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn input_block_with_multiple_ids_checks_each_output_id() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <first <second
shared content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >first
shared content
// injm end

// injm begin >second
shared content
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_ignores_input_marker_blocks_in_output_files() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin <unrelated
this is an input block, not an output block
// injm end

// injm begin >hello
expected
// injm end
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_succeeds_when_output_file_has_no_marker_blocks() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"fn main() {
    println!("no marker blocks");
}
"#,
    );

    injm()
        .args(["check"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_accepts_directories_as_patterns() {
    let temp = TempDir::new().unwrap();

    let input_directory = temp.path().join("inputs");
    let output_directory = temp.path().join("outputs");

    write_file(
        temp.path(),
        "inputs/nested/input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    write_file(
        temp.path(),
        "outputs/nested/output.rs",
        r#"// injm begin >hello
expected
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input_directory)
        .arg(&output_directory)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_fails_when_input_pattern_matches_no_files() {
    let temp = TempDir::new().unwrap();

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
content
// injm end
"#,
    );

    let missing_pattern = temp.path().join("missing").join("*.rs");

    injm()
        .arg("check")
        .arg(&missing_pattern)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no files matched pattern"));
}

#[test]
fn check_fails_when_output_pattern_matches_no_files() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
content
// injm end
"#,
    );

    let missing_pattern = temp.path().join("missing").join("*.rs");

    injm()
        .arg("check")
        .arg(&input)
        .arg(&missing_pattern)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no files matched pattern"));
}

#[test]
fn check_reports_nested_marker_parser_error() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
// injm begin <second
content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
content
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("found nested `injm begin`")
                .and(predicate::str::contains(input.display().to_string())),
        );
}

#[test]
fn check_reports_end_marker_without_begin() {
    let temp = TempDir::new().unwrap();

    let input = write_file(temp.path(), "input.rs", "// injm end\n");

    let output = write_file(temp.path(), "output.rs", "fn main() {}\n");

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("found `injm end` without `injm begin`")
                .and(predicate::str::contains(input.display().to_string())),
        );
}

#[test]
fn check_reports_begin_marker_without_end() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
content without an end marker
"#,
    );

    let output = write_file(temp.path(), "output.rs", "fn main() {}\n");

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("without `injm end`")
                .and(predicate::str::contains(input.display().to_string())),
        );
}

#[test]
fn check_handles_paths_containing_spaces() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "directory with spaces/input file.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "directory with spaces/output file.rs",
        r#"// injm begin >hello
actual
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains(output.display().to_string()));
}

#[test]
fn check_with_diff_prints_unified_diff_for_out_of_sync_block() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
println!("expected");
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
println!("actual");
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(":1-3:hello:actual")
                .and(predicate::str::contains(":1-3:hello:expected"))
                .and(predicate::str::contains("@@ -1 +1 @@"))
                .and(predicate::str::contains(r#"-println!("actual");"#))
                .and(predicate::str::contains(r#"+println!("expected");"#)),
        )
        .stderr(
            predicate::str::contains("output block `hello` is out of sync")
                .and(predicate::str::contains(output.display().to_string()))
                .and(predicate::str::contains("1-3"))
                .and(predicate::str::contains(
                    "1 output block(s) are out of sync",
                )),
        );
}

#[test]
fn check_with_diff_prints_nothing_when_blocks_are_synchronized() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
println!("hello");
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
println!("hello");
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "all marker blocks are synchronized",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_with_diff_ignores_default_output_blocks() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin
expected default content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin
different default content
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "all marker blocks are synchronized",
        ))
        .stdout(predicate::str::contains("--- a/").not())
        .stdout(predicate::str::contains("+++ b/").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_with_diff_prints_a_diff_for_every_out_of_sync_block() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <first
expected first
// injm end

// injm begin <second
expected second
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >first
actual first
// injm end

// injm begin >second
actual second
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(":1-3:first:actual")
                .and(predicate::str::contains(":1-3:first:expected"))
                .and(predicate::str::contains(":5-7:second:actual"))
                .and(predicate::str::contains(":5-7:second:expected"))
                .and(predicate::str::contains("-actual first"))
                .and(predicate::str::contains("+expected first"))
                .and(predicate::str::contains("-actual second"))
                .and(predicate::str::contains("+expected second"))
                .and(predicate::function(|stdout: &str| {
                    stdout
                        .lines()
                        .filter(|line| line.starts_with("--- "))
                        .count()
                        == 2
                })),
        )
        .stderr(
            predicate::str::contains("output block `first` is out of sync")
                .and(predicate::str::contains(
                    "output block `second` is out of sync",
                ))
                .and(predicate::str::contains(
                    "2 output block(s) are out of sync",
                )),
        );
}

#[test]
fn check_with_diff_handles_multiline_block_content() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <function
fn generated() {
    println!("new");
    println!("unchanged");
}
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >function
fn generated() {
    println!("old");
    println!("unchanged");
}
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(r#"-    println!("old");"#)
                .and(predicate::str::contains(r#"+    println!("new");"#))
                .and(predicate::str::contains(r#"     println!("unchanged");"#)),
        )
        .stderr(predicate::str::contains(
            "output block `function` is out of sync",
        ));
}

#[test]
fn check_with_diff_reports_missing_id_without_printing_diff() {
    let temp = TempDir::new().unwrap();

    let input = write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <existing
content
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >missing
content
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input)
        .arg(&output)
        .arg("--diff")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("missing input id `missing`"));
}

#[test]
fn check_with_diff_reports_duplicate_id_without_printing_diff() {
    let temp = TempDir::new().unwrap();

    let input_one = write_file(
        temp.path(),
        "input-one.rs",
        r#"// injm begin <hello
first
// injm end
"#,
    );

    let input_two = write_file(
        temp.path(),
        "input-two.rs",
        r#"// injm begin <hello
second
// injm end
"#,
    );

    let output = write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
first
// injm end
"#,
    );

    injm()
        .arg("check")
        .arg(&input_one)
        .arg(&input_two)
        .arg(&output)
        .arg("--diff")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("duplicated input id `hello`"));
}

#[test]
fn check_falls_back_to_current_directory_when_no_args_given() {
    let temp = TempDir::new().unwrap();

    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
expected
// injm end
"#,
    );

    injm()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_falls_back_to_current_directory_and_reports_out_of_sync() {
    let temp = TempDir::new().unwrap();

    write_file(
        temp.path(),
        "input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    write_file(
        temp.path(),
        "output.rs",
        r#"// injm begin >hello
actual
// injm end
"#,
    );

    injm()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output block `hello` is out of sync",
        ));
}

#[test]
fn check_falls_back_to_current_directory_recursively() {
    let temp = TempDir::new().unwrap();

    write_file(
        temp.path(),
        "src/input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    write_file(
        temp.path(),
        "src/output.rs",
        r#"// injm begin >hello
expected
// injm end
"#,
    );

    injm()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn check_falls_back_to_current_directory_recursively_reports_out_of_sync() {
    let temp = TempDir::new().unwrap();

    write_file(
        temp.path(),
        "nested/deep/input.rs",
        r#"// injm begin <hello
expected
// injm end
"#,
    );

    write_file(
        temp.path(),
        "nested/deep/output.rs",
        r#"// injm begin >hello
actual
// injm end
"#,
    );

    injm()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output block `hello` is out of sync",
        ));
}
