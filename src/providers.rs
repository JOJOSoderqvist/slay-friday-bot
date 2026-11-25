use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
enum ModelProvider {
    GrokProvider(&'static str),
    GeminiProvider(&'static str),
}

impl ModelProvider {
    const GROK_PROVIDER: ModelProvider = ModelProvider::GrokProvider("https://api.x.ai/v1");
    const GEMINI_PROVIDER: ModelProvider =
        ModelProvider::GeminiProvider("https://generativelanguage.googleapis.com/v1beta/openai");


    fn base_url(&self) -> &'static str {
        match self {
            ModelProvider::GrokProvider(base_url)
            | ModelProvider::GeminiProvider(base_url)
            => base_url
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Model {
    provider: ModelProvider,
    model_name_api: &'static str,
    model_name_display: &'static str,
    disable_reasoning: bool,
}

impl Model {

    pub const GROK_MODEL: Model = Model {
        provider: ModelProvider::GROK_PROVIDER,
        model_name_api: "grok-4.3",
        model_name_display: "Grok-4-3",
        disable_reasoning: true,
    };

    pub const GEMINI_MODEL: Model = Model {
        provider: ModelProvider::GEMINI_PROVIDER,
        model_name_api: "gemini-3.5-flash-lite",
        model_name_display: "gemini-3.5-flash",
        disable_reasoning: false,
    };

    pub const GEMINI_2_5_FLASH_LITE_MODEL: Model = Model {
        provider: ModelProvider::GEMINI_PROVIDER,
        model_name_api: "gemini-2.5-flash-lite",
        model_name_display: "gemini-2.5-flash-lite",
        disable_reasoning: true,
    };

    pub fn base_url(&self) -> &'static str {
        self.provider.base_url()
    }

    pub(crate) fn uses_chat_completions(&self) -> bool {
        matches!(self.provider, ModelProvider::GeminiProvider(_))
    }

    pub(crate) fn disables_reasoning(&self) -> bool {
        self.disable_reasoning
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
