use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
enum ModelProvider {
    MistralProvider(&'static str),
    GrokProvider(&'static str),
}

impl ModelProvider {
    const MISTRAL_PROVIDER: ModelProvider = ModelProvider::MistralProvider("https://api.mistral.ai/v1/");
    const GROK_PROVIDER: ModelProvider = ModelProvider::GrokProvider("https://api.mistral.ai/v1/chat/completions");

    fn base_url(&self) -> &'static str {
        match self {
            ModelProvider::MistralProvider(base_url) | ModelProvider::GrokProvider(base_url) => base_url
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Model {
    provider: ModelProvider,
    model_name_api: &'static str,
    model_name_display: &'static str,
}

impl Model {
    pub const MISTRAL_MODEL: Model = Model {
        provider: ModelProvider::MISTRAL_PROVIDER,
        model_name_api: "mistral-small-2603",
        model_name_display: "Mistral Medium",
    };

    pub const GLM_MODEL: Model = Model {
        provider: ModelProvider::MISTRAL_PROVIDER,
        model_name_api: "zai-glm-5-3-flash",
        model_name_display: "GLM-5-3-flash",
    };

    pub const GROK_MODEL: Model = Model {
        provider: ModelProvider::GROK_PROVIDER,
        model_name_api: "grok-4.3",
        model_name_display: "Grok-4-3",
    };

    pub fn base_url(&self) -> &'static str {
        self.provider.base_url()
    }
    pub fn api_name(&self) -> &'static str {
        self.model_name_api
    }
    pub fn display_name(&self) -> &'static str {
        self.model_name_display
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_name_display)
    }
}