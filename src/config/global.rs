use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct GlobalConfig {
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub exclude: Vec<String>,
}
