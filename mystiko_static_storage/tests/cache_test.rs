use mystiko_static_storage::{CachedStorage, CachedStorageOptions, FileStorage, GetRequest, PutRequest, Storage};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_list_folders() {
    let (raw, cache, _) = setup().await;
    raw.put("a/b/file.txt".into()).await.unwrap();
    raw.put("a/c/file.txt".into()).await.unwrap();
    let folders = cache
        .list_folders(PathBuf::default().into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(folders, vec!["a"]);
    let mut folders = cache
        .list_folders("a".into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    folders.sort();
    assert_eq!(folders, vec!["a/b", "a/c"]);
}

#[tokio::test]
async fn test_list_files() {
    let (raw, cache, _) = setup().await;
    raw.put("a/b/file.txt".into()).await.unwrap();
    raw.put("a/c/file.txt".into()).await.unwrap();
    let mut files = cache
        .list_files(PathBuf::default().into())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, vec!["a/b/file.txt", "a/c/file.txt"]);
    let mut files = cache
        .list_files("a".into())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, vec!["a/b/file.txt", "a/c/file.txt"]);
    raw.put("a/file.txt".into()).await.unwrap();
    let files = cache
        .list_files(
            mystiko_static_storage::ListFilesRequest::builder()
                .path("a")
                .non_recursively(true)
                .build(),
        )
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(files, vec!["a/file.txt"]);
}

#[tokio::test]
async fn test_exists() {
    let (raw, cache, _) = setup().await;
    raw.put("a/b/file.txt".into()).await.unwrap();
    raw.put("a/c/file.txt".into()).await.unwrap();
    assert!(cache.exists("a/b/file.txt".into()).await.unwrap().exists);
    assert!(cache.exists("a/c/file.txt".into()).await.unwrap().exists);
    assert!(!cache.exists("a/d/file.txt".into()).await.unwrap().exists);
}

#[tokio::test]
async fn test_get() {
    let (raw, cache, _) = setup().await;
    raw.put(PutRequest::builder().path("a/file.txt").data("hello world #1").build())
        .await
        .unwrap();
    let response = cache.get("a/file.txt".into()).await.unwrap();
    assert_eq!(response.data, b"hello world #1");
    raw.put(
        PutRequest::builder()
            .path("a/file.txt")
            .data("hello world #2")
            .overwrite(true)
            .build(),
    )
    .await
    .unwrap();
    let response = cache.get("a/file.txt".into()).await.unwrap();
    assert_eq!(response.data, b"hello world #1");
    let response = cache
        .get(GetRequest::builder().path("a/file.txt").no_cache(true).build())
        .await
        .unwrap();
    assert_eq!(response.data, b"hello world #2");
}

#[tokio::test]
async fn test_put() {
    let (raw, cache, _) = setup().await;
    let put_request = PutRequest::builder().path("a/file.txt").data("hello world #1").build();
    cache.put(put_request).await.unwrap();
    let get_response = raw.get("a/file.txt".into()).await.unwrap();
    assert_eq!(get_response.data, b"hello world #1");
    let put_request = PutRequest::builder()
        .path("a/file.txt")
        .data("hello world #2")
        .overwrite(true)
        .build();
    raw.put(put_request).await.unwrap();
    let get_response = cache.get("a/file.txt".into()).await.unwrap();
    assert_eq!(get_response.data, b"hello world #1");
    let get_request = GetRequest::builder().path("a/file.txt").no_cache(true).build();
    let get_response = cache.get(get_request).await.unwrap();
    assert_eq!(get_response.data, b"hello world #2");
}

