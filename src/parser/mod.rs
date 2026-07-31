mod comment;
pub(super) mod config;
mod detector;
pub(crate) mod error;
mod marker;
pub(crate) mod option;
mod pattern;

pub(crate) use error::*;
pub(crate) use option::*;
pub use pattern::*;
