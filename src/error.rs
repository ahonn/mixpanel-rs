use url;
use serde_json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Mixpanel API server error (HTTP {0})")]
    ApiServerError(u16),

    #[error("Mixpanel API rate limited (Retry after: {0:?} seconds)")]
    ApiRateLimitError(Option<u64>),

    #[error("Mixpanel API client error (HTTP {0}): {1}")]
    ApiClientError(u16, String),

    #[error("Mixpanel API payload too large (HTTP 413)")]
    ApiPayloadTooLarge,

    #[error("Mixpanel API HTTP error (HTTP {0}): {1}")]
    ApiHttpError(u16, String),

    #[error("Mixpanel API unexpected response: {0}")]
    ApiUnexpectedResponse(String),

    #[error("Time conversion error")]
    TimeError,

    #[error("Max retries reached: {0}")]
    MaxRetriesReached(String),
}

