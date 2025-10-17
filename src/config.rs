use dotenvy::dotenv;
use std::env;

pub(crate) struct BotConfig {
    pub(crate) token: String
}

impl BotConfig {
    pub(crate) fn new() -> Self {
        dotenv().ok();
        let token = env::var("TELOXIDE_TOKEN")
            .expect("Bot token not found in .env file");

        BotConfig {
            token
        }
    }
}