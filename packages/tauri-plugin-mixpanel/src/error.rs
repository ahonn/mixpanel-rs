use tauri::ipc::InvokeError;
use thiserror::Error;

use crate::persistence::PersistenceError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("Mixpanel client error: {0}")]
    MixpanelClient(mixpanel_rs::Error),
    #[error("Mixpanel plugin error: {0}")]
    MixpanelError(String),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Underlying Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl From<mixpanel_rs::Error> for Error {
    fn from(err: mixpanel_rs::Error) -> Self {
        Error::MixpanelClient(err)
    }
}

impl From<Error> for InvokeError {
    fn from(error: Error) -> Self {
        InvokeError::from_error(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
