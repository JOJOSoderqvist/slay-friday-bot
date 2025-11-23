use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct GrokGenerateTextRequest {
    pub messages: Vec<GrokMessage>,
    pub model: String,
    pub stream: bool,
}

impl GrokGenerateTextRequest {
    pub fn new(model: String, messages: Vec<GrokMessage>, _stream: bool) -> Self {
        GrokGenerateTextRequest {
            messages,
            model,
            stream: false,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GrokMessage {
    pub role: String,
    pub content: String,
}

impl GrokMessage {
    pub fn new(role: String, content: String) -> Self {
        GrokMessage { role, content }
    }
}

#[derive(Deserialize, Debug)]
pub struct GrokGenerateTextResponse {
    pub choices: Vec<GrokChoiceResponse>,
}

#[derive(Deserialize, Debug)]
pub struct GrokChoiceResponse {
    pub message: GrokMessage,
}
