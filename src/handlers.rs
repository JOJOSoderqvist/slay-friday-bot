use crate::commands::Command;
use crate::common::Model;
use crate::constants::STICKERS_MAP;
use crate::errors::ApiError;
use crate::utils::{format_time_delta, get_time_until_friday, parse_sticker_name};
use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile};
use teloxide::utils::command::BotCommands;
use tracing::{error, instrument};

#[async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_text(&self, current_text: &str) -> Result<String, ApiError>;

    async fn get_message_info(&self, text: &str) -> Option<Model>;
}

#[instrument(skip(bot, generator, cmd, msg))]
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    generator: Arc<dyn ContentGenerator>,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Friday => {
            let text = if let Some(time_left) = get_time_until_friday() {
                format!(
                    "До нефорской пятницы осталось: {} 🕷️ Готовь свой лучший аутфит. ⛓️",
                    format_time_delta(time_left)
                )
            } else {
                String::from(
                    "SLAAAAAY! 💅🔥🖤 ЭТО НЕФОРСКАЯ ПЯТНИЦА, ДЕТКА! 🤘😈⛓️ Время сиять! ✨",
                )
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

        Command::Model => {
            if let Some(msg) = msg.reply_to_message() {
                if let Some(text) = msg.text() {
                    match generator.get_message_info(text).await {
                        Some(model) => {
                            bot.send_message(
                                msg.chat.id,
                                format!("Это сообщение сгенерировано: {}", model),
                            )
                            .await?;
                        }
                        None => {
                            debug!("No entry found in storage");
                            bot.send_message(
                                msg.chat.id,
                                "Информации про это сообщение не найдено",
                            )
                            .await?;
                        }
                    }
                } else {
                    bot.send_message(msg.chat.id, "Это сообщение не сгенерировано ботом")
                        .await?;
                }
            } else {
                bot.send_message(msg.chat.id, "Команда должна быть ответом на сообщение бота")
                    .await?;
            }
        }

        Command::Sticker(raw_sticker_name) => match parse_sticker_name(raw_sticker_name) {
            Ok(sticker_name) => match STICKERS_MAP.get(sticker_name.as_str()).cloned() {
                Some(sticker_id) => {
                    bot.send_sticker(
                        msg.chat.id,
                        InputFile::file_id(FileId(sticker_id.to_string())),
                    )
                    .await?;
                }
                None => {
                    info!("No sticker with this name found");
                    bot.send_message(msg.chat.id, "Стикера с таким названием нет")
                        .await?;
                }
            },
            Err(e) => {
                info!("failed to parse sticker name {}", e.to_string());
                bot.send_message(msg.chat.id, e.to_string()).await?;
            }
        },
    }

    Ok(())
}
