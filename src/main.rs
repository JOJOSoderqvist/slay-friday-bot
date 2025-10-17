mod config;
mod commands;
mod handlers;
mod scheduler;
mod utils;

use crate::config::BotConfig;
use crate::commands::Command;
use crate::handlers::handle_command;
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting SlayFridayBot...");

    let cfg = BotConfig::new();
    let bot = Bot::new(cfg.token);


    // Commands dispatcher
    let handler = dptree::entry()
        .branch(Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
