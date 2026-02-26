use crate::handlers::root_handler::DialogueStore;
use crate::states::State;
use dashmap::DashMap;
use std::sync::Arc;
use teloxide::types::{ChatId, UserId};

pub type DialogueStorageKey = (UserId, ChatId);

// TODO: Do i need Arc?
pub struct UserDialogueStorage {
    storage: Arc<DashMap<DialogueStorageKey, State>>,
}

impl UserDialogueStorage {
    pub fn new() -> Self {
        UserDialogueStorage {
            storage: Arc::new(DashMap::new()),
        }
    }
}

impl DialogueStore for UserDialogueStorage {
    fn get_dialogue(&self, key: DialogueStorageKey) -> Option<State> {
        self.storage.get(&key).map(|v| v.clone())
    }

    fn remove_dialogue(&self, key: DialogueStorageKey) -> Option<(DialogueStorageKey, State)> {
        self.storage.remove(&key)
    }

    fn update_dialogue(&self, key: DialogueStorageKey, new_state: State) -> Option<State> {
        self.storage.insert(key, new_state)
    }
}
