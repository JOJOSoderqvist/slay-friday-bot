pub mod adapter;
pub mod common;
pub mod errors;

#[path = "repo/media_storage/dto.rs"]
pub mod media_storage_dto;

pub mod repo {
    pub mod media_storage {
        pub use crate::media_storage_dto as dto;
    }
}
