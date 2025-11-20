mod config;
mod commands;
mod handlers;
mod utils;
mod errors;
mod constants;
mod gigachat_api;

use std::process;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use crate::config::BotConfig;
use crate::commands::Command;
use crate::handlers::{handle_command, ContentGenerator};
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::gigachat_api::api::GigaChatApi;

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
    let gigachat_generator = match GigaChatApi::new(cfg.gigachat_client_id, cfg.gigachat_client_secret) {
        Ok(generator) => generator,
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let dyn_generator: Arc<dyn ContentGenerator> = Arc::new(gigachat_generator);

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
        .dependencies(dptree::deps![dyn_generator, generator_limiter])
        .enable_ctrlc_handler()
        .default_handler(|_upd| async {})
        .build()
        .dispatch()
        .await;
}
