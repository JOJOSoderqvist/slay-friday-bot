use crate::errors::ApiError;
use crate::handlers::root_handler::MediaStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tracing::instrument;

#[instrument(skip(bot, chat_id, media_store))]
pub async fn list_media(
    bot: Bot,
    chat_id: ChatId,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    match media_store.list_available_media_entries().await {
        Some(entries) => {
            let mut names: Vec<String> = entries.into_iter().map(|e| e.name).collect();

            names.sort();

            names.iter_mut().for_each(|name| {
                *name = format!("`{name}`");
            });

            bot.send_message(
                chat_id,
                format!("Доступные медиафайлы:\n{}", names.join("\n")),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }
        None => {
            debug!("No media in storage");
            bot.send_message(chat_id, "Список медиафайлов пуст").await?;
        }
    }

    Ok(())
}
