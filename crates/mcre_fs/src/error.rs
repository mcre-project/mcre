use core::{error, fmt};

use alloc::string::String;

#[derive(Debug, Clone)]
pub enum FsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    InvalidPath,
    StorageError(String),
    Unknown(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for FsError {}

pub type FsResult<T> = Result<T, FsError>;
