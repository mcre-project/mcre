use super::*;
use alloc::{string::ToString, vec::Vec};
use indexed_db_futures::object_store::ObjectStore;
use js_sys::{ArrayBuffer, Uint8Array};

use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::{Transaction, TransactionMode};

fn file_key(path: &Path) -> String {
    path.to_string()
}

fn dir_key(path: &Path) -> String {
    let s = path.to_string();
    if s.is_empty() {
        "/".to_string()
    } else {
        format!("{}/", s)
    }
}

fn map_db_err(err: indexed_db_futures::error::Error) -> FsError {
    FsError::StorageError(err.to_string())
}

#[derive(Debug, Clone)]
pub struct WebFileSystem {
    db: Database,
}

impl WebFileSystem {
    pub async fn new(app_name: &str) -> FsResult<Self> {
        let db_name = format!("{}_fs", app_name);
        let db = Database::open(db_name)
            .with_version(1u8)
            .with_on_upgrade_needed(|_event, db| {
                let data_store = db.create_object_store("data").build()?;
                data_store.put(Vec::<String>::new()).with_key("/").build()?;
                Ok(())
            })
            .await
            .map_err(|err| FsError::StorageError(err.to_string()))?;

        Ok(Self { db })
    }

    fn tx_rw(&self) -> Result<Transaction<'_>, FsError> {
        self.db
            .transaction("data")
            .with_mode(TransactionMode::Readwrite)
            .build()
            .map_err(map_db_err)
    }

    fn tx_r(&self) -> Result<Transaction<'_>, FsError> {
        self.db
            .transaction("data")
            .with_mode(TransactionMode::Readonly)
            .build()
            .map_err(|err| FsError::StorageError(err.to_string()))
    }

    async fn _delete_dir_recursive(&self, path: &Path, store: &ObjectStore<'_>) -> FsResult<()> {
        let mut stack = Vec::new();
        stack.push((path.to_owned(), true));

        while let Some((path, is_dir)) = stack.pop() {
            if !is_dir {
                self._delete_file(path.as_ref(), store).await?;
                continue;
            }
            let dkey = dir_key(path.as_ref());
            let children_opt: Option<Vec<String>> = store
                .get(&dkey)
                .await
                .map_err(|err| FsError::StorageError(err.to_string()))?;

            let Some(children) = children_opt else {
                return Err(FsError::NotFound);
            };

            for child_name in children {
                let mut child_pb = path.clone();
                let (base_name, is_dir) = if child_name.ends_with("/") {
                    (child_name.trim_end_matches("/").to_string(), true)
                } else {
                    (child_name, false)
                };
                child_pb.push(base_name);

                stack.push((child_pb, is_dir));
            }

            store
                .delete(dkey)
                .await
                .map_err(|err| FsError::StorageError(err.to_string()))?;
        }

        Ok(())
    }

    async fn _delete_file(&self, path: &Path, store: &ObjectStore<'_>) -> FsResult<()> {
        let key = file_key(path);

        store
            .delete(key)
            .await
            .map_err(|err| FsError::StorageError(err.to_string()))
    }

    async fn _create_dir(
        &self,
        path: &Path,
        store: &ObjectStore<'_>,
        create_new: bool,
    ) -> FsResult<()> {
        let dkey = dir_key(path);
        let fkey = file_key(path);

        // Check if dir already exists
        if store
            .get::<Vec<String>, _, _>(&dkey)
            .await
            .map_err(map_db_err)?
            .is_some()
        {
            if create_new {
                return Err(FsError::AlreadyExists);
            } else {
                return Ok(());
            }
        }

        // Check if file at same path
        if store
            .get::<ArrayBuffer, _, _>(fkey)
            .await
            .map_err(map_db_err)?
            .is_some()
        {
            return Err(FsError::NotADirectory);
        }

        if let Some(parent) = path.parent() {
            let parent_key = dir_key(parent);
            let siblings_opt: Option<Vec<_>> = store.get(&parent_key).await.map_err(map_db_err)?;

            let Some(mut siblings) = siblings_opt else {
                return Err(FsError::NotFound);
            };

            siblings.push(format!("{}/", path.base().unwrap()));

            store
                .put(siblings)
                .with_key(parent_key)
                .await
                .map_err(map_db_err)?;
        }

        store
            .put(Vec::<String>::new())
            .with_key(dkey)
            .await
            .map_err(map_db_err)?;

        Ok(())
    }
}

impl FileSystem for WebFileSystem {
    async fn create_dir(&self, path: &Path) -> FsResult<()> {
        let tx = self.tx_rw()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        self._create_dir(path, &store, true).await?;

        tx.commit().await.map_err(map_db_err)?;

        Ok(())
    }

    async fn create_dir_all(&self, path: &Path) -> FsResult<()> {
        let tx = self.tx_rw()?;
        let store = tx.object_store("data").map_err(map_db_err)?;
        let mut current = PathBuf::new();
        for comp in &path.0 {
            current.push(comp.as_ref());
            self._create_dir(current.as_ref(), &store, false).await?;
        }
        Ok(())
    }

