use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt::{Display, Formatter};
use teloxide::types::UserId;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Sticker,
    Gif,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, FromRow)]
pub struct MediaEntry {
    pub id: Uuid,
    pub name: String,
    pub file_id: String,
    pub media_type: MediaType,
    pub added_by: i64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Display for MediaEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for MediaEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl MediaEntry {
    pub fn new(name: String, file_id: String, user_id: UserId, media_type: MediaType) -> Self {
        MediaEntry {
            id: Uuid::new_v4(),
            name,
            file_id,
            media_type,
            added_by: user_id.0 as i64,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone, Ord, Eq, PartialEq, PartialOrd, FromRow)]
pub struct MediaUserUsage {
    pub media_id: Uuid,
    pub user_id: u64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
