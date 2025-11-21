use std::sync::Arc;
use async_trait::async_trait;
use log::error;
use rand::seq::IndexedRandom;
use tracing::instrument;
use crate::errors::ApiError;
use crate::errors::ApiError::NoModels;
use crate::handlers::ContentGenerator;

type ModelPool = Vec<Arc<dyn ContentRephraser>>;

#[async_trait]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;
}

pub struct GenerationController {
    pub models: ModelPool
}

impl GenerationController {
    pub fn new(models: Vec<Arc<dyn ContentRephraser>>) -> Self {
        GenerationController {
            models: ModelPool::from(models)
        }
    }
}


#[async_trait]
impl ContentGenerator for GenerationController {
    #[instrument(skip(self, current_text), err)]
    async fn generate_text(&self, current_text: &str) -> Result<String, ApiError> {
        let picked_model = match self.models.choose(&mut rand::rng()) {
            Some(model) => model,
            None => {
                error!("no models were provided");
                return Err(NoModels)
            }
        };
        
        let generated_text = picked_model.rephrase_text(current_text).await?;
        Ok(generated_text)
    }
}




