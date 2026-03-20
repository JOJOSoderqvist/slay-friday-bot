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
mod repo;
mod states;
mod utils;

use crate::commands::Command;
use crate::config::BotConfig;
use crate::generation_controller::{ContentRephraser, GenerationController, ModelPool};
use crate::gigachat_api::api::GigaChatApi;
use crate::grok_api::api::GrokApi;
use crate::handlers::root_handler::{
    ContentGenerator, DialogueStore, MediaStore, MessageStore, handle_command,
};
use crate::handlers::slay::inline_choice_callback;
use crate::handlers::state_dispatcher::state_dispatcher;
use crate::mistral_api::api::MistralApi;
use crate::repo::dialogue_storage::UserDialogueStorage;
use crate::repo::media_storage::storage::MediaStorage;
use crate::repo::message_history_storage::MessageHistoryStorage;
use crate::utils::get_client_with_proxy;
use std::process;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};
use url::Url;

#[tokio::main]
async fn main() {
    let cfg = match BotConfig::new() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("error happened configuring BOT: {}", e);
            process::exit(1);
        }
    };

    let custom_proxy_client = match get_client_with_proxy(cfg.proxy_url.as_str()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error happened configuring custom client: {}", e);
            process::exit(1);
        }
    };

    let bot = Bot::with_client(cfg.tg_token, custom_proxy_client.clone());

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

    let grok_generator = match GrokApi::new(cfg.grok_token, custom_proxy_client.clone()) {
        Ok(generator) => Arc::new(generator) as Arc<dyn ContentRephraser>,
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let media_storage = match MediaStorage::new("sticker_storage.json".to_string()).await {
        Ok(storage) => Arc::new(storage) as Arc<dyn MediaStore>,
        Err(e) => {
            eprintln!("error happened configuring sticker storage: {}", e);
            process::exit(1);
        }
    };

    let message_history_storage = Arc::new(MessageHistoryStorage::new()) as Arc<dyn MessageStore>;

    let model_pool = ModelPool::from(vec![gigachat_generator, mistral_generator, grok_generator]);

    let generation_controller =
        Arc::new(GenerationController::new(model_pool)) as Arc<dyn ContentGenerator>;

    let (loki_layer, task) = match tracing_loki::builder()
        .label("service_name", "slay-friday-bot")
        .unwrap()
        .build_url(Url::parse("http://loki:3100").unwrap())
    {
        Ok((layer, task)) => (layer, task),
        Err(e) => {
            eprintln!("error happened configuring loki: {}", e);
            process::exit(1);
        }
    };

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(loki_layer)
        .with(fmt::layer());

    subscriber.init();
    tokio::spawn(task);

    let command_handler = dptree::entry()
        .filter_command::<Command>()
        .endpoint(handle_command);

    let dialogue_store = Arc::new(UserDialogueStorage::new()) as Arc<dyn DialogueStore>;

    let callback_handler = Update::filter_callback_query().endpoint(inline_choice_callback);

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .endpoint(state_dispatcher);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            generation_controller,
            media_storage,
            message_history_storage,
            dialogue_store
        ])
        .enable_ctrlc_handler()
        .default_handler(|_upd| async {})
        .build()
        .dispatch()
        .await;
}
