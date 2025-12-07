use super::*;
use std::io;
use std::path::{Path as StdPath, PathBuf as StdPathBuf};
use tokio::fs;

/// FileSystem implementation for native std that uses application-specific data directories
#[derive(Debug, Clone)]
pub struct NativeFileSystem {
    base_dir: StdPathBuf,
}

impl NativeFileSystem {
    /// Creates a new NativeFileSystem using platform-specific application data directory
    pub async fn new(app_name: &str) -> Result<Self, io::Error> {
        let base_dir = Self::get_app_data_dir(app_name)?;
        fs::create_dir_all(&base_dir).await?;
        Ok(Self { base_dir })
    }

    /// Creates with a custom base directory
    pub async fn with_base_dir<P: AsRef<StdPath>>(base_dir: P) -> Result<Self, io::Error> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir).await?;
        Ok(Self { base_dir })
    }

    /// Gets the base directory path
    pub fn base_dir(&self) -> &StdPath {
        &self.base_dir
    }

    /// Gets platform-specific application data directory
    fn get_app_data_dir(app_name: &str) -> Result<StdPathBuf, io::Error> {
        #[cfg(target_os = "windows")]
        {
            let mut path = dirs::data_dir().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
            })?;
            path.push(app_name);
            Ok(path)
        }

        #[cfg(target_os = "macos")]
        {
            let mut path = dirs::data_dir().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
            })?;
            path.push("Application Support");
            path.push(app_name);
            Ok(path)
        }

        #[cfg(target_os = "linux")]
        {
            let mut path = dirs::data_dir().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
            })?;
            path.push(app_name);
            Ok(path)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported operating system",
            ))
        }
    }

    /// Converts custom Path to std::path::PathBuf
    fn to_std_path(&self, path: &Path) -> StdPathBuf {
        let mut std_path = self.base_dir.clone();

        for component in &path.0 {
            std_path.push(&**component);
        }

        std_path
    }

    /// Converts std::io::Error to FsError
    fn io_error_to_fs_error(&self, error: io::Error, path: &Path) -> FsError {
        match error.kind() {
            io::ErrorKind::NotFound => FsError::NotFound,
            io::ErrorKind::PermissionDenied => FsError::PermissionDenied,
            io::ErrorKind::AlreadyExists => FsError::AlreadyExists,
            io::ErrorKind::InvalidInput => FsError::InvalidPath,
            io::ErrorKind::InvalidData => FsError::InvalidPath,
            io::ErrorKind::NotADirectory => FsError::NotADirectory,
            io::ErrorKind::IsADirectory => FsError::IsADirectory,
            io::ErrorKind::DirectoryNotEmpty => {
                FsError::StorageError("Directory not empty".to_string())
            }
            _ => FsError::StorageError(format!("IO error at {}: {}", path, error)),
        }
    }
}

impl FileSystem for NativeFileSystem {
    async fn create_dir(&self, path: &Path) -> FsResult<()> {
        let std_path = self.to_std_path(path);

        fs::create_dir(&std_path)
            .await
            .map_err(|e| self.io_error_to_fs_error(e, path))
    }

    async fn create_dir_all(&self, path: &Path) -> FsResult<()> {
        let std_path = self.to_std_path(path);

        fs::create_dir_all(&std_path)
            .await
            .map_err(|e| self.io_error_to_fs_error(e, path))
    }

    async fn write(&self, path: &Path, data: &[u8]) -> FsResult<()> {
        let std_path = self.to_std_path(path);

        // Ensure parent directory exists
        if let Some(parent) = std_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).await.map_err(|e| {
                FsError::StorageError(format!("Failed to create parent directory: {}", e))
            })?;
        }

        fs::write(&std_path, data)
            .await
            .map_err(|e| self.io_error_to_fs_error(e, path))
    }

    async fn read(&self, path: &Path) -> FsResult<Box<[u8]>> {
        let std_path = self.to_std_path(path);

        fs::read(&std_path)
            .await
            .map(|data| data.into_boxed_slice())
            .map_err(|e| self.io_error_to_fs_error(e, path))
    }

    async fn delete(&self, path: &Path) -> FsResult<()> {
        let std_path = self.to_std_path(path);

        let metadata = fs::metadata(&std_path).await;

        match metadata {
            Ok(metadata) => {
                if metadata.is_file() {
                    fs::remove_file(&std_path)
                        .await
                        .map_err(|e| self.io_error_to_fs_error(e, path))
                } else if metadata.is_dir() {
                    // Try to remove as empty directory first
                    match fs::remove_dir(&std_path).await {
                        Ok(()) => Ok(()),
                        Err(e) if e.kind() == io::ErrorKind::DirectoryNotEmpty => {
                            fs::remove_dir_all(&std_path)
                                .await
                                .map_err(|e| self.io_error_to_fs_error(e, path))
                        }
                        Err(e) => Err(self.io_error_to_fs_error(e, path)),
                    }
                } else {
                    Err(FsError::InvalidPath)
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Err(FsError::NotFound)
                } else {
                    Err(self.io_error_to_fs_error(e, path))
                }
            }
        }
    }

    async fn read_dir(&self, path: &Path) -> FsResult<Box<[PathBuf]>> {
        let std_path = self.to_std_path(path);

        let mut read_dir = fs::read_dir(&std_path)
            .await
            .map_err(|e| self.io_error_to_fs_error(e, path))?;

        let mut results = Vec::new();

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| FsError::StorageError(format!("Failed to read directory entry: {}", e)))?
        {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy().into_owned();

            let mut path_buf = PathBuf::new();
            path_buf.push(name_str);

            results.push(path_buf);
        }

        Ok(results.into_boxed_slice())
    }

    async fn metadata(&self, path: &Path) -> FsResult<FileMetadata> {
        let std_path = self.to_std_path(path);

        let metadata = fs::metadata(&std_path)
            .await
            .map_err(|e| self.io_error_to_fs_error(e, path))?;

        Ok(FileMetadata {
            is_dir: metadata.is_dir(),
            len: metadata.len(),
        })
    }
}
