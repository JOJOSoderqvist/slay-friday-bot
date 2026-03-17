use crate::errors::ApiError;
use crate::errors::ApiError::StickerAlreadyExists;
use crate::handlers::root_handler::{DialogueStore, StickerStore};
use crate::handlers::utils::{get_current_state, get_key, get_user_id_from_option};
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::User;
use tracing::{error, instrument};

#[instrument(skip(bot, chat_id, from, dialogue))]
pub async fn trigger_rename(
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

    bot.send_message(
        chat_id,
        "Введите название стикера, который хотите переименовать",
    )
    .await?;
    dialogue.update_dialogue(key, State::TriggeredRenameCmd);
    Ok(())
}

#[instrument(skip(bot, msg, dialogue, sticker_store))]
pub async fn rename_sticker(
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

    let Some(State::TriggeredRenameCmd) = get_current_state(&msg, dialogue.clone()) else {
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

    if !sticker_store.is_already_created(sticker_name).await {
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

    dialogue.update_dialogue(
        key,
        State::PerformRename {
            old_name: sticker_name.to_string(),
        },
    );
    bot.send_message(msg.chat.id, "Введите новое название")
        .await?;

    Ok(())
}

pub async fn process_new_sticker_name(
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

    let Some(State::PerformRename { old_name }) = get_current_state(&msg, dialogue.clone()) else {
        return Ok(());
    };

    let Some(new_name) = msg.text() else {
        bot.send_message(msg.chat.id, "Сообщение пустое, пожалуйста укажите название")
            .await?;
        return Ok(());
    };

    match sticker_store
        .rename_sticker(old_name.as_str(), new_name)
        .await
    {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                format!("Новое имя '{}' сохранено! 🎉", new_name),
            )
            .await?;

            dialogue.remove_dialogue(&key);
        }

        Err(StickerAlreadyExists) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Стикер с именем {} уже существует, попробуй другое",
                    new_name
                ),
            )
            .await?;
        }

        Err(e) => {
            error!(err = %e, "Failed to handle sticker renae");
            bot.send_message(msg.chat.id, format!("Произошла неизвестная ошибка {}", e))
                .await?;

            dialogue.remove_dialogue(&key);
        }
    }

    Ok(())
}