#[tokio::test]
async fn test_remove_files() {
    let (raw, cache, _) = setup().await;
    let put_request1 = PutRequest::builder().path("a/file.txt").data("hello world #1").build();
    let put_request2 = PutRequest::builder().path("b/file.txt").data("hello world #1").build();
    cache.put(put_request1).await.unwrap();
    raw.put(put_request2).await.unwrap();

    cache
        .remove_files(vec![PathBuf::from("a/file.txt")].into())
        .await
        .unwrap();
    assert!(!raw.exists("a/file.txt".into()).await.unwrap().exists);
    assert!(!cache.exists("a/file.txt".into()).await.unwrap().exists);
    assert!(cache.get("a/file.txt".into()).await.is_err());

    cache
        .remove_files(vec![PathBuf::from("b/file.txt")].into())
        .await
        .unwrap();
    assert!(!raw.exists("b/file.txt".into()).await.unwrap().exists);
    assert!(!cache.exists("b/file.txt".into()).await.unwrap().exists);
    assert!(cache.get("b/file.txt".into()).await.is_err());
}

#[tokio::test]
async fn test_remove_file() {
    let (raw, cache, _) = setup().await;
    let put_request1 = PutRequest::builder().path("a/file.txt").data("hello world #1").build();
    let put_request2 = PutRequest::builder().path("b/file.txt").data("hello world #1").build();
    cache.put(put_request1).await.unwrap();
    raw.put(put_request2).await.unwrap();

    cache.remove_file("a/file.txt".into()).await.unwrap();
    assert!(!raw.exists("a/file.txt".into()).await.unwrap().exists);
    assert!(!cache.exists("a/file.txt".into()).await.unwrap().exists);
    assert!(cache.get("a/file.txt".into()).await.is_err());

    cache.remove_file("b/file.txt".into()).await.unwrap();
    assert!(!raw.exists("b/file.txt".into()).await.unwrap().exists);
    assert!(!cache.exists("b/file.txt".into()).await.unwrap().exists);
    assert!(cache.get("b/file.txt".into()).await.is_err());
}

#[tokio::test]
async fn test_remove_folder() {
    let (raw, cache, _) = setup().await;
    let put_request1 = PutRequest::builder().path("a/file.txt").data("hello world #1").build();
    let put_request2 = PutRequest::builder()
        .path("a/b/file.txt")
        .data("hello world #1")
        .build();
    let put_request3 = PutRequest::builder()
        .path("a/b/c/file.txt")
        .data("hello world #1")
        .build();
    cache.put(put_request1).await.unwrap();
    cache.put(put_request2).await.unwrap();
    raw.put(put_request3).await.unwrap();

    cache
        .remove_files(
            vec![
                PathBuf::from("a/file.txt"),
                PathBuf::from("a/b/file.txt"),
                PathBuf::from("a/b/c/file.txt"),
            ]
            .into(),
        )
        .await
        .unwrap();
    cache.remove_folder("a/b/c".into()).await.unwrap();
    assert!(!raw.exists("a/b/c".into()).await.unwrap().exists);
    assert!(!cache.exists("a/b/c".into()).await.unwrap().exists);
    cache.remove_folder("a/b".into()).await.unwrap();
    assert!(!raw.exists("a/b".into()).await.unwrap().exists);
    assert!(!cache.exists("a/b".into()).await.unwrap().exists);
    assert!(cache.get("a/b/file.txt".into()).await.is_err());
    cache.remove_folder("a".into()).await.unwrap();
    assert!(!raw.exists("a".into()).await.unwrap().exists);
    assert!(!cache.exists("a".into()).await.unwrap().exists);
    assert!(cache.get("a/file.txt".into()).await.is_err());
}

async fn setup() -> (Arc<FileStorage>, CachedStorage<FileStorage>, Vec<tempfile::TempDir>) {
    let raw_dir = tempfile::tempdir().unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let raw = Arc::new(FileStorage::new(raw_dir.path()).await.unwrap());
    let cache = CachedStorage::new(
        CachedStorageOptions::<PathBuf, FileStorage>::builder()
            .raw(raw.clone())
            .cache_folder(PathBuf::from(cache_dir.path()))
            .build(),
    )
    .await
    .unwrap();
    (raw, cache, vec![raw_dir, cache_dir])
}
