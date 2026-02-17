use strum::IntoEnumIterator;
use crate::errors::ApiError;
use crate::handlers::root_handler::MyDialogue;
use teloxide::types::Message;
use teloxide::Bot;
use teloxide::prelude::*;
use crate::commands::Command;
use crate::utils::setup_inline_callback_keyboard;

pub async fn slay(bot: Bot, msg: Message, my_dialogue: MyDialogue) -> Result<(), ApiError> {
    if msg.from.is_none() {
        bot.send_message(msg.chat.id, "Каналы не поддерживаются").await?;
        return Ok(())
    }

    let available_commands = Command::iter().collect();

    let inline_keyboard = match setup_inline_callback_keyboard(available_commands, 4) {
        Some(k) => k,
        None => {
            bot.send_message(msg.chat.id, "Нет доступных команд").await?;
            return Ok(())
        }
    };

    bot.send_message(msg.chat.id, "Выберете команду")
        .reply_markup(inline_keyboard)
        .await?;
    
    Ok(())
}
