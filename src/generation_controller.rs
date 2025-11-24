use std::collections::VecDeque;
use crate::errors::ApiError;
use crate::errors::ApiError::{GenFailed, NoModels};
use crate::handlers::ContentGenerator;
use async_trait::async_trait;
use rand::seq::{SliceRandom};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use tracing::instrument;
use crate::common::Model;

pub type ModelPool = Vec<Arc<dyn ContentRephraser>>;

#[async_trait]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;

    fn get_model_name(&self) -> Model;
}

pub struct GenerationController {
    pub models: ModelPool,
    storage: Mutex<VecDeque<(Model, String)>>
}
impl GenerationController {
    pub fn new(models: ModelPool) -> Self {
        let mut storage: VecDeque<(Model, String)> = VecDeque::new();
        storage.reserve(10);

        GenerationController {
            models,
            storage: Mutex::new(storage)
        }
    }

    async fn add_storage_entry(&self, model: Model, text: &str) {
        let mut storage_lock = self.storage.lock().await;
        if storage_lock.len() == 10 {
            storage_lock.pop_front();
        }

        storage_lock.push_back((model, text.to_string()));
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
                    self.add_storage_entry(model_name, new_text.as_str()).await;

                    return Ok(new_text)
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

    async fn get_message_info(&self, text: &str) -> Option<Model> {
        let storage_lock = self.storage.lock().await;


        for entry in storage_lock.iter() {
            if entry.1 == text {
                return Some(entry.0)
            }
        }

        None
    }
}
