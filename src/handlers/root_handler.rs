use crate::commands::Command;
use crate::common::Model;
use crate::errors::ApiError;
use crate::handlers::add_media::trigger_add;
use crate::handlers::delete_media::trigger_delete;
use crate::handlers::friday::friday;
use crate::handlers::get_media::get_media;
use crate::handlers::list_available_media::list_default;
use crate::handlers::model_info::model_info;
use crate::handlers::rename_media::trigger_rename;
use crate::handlers::slay::slay;
use crate::repo::dialogue_storage::DialogueStorageKey;
use crate::repo::media_storage_postgres::dto::MediaEntry;
use crate::repo::message_history_storage::HistoryEntry;
use crate::states::State;
use async_trait::async_trait;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::{Message, Requester, UserId};
use teloxide::types::ChatId;
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
pub trait MediaStore: Send + Sync {
    async fn add_media_entry(&self, media_entry: MediaEntry) -> Result<(), ApiError>;
    async fn get_media_entry(
        &self,
        media_entry_name: &str,
        user_id: UserId,
    ) -> Result<Option<MediaEntry>, ApiError>;
    async fn rename_media_entry(
        &self,
        old_entry_name: &str,
        new_entry_name: &str,
    ) -> Result<(), ApiError>;
    async fn list_available_media_entries(&self) -> Result<Vec<MediaEntry>, ApiError>;
    async fn list_user_specific_media_entries(
        &self,
        user_id: UserId,
    ) -> Result<Vec<MediaEntry>, ApiError>;

    async fn remove_media_entry(&self, media_entry_name: &str) -> Result<bool, ApiError>;
    async fn is_already_created(&self, media_entry_name: &str) -> Result<bool, ApiError>;
}

pub trait DialogueStore: Send + Sync {
    fn get_dialogue(&self, key: &DialogueStorageKey) -> Option<State>;
    fn remove_dialogue(&self, key: &DialogueStorageKey) -> Option<(DialogueStorageKey, State)>;
    fn update_dialogue(&self, key: DialogueStorageKey, new_state: State) -> Option<State>;
}

#[instrument(skip(bot, generator, cmd, msg, media_store, message_store, dialogue))]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    generator: Arc<dyn ContentGenerator>,
    media_store: Arc<dyn MediaStore>,
    message_store: Arc<dyn MessageStore>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    match cmd {
        Command::Help => help(bot, msg.chat.id).await?,

        Command::Friday => friday(bot, msg.chat.id, generator, message_store).await?,

        Command::Model => model_info(bot, msg, message_store).await?,

        Command::ListMedia => list_default(bot, msg.chat.id, media_store).await?,

        Command::AddMedia => trigger_add(bot, msg.chat.id, msg.from, dialogue).await?,

        Command::Cancel => cancel(bot, msg, dialogue).await?,

        Command::GetMedia(name) => get_media(bot, msg, name, media_store).await?,

        Command::RenameMedia => trigger_rename(bot, msg.chat.id, msg.from, dialogue).await?,

        Command::DeleteMedia => trigger_delete(bot, msg.chat.id, msg.from, dialogue).await?,
        Command::Slay => slay(bot, msg.chat.id, msg.from).await?,
    }

    Ok(())
}

#[instrument(skip(bot, chat_id))]
pub async fn help(bot: Bot, chat_id: ChatId) -> Result<(), ApiError> {
    bot.send_message(chat_id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, msg: Message, dialogue: Arc<dyn DialogueStore>) -> Result<(), ApiError> {
    let key = (msg.from.unwrap().id, msg.chat.id);
    if dialogue.get_dialogue(&key).is_some() {
        dialogue.remove_dialogue(&key);
        bot.send_message(msg.chat.id, "Операция отменена.").await?;
    }
    Ok(())
}
