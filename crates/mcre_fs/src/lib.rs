#![cfg_attr(all(not(feature = "std"), not(feature = "web")), no_std)]

extern crate alloc;

mod error;
#[cfg(feature = "std")]
mod native;
mod path;
#[cfg(feature = "web")]
mod web;

use alloc::boxed::Box;

pub use error::*;
#[cfg(feature = "std")]
pub use native::NativeFileSystem;
pub use path::*;
#[cfg(feature = "web")]
pub use web::*;

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub is_dir: bool,
    pub len: u64,
}

#[allow(async_fn_in_trait)]
pub trait FileSystem {
    async fn create_dir(&self, path: &Path) -> FsResult<()>;
    async fn create_dir_all(&self, path: &Path) -> FsResult<()>;
    async fn write(&self, path: &Path, data: &[u8]) -> FsResult<()>;
    async fn read(&self, path: &Path) -> FsResult<Box<[u8]>>;
    async fn delete(&self, path: &Path) -> FsResult<()>;
    async fn read_dir(&self, path: &Path) -> FsResult<Box<[PathBuf]>>;
    async fn metadata(&self, path: &Path) -> FsResult<FileMetadata>;

    async fn exists(&self, path: &Path) -> bool {
        self.metadata(path).await.is_ok()
    }
}
