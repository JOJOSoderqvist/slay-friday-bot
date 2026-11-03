use crate::errors::BotConfigError;
use crate::errors::BotConfigError::{
    BotTokenNotFound, DBURLNotFound, LogLevelNotFound, MistralTokenNotFound, ParseLogLevelError,
    XAITokenNotFound,
};
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;
use tracing::Level;

pub struct BotConfig {
    pub tg_token: String,
    pub mistral_token: String,
    pub grok_token: String,
    pub log_level: Level,
    pub db_conn_str: String,
}

impl BotConfig {
    pub fn new() -> Result<Self, BotConfigError> {
        dotenv().ok();
        let tg_token = env::var("TELOXIDE_TOKEN").map_err(BotTokenNotFound)?;

        let grok_token = env::var("XAI_API_KEY").map_err(XAITokenNotFound)?;

        let mistral_token = env::var("MISTRAL_TOKEN").map_err(MistralTokenNotFound)?;

        let log_level_str = env::var("LOG_LEVEL").map_err(LogLevelNotFound)?;

        let log_level =
            Level::from_str(&log_level_str).map_err(|_| ParseLogLevelError(log_level_str))?;

        let db_conn_str = env::var("DATABASE_URL").map_err(DBURLNotFound)?;

        Ok(BotConfig {
            tg_token,
            mistral_token,
            grok_token,
            log_level,
            db_conn_str,
        })
    }
}
