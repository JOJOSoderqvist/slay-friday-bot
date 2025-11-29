use crate::common::Model;
use crate::errors::ApiError;
use crate::errors::ApiError::{GenFailed, NoModels};
use crate::handlers::ContentGenerator;
use crate::repo::message_history_storage::HistoryEntry;
use crate::repo::sticker_storage::dto::StickerEntry;
use async_trait::async_trait;
use rand::seq::SliceRandom;
use std::error::Error;
use std::sync::Arc;
use tracing::error;
use tracing::instrument;

pub type ModelPool = Vec<Arc<dyn ContentRephraser>>;

#[async_trait]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;

    fn get_model_name(&self) -> Model;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn add_message(&self, message: HistoryEntry);
    async fn get_message_info(&self, message: &str) -> Option<Model>;
}

pub struct GenerationController {
    pub models: ModelPool,
    pub storage: Arc<dyn MessageStore>,
}

impl GenerationController {
    pub fn new(models: ModelPool, storage: Arc<dyn MessageStore>) -> Self {
        GenerationController { models, storage }
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
                Ok(new_text) => {
                    let model_name = sh.get_model_name();
                    self.storage
                        .add_message(HistoryEntry::new(model_name, current_text.to_string()))
                        .await;

                    return Ok(new_text);
                }

                Err(err) => {
                    error!(error = %err, "failed to generated content, trying next model");
                    continue;
                }
            }
        }

        error!("Generation with all models has failed");
        Err(GenFailed)
    }

    // TODO: Cringe
    async fn get_message_info(&self, text: &str) -> Option<Model> {
        self.storage.get_message_info(text).await
    }
}
