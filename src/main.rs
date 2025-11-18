mod config;
mod commands;
mod handlers;
mod scheduler;
mod utils;
mod gigachat;
mod errors;
mod dto;
mod constants;

use std::process;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use crate::config::BotConfig;
use crate::commands::Command;
use crate::handlers::handle_command;
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::gigachat::GigaChatApi;

#[tokio::main]
async fn main() {
    let cfg = match BotConfig::new() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("error happened configuring BOT: {}", e);
            process::exit(1);
        }
    };

    let bot = Bot::new(cfg.tg_token);
    let generator = match GigaChatApi::new(cfg.gigachat_client_id, cfg.gigachat_client_secret) {
        Ok(generator) => Arc::new(generator),
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let generator_limiter = Arc::new(AtomicUsize::new(0));

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(fmt::layer());

    subscriber.init();


    // Commands dispatcher
    let handler = dptree::entry()
        .branch(Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![generator, generator_limiter])
        .enable_ctrlc_handler()
        .default_handler(|_upd| async {})
        .build()
        .dispatch()
        .await;
}
