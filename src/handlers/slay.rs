use crate::StickerStore;
use crate::commands::Command;
use crate::errors::ApiError;
use crate::handlers::add_sticker::trigger_add;
use crate::handlers::delete_sticker::trigger_delete;
use crate::handlers::friday::friday;
use crate::handlers::list_stickers::list_stickers;
use crate::handlers::rename_sticker::trigger_rename;
use crate::handlers::root_handler::{ContentGenerator, DialogueStore, MessageStore, help};
use crate::handlers::utils::get_user_id_from_option;
use crate::utils::{reply_suggestions_keyboard, setup_inline_callback_keyboard};
use std::sync::Arc;
use strum::IntoEnumIterator;
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::User;
use tracing::{info, warn};

pub async fn slay(bot: Bot, chat_id: ChatId, from: Option<User>) -> Result<(), ApiError> {
    let Some(_) = get_user_id_from_option(&from) else {
        bot.send_message(chat_id, "Каналы не поддерживаются")
            .await?;
        return Ok(());
    };

    let available_commands = Command::iter()
        .filter(|cmd| *cmd != Command::Slay && *cmd != Command::Model)
        .collect::<Vec<Command>>();

    let inline_keyboard = match setup_inline_callback_keyboard(available_commands.as_slice()) {
        Some(k) => k,
        None => {
            bot.send_message(chat_id, "Нет доступных команд").await?;
            return Ok(());
        }
    };

    bot.send_message(chat_id, "Выберите команду")
        .reply_markup(inline_keyboard)
        .await?;

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

    bot.answer_callback_query(q.id.clone()).await?;

    let Some(chat_id) = q.chat_id() else {
        warn!("chat id not found for this query");
        return Ok(());
    };

    let key = (q.from.id, chat_id);

    let Some(data) = q.data else {
        warn!("callback had no data");
        return Ok(());
    };

    match data.parse::<Command>()? {
        Command::Help => {
            info!("help cmd parsed");
            help(bot, chat_id).await?;
            dialogue.remove_dialogue(&key);
            Ok(())
        }
        Command::Friday => {
            dialogue.remove_dialogue(&key);
            friday(bot, chat_id, generator, message_store).await?;
            Ok(())
        }
        Command::ListStickers => {
            dialogue.remove_dialogue(&key);
            list_stickers(bot, chat_id, sticker_store).await?;
            Ok(())
        }
        Command::Sticker(_) => {
            dialogue.remove_dialogue(&key);

            let stickers_list = sticker_store.list_stickers().await;
            let mut available_stickers = match stickers_list {
                None => {
                    bot.send_message(chat_id, "No stickers available").await?;
                    dialogue.remove_dialogue(&key);
                    return Ok(());
                }
                Some(stickers) => stickers,
            };

            available_stickers.sort();

            let keyboard = reply_suggestions_keyboard(available_stickers.as_slice(), "/get");

            bot.send_message(chat_id, "Выберите стикер")
                .reply_markup(keyboard)
                .await?;

            Ok(())
        }
        Command::Cancel => {
            dialogue.remove_dialogue(&key);
            Ok(())
        }

        Command::AddSticker => {
            if let Some(d) = dialogue.get_dialogue(&key) {
                info!("updated state: {d}, u_id: {}, c_id: {}", key.0, key.1)
            } else {
                info!("no state")
            }

            dialogue.remove_dialogue(&key);
            info!("trigger add");
            trigger_add(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }

        Command::RenameSticker => {
            dialogue.remove_dialogue(&key);
            trigger_rename(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }

        Command::DeleteSticker => {
            dialogue.remove_dialogue(&key);
            trigger_delete(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }
        cmd => {
            bot.send_message(chat_id, format!("Команда {cmd} пока не поддерживается"))
                .await?;
            dialogue.remove_dialogue(&key);
            Ok(())
        }
    }
}
