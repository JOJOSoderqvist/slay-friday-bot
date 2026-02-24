use crate::errors::ApiError;
use crate::handlers::root_handler::StickerStore;
use crate::utils::reply_suggestions_keyboard;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::instrument;

#[instrument(skip(bot, msg, sticker_store))]
pub async fn list_stickers(
    bot: Bot,
    msg: Message,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    match sticker_store.list_stickers().await {
        Some(mut entries) => {
            entries.sort();

            let keyboard = reply_suggestions_keyboard(entries.as_slice(), None);

            bot.send_message(msg.chat.id, "Доступные стикеры:".to_string())
                .reply_markup(keyboard)
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
