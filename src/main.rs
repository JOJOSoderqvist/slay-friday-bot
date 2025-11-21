mod config;
mod commands;
mod handlers;
mod utils;
mod errors;
mod constants;
mod gigachat_api;
mod mistral_api;
mod generation_controller;

use std::process;
use std::sync::Arc;
use crate::config::BotConfig;
use crate::commands::Command;
use crate::handlers::{handle_command, ContentGenerator};
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::generation_controller::{ContentRephraser, GenerationController};
use crate::gigachat_api::api::GigaChatApi;
use crate::mistral_api::api::MistralApi;

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
        Ok(generator) => Arc::new(generator) as Arc<dyn ContentRephraser>,
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let mistral_generator = Arc::new(MistralApi::new(cfg.mistral_token))
        as Arc<dyn ContentRephraser>;

    let generation_controller =
        Arc::new(GenerationController::new(vec![gigachat_generator, mistral_generator])) as Arc<dyn ContentGenerator>;

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
        .dependencies(dptree::deps![generation_controller])
        .enable_ctrlc_handler()
        .default_handler(|_upd| async {})
        .build()
        .dispatch()
        .await;
}
