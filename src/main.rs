mod adapter;
mod commands;
mod providers;
mod config;
mod constants;
mod content_rephraser;
mod errors;
mod generation_controller;
mod handlers;
mod repo;
mod states;
mod utils;

use crate::adapter::postgres::PgStore;
use crate::commands::Command;
use crate::config::BotConfig;
use crate::generation_controller::{ContentRephraser, GenerationController, ModelPool};
use crate::handlers::root_handler::{
    ContentGenerator, DialogueStore, MediaStore, MessageStore, handle_command,
};
use crate::handlers::slay::inline_choice_callback;
use crate::handlers::state_dispatcher::state_dispatcher;
use crate::repo::dialogue_storage::UserDialogueStorage;
use crate::repo::media_storage_postgres::storage::PGMediaStorage;
use crate::repo::message_history_storage::MessageHistoryStorage;
use std::process;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};
use crate::adapter::generation::open_ai_sdk::OpenAIClient;
use crate::providers::Model;

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


    let listener = webhooks::axum(bot.clone(), cfg.webhook_options)
        .await
        .expect("Failed to start webhook listener");

    let gemini_gen = Arc::new(OpenAIClient::new(
        Model::GEMINI_MODEL,
        cfg.gemini_token.clone(),
    )) as Arc<dyn ContentRephraser>;

    let gemini_2_5_gen = Arc::new(OpenAIClient::new(
        Model::GEMINI_2_5_FLASH_LITE_MODEL,
        cfg.gemini_token,
    )) as Arc<dyn ContentRephraser>;

    let grok_gen = Arc::new(OpenAIClient::new(
        Model::GROK_MODEL,
        cfg.grok_token,
    )) as Arc<dyn ContentRephraser>;


    let pg_pool = match PgStore::new(cfg.db_conn_str.as_str()).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error happened configuring sticker storage: {}", e);
            process::exit(1);
        }
    };

    let media_storage = Arc::new(PGMediaStorage::new(pg_pool)) as Arc<dyn MediaStore>;

    let message_history_storage = Arc::new(MessageHistoryStorage::new()) as Arc<dyn MessageStore>;

    let model_pool = ModelPool::from(vec![gemini_gen, gemini_2_5_gen, grok_gen]);

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
        .dispatch_with_listener(listener, LoggingErrorHandler::with_custom_text("Error from webhook listener"))
        .await;
}
