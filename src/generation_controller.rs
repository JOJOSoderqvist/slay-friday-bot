use crate::common::Model;
use crate::errors::ApiError;
use crate::errors::ApiError::{GenFailed, NoModels};
use crate::handlers::ContentGenerator;
use async_trait::async_trait;
use mockall::automock;
use rand::seq::SliceRandom;
use std::sync::Arc;
use tracing::error;
use tracing::instrument;

pub type ModelPool = Vec<Arc<dyn ContentRephraser>>;

#[async_trait]
#[automock]
pub trait ContentRephraser: Send + Sync {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError>;

    fn get_model_name(&self) -> Model;
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
    async fn generate_text(&self, current_text: &str) -> Result<(String, Model), ApiError> {
        if self.models.is_empty() {
            error!("no models were provided");
            return Err(NoModels);
        }

        let mut local_models = self.models.clone();
        local_models.shuffle(&mut rand::rng());

        for sh in local_models {
            match sh.rephrase_text(current_text).await {
                Ok(new_text) => {
                    return Ok((new_text, sh.get_model_name()));
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
}

#[tokio::test]
async fn generation_controller_fails_test() {
    let controller = GenerationController::new(vec![]);
    let res = controller.generate_text("some test text").await;

    assert!(matches!(res, Err(NoModels)))
}

#[tokio::test]
async fn generation_controller_fallback_test() {
    let mut failing = MockContentRephraser::new();
    failing
        .expect_rephrase_text()
        .returning(|_| Box::pin(async { Err(GenFailed) }));

    failing.expect_get_model_name().return_const(Model::Grok);

    let mut succeeding = MockContentRephraser::new();
    succeeding
        .expect_rephrase_text()
        .returning(|_| Box::pin(async { Ok("new text".to_string()) }));

    succeeding
        .expect_get_model_name()
        .return_const(Model::Mistral);

    let controller = GenerationController::new(vec![Arc::new(failing), Arc::new(succeeding)]);

    let res = controller.generate_text("some test text").await;

    assert!(res.is_ok());

    let (text, model) = res.unwrap();
    assert!(matches!(text.as_str(), "new text"));
    assert!(matches!(model, Model::Mistral));
}

#[tokio::test]
async fn generation_controller_failed_test() {
    let mut failing = MockContentRephraser::new();
    failing
        .expect_rephrase_text()
        .returning(|_| Box::pin(async { Err(GenFailed) }));

    failing.expect_get_model_name().return_const(Model::Grok);

    let controller = GenerationController::new(vec![Arc::new(failing)]);

    let res = controller.generate_text("some test text").await;

    assert!(matches!(res, Err(GenFailed)));
}
