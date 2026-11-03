mod adapter;
mod commands;
mod common;
mod config;
mod constants;
mod errors;
mod generation_controller;
mod grok_api;
mod handlers;
mod mistral_api;
mod repo;
mod states;
mod utils;

use crate::adapter::postgres::PgStore;
use crate::commands::Command;
use crate::config::BotConfig;
use crate::generation_controller::{ContentRephraser, GenerationController, ModelPool};
use crate::grok_api::api::GrokApi;
use crate::handlers::root_handler::{
    ContentGenerator, DialogueStore, MediaStore, MessageStore, handle_command,
};
use crate::handlers::slay::inline_choice_callback;
use crate::handlers::state_dispatcher::state_dispatcher;
use crate::mistral_api::api::MistralApi;
use crate::repo::dialogue_storage::UserDialogueStorage;
use crate::repo::media_storage_postgres::storage::PGMediaStorage;
use crate::repo::message_history_storage::MessageHistoryStorage;
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

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(fmt::layer().with_ansi(false))
        .init();

    let bot = Bot::new(cfg.tg_token);

    let mistral_generator =
        Arc::new(MistralApi::new(cfg.mistral_token)) as Arc<dyn ContentRephraser>;

    let grok_generator = match GrokApi::new(cfg.grok_token) {
        Ok(generator) => Arc::new(generator) as Arc<dyn ContentRephraser>,
        Err(e) => {
            eprintln!("error happened configuring generator: {}", e);
            process::exit(1);
        }
    };

    let pg_pool = match PgStore::new(cfg.db_conn_str.as_str()).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error happened configuring sticker storage: {}", e);
            process::exit(1);
        }
    };

    let media_storage = Arc::new(PGMediaStorage::new(pg_pool)) as Arc<dyn MediaStore>;

    let message_history_storage = Arc::new(MessageHistoryStorage::new()) as Arc<dyn MessageStore>;

    let model_pool = ModelPool::from(vec![mistral_generator, grok_generator]);

    let generation_controller =
        Arc::new(GenerationController::new(model_pool)) as Arc<dyn ContentGenerator>;

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
