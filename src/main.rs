mod commands;
mod common;
mod config;
mod constants;
mod errors;
mod generation_controller;
mod gigachat_api;
mod grok_api;
mod handlers;
mod mistral_api;
mod utils;

use crate::commands::Command;
use crate::config::BotConfig;
use crate::generation_controller::{ContentRephraser, GenerationController, ModelPool};
use crate::gigachat_api::api::GigaChatApi;
use crate::grok_api::api::GrokApi;
use crate::handlers::{ContentGenerator, handle_command};
use crate::mistral_api::api::MistralApi;
use std::process;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

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
    let gigachat_generator =
        match GigaChatApi::new(cfg.gigachat_client_id, cfg.gigachat_client_secret) {
            Ok(generator) => Arc::new(generator) as Arc<dyn ContentRephraser>,
            Err(e) => {
                eprintln!("error happened configuring generator: {}", e);
                process::exit(1);
            }
        };

    let mistral_generator =
        Arc::new(MistralApi::new(cfg.mistral_token)) as Arc<dyn ContentRephraser>;

    let grok_generator = match GrokApi::new(cfg.grok_token, cfg.proxy_url) {
        Ok(generator) => Arc::new(generator) as Arc<dyn ContentRephraser>,
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let model_pool = ModelPool::from(vec![gigachat_generator, mistral_generator, grok_generator]);

    let generation_controller =
        Arc::new(GenerationController::new(model_pool)) as Arc<dyn ContentGenerator>;

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(fmt::layer());

    subscriber.init();

    // Commands dispatcher
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![generation_controller])
        .enable_ctrlc_handler()
        .default_handler(|_upd| async {})
        .build()
        .dispatch()
        .await;
}
