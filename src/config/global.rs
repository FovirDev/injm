use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub(crate) struct GlobalConfig {
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub exclude: Vec<String>,
}
