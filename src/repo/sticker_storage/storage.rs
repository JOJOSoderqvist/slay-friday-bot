use std::collections::HashMap;

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

use crate::errors::ApiError::{StickerAlreadyExists, StickerNotFound};
use crate::errors::RepoError::{ChangeFileError, FailedToOpenFile, ReadJSONError, WriteJSONError};
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
    pub async fn new(filename: String) -> Result<Self, RepoError> {
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

    async fn get_stickers(file: &mut File) -> Result<Vec<StickerEntry>, RepoError> {
        let mut stickers_string = String::new();
        file.read_to_string(&mut stickers_string)
            .await
            .map_err(FailedToOpenFile)?;
        let stickers: Vec<StickerEntry> =
            serde_json::from_str(stickers_string.as_str()).map_err(ReadJSONError)?;
        Ok(stickers)
    }

    async fn write_stickers(file: &mut File, stickers: Vec<StickerEntry>) -> Result<(), RepoError> {
        let sticker_raw = serde_json::to_string(&stickers).map_err(WriteJSONError)?;
        file.set_len(0).await.map_err(ChangeFileError)?;
        file.seek(SeekFrom::Start(0))
            .await
            .map_err(ChangeFileError)?;

        file.write_all(sticker_raw.as_bytes())
            .await
            .map_err(ChangeFileError)?;

        file.flush().await.map_err(ChangeFileError)?;
        Ok(())
    }
}

#[async_trait]
impl StickerStore for StickerStorage {
    async fn add_sticker(&self, sticker: StickerEntry) -> Result<(), ApiError> {
        {
            let mut file_lock = self.storage.lock().await;
            let mut stickers = StickerStorage::get_stickers(&mut file_lock).await?;

            if !stickers.contains(&sticker) {
                stickers.push(sticker.clone());
                StickerStorage::write_stickers(&mut file_lock, stickers).await?;
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
    async fn rename_sticker(&self, old_name: &str, new_name: &str) -> Result<(), ApiError> {
        {
            let mut file_lock = self.storage.lock().await;
            let mut stickers = StickerStorage::get_stickers(&mut file_lock).await?;
            if let Some(sticker) = stickers
                .iter_mut()
                .find(|sticker| sticker.name == old_name)
            {
                sticker.name = new_name.to_string();
                StickerStorage::write_stickers(&mut file_lock, stickers).await?;
            } else {
                return Err(StickerNotFound);
            }
        }

        {
            let mut cache_lock = self.cache.lock().await;

            if let Some(file_id) = cache_lock.remove(old_name) {
                cache_lock.insert(new_name.to_string(), file_id);
            }
        }

        Ok(())
    }
    async fn list_stickers(&self) -> Option<Vec<StickerEntry>> {
        let entries: Vec<StickerEntry> = {
            let cache_lock = self.cache.lock().await;

            let mut entries: Vec<StickerEntry> = Vec::new();
            entries.reserve(cache_lock.len());
            for (name, file_id) in cache_lock.iter() {
                entries.push(StickerEntry::new(name.clone(), file_id.clone()));
            }
            entries
        };

        match entries.len() {
            0 => None,
            _ => Some(entries),
        }
    }
    async fn remove_sticker(&self, sticker_name: &str) -> Result<(), ApiError> {
        {
            let mut file_lock = self.storage.lock().await;
            let mut stickers = StickerStorage::get_stickers(&mut file_lock).await?;
            if let Some(index) = stickers
                .iter()
                .position(|sticker| sticker.name == sticker_name)
            {
                stickers.swap_remove(index);
                StickerStorage::write_stickers(&mut file_lock, stickers).await?;
            } else {
                return Err(StickerNotFound);
            }
        }

        {
            let mut cache_lock = self.cache.lock().await;
            cache_lock.remove(sticker_name);
        }

        Ok(())
    }
}
