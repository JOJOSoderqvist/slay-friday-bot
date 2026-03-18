use crate::MediaStore;
use crate::commands::Command;
use crate::errors::ApiError;
use crate::handlers::add_media::trigger_add;
use crate::handlers::delete_media::trigger_delete;
use crate::handlers::friday::friday;
use crate::handlers::list_available_media::list_media;
use crate::handlers::rename_media::trigger_rename;
use crate::handlers::root_handler::{ContentGenerator, DialogueStore, MessageStore, help};
use crate::handlers::utils::get_user_id_from_option;
use crate::utils::{reply_suggestions_keyboard, setup_inline_callback_keyboard};
use std::sync::Arc;
use strum::IntoEnumIterator;
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::User;
use tracing::warn;

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
    media_store: Arc<dyn MediaStore>,
    dialogue: Arc<dyn DialogueStore>,
) -> Result<(), ApiError> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(chat_id) = q.chat_id() else {
        warn!("chat id not found for this query");
        return Ok(());
    };

    let key = (q.from.id, chat_id);
    dialogue.remove_dialogue(&key);

    let Some(data) = q.data else {
        warn!("callback had no data");
        return Ok(());
    };

    match data.parse::<Command>()? {
        Command::Help => {
            help(bot, chat_id).await?;
            Ok(())
        }
        Command::Friday => {
            friday(bot, chat_id, generator, message_store).await?;
            Ok(())
        }
        Command::ListMedia => {
            list_media(bot, chat_id, media_store).await?;
            Ok(())
        }
        Command::GetMedia(_) => {
            let media_list = media_store.list_available_media_entries().await;
            let mut available_media_entries = match media_list {
                None => {
                    bot.send_message(chat_id, "No stickers available").await?;
                    return Ok(());
                }
                Some(media) => media,
            };

            available_media_entries.sort();

            let keyboard = reply_suggestions_keyboard(available_media_entries.as_slice(), "/get");

            bot.send_message(chat_id, "Выберите медиафайл")
                .reply_markup(keyboard)
                .await?;

            Ok(())
        }
        Command::Cancel => {
            dialogue.remove_dialogue(&key);
            Ok(())
        }

        Command::AddMedia => {
            trigger_add(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }

        Command::RenameMedia => {
            trigger_rename(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }

        Command::DeleteMedia => {
            trigger_delete(bot, chat_id, Some(q.from), dialogue).await?;
            Ok(())
        }
        cmd => {
            bot.send_message(chat_id, format!("Команда {cmd} пока не поддерживается"))
                .await?;
            Ok(())
        }
    }
}
