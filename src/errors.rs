use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to perform request {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse url {0}")]
    ParseUrlError(#[from] url::ParseError),

    #[error("Failed to transform json {0}")]
    TransformJSONError(#[from] serde_json::Error),

    #[error("No content was generated")]
    NoContent
}