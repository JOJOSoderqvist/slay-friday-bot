use crate::handlers::root_handler::DialogueStore;
use crate::repo::dialogue_storage::DialogueStorageKey;
use crate::states::State;
use std::sync::Arc;
use teloxide::types::{Message, User, UserId};

pub fn get_user_id_from_option(from: &Option<User>) -> Option<UserId> {
    from.as_ref().map(|u| u.id)
}

pub fn get_key(msg: &Message) -> Option<DialogueStorageKey> {
    let u_id = msg.from.as_ref()?.id;
    let chat_id = msg.chat.id;
    Some((u_id, chat_id))
}

pub fn get_current_state(msg: &Message, dialogue: Arc<dyn DialogueStore>) -> Option<State> {
    let key = get_key(msg)?;
    dialogue.get_dialogue(&key)
}
