use async_trait::async_trait;
use log::info;
use reqwest::{Client, Url};
use tracing::{debug, error, instrument, warn};
use crate::errors::ApiError;
use crate::errors::ApiError::{ApiStatusError, DecodeResponseError, NoContent, RequestError};
use crate::handlers::ContentGenerator;
use crate::mistral_api::dto::{MistralGenerateTextRequest, MistralGenerateTextResponse, MistralMessage};

#[derive(Debug)]
pub struct MistralApi {
    server: Client,
    token: String
}

impl MistralApi {
    pub fn new(token: String) -> Self {
        MistralApi{
            server: Client::new(),
            token
        }
    }
}

#[async_trait]
impl ContentGenerator for MistralApi {
    #[instrument(skip(self, current_text), err)]
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        info!("Starting generation through Mistral");

        let system_message = MistralMessage::new_system_message();
        let content = MistralMessage::new(current_text.to_string());

        let request = MistralGenerateTextRequest::new(vec![system_message, content]);

        info!("Sending generation request to Mistral...");

        for attempt in 1..=2 {
            debug!("Sending request, attempt {}", attempt);

            let generate_content_url =
                Url::parse("https://api.mistral.ai/v1/chat/completions")?;

            let response = self.server
                .post(generate_content_url)
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", self.token))
                .json(&request)
                .send()
                .await
                .map_err(RequestError)?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                error!(%status, %body, "Mistral generation failed");
                return Err(ApiStatusError { status, body });
            }

            let resp_text = response.text().await.map_err(DecodeResponseError)?;
            let response: MistralGenerateTextResponse = serde_json::from_str(&resp_text)?;

            if let Some(new_text) = response.choices.into_iter().next() {
                info!("Text rephrased successfully");
                return Ok(new_text.message.content)
            }
        }

        warn!("Mistral returned 200 OK but empty choices");
        Err(NoContent)
    }
}