use crate::errors::ApiError;
use crate::errors::ApiError::MediaAlreadyExists;
use crate::handlers::root_handler::{DialogueStore, MediaStore};
use crate::handlers::utils::{
    extract_media_file_id, get_current_state, get_key, get_user_id_from_option,
};
use crate::repo::media_storage_postgres::dto::MediaEntry;
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::User;
use tracing::{error, instrument};

#[instrument(skip(bot, chat_id, dialogue))]
pub async fn trigger_add(
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

    bot.send_message(chat_id, "Введите название медиафайла")
        .await?;
    dialogue.update_dialogue(key, State::TriggeredAddCmd);

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, media_store))]
pub async fn process_new_name(
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

    let Some(State::TriggeredAddCmd) = get_current_state(&msg, dialogue.clone()) else {
        return Ok(());
    };

    let Some(media_name) = msg.text() else {
        bot.send_message(
            msg.chat.id,
            "Сообщение пустое, либо это не текстовое сообщение",
        )
        .await?;
        return Ok(());
    };

    match media_store.is_already_created(media_name).await {
        Ok(is_created) => {
            if is_created {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Медиафайл с именем {} уже существует, попробуй другое",
                        media_name
                    ),
                )
                .await?;
                return Ok(());
            }
        }

        Err(e) => {
            bot.send_message(
                msg.chat.id,
                "Произошла ошибка при проверке стикера на существование",
            )
            .await?;

            error!(error = %e, "Failed to check sticker existance");
            return Ok(());
        }
    }

    dialogue.update_dialogue(
        key,
        State::PerformAdd {
            media_entry_name: media_name.to_string(),
        },
    );

    bot.send_message(msg.chat.id, "Отправьте стикер или gif")
        .await?;

    Ok(())
}

#[instrument(skip(bot, msg, dialogue, media_store))]
pub async fn receive_media(
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

    let Some(State::PerformAdd { media_entry_name }) = get_current_state(&msg, dialogue.clone())
    else {
        return Ok(());
    };

    let (Some(file_id), Some(media_type)) = extract_media_file_id(&msg) else {
        bot.send_message(
            msg.chat.id,
            "Это не стикер и не gif. Отправьте стикер, gif или команду /cancel.",
        )
        .await?;
        return Ok(());
    };

    let media_entry = MediaEntry::new(media_entry_name, file_id.to_string(), key.0, media_type);
    match media_store.add_media_entry(media_entry).await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "Медиафайл сохранен! 🎉")
                .await?;
            dialogue.remove_dialogue(&key);
        }
        Err(MediaAlreadyExists) => {
            bot.send_message(
                msg.chat.id,
                "Медиафайл с этим именем уже существует. Попробуйте другое имя",
            )
            .await?;
        }
        Err(e) => {
            error!(err = %e, "Failed to handle media creation");

            bot.send_message(
                msg.chat.id,
                format!("Произошла ошибка сохранения медиафайла: {}", e),
            )
            .await?;

            dialogue.remove_dialogue(&key);
        }
    }

    Ok(())
}
