use crate::errors::ApiError;
use crate::handlers::root_handler::StickerStore;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, msg, sticker_store))]
pub async fn delete_sticker(
    bot: Bot,
    msg: Message,
    sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Пожалуйста, укажите название стикера")
            .await?;
        return Ok(());
    }

    match sticker_store.remove_sticker(sticker_name.as_str()).await {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                format!("Стикер {} успешно удален", sticker_name),
            )
            .await?;
        }

        Err(ApiError::StickerNotFound) => {
            bot.send_message(
                msg.chat.id,
                format!("Стикер с названием {} не найден", sticker_name),
            )
            .await?;
        }

        Err(e) => {
            error!(err = %e, "Failed to handle sticker deletion");

            bot.send_message(
                msg.chat.id,
                format!("Произошла ошибка удаления стикера: {}", e),
            )
            .await?;
        }
    }

    Ok(())
}
