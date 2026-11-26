use crate::errors::BotConfigError;
use crate::errors::BotConfigError::{
    BotTokenNotFound, DBURLNotFound, FailedToParseWebHookURL, GeminiTokenNotFound, InvalidPort,
    LogLevelNotFound, ParseLogLevelError, ReadPortError, WebHookURLNotFound, XAITokenNotFound,
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::str::FromStr;
use teloxide::update_listeners::webhooks::Options;
use tracing::Level;
use url::Url;

pub struct BotConfig {
    pub tg_token: String,
    pub grok_token: String,
    pub gemini_token: String,
    pub log_level: Level,
    pub db_conn_str: String,
    pub webhook_options: Options,
}

impl BotConfig {
    pub fn new() -> Result<Self, BotConfigError> {
        dotenv().ok();
        let tg_token = env::var("TELOXIDE_TOKEN").map_err(BotTokenNotFound)?;

        let grok_token = env::var("XAI_API_KEY").map_err(XAITokenNotFound)?;

        let gemini_token = env::var("GEMINI_API_KEY").map_err(GeminiTokenNotFound)?;

        let log_level_str = env::var("LOG_LEVEL").map_err(LogLevelNotFound)?;

        let log_level =
            Level::from_str(&log_level_str).map_err(|_| ParseLogLevelError(log_level_str))?;

        let db_conn_str = env::var("DATABASE_URL").map_err(DBURLNotFound)?;

        let port = match env::var("PORT") {
            Ok(port) => port.parse::<NonZeroU16>().map_err(InvalidPort)?.get(),
            Err(env::VarError::NotPresent) => 8080,
            Err(err) => return Err(ReadPortError(err)),
        };
        let addr: SocketAddr = ([0, 0, 0, 0], port).into();
        let url = env::var("WEBHOOK_APP_URL").map_err(WebHookURLNotFound)?;
        let url = Url::parse(&url).map_err(FailedToParseWebHookURL)?;

        let webhook_options = Options::new(addr, url);

        Ok(BotConfig {
            tg_token,
            grok_token,
            gemini_token,
            log_level,
            db_conn_str,
            webhook_options,
        })
    }
}
