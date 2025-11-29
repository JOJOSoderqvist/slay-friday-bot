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
use crate::generation_controller::{
    ContentRephraser, GenerationController, MessageStore, ModelPool,
};
use crate::gigachat_api::api::GigaChatApi;
use crate::grok_api::api::GrokApi;
use crate::handlers::{ContentGenerator, StickerStore, handle_command};
use crate::mistral_api::api::MistralApi;
use crate::repo::message_history_storage::MessageHistoryStorage;
use crate::repo::sticker_storage::storage::StickerStorage;
use crate::states::State;
use std::process;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
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

    let sticker_storage = match StickerStorage::new("sticker_storage.json".to_string()).await {
        Ok(storage) => Arc::new(storage) as Arc<dyn StickerStore>,
        Err(e) => {
            eprintln!("error happened configuring sticker storage: {}", e);
            process::exit(1);
        }
    };

    let message_history_storage = Arc::new(MessageHistoryStorage::new()) as Arc<dyn MessageStore>;

    let model_pool = ModelPool::from(vec![gigachat_generator, mistral_generator, grok_generator]);

    let generation_controller = Arc::new(GenerationController::new(
        model_pool,
        message_history_storage,
    )) as Arc<dyn ContentGenerator>;

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(fmt::layer());

    subscriber.init();


    type MyDialogue = Dialogue<State, InMemStorage<State>>;
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(handle_command),
                )
                .branch(
                    dptree::filter_map_async(|dialogue: MyDialogue| async move {
                        dialogue.get().await.ok().flatten()
                    })
                        .branch(
                            case![State::ReceiveSticker { name }]
                                .endpoint(handlers::receive_sticker),
                        ),
                ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            generation_controller,
            sticker_storage,
            InMemStorage::<State>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
