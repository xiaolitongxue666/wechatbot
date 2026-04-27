use crate::error::{Result, WeChatBotError};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct StoredMedia {
    pub storage_backend: String,
    pub storage_key: String,
    pub size_bytes: i64,
    pub sha256: String,
}

#[async_trait]
pub trait MediaStore: Send + Sync {
    async fn save(
        &self,
        session_id: &str,
        message_id: &str,
        media_type: &str,
        media_bytes: &[u8],
    ) -> Result<StoredMedia>;
}

pub struct LocalFsMediaStore {
    root: PathBuf,
}

impl LocalFsMediaStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl MediaStore for LocalFsMediaStore {
    async fn save(
        &self,
        session_id: &str,
        message_id: &str,
        media_type: &str,
        media_bytes: &[u8],
    ) -> Result<StoredMedia> {
        let mut hasher = Sha256::new();
        hasher.update(media_bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        let file_name = format!("{message_id}_{sha256}");
        let relative = PathBuf::from(session_id).join(media_type).join(file_name);
        let output_path = self.root.join(&relative);

        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::File::create(&output_path).await?;
        file.write_all(media_bytes).await?;
        file.flush().await?;
        Ok(StoredMedia {
            storage_backend: "localfs".into(),
            storage_key: relative.to_string_lossy().to_string(),
            size_bytes: media_bytes.len() as i64,
            sha256,
        })
    }
}

pub struct S3CompatibleMediaStore {
    bucket: String,
    endpoint: String,
}

impl S3CompatibleMediaStore {
    pub fn new(bucket: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl MediaStore for S3CompatibleMediaStore {
    async fn save(
        &self,
        session_id: &str,
        message_id: &str,
        media_type: &str,
        media_bytes: &[u8],
    ) -> Result<StoredMedia> {
        let mut hasher = Sha256::new();
        hasher.update(media_bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        let object_key = format!("{}/{}/{}_{}", session_id, media_type, message_id, sha256);
        if self.bucket.is_empty() || self.endpoint.is_empty() {
            return Err(WeChatBotError::Other("s3 store is not configured".into()));
        }
        Ok(StoredMedia {
            storage_backend: "s3".into(),
            storage_key: object_key,
            size_bytes: media_bytes.len() as i64,
            sha256,
        })
    }
}
