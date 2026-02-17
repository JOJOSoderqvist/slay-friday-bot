use crate::errors::ApiError;
use crate::errors::ApiError::{DialogueStorageError, StickerAlreadyExists};
use crate::handlers::root_handler::{MyDialogue, StickerStore};
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, msg, dialogue, sticker_name, sticker_store))]
pub async fn add_sticker(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Пожалуйста, укажите название: /add <name>")
            .await?;
        return Ok(());
    }

    if sticker_store
        .is_already_created(sticker_name.as_str())
        .await
    {
        bot.send_message(
            msg.chat.id,
            format!(
                "Стикер с именем {} уже существует, попробуй другое",
                sticker_name
            ),
        )
        .await?;
        return Ok(());
    };

    bot.send_message(
        msg.chat.id,
        format!("Отправь мне стикер для '{}'", sticker_name),
    )
    .await?;

    dialogue
        .update(State::ReceiveSticker { name: sticker_name })
        .await
        .map_err(DialogueStorageError)?;

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, name, sticker_store))]
pub async fn receive_sticker(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if let Some(sticker) = msg.sticker() {
        let entry = StickerEntry::new(name.clone(), sticker.file.id.clone().to_string());

        match sticker_store.add_sticker(entry).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, format!("Стикер '{}' сохранен! 🎉", name))
                    .await?;

                dialogue.exit().await?;
            }
            Err(StickerAlreadyExists) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Стикер '{}' уже существует. Попробуйте другое имя", name),
                )
                .await?;

                dialogue.exit().await?;
            }

            Err(e) => {
                error!(err = %e, "Failed to handle sticker creation");

                bot.send_message(
                    msg.chat.id,
                    format!("Произошла ошибка сохранения стикера: {}", e),
                )
                .await?;
                dialogue.exit().await?;
            }
        }
    } else {
        debug!("Not a sticker");

        bot.send_message(msg.chat.id, "Это не стикер. Отправьте стикер или /cancel.")
            .await?;
    }
    Ok(())
}
