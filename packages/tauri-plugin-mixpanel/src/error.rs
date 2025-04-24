use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum Error {
    #[error("Mixpanel error: {0}")]
    MixpanelError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
