//! Integration tests across crate boundaries.

mod audio;
mod autosave;
#[cfg(feature = "dev-bridge")]
mod dev_bridge;
mod guide_setting;
mod migration;
mod sqlite_storage;
