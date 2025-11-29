use crate::commands::Command;
use crate::common::Model;
use crate::constants::STICKERS_MAP;
use crate::errors::ApiError;
use crate::errors::ApiError::{DialogueStorageError, TelegramError};
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use crate::utils::{format_time_delta, get_time_until_friday, parse_sticker_name};
use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile};
use teloxide::utils::command::BotCommands;
use tracing::field::debug;
use tracing::{error, instrument};

#[async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_text(&self, current_text: &str) -> Result<String, ApiError>;

    async fn get_message_info(&self, text: &str) -> Option<Model>;
}

#[async_trait]
pub trait StickerStore: Send + Sync {
    // TODO: error type?
    async fn add_sticker(&self, sticker: StickerEntry) -> Result<(), ApiError>;
    // Option??
    async fn get_sticker(&self, sticker_name: &str) -> Option<StickerEntry>;
    async fn rename_sticker(&self, old_name: &str, new_name: &str) -> Result<(), ApiError>;
    async fn list_stickers(&self) -> Option<Vec<StickerEntry>>;
    async fn remove_sticker(&self, sticker_name: &str) -> Result<(), ApiError>;
}

#[instrument(skip(bot, generator, cmd, msg, sticker_store, dialogue))]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    generator: Arc<dyn ContentGenerator>,
    sticker_store: Arc<dyn StickerStore>,
    dialogue: MyDialogue,
) -> Result<(), ApiError> {
    match cmd {
        Command::Help => handle_help(bot, msg).await?,

        Command::Friday => handle_friday(bot, msg, generator).await?,

        Command::Model => handle_model_info(bot, msg, generator).await?,

        Command::ListStickers => handle_list_stickers(bot, msg, sticker_store).await?,

        Command::AddSticker(name) => handle_add_sticker_command(bot, msg, dialogue, name).await?,

        Command::Cancel => handle_cancel(bot, msg, dialogue).await?,

        Command::Sticker(name) => handle_get_sticker(bot, msg, name, sticker_store).await?,
    }

    Ok(())
}

#[instrument(skip(bot, msg))]
async fn handle_help(bot: Bot, msg: Message) -> Result<(), ApiError> {
    info!("Help command");
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

#[instrument(skip(bot, msg, generator))]
async fn handle_friday(
    bot: Bot,
    msg: Message,
    generator: Arc<dyn ContentGenerator>,
) -> Result<(), ApiError> {
    let text = if let Some(time_left) = get_time_until_friday() {
        format!(
            "До нефорской пятницы осталось: {} 🕷️ Готовь свой лучший аутфит. ⛓️",
            format_time_delta(time_left)
        )
    } else {
        String::from("SLAAAAAY! 💅🔥🖤 ЭТО НЕФОРСКАЯ ПЯТНИЦА, ДЕТКА! 🤘😈⛓️ Время сиять! ✨")
    };

    match generator.generate_text(text.as_str()).await {
        Ok(new_text) => {
            bot.send_message(msg.chat.id, new_text).await?;
        }
        Err(err) => {
            error!(error = %err, "Failed to rephrase text");
            bot.send_message(msg.chat.id, text).await?;
        }
    }

    Ok(())
}

async fn handle_model_info(
    bot: Bot,
    msg: Message,
    generator: Arc<dyn ContentGenerator>,
) -> Result<(), ApiError> {
    let reply_msg = match msg.reply_to_message() {
        Some(m) => m,
        None => {
            bot.send_message(msg.chat.id, "Команда должна быть ответом на сообщение бота")
                .await?;
            return Ok(());
        }
    };

    let text = match reply_msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "Это сообщение не сгенерировано ботом")
                .await?;
            return Ok(());
        }
    };

    match generator.get_message_info(text).await {
        Some(model) => {
            bot.send_message(
                msg.chat.id,
                format!("Это сообщение сгенерировано: {}", model),
            )
            .await?;
        }
        None => {
            debug!("No entry found in storage");
            bot.send_message(msg.chat.id, "Информации про это сообщение не найдено")
                .await?;
        }
    }

    Ok(())
}

async fn handle_get_sticker(
    bot: Bot,
    msg: Message,
    sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    match sticker_store.get_sticker(sticker_name.as_str()).await {
        Some(entry) => {
            bot.send_sticker(msg.chat.id, InputFile::file_id(FileId(entry.file_id)))
                .await?;
        }
        None => {
            debug!("Sticker with name '{}' not found", sticker_name);
            bot.send_message(msg.chat.id, "Стикера с таким названием нет")
                .await?;
        }
    }

    Ok(())
}
#[instrument(skip(bot, msg, sticker_store))]
async fn handle_list_stickers(
    bot: Bot,
    msg: Message,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    match sticker_store.list_stickers().await {
        Some(entries) => {
            let names: Vec<String> = entries.into_iter().map(|e| e.name).collect();

            bot.send_message(
                msg.chat.id,
                format!("Доступные стикеры:\n{}", names.join("\n")),
            )
            .await?;
        }
        None => {
            debug!("No stickers in storage");
            bot.send_message(msg.chat.id, "Список стикеров пуст")
                .await?;
        }
    }

    Ok(())
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[instrument(skip(bot, msg, dialogue, sticker_name))]
async fn handle_add_sticker_command(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    sticker_name: String,
) -> Result<(), ApiError> {
    if sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Пожалуйста, укажите название: /add <name>")
            .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        format!("Отправь мне стикер для '{}'", sticker_name),
    )
    .await?;

    dialogue
        .update(State::ReceiveSticker { name: sticker_name })
        .await
        .map_err(DialogueStorageError)?;

    Ok(())
}

async fn handle_cancel(bot: Bot, msg: Message, dialogue: MyDialogue) -> Result<(), ApiError> {
    bot.send_message(msg.chat.id, "Операция отменена.").await?;
    dialogue.exit().await.map_err(DialogueStorageError)?;
    Ok(())
}

pub async fn receive_sticker(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    (name): (String),
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if let Some(sticker) = msg.sticker() {
        let entry = StickerEntry::new(name.clone(), sticker.file.id.clone().to_string());

        match sticker_store.add_sticker(entry).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, format!("Стикер '{}' сохранен! 🎉", name))
                    .await?;

                dialogue.exit().await?;
            }
            Err(ApiError::StickerAlreadyExists) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Стикер '{}' уже существует. Попробуйте другое имя", name),
                )
                .await?;

                dialogue.exit().await?;
            }

            Err(e) => {
                error!(err = %e, "Failed to handle sticker creation");

                bot.send_message(msg.chat.id, format!("Error saving sticker: {}", e))
                    .await?;
                dialogue.exit().await?;
            }
        }
    } else {
        debug!("Not a sticker");

        bot.send_message(msg.chat.id, "Это не стикер. Отправьте стикер или /cancel.")
            .await?;
    }
    Ok(())
}
