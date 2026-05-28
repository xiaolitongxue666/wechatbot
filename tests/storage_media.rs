//! Backend integration tests: media storage.

mod common;

use tempfile::TempDir;
use wechatbot::storage::media::{LocalFsMediaStore, MediaStore, S3CompatibleMediaStore};

#[tokio::test]
async fn local_fs_media_store_save() {
    let tmp = TempDir::new().unwrap();
    let store = LocalFsMediaStore::new(tmp.path());

    let data = b"hello world media test";
    let stored = store
        .save("session-a", "msg-001", "image", data)
        .await
        .unwrap();

    assert_eq!(stored.storage_backend, "localfs");
    assert_eq!(stored.size_bytes, data.len() as i64);
    assert!(!stored.sha256.is_empty());
    assert!(stored.storage_key.contains("msg-001"));
    assert!(stored.storage_key.contains(&stored.sha256));

    let expected_path = tmp.path().join(&stored.storage_key);
    let content = tokio::fs::read(&expected_path).await.unwrap();
    assert_eq!(content, data);
}

#[tokio::test]
async fn local_fs_media_store_sha256_deterministic() {
    let tmp = TempDir::new().unwrap();
    let store = LocalFsMediaStore::new(tmp.path());
    let data = b"same data";

    let a = store.save("s", "m1", "image", data).await.unwrap();
    let b = store.save("s", "m2", "image", data).await.unwrap();

    assert_eq!(a.sha256, b.sha256, "SHA-256 should be deterministic");
}

#[tokio::test]
async fn s3_store_rejects_unconfigured() {
    let store = S3CompatibleMediaStore::new("", "");
    let result = store.save("s", "m", "image", b"test").await;
    assert!(
        result.is_err(),
        "S3 store with empty config should return error"
    );
}
