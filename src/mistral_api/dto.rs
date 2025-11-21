use serde::{Deserialize, Serialize};
use crate::constants::{TEXT_MODIFY_PROMPT};

#[derive(Serialize, Debug)]
pub struct MistralGenerateTextRequest {
    pub model: String,
    pub messages: Vec<MistralMessage>
}

impl MistralGenerateTextRequest {
    pub fn new(messages: Vec<MistralMessage>) -> Self {
        MistralGenerateTextRequest{
            model: "mistral-small-latest".to_string(),
            messages
        }
    }
}

#[derive(Serialize, Debug)]
pub struct MistralMessage {
    pub role: String,
    pub content: String
}

impl MistralMessage {
    pub fn new(content_text: String) -> Self {
        MistralMessage {
            role: "user".to_string(),
            content: content_text,
        }
    }

    pub fn new_system_message() -> Self {
        MistralMessage {
            role: "system".to_string(),
            content: TEXT_MODIFY_PROMPT.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MistralGenerateTextResponse {
    pub choices: Vec<MistralChoiceResponse>
}

#[derive(Deserialize, Debug)]
pub struct MistralChoiceResponse {
    pub message: MistralMessageResponse
}

#[derive(Deserialize, Debug)]
pub struct MistralMessageResponse {
    pub content: String
}