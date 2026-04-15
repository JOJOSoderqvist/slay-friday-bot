use crate::errors::ApiError;
use crate::handlers::root_handler::MediaStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, User};
use tracing::{error, instrument};
use crate::handlers::utils::get_user_id_from_option;
use crate::repo::media_storage_postgres::dto::MediaEntry;

#[instrument(skip(bot, chat_id, media_store))]
pub async fn list_default(
    bot: Bot,
    chat_id: ChatId,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    match media_store.list_available_media_entries().await {
        Ok(mut entries) => {
            if entries.is_empty() {
                debug!("No media in storage");
                bot.send_message(chat_id, "Список медиафайлов пуст").await?;
                return Ok(())
            }

            let names: Vec<String> = entries.into_iter()
                .map(|e| {
                    format!("`{}`", e.name)
                })
                .collect();

            bot.send_message(
                chat_id,
                format!("Доступные медиафайлы:\n{}", names.join("\n")),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }

        Err(e) => {
            bot.send_message(chat_id, "Произошла ошибка получения медиафайлов").await?;
            error!(error = %e, "Failed to get mediafiles");
            return Err(e);
        }
    }

    Ok(())
}

pub async fn list_for_user(
    bot: Bot,
    from: Option<User>,
    chat_id: ChatId,
    media_store: Arc<dyn MediaStore>,
) -> Result<Option<Vec<MediaEntry>>, ApiError> {
    let Some(user_id) = get_user_id_from_option(&from) else {
        bot.send_message(chat_id, "Каналы не поддерживаются")
            .await?;
        return Ok(None);
    };

    match media_store.list_user_specific_media_entries(user_id).await {
        Ok(entries) => {
            if entries.is_empty() {
                debug!("No media in storage");
                bot.send_message(chat_id, "Список медиафайлов пуст").await?;
                return Ok(None)
            }

            Ok(Some(entries))
        }

        Err(e) => {
            bot.send_message(chat_id, "Произошла ошибка получения медиафайлов").await?;
            error!(error = %e, "Failed to get mediafiles");
            Ok(None)
        }
    }
}
