use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::commands::Command;
use crate::utils::{get_time_until_friday, format_timedelta};

pub async fn handle_command(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Friday => {
            let time_left = get_time_until_friday();
            let text = if time_left.num_days() >= 6 {
                "SLAAAAAY! 💅🔥🖤 ЭТО НЕФОРСКАЯ ПЯТНИЦА, ДЕТКА! 🤘😈⛓️ Время сиять! ✨"
                    .to_string()
            } else {
                format!(
                    "До нефорской пятницы осталось: {} 🕷️ Готовь свой лучший аутфит. ⛓️",
                    format_timedelta(time_left)
                )
            };
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Stop => {
            bot.send_message(msg.chat.id, "Отключаю slay-уведомления. 💔").await?;
        }
    };

    Ok(())
}
