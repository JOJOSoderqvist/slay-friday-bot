use crate::errors::ApiError;
use crate::errors::ApiError::{GenFailed, NoModels};
use crate::handlers::ContentGenerator;
use async_trait::async_trait;
use rand::seq::{SliceRandom};
use std::sync::Arc;
use tracing::error;
use tracing::instrument;

pub type ModelPool = Vec<Arc<dyn ContentRephraser>>;

#[async_trait]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;
}

pub struct GenerationController {
    pub models: ModelPool,
}
impl GenerationController {
    pub fn new(models: ModelPool) -> Self {
        GenerationController { models }
    }
}

#[async_trait]
impl ContentGenerator for GenerationController {
    #[instrument(skip(self, current_text), err)]
    async fn generate_text(&self, current_text: &str) -> Result<String, ApiError> {
        if self.models.is_empty() {
            error!("no models were provided");
            return Err(NoModels);
        }

        let mut local_models = self.models.clone();
        local_models.shuffle(&mut rand::rng());

        for sh in local_models {
            match sh.rephrase_text(current_text).await {
                Ok(new_text) => return Ok(new_text),

                Err(err) => {
                    error!(error = %err, "failed to generated content, trying next model");
                    continue;
                }
            }
        }

        error!("Generation with all models has failed");
        Err(GenFailed)
    }
}