    async fn write(&self, path: &Path, data: &[u8]) -> FsResult<()> {
        let fkey = file_key(path);
        let dkey = dir_key(path);
        let tx = self.tx_rw()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        // Check if dir at path
        if store
            .get::<Vec<String>, _, _>(&dkey)
            .await
            .map_err(map_db_err)?
            .is_some()
        {
            return Err(FsError::IsADirectory);
        }

        if let Some(parent) = path.parent() {
            let parent_key = dir_key(parent);
            let siblings_opt: Option<Vec<_>> = store.get(&parent_key).await.map_err(map_db_err)?;

            let Some(mut siblings) = siblings_opt else {
                return Err(FsError::NotFound);
            };

            siblings.push(path.base().unwrap().to_string());

            store
                .put(siblings)
                .with_key(parent_key)
                .await
                .map_err(map_db_err)?;
        }

        let buffer = Uint8Array::new_with_length(data.len() as u32);
        buffer.copy_from(data);

        let buffer = buffer.buffer();

        store
            .put(buffer)
            .with_key(&*fkey)
            .await
            .map_err(map_db_err)?;

        tx.commit().await.map_err(map_db_err)?;

        Ok(())
    }

    async fn read(&self, path: &Path) -> FsResult<Box<[u8]>> {
        let fkey = file_key(path);
        let tx = self.tx_r()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        let buffer_opt: Option<ArrayBuffer> = store.get(fkey).await.map_err(map_db_err)?;

        let Some(buffer) = buffer_opt else {
            return Err(FsError::NotFound);
        };

        let data = Uint8Array::new(&buffer).to_vec();

        tx.commit().await.map_err(map_db_err)?;
        Ok(data.into_boxed_slice())
    }

    async fn delete(&self, path: &Path) -> FsResult<()> {
        let fkey = file_key(path);
        let dkey = dir_key(path);
        let tx = self.tx_rw()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        let d_entry_opt: Option<_> = store
            .get::<Vec<String>, _, _>(&dkey)
            .await
            .map_err(map_db_err)?;

        let is_dir = d_entry_opt.is_some();

        if is_dir {
            self._delete_dir_recursive(path, &store).await?;

            // Remove from parent
            if let Some(parent_path) = path.parent() {
                let p_dkey = dir_key(parent_path);

                let p_entry_opt: Option<Vec<String>> =
                    store.get(&p_dkey).await.map_err(map_db_err)?;

                let base = format!("{}/", path.base().unwrap());

                if let Some(mut children) = p_entry_opt {
                    children.retain(|c| c != &base);
                    store
                        .put(children)
                        .with_key(p_dkey)
                        .await
                        .map_err(map_db_err)?;
                }
            }
        } else {
            let f_entry_opt: Option<ArrayBuffer> = store.get(&fkey).await.map_err(map_db_err)?;

            if f_entry_opt.is_none() {
                return Err(FsError::NotFound);
            }

            // Remove from parent
            if let Some(parent_path) = path.parent() {
                let p_dkey = dir_key(parent_path);

                let p_entry_opt: Option<Vec<String>> =
                    store.get(&p_dkey).await.map_err(map_db_err)?;

                let base = path.base().unwrap().to_string();

                if let Some(mut children) = p_entry_opt {
                    children.retain(|c| c != &base);
                    store
                        .put(children)
                        .with_key(p_dkey)
                        .await
                        .map_err(map_db_err)?;
                }
            }

            // Delete file
            store.delete(&fkey).await.map_err(map_db_err)?;
        }

        tx.commit().await.map_err(map_db_err)?;

        Ok(())
    }

    async fn read_dir(&self, path: &Path) -> FsResult<Box<[PathBuf]>> {
        let dkey = dir_key(path);
        let tx = self.tx_r()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        let children_opt: Option<Vec<String>> = store.get(&dkey).await.map_err(map_db_err)?;

        let Some(children) = children_opt else {
            return Err(FsError::NotFound);
        };

        let mut res = Vec::with_capacity(children.len());
        for name in children {
            let mut pb = path.to_owned();
            pb.push(name.trim_end_matches("/"));
            res.push(pb);
        }

        tx.commit().await.map_err(map_db_err)?;
        Ok(res.into_boxed_slice())
    }

    async fn metadata(&self, path: &Path) -> FsResult<FileMetadata> {
        let fkey = file_key(path);
        let dkey = dir_key(path);
        let tx = self.tx_r()?;
        let store = tx.object_store("data").map_err(map_db_err)?;

        // Check dir
        if store
            .get::<Vec<String>, _, _>(&dkey)
            .await
            .map_err(map_db_err)?
            .is_some()
        {
            tx.commit().await.map_err(map_db_err)?;

            return Ok(FileMetadata {
                is_dir: true,
                len: 0,
            });
        }

        // Check file
        if let Some(buffer) = store
            .get::<ArrayBuffer, _, _>(&fkey)
            .await
            .map_err(map_db_err)?
        {
            tx.commit().await.map_err(map_db_err)?;

            return Ok(FileMetadata {
                is_dir: false,
                len: Uint8Array::new(&buffer).length() as u64,
            });
        }

        tx.commit().await.map_err(map_db_err)?;

        Err(FsError::NotFound)
    }
}
