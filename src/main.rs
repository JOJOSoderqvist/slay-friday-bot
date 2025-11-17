mod config;
mod commands;
mod handlers;
mod scheduler;
mod utils;
mod gigachat;
mod errors;
mod dto;
mod constants;

use std::sync::Arc;
use crate::config::BotConfig;
use crate::commands::Command;
use crate::handlers::handle_command;
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;
use crate::gigachat::GigaChatApi;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting SlayFridayBot...");

    let cfg = BotConfig::new();
    let bot = Bot::new(cfg.tg_token);
    let generator = GigaChatApi::new(cfg.gigachat_client_id, cfg.gigachat_client_secret);


    // Commands dispatcher
    let handler = dptree::entry()
        .branch(Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(generator)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
