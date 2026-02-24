use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::mem::swap;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct StickerEntry {
    pub name: String,
    pub file_id: String,
}

impl Display for StickerEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl StickerEntry {
    pub fn new(name: String, file_id: String) -> Self {
        StickerEntry { name, file_id }
    }
}

impl PartialEq for StickerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
