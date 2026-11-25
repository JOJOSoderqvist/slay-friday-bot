use crate::providers::Model;
use crate::errors::ApiError;
use async_trait::async_trait;

#[allow(clippy::double_must_use)]
#[async_trait]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;
    fn get_model_name(&self) -> Model;
}
