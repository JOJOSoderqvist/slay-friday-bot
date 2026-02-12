use crate::common::{Model, ensure_success};
use crate::errors::ApiError;
use crate::errors::ApiError::{DecodeResponseError, NoContent, RequestError};
use crate::generation_controller::ContentRephraser;
use crate::mistral_api::dto::{
    MistralGenerateTextRequest, MistralGenerateTextResponse, MistralMessage,
};
use async_trait::async_trait;
use log::info;
use reqwest::{Client, Url};
use tracing::{debug, error, instrument, warn};

#[derive(Debug)]
pub struct MistralApi {
    server: Client,
    token: String,
    model: Model,
}

impl MistralApi {
    pub fn new(token: String) -> Self {
        MistralApi {
            server: Client::new(),
            token,
            model: Model::Mistral,
        }
    }
}

#[async_trait]
impl ContentRephraser for MistralApi {
    #[instrument(skip(self, current_text), err)]
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        info!("Starting generation through Mistral");

        let system_message = MistralMessage::new_system_message();
        let content = MistralMessage::new(current_text.to_string());

        let request = MistralGenerateTextRequest::new(vec![system_message, content]);

        info!("Sending generation request to Mistral...");

        for attempt in 1..=2 {
            debug!("Sending request, attempt {}", attempt);

            let generate_content_url = Url::parse("https://api.mistral.ai/v1/chat/completions")?;

            let response = self
                .server
                .post(generate_content_url)
                .header(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", self.token),
                )
                .json(&request)
                .send()
                .await
                .map_err(RequestError)?;

            let response = match ensure_success(self.model, response).await {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt == 1 {
                        continue;
                    }
                    error!(error = %e, "Failed to generate content through Mistral");
                    return Err(e);
                }
            };

            let response: MistralGenerateTextResponse =
                response.json().await.map_err(DecodeResponseError)?;

            if let Some(new_text) = response.choices.into_iter().next() {
                info!("Text rephrased successfully");
                return Ok(new_text.message.content);
            }
        }

        warn!("Mistral returned 200 OK but empty choices");
        Err(NoContent)
    }

    fn get_model_name(&self) -> Model {
        self.model
    }
}
