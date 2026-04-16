// use std::collections::HashMap;
//
// use async_trait::async_trait;
// use tokio::fs::{File, OpenOptions};
// use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
// use tokio::sync::{Mutex, RwLock};
//
// use crate::errors::ApiError::{MediaAlreadyExists, MediaNotFound, StorageError};
// use crate::errors::RepoError::{
//     ChangeFileError, FailedToOpenFile, FailedToReadFile, ReadJSONError, WriteJSONError,
// };
// use crate::errors::{ApiError, RepoError};
// use crate::handlers::root_handler::MediaStore;
// use crate::repo::media_storage_postgres::dto::MediaEntry;
//
// pub struct MediaStorage {
//     storage: Mutex<File>,
//     cache: RwLock<HashMap<String, String>>,
// }
//
// impl MediaStorage {
//     // TODO: Make as config
//     pub async fn new(filename: String) -> Result<Self, RepoError> {
//         let mut file = OpenOptions::new()
//             .read(true)
//             .write(true)
//             .create(true)
//             .truncate(false)
//             .open(filename.as_str())
//             .await
//             .map_err(FailedToOpenFile)?;
//
//         file.seek(SeekFrom::Start(0))
//             .await
//             .map_err(ChangeFileError)?;
//
//         let mut media_str = String::new();
//         file.read_to_string(&mut media_str)
//             .await
//             .map_err(FailedToReadFile)?;
//
//         let available_media: Vec<MediaEntry> = if media_str.is_empty() {
//             Vec::new()
//         } else {
//             serde_json::from_str(&media_str).map_err(ReadJSONError)?
//         };
//
//         let mut cache: HashMap<String, String> = HashMap::with_capacity(available_media.len());
//
//         for entry in available_media {
//             cache.insert(entry.name, entry.file_id);
//         }
//
//         Ok(MediaStorage {
//             storage: Mutex::new(file),
//             cache: RwLock::new(cache),
//         })
//     }
//
//     async fn persist_cache(
//         &self,
//         cache_snapshot: &HashMap<String, String>,
//     ) -> Result<(), RepoError> {
//         let media_entries: Vec<MediaEntry> = cache_snapshot
//             .iter()
//             .map(|(name, file_id)| MediaEntry::new(name.clone(), file_id.clone()))
//             .collect();
//
//         let raw_media_entries = serde_json::to_string(&media_entries).map_err(WriteJSONError)?;
//         let mut file_lock = self.storage.lock().await;
//
//         file_lock.set_len(0).await.map_err(ChangeFileError)?;
//         file_lock
//             .seek(SeekFrom::Start(0))
//             .await
//             .map_err(ChangeFileError)?;
//
//         file_lock
//             .write_all(raw_media_entries.as_bytes())
//             .await
//             .map_err(ChangeFileError)?;
//
//         file_lock.flush().await.map_err(ChangeFileError)?;
//
//         Ok(())
//     }
// }
//
// #[async_trait]
// impl MediaStore for MediaStorage {
//     async fn add_media_entry(&self, media_entry: MediaEntry) -> Result<(), ApiError> {
//         let mut cache = self.cache.write().await;
//
//         if cache.contains_key(&media_entry.name) {
//             return Err(MediaAlreadyExists);
//         }
//
//         cache.insert(media_entry.name.clone(), media_entry.file_id);
//
//         if let Err(e) = self.persist_cache(&cache).await {
//             cache.remove(media_entry.name.as_str());
//
//             return Err(StorageError(e));
//         }
//
//         Ok(())
//     }
//
//     async fn get_media_entry(&self, media_entry_name: &str) -> Option<MediaEntry> {
//         let cache = self.cache.read().await;
//         cache
//             .get(media_entry_name)
//             .map(|file_id| MediaEntry::new(media_entry_name.to_string(), file_id.clone()))
//     }
//     async fn rename_media_entry(&self, old_name: &str, new_name: &str) -> Result<(), ApiError> {
//         let mut cache = self.cache.write().await;
//
//         if !cache.contains_key(old_name) {
//             return Err(MediaNotFound);
//         }
//
//         if cache.contains_key(new_name) {
//             return Err(MediaAlreadyExists);
//         }
//
//         if let Some(file_id) = cache.remove(old_name) {
//             cache.insert(new_name.to_string(), file_id.clone());
//
//             if let Err(e) = self.persist_cache(&cache).await {
//                 cache.remove(new_name);
//                 cache.insert(old_name.to_string(), file_id);
//
//                 return Err(StorageError(e));
//             }
//         }
//
//         Ok(())
//     }
//     async fn list_available_media_entries(&self) -> Option<Vec<MediaEntry>> {
//         let cache = self.cache.read().await;
//
//         if cache.is_empty() {
//             return None;
//         }
//
//         let entries: Vec<MediaEntry> = cache
//             .iter()
//             .map(|(name, file_id)| MediaEntry::new(name.clone(), file_id.clone()))
//             .collect();
//
//         Some(entries)
//     }
//     async fn remove_media_entry(&self, media_entry_name: &str) -> Result<(), ApiError> {
//         let mut cache = self.cache.write().await;
//
//         match cache.remove(media_entry_name) {
//             Some(file_id) => {
//                 if let Err(e) = self.persist_cache(&cache).await {
//                     cache.insert(media_entry_name.to_string(), file_id);
//
//                     return Err(StorageError(e));
//                 }
//
//                 Ok(())
//             }
//
//             None => Err(MediaNotFound),
//         }
//     }
//
//     async fn is_already_created(&self, media_entry_name: &str) -> bool {
//         let cache = self.cache.read().await;
//
//         if cache.get(media_entry_name).is_some() {
//             return true;
//         }
//
//         false
//     }
// }
