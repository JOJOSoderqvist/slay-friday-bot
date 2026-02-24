use crate::StickerStore;
use crate::commands::Command;
use crate::errors::ApiError;
use crate::errors::ApiError::DialogueStorageError;
use crate::handlers::friday::friday;
use crate::handlers::list_stickers::list_stickers;
use crate::handlers::root_handler::{ContentGenerator, MessageStore, MyDialogue, help};
use crate::repo::sticker_storage::dto::StickerEntry;
use crate::states::State;
use crate::utils::{reply_suggestions_keyboard, setup_inline_callback_keyboard};
use log::{info, warn};
use std::cmp::PartialEq;
use std::sync::Arc;
use strum::IntoEnumIterator;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{Message, MessageId, User};

pub async fn slay(bot: Bot, msg: Message, dialogue: MyDialogue) -> Result<(), ApiError> {
    info!("entered slay command");
    let user = match msg.from {
        None => {
            bot.send_message(msg.chat.id, "Каналы не поддерживаются")
                .await?;
            return Ok(());
        }
        Some(ref id) => id,
    };

    let available_commands = Command::iter()
        .filter(|cmd| {
            *cmd != Command::Slay
                && *cmd != Command::Model
                && *cmd != Command::AddSticker(String::new())
                && *cmd != Command::RenameSticker(String::new())
        })
        .collect();

    let inline_keyboard = match setup_inline_callback_keyboard(available_commands, 4) {
        Some(k) => k,
        None => {
            bot.send_message(msg.chat.id, "Нет доступных команд")
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, "Выберите команду")
        .reply_markup(inline_keyboard)
        .await?;

    dialogue
        .update(State::ShowInline {
            user_id: user.id,
            original_msg: msg,
        })
        .await
        .map_err(DialogueStorageError)?;

    Ok(())
}

pub async fn inline_choice_callback(
    bot: Bot,
    q: CallbackQuery,
    generator: Arc<dyn ContentGenerator>,
    message_store: Arc<dyn MessageStore>,
    sticker_store: Arc<dyn StickerStore>,
    dialogue: MyDialogue,
) -> Result<(), ApiError> {
    info!("entering callback");
    let current_state = dialogue.get().await?.unwrap();

    let State::ShowInline {
        user_id,
        original_msg: stored_msg,
    } = current_state
    else {
        return Ok(());
    };

    if q.from.id != user_id {
        return Ok(());
    }

    bot.answer_callback_query(q.id).await?;

    let Some(data) = q.data else {
        warn!("callback had no data");
        return Ok(());
    };

    match data.parse::<Command>()? {
        Command::Help => {
            info!("help cmd parsed");
            help(bot, stored_msg).await?;
            dialogue.exit().await?;
            Ok(())
        }
        Command::Friday => {
            friday(bot, stored_msg, generator, message_store).await?;
            dialogue.exit().await?;
            Ok(())
        }
        Command::ListStickers => {
            list_stickers(bot, stored_msg, sticker_store).await?;
            dialogue.exit().await?;
            Ok(())
        }
        Command::Sticker(_) => {
            let stickers_list = sticker_store.list_stickers().await;
            let mut available_stickers = match stickers_list {
                None => {
                    bot.send_message(stored_msg.chat.id, "No stickers available")
                        .await?;
                    dialogue.exit().await?;
                    return Ok(());
                }
                Some(stickers) => stickers,
            };

            available_stickers.sort();

            let keyboard = reply_suggestions_keyboard(
                available_stickers.as_slice(),
                Some(Command::Sticker(String::default())),
            ); // TODO: cringe
            bot.send_message(stored_msg.chat.id, "Выберите опцию")
                .reply_to(stored_msg.id)
                .reply_markup(keyboard)
                .await?;
            dialogue.exit().await?;
            Ok(())
        }
        Command::Cancel => {
            dialogue.exit().await?;
            Ok(())
        }
        cmd => {
            bot.send_message(
                stored_msg.chat.id,
                format!("Команда {cmd} пока не поддерживается"),
            )
            .await?;
            dialogue.exit().await?;
            Ok(())
        }
    }
}
