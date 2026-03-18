use crate::errors::ApiError;
use crate::handlers::add_media::{process_new_name, receive_media};
use crate::handlers::delete_media::delete_media;
use crate::handlers::rename_media::{process_new_media_name, rename_media};
use crate::handlers::root_handler::{DialogueStore, MediaStore};
use crate::states::State;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::types::Message;

pub async fn state_dispatcher(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
    media_store: Arc<dyn MediaStore>,
) -> Result<(), ApiError> {
    let chat_id = msg.chat.id;
    let Some(user) = msg.from.clone() else {
        return Ok(());
    };

    let key = (user.id, chat_id);

    match dialogue.get_dialogue(&key) {
        Some(State::TriggeredAddCmd) => {
            process_new_name(bot, msg, dialogue, media_store).await?;
            Ok(())
        }

        Some(State::PerformAdd { .. }) => {
            receive_media(bot, msg, dialogue, media_store).await?;
            Ok(())
        }

        Some(State::TriggeredRenameCmd) => {
            rename_media(bot, msg, dialogue, media_store).await?;
            Ok(())
        }

        Some(State::PerformRename { .. }) => {
            process_new_media_name(bot, msg, dialogue, media_store).await?;
            Ok(())
        }

        Some(State::TriggerDeleteCmd) => {
            delete_media(bot, msg, dialogue, media_store).await?;
            Ok(())
        }

        None => Ok(()),
    }
}
