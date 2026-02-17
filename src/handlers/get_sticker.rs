use crate::errors::ApiError;
use crate::handlers::root_handler::StickerStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile};

pub async fn get_sticker(
    bot: Bot,
    msg: Message,
    sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Укажите имя стикера".to_string())
            .await?;
        return Ok(());
    }

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
