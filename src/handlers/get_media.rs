use crate::errors::ApiError;
use crate::handlers::root_handler::MediaStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile};

pub async fn get_media(
    bot: Bot,
    msg: Message,
    media_entry_name: String,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    if media_entry_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Укажите имя медиа файла".to_string())
            .await?;
        return Ok(());
    }

    match media_store.get_media_entry(media_entry_name.as_str()).await {
        Some(entry) => {
            bot.send_sticker(msg.chat.id, InputFile::file_id(FileId(entry.file_id)))
                .await?;
        }
        None => {
            debug!("Media with name '{}' not found", media_entry_name);
            bot.send_message(msg.chat.id, "Медиа с таким названием нет")
                .await?;
        }
    }

    Ok(())
}
