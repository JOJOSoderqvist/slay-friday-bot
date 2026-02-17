use crate::errors::ApiError;
use crate::handlers::root_handler::MessageStore;
use log::debug;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;

pub async fn model_info(
    bot: Bot,
    msg: Message,
    store: Arc<dyn MessageStore>,
) -> Result<(), ApiError> {
    let reply_msg = match msg.reply_to_message() {
        Some(m) => m,
        None => {
            bot.send_message(msg.chat.id, "Команда должна быть ответом на сообщение бота")
                .await?;
            return Ok(());
        }
    };

    let text = match reply_msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "Это сообщение не сгенерировано ботом")
                .await?;
            return Ok(());
        }
    };

    match store.get_message_info(text).await {
        Some(model) => {
            bot.send_message(
                msg.chat.id,
                format!("Это сообщение сгенерировано: {}", model),
            )
            .await?;
        }
        None => {
            debug!("No entry found in storage");
            bot.send_message(msg.chat.id, "Информации про это сообщение не найдено")
                .await?;
        }
    }

    Ok(())
}
