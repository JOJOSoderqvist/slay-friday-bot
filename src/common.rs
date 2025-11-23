use crate::errors::ApiError;
use crate::errors::ApiError::ApiStatusError;
use reqwest::Response;
use std::fmt::Display;
use tracing::error;

#[derive(Debug, Copy, Clone)]
pub enum Model {
    Gigachat,
    Mistral,
    Grok,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Gigachat => write!(f, "Gigachat"),
            Model::Mistral => write!(f, "Mistral"),
            Model::Grok => write!(f, "Grok"),
        }
    }
}

pub async fn ensure_success(model: Model, response: Response) -> Result<Response, ApiError> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    error!(%status, %body, %model, "Generation failed");

    Err(ApiStatusError {
        model,
        status,
        body,
    })
}
