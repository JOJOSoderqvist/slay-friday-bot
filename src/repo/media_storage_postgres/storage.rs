use std::sync::Arc;
use async_trait::async_trait;
use crate::adapter::postgres::PgStore;
use crate::errors::ApiError;
use crate::errors::ApiError::{MediaAlreadyExists, MediaNotFound, StorageError};
use crate::errors::RepoError;
use crate::errors::RepoError::DBError;
use crate::handlers::root_handler::MediaStore;
use crate::repo::media_storage_postgres::dto::MediaEntry;

pub struct PGMediaStorage {
    storage: PgStore
}

impl PGMediaStorage {
    pub fn new(pool: PgStore) -> Self {
        Self {
            storage: pool,
        }
    }

    async fn add_media_entry(&self, media_entry: MediaEntry) -> Result<(), ApiError> {
        match sqlx::query(
            r"insert into media (id, name, file_id, media_type, added_by)
                values ($1, $2, $3, $4, $5);"
            )
            .bind(media_entry.id)
            .bind(media_entry.name)
            .bind(media_entry.file_id)
            .bind(media_entry.media_type)
            .bind(media_entry.added_by)
            .execute(&self.storage.pool)
            .await
        {
            Ok(..) => Ok(()),
            Err(sqlx::Error::Database(e)) => {
                if e.is_unique_violation() {
                    return Err(MediaAlreadyExists)
                }

                // TODO: WTF is this??
                Err(StorageError(DBError(sqlx::Error::Database(e))))
            }

            Err(e) => {
                Err(StorageError(DBError(e)))
            }
        }
    }

    async fn get_media_entry(&self, media_entry_name: &str) -> Result<Option<MediaEntry>, ApiError> {
        let media_entry = sqlx::query_as(
            r"select id, name, file_id, media_type, added_by, created_at, updated_at
                  from media where name = $1;"
        )
            .bind(media_entry_name)
            .fetch_optional(&self.storage.pool)
            .await
            .map_err(DBError)?;


        Ok(media_entry)
    }

    async fn rename_media_entry(&self, old_entry_name: &str, new_entry_name: &str) -> Result<(), ApiError> {
        let res = sqlx::query(
            r"update media set name = $1 where name = $2"
        )
            .bind(new_entry_name)
            .bind(old_entry_name)
            .execute(&self.storage.pool)
            .await
            .map_err(DBError)?;
        
        
        if res.rows_affected() != 1 {
            return Err(MediaNotFound)
        }
        
        Ok(())
    }

    async fn list_available_media_entries(&self) -> Option<Vec<MediaEntry>> {
        todo!()
    }

    async fn remove_media_entry(&self, media_entry_name: &str) -> Result<(), ApiError> {
        todo!()
    }

    async fn is_already_created(&self, media_entry_name: &str) -> bool {
        todo!()
    }
}

#[async_trait]
impl MediaStore for PGMediaStorage {
    async fn add_media_entry(&self, media_entry: MediaEntry) -> Result<(), ApiError> {
        todo!()
    }

    async fn get_media_entry(&self, media_entry_name: &str) -> Option<MediaEntry> {
        todo!()
    }

    async fn rename_media_entry(&self, old_entry_name: &str, new_entry_name: &str) -> Result<(), ApiError> {
        todo!()
    }

    async fn list_available_media_entries(&self) -> Option<Vec<MediaEntry>> {
        todo!()
    }

    async fn remove_media_entry(&self, media_entry_name: &str) -> Result<(), ApiError> {
        todo!()
    }

    async fn is_already_created(&self, media_entry_name: &str) -> bool {
        todo!()
    }
}