use crate::errors::BotConfigError;
use crate::errors::BotConfigError::{
    BotTokenNotFound, GigaChatClientIDNotFound, GigaChatClientSecretNotFound, LogLevelNotFound,
    MistralTokenNotFound, ParseLogLevelError,
};
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;
use tracing::Level;

pub struct BotConfig {
    pub tg_token: String,
    pub gigachat_client_id: String,
    pub gigachat_client_secret: String,
    pub mistral_token: String,
    pub log_level: Level,
}

impl BotConfig {
    pub fn new() -> Result<Self, BotConfigError> {
        dotenv().ok();
        let tg_token = env::var("TELOXIDE_TOKEN").map_err(BotTokenNotFound)?;

        let gigachat_client_id =
            env::var("GIGACHAT_CLIENT_ID").map_err(GigaChatClientIDNotFound)?;

        let gigachat_client_secret =
            env::var("GIGACHAT_CLIENT_SECRET").map_err(GigaChatClientSecretNotFound)?;

        let mistral_token = env::var("MISTRAL_TOKEN").map_err(MistralTokenNotFound)?;

        let log_level_str = env::var("LOG_LEVEL").map_err(LogLevelNotFound)?;

        let log_level =
            Level::from_str(&log_level_str).map_err(|_| ParseLogLevelError(log_level_str))?;

        Ok(BotConfig {
            tg_token,
            gigachat_client_id,
            gigachat_client_secret,
            mistral_token,
            log_level,
        })
    }
}
