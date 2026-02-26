use crate::errors::ApiError;
use crate::handlers::add_sticker::receive_sticker;
use crate::handlers::rename_sticker::receive_new_sticker_name;
use crate::handlers::root_handler::{ContentGenerator, DialogueStore, MessageStore, StickerStore};
use crate::handlers::slay::inline_choice_callback;
use crate::states::State;
use log::info;
use mockall::predicate::ge;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::CallbackQuery;
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
        Some(State::ReceiveNewName { .. }) => {
            receive_new_sticker_name(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }
        Some(State::ReceiveSticker { .. }) => {
            info!("MATCHER STICKER RECEIVE STATE");

            receive_sticker(bot, msg, dialogue, sticker_store).await?;
            Ok(())
        }
        // Some(State::ShowInline { .. }) => {
        //     inline_choice_callback(bot, q, generator, message_store, sticker_store, dialogue).await?;
        //     Ok(())
        // }
        None | Some(_) => Ok(()),
    }
}
