use crate::providers::Model;
use crate::constants::TEXT_MODIFY_PROMPT;
use crate::errors::ApiError;
use crate::errors::ApiError::NoContent;
use crate::content_rephraser::ContentRephraser;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::CreateResponseArgs;
use async_trait::async_trait;

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model: Model
}

impl OpenAIClient {
    pub fn new(model: Model, api_key: String) -> Self {
        let cfg = OpenAIConfig::new()
            .with_api_base(model.base_url())
            .with_api_key(api_key);

        Self {
            client: Client::with_config(cfg),
            model,
        }
    }
}

#[async_trait]
impl ContentRephraser for OpenAIClient {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        let request = CreateResponseArgs::default()
            .model(self.model.api_name())
            .instructions(TEXT_MODIFY_PROMPT)
            .input(current_text)
            .build()?;

        let response = self.client
            .responses()
            .create(request)
            .await?;

        match response.output_text() {
            Some(text) => Ok(text),
            None => Err(NoContent)
        }
    }

    fn get_model_name(&self) -> Model {
        self.model
    }
}
