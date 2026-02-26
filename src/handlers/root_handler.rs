use crate::commands::Command;
use crate::common::Model;
use crate::errors::ApiError;
use crate::handlers::add_sticker::trigger_add;
use crate::handlers::delete_sticker::delete_sticker;
use crate::handlers::friday::friday;
use crate::handlers::get_sticker::get_sticker;
use crate::handlers::list_stickers::list_stickers;
use crate::handlers::model_info::model_info;
use crate::handlers::rename_sticker::trigger_rename;
use crate::handlers::slay::slay;
use crate::repo::dialogue_storage::DialogueStorageKey;
use crate::repo::message_history_storage::HistoryEntry;
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use async_trait::async_trait;
use log::info;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::{Message, Requester};
use teloxide::utils::command::BotCommands;
use tracing::instrument;

#[async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_text(&self, current_text: &str) -> Result<(String, Model), ApiError>;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn add_message(&self, message: HistoryEntry);
    async fn get_message_info(&self, message: &str) -> Option<Model>;
}

#[async_trait]
pub trait StickerStore: Send + Sync {
    async fn add_sticker(&self, sticker: StickerEntry) -> Result<(), ApiError>;
    async fn get_sticker(&self, sticker_name: &str) -> Option<StickerEntry>;
    async fn rename_sticker(&self, old_name: &str, new_name: &str) -> Result<(), ApiError>;
    async fn list_stickers(&self) -> Option<Vec<StickerEntry>>;
    async fn remove_sticker(&self, sticker_name: &str) -> Result<(), ApiError>;
    async fn is_already_created(&self, sticker_name: &str) -> bool;
}

pub trait DialogueStore: Send + Sync {
    fn get_dialogue(&self, key: DialogueStorageKey) -> Option<State>;
    fn remove_dialogue(&self, key: DialogueStorageKey) -> Option<(DialogueStorageKey, State)>;
    fn update_dialogue(&self, key: DialogueStorageKey, new_state: State) -> Option<State>;
}

#[instrument(skip(bot, generator, cmd, msg, sticker_store, message_store, dialogue))]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    generator: Arc<dyn ContentGenerator>,
    sticker_store: Arc<dyn StickerStore>,
    message_store: Arc<dyn MessageStore>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    match cmd {
        Command::Help => help(bot, msg).await?,

        Command::Friday => friday(bot, msg, generator, message_store).await?,

        Command::Model => model_info(bot, msg, message_store).await?,

        Command::ListStickers => list_stickers(bot, msg, sticker_store).await?,

        Command::AddSticker => trigger_add(bot, msg, dialogue).await?,

        Command::Cancel => cancel(bot, msg, dialogue).await?,

        Command::Sticker(name) => get_sticker(bot, msg, name, sticker_store).await?,

        Command::RenameSticker => trigger_rename(bot, msg, dialogue).await?,

        Command::DeleteSticker => delete_sticker(bot, msg, dialogue, sticker_store).await?,
        Command::Slay => slay(bot, msg, dialogue).await?,
    }

    Ok(())
}

#[instrument(skip(bot, msg))]
pub async fn help(bot: Bot, msg: Message) -> Result<(), ApiError> {
    info!("Help command");
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, msg: Message, dialogue: Arc<dyn DialogueStore>) -> Result<(), ApiError> {
    let key = (msg.from.unwrap().id, msg.chat.id);
    if dialogue.get_dialogue(key).is_some() {
        dialogue.remove_dialogue(key);
        bot.send_message(msg.chat.id, "Операция отменена.").await?;
    }
    Ok(())
}
