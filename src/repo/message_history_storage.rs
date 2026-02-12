use std::collections::VecDeque;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::common::Model;
use crate::handlers::MessageStore;

const HISTORY_STORAGE_SIZE: usize = 20;
pub struct HistoryEntry {
    model: Model,
    message: String,
}

impl HistoryEntry {
    pub fn new(model: Model, message: String) -> Self {
        HistoryEntry { model, message }
    }
}

pub struct MessageHistoryStorage {
    storage: RwLock<VecDeque<HistoryEntry>>,
}

impl MessageHistoryStorage {
    pub fn new() -> Self {
        let mut storage: VecDeque<HistoryEntry> = VecDeque::new();
        storage.reserve(HISTORY_STORAGE_SIZE);

        MessageHistoryStorage {
            storage: RwLock::new(storage),
        }
    }
}

#[async_trait]
impl MessageStore for MessageHistoryStorage {
    async fn add_message(&self, message: HistoryEntry) {
        let mut storage = self.storage.write().await;

        if storage.len() == HISTORY_STORAGE_SIZE {
            storage.pop_front();
        }

        storage.push_back(message);
    }

    async fn get_message_info(&self, message: &str) -> Option<Model> {
        let storage = self.storage.read().await;

        if let Some(entry) = storage.iter().find(|entry| entry.message == message) {
            return Some(entry.model);
        }

        None
    }
}
