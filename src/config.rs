use dotenvy::dotenv;
use std::env;

pub struct BotConfig {
    pub tg_token: String,
    pub gigachat_client_id: String,
    pub gigachat_client_secret: String
}

impl BotConfig {
    pub fn new() -> Self {
        dotenv().ok();
        let tg_token = env::var("TELOXIDE_TOKEN")
            .expect("Bot token not found in .env file");

        let gigachat_client_id = env::var("GIGACHAT_CLIENT_ID")
            .expect("GIGACHAT_CLIENT_ID token not found in .env file");

        let gigachat_client_secret = env::var("GIGACHAT_CLIENT_SECRET")
            .expect("GIGACHAT_CLIENT_ID token not found in .env file");

        BotConfig {
            tg_token,
            gigachat_client_id,
            gigachat_client_secret
        }
    }
}