pub mod check;
pub mod inject;
pub mod list;
pub mod root;

fn merged_with_fallback(cli: Vec<String>, input: Vec<String>, output: Vec<String>) -> Vec<String> {
    let mut merged: Vec<String> = cli.into_iter().chain(input).chain(output).collect();
    if merged.is_empty() {
        merged.push(".".to_string());
    }
    merged
}
