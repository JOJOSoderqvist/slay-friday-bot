use crate::StickerStore;
use crate::commands::Command;
use crate::errors::ApiError;
use crate::errors::ApiError::DialogueStorageError;
use crate::handlers::friday::friday;
use crate::handlers::list_stickers::list_stickers;
use crate::handlers::root_handler::{ContentGenerator, MessageStore, MyDialogue, help, DialogueStore};
use crate::states::State;
use crate::utils::{reply_suggestions_keyboard, setup_inline_callback_keyboard};
use log::{info, warn};
use std::sync::Arc;
use strum::IntoEnumIterator;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{Message};
use url::quirks::origin;
use crate::handlers::add_sticker::add_sticker;

pub async fn slay(bot: Bot, msg: Message, dialogue: Arc<dyn DialogueStore>) -> Result<(), ApiError> {
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
                // && *cmd != Command::AddSticker(String::new())
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

    let key = (msg.from.clone().unwrap().id, msg.chat.id);
    dialogue.update_dialogue(key, State::ShowInline {
        user_id: user.id,
        original_msg: msg,
    });

    Ok(())
}

pub async fn inline_choice_callback(
    bot: Bot,
    q: CallbackQuery,
    generator: Arc<dyn ContentGenerator>,
    message_store: Arc<dyn MessageStore>,
    sticker_store: Arc<dyn StickerStore>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    info!("entering callback");

    bot.answer_callback_query(q.id).await?;

    // TODO: Бредик
    let msg_chat_id = match q.message {
        Some(msg) => msg.chat().id,
        None => {
            return Ok(());
        }
    };


    let key = (q.from.id, msg_chat_id);

    let (user_id, stored_msg) = match dialogue.get_dialogue(key) {
        Some(State::ShowInline {user_id, original_msg}) => {
            (user_id, original_msg)
        }
        _ => {
            return Ok(())
        }
    };

    if q.from.id != user_id {
        return Ok(());
    }


    let Some(data) = q.data else {
        warn!("callback had no data");
        return Ok(());
    };

    match data.parse::<Command>()? {
        Command::Help => {
            info!("help cmd parsed");
            help(bot, stored_msg).await?;
            dialogue.remove_dialogue(key);
            Ok(())
        }
        Command::Friday => {
            friday(bot, stored_msg, generator, message_store).await?;
            dialogue.remove_dialogue(key);
            Ok(())
        }
        Command::ListStickers => {
            list_stickers(bot, stored_msg, sticker_store).await?;
            dialogue.remove_dialogue(key);
            Ok(())
        }
        Command::Sticker(_) => {
            let stickers_list = sticker_store.list_stickers().await;
            let mut available_stickers = match stickers_list {
                None => {
                    bot.send_message(stored_msg.chat.id, "No stickers available")
                        .await?;
                    dialogue.remove_dialogue(key);
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
            dialogue.remove_dialogue(key);
            Ok(())
        }
        Command::Cancel => {
            dialogue.remove_dialogue(key);
            Ok(())
        }

        Command::AddSticker(_) => {
            todo!()
            // add_sticker(bot, stored_msg, dialogue)
        }
        cmd => {
            bot.send_message(
                stored_msg.chat.id,
                format!("Команда {cmd} пока не поддерживается"),
            )
            .await?;
            dialogue.remove_dialogue(key);
            Ok(())
        }
    }
}
