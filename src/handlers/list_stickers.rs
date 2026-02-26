use crate::errors::ApiError;
use crate::handlers::root_handler::StickerStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tracing::instrument;

#[instrument(skip(bot, msg, sticker_store))]
pub async fn list_stickers(
    bot: Bot,
    msg: Message,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    match sticker_store.list_stickers().await {
        Some(entries) => {
            let mut names: Vec<String> = entries.into_iter().map(|e| e.name).collect();

            names.sort();

            names.iter_mut().for_each(|name| {
                *name = format!("`{name}`");
            });

            bot.send_message(
                msg.chat.id,
                format!("Доступные стикеры:\n{}", names.join("\n")),
            )
            .parse_mode(ParseMode::MarkdownV2)
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
