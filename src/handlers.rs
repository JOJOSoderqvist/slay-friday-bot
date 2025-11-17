use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::commands::Command;
use crate::gigachat::GigaChatApi;
use crate::utils::{get_time_until_friday, format_time_delta};

pub async fn handle_command(bot: Bot, msg: Message, cmd: Command, generator: Arc<GigaChatApi>) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Friday => {
            let text = if let Some(time_left) = get_time_until_friday() {
                format!(
                    "До нефорской пятницы осталось: {} 🕷️ Готовь свой лучший аутфит. ⛓️",
                    format_time_delta(time_left)
                )
            } else {
                String::from("SLAAAAAY! 💅🔥🖤 ЭТО НЕФОРСКАЯ ПЯТНИЦА, ДЕТКА! 🤘😈⛓️ Время сиять! ✨")
            };

            match generator.rephrase_text(text.as_str()).await {
                Ok(new_text) => {
                    bot.send_message(msg.chat.id, new_text).await?;
                }
                Err(err) => {
                    log::error!("Failed to rephrase text: {:?}", err);
                    bot.send_message(msg.chat.id, text).await?;
                }
            }

        }
        Command::Stop => {
            bot.send_message(msg.chat.id, "Отключаю slay-уведомления. 💔").await?;
        }
    };

    Ok(())
}
