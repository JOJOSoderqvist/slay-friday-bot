use crate::providers::Model;
use crate::constants::TEXT_MODIFY_PROMPT;
use crate::errors::ApiError;
use crate::errors::ApiError::NoContent;
use crate::content_rephraser::ContentRephraser;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs};
use async_openai::types::responses::{CreateResponseArgs, Reasoning, ReasoningEffort};
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

    async fn rephrase_with_chat(&self, current_text: &str) -> Result<String, ApiError> {
        let mut request = CreateChatCompletionRequestArgs::default();
        request
            .model(self.model.api_name())
            .messages(vec![
                ChatCompletionRequestMessage::System(TEXT_MODIFY_PROMPT.into()),
                ChatCompletionRequestMessage::User(current_text.into()),
            ]);

        if self.model.disables_reasoning() {
            request.reasoning_effort(ReasoningEffort::None);
        }

        let response = self.client.chat().create(request.build()?).await?;
        response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|text| !text.trim().is_empty())
            .ok_or(NoContent)
    }
}

#[async_trait]
impl ContentRephraser for OpenAIClient {
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        if self.model.uses_chat_completions() {
            return self.rephrase_with_chat(current_text).await;
        }

        let mut request = CreateResponseArgs::default();
        request
            .model(self.model.api_name())
            .instructions(TEXT_MODIFY_PROMPT)
            .input(current_text);

        if self.model.disables_reasoning() {
            request.reasoning(Reasoning {
                effort: Some(ReasoningEffort::None),
                ..Default::default()
            });
        }

        let response = self.client
            .responses()
            .create(request.build()?)
            .await?;

        eprintln!("{:?}", response);

        match response.output_text() {
            Some(text) => Ok(text),
            None => Err(NoContent)
        }
    }

    fn get_model_name(&self) -> Model {
        self.model
    }
}
