use crate::errors::ApiError;
use crate::errors::ApiError::{StickerAlreadyExists};
use crate::handlers::root_handler::{DialogueStore, StickerStore};
use crate::handlers::utils::{get_current_state, get_key, is_user};
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, msg, dialogue))]
pub async fn trigger_add(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    if !is_user(&msg) {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    }

    let key = (msg.from.unwrap().id, msg.chat.id);

    bot.send_message(msg.chat.id, "Введите название стикера")
        .await?;
    dialogue.update_dialogue(key, State::TriggeredAddCmd);

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, sticker_store))]
pub async fn process_new_name(
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

    if sticker_store.is_already_created(sticker_name).await {
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

    dialogue.update_dialogue(
        key,
        State::PerformAdd {
            sticker_name: sticker_name.to_string(),
        },
    );

    bot.send_message(msg.chat.id, "Отправьте стикер").await?;

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, sticker_store))]
pub async fn receive_sticker(
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

    let Some(State::PerformAdd { sticker_name }) = get_current_state(&msg, dialogue.clone()) else {
        return Ok(());
    };

    if let Some(sticker) = msg.sticker() {
        let entry = StickerEntry::new(sticker_name.clone(), sticker.file.id.clone().to_string());

        match sticker_store.add_sticker(entry).await {
            Ok(_) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Стикер '{}' сохранен! 🎉", sticker_name),
                )
                .await?;

                dialogue.remove_dialogue(key);
            }
            Err(StickerAlreadyExists) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Стикер '{}' уже существует. Попробуйте другое имя",
                        sticker_name
                    ),
                )
                .await?;
            }

            Err(e) => {
                error!(err = %e, "Failed to handle sticker creation");

                bot.send_message(
                    msg.chat.id,
                    format!("Произошла ошибка сохранения стикера: {}", e),
                )
                .await?;

                dialogue.remove_dialogue(key);
            }
        }
    } else {
        bot.send_message(msg.chat.id, "Это не стикер. Отправьте стикер или /cancel.")
            .await?;
    }
    Ok(())
}
