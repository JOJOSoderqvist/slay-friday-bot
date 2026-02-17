use std::collections::HashMap;

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::{Mutex, RwLock};

use crate::errors::ApiError::{StickerAlreadyExists, StickerNotFound, StorageError};
use crate::errors::RepoError::{
    ChangeFileError, FailedToOpenFile, FailedToReadFile, ReadJSONError, WriteJSONError,
};
use crate::errors::{ApiError, RepoError};
use crate::handlers::root_handler::StickerStore;
use crate::repo::sticker_storage::dto::StickerEntry;

pub struct StickerStorage {
    storage: Mutex<File>,
    cache: RwLock<HashMap<String, String>>,
}

impl StickerStorage {
    // TODO: Make as config
    pub async fn new(filename: String) -> Result<Self, RepoError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(filename.as_str())
            .await
            .map_err(FailedToOpenFile)?;

        file.seek(SeekFrom::Start(0))
            .await
            .map_err(ChangeFileError)?;

        let mut stickers_string = String::new();
        file.read_to_string(&mut stickers_string)
            .await
            .map_err(FailedToReadFile)?;

        let stickers: Vec<StickerEntry> = if stickers_string.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&stickers_string).map_err(ReadJSONError)?
        };

        let mut cache: HashMap<String, String> = HashMap::with_capacity(stickers.len());

        for entry in stickers {
            cache.insert(entry.name, entry.file_id);
        }

        Ok(StickerStorage {
            storage: Mutex::new(file),
            cache: RwLock::new(cache),
        })
    }

    async fn persist_cache(
        &self,
        cache_snapshot: &HashMap<String, String>,
    ) -> Result<(), RepoError> {
        let stickers: Vec<StickerEntry> = cache_snapshot
            .iter()
            .map(|(name, file_id)| StickerEntry::new(name.clone(), file_id.clone()))
            .collect();

        let sticker_raw = serde_json::to_string(&stickers).map_err(WriteJSONError)?;
        let mut file_lock = self.storage.lock().await;

        file_lock.set_len(0).await.map_err(ChangeFileError)?;
        file_lock
            .seek(SeekFrom::Start(0))
            .await
            .map_err(ChangeFileError)?;

        file_lock
            .write_all(sticker_raw.as_bytes())
            .await
            .map_err(ChangeFileError)?;

        file_lock.flush().await.map_err(ChangeFileError)?;

        Ok(())
    }
}

#[async_trait]
impl StickerStore for StickerStorage {
    async fn add_sticker(&self, sticker: StickerEntry) -> Result<(), ApiError> {
        let mut cache = self.cache.write().await;

        if cache.contains_key(&sticker.name) {
            return Err(StickerAlreadyExists);
        }

        cache.insert(sticker.name.clone(), sticker.file_id);

        if let Err(e) = self.persist_cache(&cache).await {
            cache.remove(sticker.name.as_str());

            return Err(StorageError(e));
        }

        Ok(())
    }

    async fn get_sticker(&self, sticker_name: &str) -> Option<StickerEntry> {
        let cache = self.cache.read().await;
        cache
            .get(sticker_name)
            .map(|file_id| StickerEntry::new(sticker_name.to_string(), file_id.clone()))
    }
    async fn rename_sticker(&self, old_name: &str, new_name: &str) -> Result<(), ApiError> {
        let mut cache = self.cache.write().await;

        if !cache.contains_key(old_name) {
            return Err(StickerNotFound);
        }

        if cache.contains_key(new_name) {
            return Err(StickerAlreadyExists);
        }

        if let Some(file_id) = cache.remove(old_name) {
            cache.insert(new_name.to_string(), file_id.clone());

            if let Err(e) = self.persist_cache(&cache).await {
                cache.remove(new_name);
                cache.insert(old_name.to_string(), file_id);

                return Err(StorageError(e));
            }
        }

        Ok(())
    }
    async fn list_stickers(&self) -> Option<Vec<StickerEntry>> {
        let cache = self.cache.read().await;

        if cache.is_empty() {
            return None;
        }

        let entries: Vec<StickerEntry> = cache
            .iter()
            .map(|(name, file_id)| StickerEntry::new(name.clone(), file_id.clone()))
            .collect();

        Some(entries)
    }
    async fn remove_sticker(&self, sticker_name: &str) -> Result<(), ApiError> {
        let mut cache = self.cache.write().await;

        match cache.remove(sticker_name) {
            Some(file_id) => {
                if let Err(e) = self.persist_cache(&cache).await {
                    cache.insert(sticker_name.to_string(), file_id);

                    return Err(StorageError(e));
                }

                Ok(())
            }

            None => Err(StickerNotFound),
        }
    }

    async fn is_already_created(&self, sticker_name: &str) -> bool {
        let cache = self.cache.read().await;

        if cache.get(sticker_name).is_some() {
            return true;
        }

        false
    }
}
