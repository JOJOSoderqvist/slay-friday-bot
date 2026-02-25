use crate::errors::ApiError;
use crate::errors::ApiError::{DialogueStorageError, StickerAlreadyExists};
use crate::handlers::root_handler::{DialogueStore, MyDialogue, StickerStore};
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use log::{debug, info};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, msg, dialogue, sticker_name, sticker_store))]
pub async fn add_sticker(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
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


    let key = (msg.from.unwrap().id, msg.chat.id); // TODO: remove unwrap
    dialogue.update_dialogue(key, State::ReceiveSticker {name: sticker_name});

    info!("UPDATED DIALOGUE FROM ADD STICKER");

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, sticker_store))]
pub async fn receive_sticker(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    info!("RECEIVED STICKER");

    let user_id = match msg.from.clone().map(|u| u.id) {
        Some(id) => id,
        None => {
            bot.send_message(msg.chat.id, "Каналы не поддерживаются").await?;
            return Ok(());
        }
    };

    if let Some(sticker) = msg.sticker() {
        let key = (user_id, msg.chat.id);
        let new_name = match dialogue.get_dialogue(key) {
            Some(State::ReceiveSticker {name}) => {
                name
            }
            _ => {
                return Ok(())
            }
        };


        info!("NEW NAME: {}", new_name);

        let entry = StickerEntry::new(new_name.clone(), sticker.file.id.clone().to_string());

        match sticker_store.add_sticker(entry).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, format!("Стикер '{}' сохранен! 🎉", new_name))
                    .await?;

                dialogue.remove_dialogue(key);
            }
            Err(StickerAlreadyExists) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Стикер '{}' уже существует. Попробуйте другое имя", new_name),
                )
                .await?;

                dialogue.remove_dialogue(key);
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
        debug!("Not a sticker");

        bot.send_message(msg.chat.id, "Это не стикер. Отправьте стикер или /cancel.")
            .await?;
    }
    Ok(())
}
