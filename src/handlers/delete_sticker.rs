use crate::errors::ApiError;
use crate::handlers::root_handler::{DialogueStore, StickerStore};
use crate::handlers::utils::{get_current_state, get_key, is_user};
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};
use crate::repo::dialogue_storage::DialogueStorageKey;

#[instrument(skip(bot, msg, dialogue))]
pub async fn trigger_delete(
    bot: Bot,
    msg: Message,
    optional_key: Option<DialogueStorageKey>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    if !is_user(&msg) {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    }

    let key = match optional_key {
        Some(k) => k,
        None => {
            (msg.from.unwrap().id, msg.chat.id)
        }
    };
    bot.send_message(msg.chat.id, "Введите название стикера для удаления")
        .await?;
    dialogue.update_dialogue(key, State::TriggerDeleteCmd);

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, sticker_store))]
pub async fn delete_sticker(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    let Some(key) = get_key(&msg) else {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    };

    let Some(State::TriggeredAddCmd) = get_current_state(&msg, dialogue.clone()) else {
        return Ok(());
    };

    let Some(sticker_name) = msg.text() else {
        bot.send_message(
            msg.chat.id,
            "Сообщение пустое, либо это не текстовое сообщение",
        )
        .await?;
        return Ok(());
    };

    match sticker_store.remove_sticker(sticker_name).await {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                format!("Стикер {} успешно удален", sticker_name),
            )
            .await?;
            dialogue.remove_dialogue(key);
        }

        Err(ApiError::StickerNotFound) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Стикер с названием {} не найден, попробуйте другое название",
                    sticker_name
                ),
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
            dialogue.remove_dialogue(key);
        }
    }

    Ok(())
}
