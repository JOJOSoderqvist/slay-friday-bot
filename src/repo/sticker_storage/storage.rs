use std::collections::HashMap;

use async_trait::async_trait;
use teloxide::payloads::SetMyCommandsSetters;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::errors::ApiError::StickerAlreadyExists;
use crate::errors::RepoError::FailedToOpenFile;
use crate::errors::{ApiError, RepoError};
use crate::generation_controller::StickerStore;
use crate::repo::sticker_storage::dto::StickerEntry;

pub struct StickerStorage {
    storage: Mutex<File>,
    cache: Mutex<HashMap<String, String>>,
    filename: String,
}

impl StickerStorage {
    // TODO: Make as config
    async fn new(filename: String) -> Result<Self, RepoError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename.as_str())
            .await
            .map_err(FailedToOpenFile)?;

        let cache: HashMap<String, String> = HashMap::new();
        Ok(StickerStorage {
            storage: Mutex::new(file),
            cache: Mutex::new(cache),
            filename,
        })
    }
}

#[async_trait]
impl StickerStore for StickerStorage {
    async fn add_sticker(&self, sticker: StickerEntry) -> Result<(), ApiError> {
        {
            let mut file_lock = self.storage.lock().await;
            let mut stickers_string = String::new();
            file_lock.read_to_string(&mut stickers_string);
            let mut stickers: Vec<StickerEntry> = serde_json::from_str(stickers_string.as_str())?;

            if !stickers.contains(&sticker) {
                stickers.push(sticker.clone());

                let sticker_raw = serde_json::to_string(&stickers)?;
                file_lock.write_all(sticker_raw.as_bytes()).await?;
                file_lock.flush().await?;
            } else {
                return Err(StickerAlreadyExists);
            }
        }

        {
            let mut cache_lock = self.cache.lock().await;
            cache_lock.insert(sticker.name, sticker.file_id);
        }

        Ok(())
    }

    async fn get_sticker(&self, sticker_name: &str) -> Option<StickerEntry> {
        let cache_lock = self.cache.lock().await;
        match cache_lock.get(sticker_name).cloned() {
            Some(file_id) => Some(StickerEntry::new(sticker_name.to_string(), file_id)),
            None => None,
        }
    }
    async fn rename_sticker(&self, old_name: &str, new_name: &str) -> Result<(), ApiError> {}
    async fn list_stickers(&self) -> Option<Vec<StickerEntry>> {
        let entries: Vec<StickerEntry> = {
            let cache_lock = self.cache.lock().await;

            let mut entries: Vec<StickerEntry> = Vec::new();
            entries.reserve(cache_lock.len());
            for (name, file_id) in cache_lock.iter() {
                entries.push(StickerEntry::new(*name, *file_id));
            }
            entries
        };

        match entries.len() {
            0 => None,
            _ => Some(entries),
        }
    }
    async fn remove_sticker(&self) -> Result<(), ApiError> {}
}
