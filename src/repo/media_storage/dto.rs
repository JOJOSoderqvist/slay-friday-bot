use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct MediaEntry {
    pub name: String,
    pub file_id: String,
}

impl Display for MediaEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl MediaEntry {
    pub fn new(name: String, file_id: String) -> Self {
        MediaEntry { name, file_id }
    }
}

impl PartialEq for MediaEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
