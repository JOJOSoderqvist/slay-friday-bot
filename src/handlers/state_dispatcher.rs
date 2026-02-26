use crate::errors::ApiError;
use crate::handlers::add_sticker::{process_new_name, receive_sticker};
use crate::handlers::delete_sticker::delete_sticker;
use crate::handlers::rename_sticker::{process_new_sticker_name, rename_sticker};
use crate::handlers::root_handler::{DialogueStore, StickerStore};
use crate::states::State;
use log::info;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::types::Message;

pub async fn state_dispatcher(
    bot: Bot,
    msg: Message,
    dialogue: Arc<dyn DialogueStore>,
    sticker_store: Arc<dyn StickerStore>,
) -> Result<(), ApiError> {
    let chat_id = msg.chat.id;
    let Some(user) = msg.from.clone() else {
        return Ok(());
    };

    info!("started matching state");

    let key = (user.id, chat_id);
    match dialogue.get_dialogue(key) {
        Some(State::TriggeredAddCmd) => {
            info!("processing new sticker name");
            process_new_name(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }

        Some(State::PerformAdd { .. }) => {
            receive_sticker(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }

        Some(State::TriggeredRenameCmd) => {
            rename_sticker(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }

        Some(State::PerformRename { .. }) => {
            process_new_sticker_name(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }

        Some(State::TriggerDeleteCmd) => {
            delete_sticker(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }
        None | Some(_) => Ok(()),
    }
}
