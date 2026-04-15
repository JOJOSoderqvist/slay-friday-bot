use crate::errors::ApiError;
use crate::handlers::root_handler::MediaStore;
use crate::handlers::utils::get_user_id_from_option;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile};
use tracing::{debug, error};

pub async fn get_media(
    bot: Bot,
    msg: Message,
    media_entry_name: String,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    let Some(user_id) = get_user_id_from_option(&msg.from) else {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    };

    if media_entry_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Укажите имя медиа файла".to_string())
            .await?;
        return Ok(());
    }

    match media_store
        .get_media_entry(media_entry_name.as_str(), user_id)
        .await
    {
        Ok(Some(entry)) => {
            bot.send_sticker(msg.chat.id, InputFile::file_id(FileId(entry.file_id)))
                .await?;
        }
        Ok(None) => {
            debug!("Media with name '{}' not found", media_entry_name);
            bot.send_message(msg.chat.id, "Медиа с таким названием нет")
                .await?;
        }

        Err(e) => {
            bot.send_message(msg.chat.id, "Не удалось получить стикер")
                .await?;
            error!(error = %e, "Failed to get sticker");
        }
    }

    Ok(())
}
