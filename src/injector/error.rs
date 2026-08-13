use thiserror::Error;

pub type Result<T> = std::result::Result<T, InjectorError>;

#[derive(Debug, Error)]
pub enum InjectorError {
    #[error("empty input content")]
    EmptyInputContent,

    #[error("invalid range: ({begin}, {end})")]
    InvalidRange { begin: usize, end: usize },

    #[error("line {line_number} mixes tabs and spaces in indentation")]
    MixedIndentChar { line_number: usize },
}
