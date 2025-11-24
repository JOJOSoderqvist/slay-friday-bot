use crate::common::{Model, ensure_success};
use crate::constants::TEXT_MODIFY_PROMPT;
use crate::errors::ApiError;
use crate::errors::ApiError::{DecodeResponseError, NoContent, RequestError};
use crate::generation_controller::ContentRephraser;
use crate::grok_api::dto::{GrokGenerateTextRequest, GrokGenerateTextResponse, GrokMessage};
use async_trait::async_trait;
use log::{info, warn};
use reqwest::{Client, Proxy};
use std::error::Error;
use url::Url;

pub struct GrokApi {
    client: Client,
    token: String,
    model: Model,
}
impl GrokApi {
    pub fn new(token: String, proxy_url: String) -> Result<Self, Box<dyn Error>> {
        let proxy = Proxy::all(proxy_url)?;

        let client = Client::builder().proxy(proxy).build()?;

        Ok(GrokApi {
            client,
            token,
            model: Model::Grok,
        })
    }
}

#[async_trait]
impl ContentRephraser for GrokApi {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        let system_message = GrokMessage::new("system".to_string(), TEXT_MODIFY_PROMPT.to_string());
        let user_message = GrokMessage::new("user".to_string(), current_text.to_string());

        let request = GrokGenerateTextRequest::new(
            "grok-4-1-fast-non-reasoning".to_string(),
            vec![system_message, user_message],
            false,
        );

        for attempt in 1..=2 {
            let request_url = Url::parse("https://api.x.ai/v1/chat/completions")?;

            let response = self
                .client
                .post(request_url)
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
                    return Err(e);
                }
            };

            let response: GrokGenerateTextResponse =
                response.json().await.map_err(DecodeResponseError)?;

            if let Some(new_text) = response.choices.into_iter().next() {
                info!("Text rephrased successfully");
                return Ok(new_text.message.content);
            }
        }

        warn!("No content was generated");
        Err(NoContent)
    }

    fn get_model_name(&self) -> Model {
        self.model
    }
}
