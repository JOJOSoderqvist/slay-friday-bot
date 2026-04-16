use crate::handlers::root_handler::DialogueStore;
use crate::repo::dialogue_storage::DialogueStorageKey;
use crate::repo::media_storage_postgres::dto::MediaType;
use crate::states::State;
use std::sync::Arc;
use teloxide::types::{FileId, Message, User, UserId};

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

pub fn extract_media_file_id(msg: &Message) -> (Option<&FileId>, Option<MediaType>) {
    match msg.animation() {
        Some(a) => (Some(&a.file.id), Some(MediaType::Gif)),
        None => match msg.sticker() {
            Some(s) => (Some(&s.file.id), Some(MediaType::Sticker)),
            None => (None, None),
        },
    }
}
