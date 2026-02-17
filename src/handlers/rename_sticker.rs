use crate::errors::ApiError;
use crate::errors::ApiError::{DialogueStorageError, StickerAlreadyExists};
use crate::handlers::root_handler::{MyDialogue, StickerStore};
use crate::states::State;
use log::info;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, msg, dialogue, sticker_name, sticker_store))]
pub async fn rename_sticker(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    if sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Пожалуйста, укажите название")
            .await?;
        return Ok(());
    }

    if !sticker_store
        .is_already_created(sticker_name.as_str())
        .await
    {
        bot.send_message(
            msg.chat.id,
            format!(
                "Стикер с именем {} не существует, попробуй другое",
                sticker_name
            ),
        )
        .await?;
        return Ok(());
    };

    bot.send_message(
        msg.chat.id,
        format!("Отправь новое имя для стикера '{}'", sticker_name),
    )
    .await?;

    dialogue
        .update(State::ReceiveNewName {
            old_name: sticker_name,
        })
        .await
        .map_err(DialogueStorageError)?;

    Ok(())
}

pub async fn receive_new_sticker_name(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    old_sticker_name: String,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    let new_sticker_name = if let Some(name) = msg.text() {
        name
    } else {
        bot.send_message(msg.chat.id, "Это сообщение - не текст".to_string())
            .await?;

        dialogue.exit().await?;
        return Ok(());
    };

    if new_sticker_name.trim().is_empty() {
        bot.send_message(msg.chat.id, "Пожалуйста, укажите название".to_string())
            .await?;

        dialogue.exit().await?;
        return Ok(());
    }

    match sticker_store
        .rename_sticker(old_sticker_name.as_str(), new_sticker_name)
        .await
    {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                format!("Новое имя '{}' сохранено! 🎉", new_sticker_name),
            )
            .await?;
            dialogue.exit().await?;
        }

        Err(StickerAlreadyExists) => {
            info!("Sticker with name {} already exists", new_sticker_name);
            bot.send_message(
                msg.chat.id,
                format!(
                    "Стикер с именем {} уже существует, попробуй другое",
                    new_sticker_name
                ),
            )
            .await?;

            dialogue.exit().await?;
        }

        Err(e) => {
            error!(err = %e, "Failed to handle sticker renae");
            bot.send_message(msg.chat.id, format!("Произошла неизвестная ошибка {}", e))
                .await?;

            dialogue.exit().await?;
        }
    }

    Ok(())
}
