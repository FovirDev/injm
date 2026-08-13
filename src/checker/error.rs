use thiserror::Error;

pub type Result<T> = std::result::Result<T, CheckerError>;

#[derive(Debug, Error)]
pub enum CheckerError {
    #[error(transparent)]
    Injector(#[from] crate::injector::InjectorError),
}
