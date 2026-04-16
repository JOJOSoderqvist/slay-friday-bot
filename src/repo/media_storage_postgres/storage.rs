use crate::adapter::postgres::PgStore;
use crate::errors::ApiError;
use crate::errors::ApiError::{MediaAlreadyExists, MediaNotFound, StorageError};
use crate::errors::RepoError::DBError;
use crate::handlers::root_handler::MediaStore;
use crate::repo::media_storage_postgres::dto::MediaEntry;
use async_trait::async_trait;
use teloxide::types::UserId;
use tracing::warn;

pub struct PGMediaStorage {
    storage: PgStore,
}

// TODO: Change return type from ApiError to DBError
impl PGMediaStorage {
    pub fn new(pool: PgStore) -> Self {
        Self { storage: pool }
    }
}

#[async_trait]
impl MediaStore for PGMediaStorage {
    async fn add_media_entry(&self, media_entry: MediaEntry) -> Result<(), ApiError> {
        match sqlx::query(
            r"insert into media (id, name, file_id, media_type, added_by)
                values ($1, $2, $3, $4, $5);",
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
                    return Err(MediaAlreadyExists);
                }

                // TODO: WTF is this??
                Err(StorageError(DBError(sqlx::Error::Database(e))))
            }

            Err(e) => Err(StorageError(DBError(e))),
        }
    }

    async fn get_media_entry(
        &self,
        media_entry_name: &str,
        user_id: UserId,
    ) -> Result<Option<MediaEntry>, ApiError> {
        let mut tx = self.storage.pool.begin().await.map_err(DBError)?;

        let media_entry = sqlx::query_as::<_, MediaEntry>(
            r"select id, name, file_id, media_type, added_by, created_at, updated_at
                  from media where name = $1;",
        )
        .bind(media_entry_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DBError)?;

        let entry: MediaEntry = match media_entry {
            None => {
                tx.commit().await.map_err(DBError)?;
                return Ok(None);
            }
            Some(e) => e,
        };

        let res = sqlx::query(
            r"insert into media_user_usage (media_id, user_id)
                values ($1, $2)
                on CONFLICT (media_id, user_id) do update
                set usage_count = media_user_usage.usage_count + 1;",
        )
        .bind(entry.id)
        .bind(user_id.0 as i64)
        .execute(&mut *tx)
        .await
        .map_err(DBError)?;

        if res.rows_affected() != 1 {
            warn!(%entry.id, %user_id, "failed to increment usage count")
        }

        tx.commit().await.map_err(DBError)?;
        Ok(Some(entry))
    }

    async fn rename_media_entry(
        &self,
        old_entry_name: &str,
        new_entry_name: &str,
    ) -> Result<(), ApiError> {
        let res = sqlx::query(r"update media set name = $1 where name = $2")
            .bind(new_entry_name)
            .bind(old_entry_name)
            .execute(&self.storage.pool)
            .await
            .map_err(DBError)?;

        if res.rows_affected() != 1 {
            return Err(MediaNotFound);
        }

        Ok(())
    }

    async fn list_available_media_entries(&self) -> Result<Vec<MediaEntry>, ApiError> {
        let media_entries = sqlx::query_as::<_, MediaEntry>(
            r"select m.id, m.name, m.file_id, m.media_type, m.added_by, m.created_at, m.updated_at from media m
                order by m.name desc;"
        )
            .fetch_all(&self.storage.pool)
            .await
            .map_err(DBError)?;

        Ok(media_entries)
    }

    async fn list_user_specific_media_entries(
        &self,
        user_id: UserId,
    ) -> Result<Vec<MediaEntry>, ApiError> {
        let media_entries = sqlx::query_as::<_, MediaEntry>(
            r"select m.id, m.name, m.file_id, m.media_type, m.added_by, m.created_at, m.updated_at
                from media m
                left join media_user_usage mu on mu.user_id = $1 and mu.media_id = m.id
                order by coalesce(mu.usage_count, 0) desc;",
        )
        .bind(user_id.0 as i64)
        .fetch_all(&self.storage.pool)
        .await
        .map_err(DBError)?;

        Ok(media_entries)
    }

    async fn remove_media_entry(&self, media_entry_name: &str) -> Result<bool, ApiError> {
        let res = sqlx::query(r"delete from media where name = $1;")
            .bind(media_entry_name)
            .execute(&self.storage.pool)
            .await
            .map_err(DBError)?;

        Ok(res.rows_affected() == 1)
    }

    async fn is_already_created(&self, media_entry_name: &str) -> Result<bool, ApiError> {
        let res = sqlx::query("select id from media where name = $1;")
            .bind(media_entry_name)
            .fetch_optional(&self.storage.pool)
            .await
            .map_err(DBError)?;

        Ok(res.is_some())
    }
}
