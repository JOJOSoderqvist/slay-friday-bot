use crate::errors::ApiError;
use crate::handlers::root_handler::{ContentGenerator, MessageStore};
use crate::repo::message_history_storage::HistoryEntry;
use crate::utils::{format_time_delta, get_time_until_friday};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use tracing::{error, instrument};

#[instrument(skip(bot, chat_id, generator, store))]
pub async fn friday(
    bot: Bot,
    chat_id: ChatId,
    generator: Arc<dyn ContentGenerator>,
    store: Arc<dyn MessageStore>,
) -> Result<(), ApiError> {
    let text = if let Some(time_left) = get_time_until_friday() {
        format!(
            "До нефорской пятницы осталось: {} 🕷️ Готовь свой лучший аутфит. ⛓️",
            format_time_delta(time_left)
        )
    } else {
        String::from("SLAAAAAY! 💅🔥🖤 ЭТО НЕФОРСКАЯ ПЯТНИЦА, ДЕТКА! 🤘😈⛓️ Время сиять! ✨")
    };

    match generator.generate_text(text.as_str()).await {
        Ok((new_text, model_name)) => {
            store
                .add_message(HistoryEntry::new(model_name, new_text.clone()))
                .await;

            bot.send_message(chat_id, new_text).await?;
        }
        Err(err) => {
            error!(error = %err, "Failed to rephrase text");
            bot.send_message(chat_id, text).await?;
        }
    }

    Ok(())
}
