use crate::errors::ApiError;
use crate::handlers::root_handler::{DialogueStore, MediaStore};
use crate::handlers::utils::{get_current_state, get_key, get_user_id_from_option};
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::User;
use tracing::{error, instrument};

#[instrument(skip(bot, chat_id, from, dialogue))]
pub async fn trigger_delete(
    bot: Bot,
    chat_id: ChatId,
    from: Option<User>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    let Some(user_id) = get_user_id_from_option(&from) else {
        bot.send_message(chat_id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    };

    let key = (user_id, chat_id);

    bot.send_message(chat_id, "Введите название медиафайла для удаления")
        .await?;
    dialogue.update_dialogue(key, State::TriggerDeleteCmd);

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, media_store))]
pub async fn delete_media(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    let Some(key) = get_key(&msg) else {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    };

    let Some(State::TriggerDeleteCmd) = get_current_state(&msg, dialogue.clone()) else {
        return Ok(());
    };

    let Some(media_entry_name) = msg.text() else {
        bot.send_message(
            msg.chat.id,
            "Сообщение пустое, либо это не текстовое сообщение",
        )
        .await?;
        return Ok(());
    };

    match media_store.remove_media_entry(media_entry_name).await {
        Ok(res) => {
            if res {
                bot.send_message(
                    msg.chat.id,
                    format!("Медиа {} успешно удалено", media_entry_name),
                )
                    .await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    format!("Медиа с названием {} нет", media_entry_name),
                )
                    .await?;
            }

            dialogue.remove_dialogue(&key);
        }

        Err(e) => {
            error!(err = %e, "Failed to handle media deletion");

            bot.send_message(
                msg.chat.id,
                format!("Произошла ошибка удаления медиафайла: {}", e),
            )
            .await?;
            dialogue.remove_dialogue(&key);
        }
    }

    Ok(())
}
