use std::sync::Arc;
use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tracing::{error, instrument};
use crate::commands::Command;
use crate::errors::ApiError;
use crate::utils::{get_time_until_friday, format_time_delta};


#[async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_text(&self, current_text: &str) -> Result<String, ApiError>;
}

#[instrument(skip(bot, generator, cmd, msg))]
pub async fn handle_command(bot: Bot,
                            msg: Message,
                            cmd: Command,
                            generator: Arc<dyn ContentGenerator>) -> ResponseResult<()>
{
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


            match generator.generate_text(text.as_str()).await {
                Ok(new_text) => {
                    bot.send_message(msg.chat.id, new_text).await?;
                }
                Err(err) => {
                    error!(error = %err, "Failed to rephrase text via GigaChat");
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
