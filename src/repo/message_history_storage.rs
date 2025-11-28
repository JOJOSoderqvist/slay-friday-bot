use std::collections::VecDeque;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{common::Model, generation_controller::MessageStore};

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
    storage: Mutex<VecDeque<HistoryEntry>>,
}

impl MessageHistoryStorage {
    pub fn new() -> Self {
        let mut storage: VecDeque<HistoryEntry> = VecDeque::new();
        storage.reserve(10);

        MessageHistoryStorage {
            storage: Mutex::new(storage),
        }
    }
}

#[async_trait]
impl MessageStore for MessageHistoryStorage {
    async fn add_message(&self, message: HistoryEntry) {
        let mut storage_lock = self.storage.lock().await;
        if storage_lock.len() == 10 {
            storage_lock.pop_front();
        }

        storage_lock.push_back(message);
    }

    async fn get_message_info(&self, message: &str) -> Option<Model> {
        let storage_lock = self.storage.lock().await;

        for entry in storage_lock.iter() {
            if entry.message == message {
                return Some(entry.model);
            }
        }

        None
    }
}
