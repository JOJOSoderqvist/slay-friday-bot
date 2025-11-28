use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StickerEntry {
    pub name: String,
    pub file_id: String,
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
