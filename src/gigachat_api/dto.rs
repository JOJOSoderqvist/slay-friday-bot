use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampMilliSeconds};
use std::time;
use crate::constants::TEXT_MODIFY_PROMPT;

#[derive(Serialize, Debug)]
pub struct GigaChatAuthRequest {
    pub scope: String,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct GigaChatAuthResponse {
    pub access_token: String,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub expires_at: time::SystemTime
}

#[derive(Serialize)]
pub struct GigaChatGenerateTextRequest {
    pub model: String,
    pub messages: Vec<GigaChatMessage>
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GigaChatRole {
    System,
    User,
    Assistant
}

impl From<String> for GigaChatRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "user" => GigaChatRole::User,
            "system" => GigaChatRole::System,
            "assistant" => GigaChatRole::Assistant,
            _ => GigaChatRole::User,
        }
    }
}

#[derive(Serialize)]
pub struct GigaChatMessage {
    pub role: GigaChatRole,
    pub content: String
}

impl GigaChatMessage {
    pub fn new(role: GigaChatRole, content: String) -> Self {
        GigaChatMessage {
            role,
            content
        }
    }

    // TODO: Мб лучше создавать не тут
    pub fn new_system_message() -> Self {
        GigaChatMessage{
            role: GigaChatRole::System,
            content: TEXT_MODIFY_PROMPT.to_string()
        }
    }
}


#[derive(Deserialize)]
pub struct GigaChatGenerateTextResponse {
    pub choices: Vec<GigaChatChoice>
}

#[derive(Deserialize)]
pub struct GigaChatChoice {
    pub message: GigaChatChoiceMessage,
}

#[derive(Deserialize)]
pub struct GigaChatChoiceMessage {
    pub content: String,
}