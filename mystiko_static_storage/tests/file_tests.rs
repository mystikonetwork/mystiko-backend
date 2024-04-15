use mystiko_static_storage::{FileStorage, ListFilesRequest, PutRequest, RemoveFolderRequest, Storage};
use std::path::PathBuf;
use tokio::fs;

#[tokio::test]
async fn test_list_folders() {
    let (_, storage) = setup().await;
    assert!(storage.list_folders("a".into()).await.is_err());
    storage.put("a/test.txt".into()).await.unwrap();
    let folders = storage
        .list_folders(PathBuf::default().into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(folders, vec!["a"]);
    assert!(storage.list_folders("a".into()).await.unwrap().folders.is_empty());
    storage.put("a/b/test.txt".into()).await.unwrap();
    storage.put("a/b/c/test.txt".into()).await.unwrap();
    let folders = storage
        .list_folders("a".into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(folders, vec!["a/b"]);
}

#[tokio::test]
async fn test_list_files() {
    let (_, storage) = setup().await;
    assert!(storage.list_files("a".into()).await.is_err());
    storage.put("a/test.txt".into()).await.unwrap();
    storage.put("a/b/test.txt".into()).await.unwrap();
    storage.put("a/b/c/test.txt".into()).await.unwrap();
    let files = storage
        .list_files("a".into())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(files, vec!["a/test.txt", "a/b/test.txt", "a/b/c/test.txt",]);
    let files = storage
        .list_files(ListFilesRequest::builder().path("a").non_recursively(true).build())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(files, vec!["a/test.txt"]);
}

#[tokio::test]
async fn test_exists() {
    let (_, storage) = setup().await;
    assert!(!storage.exists("a/test.txt".into()).await.unwrap().exists);
    storage.put("a/test.txt".into()).await.unwrap();
    assert!(storage.exists("a/test.txt".into()).await.unwrap().exists);
}

#[tokio::test]
async fn test_get() {
    let (_, storage) = setup().await;
    assert!(storage.get("a/test.txt".into()).await.is_err());
}

#[tokio::test]
async fn test_put() {
    let (_, storage) = setup().await;
    let put_request = PutRequest::builder().path("a/test.txt").data("hello world!").build();
    storage.put(put_request).await.unwrap();
    assert_eq!(
        String::from_utf8(storage.get("a/test.txt".into()).await.unwrap().data).unwrap(),
        "hello world!"
    );
}

#[tokio::test]
async fn test_put_overwrite() {
    let (_, storage) = setup().await;
    storage.put("a/test.txt".into()).await.unwrap();
    let mut put_request = PutRequest::builder()
        .path("a/test.txt")
        .data("hello world!")
        .overwrite(false)
        .build();
    storage.put(put_request.clone()).await.unwrap();
    assert_eq!(
        String::from_utf8(storage.get("a/test.txt".into()).await.unwrap().data).unwrap(),
        ""
    );
    put_request.overwrite = true;
    storage.put(put_request).await.unwrap();
    assert_eq!(
        String::from_utf8(storage.get("a/test.txt".into()).await.unwrap().data).unwrap(),
        "hello world!"
    );
}

#[tokio::test]
async fn test_remove_files() {
    let (_, storage) = setup().await;
    storage.put("a/test.txt".into()).await.unwrap();
    storage.put("a/b/test.txt".into()).await.unwrap();
    storage.put("a/b/c/test.txt".into()).await.unwrap();
    let files: Vec<PathBuf> = vec![PathBuf::from("a/test.txt"), PathBuf::from("a/b/test.txt")];
    storage.remove_files(files.into()).await.unwrap();
    storage.remove_file("a/b/c/test.txt".into()).await.unwrap();
    assert!(!storage.exists("a/test.txt".into()).await.unwrap().exists);
    assert!(!storage.exists("a/b/test.txt".into()).await.unwrap().exists);
    assert!(!storage.exists("a/b/c/test.txt".into()).await.unwrap().exists);
}

#[tokio::test]
async fn test_remove_folder() {
    let (_, storage) = setup().await;
    storage.put("a/test.txt".into()).await.unwrap();
    storage.put("a/b/test.txt".into()).await.unwrap();
    storage.put("a/b/c/test.txt".into()).await.unwrap();
    assert!(storage
        .remove_folder(RemoveFolderRequest::builder().path("a").non_recursively(true).build())
        .await
        .is_err());
    storage.remove_folder("a/b".into()).await.unwrap();
    assert!(storage.exists("a/test.txt".into()).await.unwrap().exists);
    assert!(!storage.exists("a/b/test.txt".into()).await.unwrap().exists);
    assert!(!storage.exists("a/b/c/test.txt".into()).await.unwrap().exists);
    storage
        .remove_folder(RemoveFolderRequest::builder().path("a").build())
        .await
        .unwrap();
    assert!(!storage.exists("a/test.txt".into()).await.unwrap().exists);
}

#[tokio::test]
async fn test_construct_with_non_existing_path() {
    let path = tempfile::tempdir().unwrap().path().to_path_buf();
    let storage = FileStorage::new(path.clone()).await.unwrap();
    storage.put("a/test.txt".into()).await.unwrap();
    assert!(storage.exists("a/test.txt".into()).await.unwrap().exists);
    fs::remove_dir_all(path).await.unwrap();
}

async fn setup() -> (tempfile::TempDir, FileStorage) {
    let temp_dir = tempfile::tempdir().unwrap();
    let base = PathBuf::from(temp_dir.path());
    (temp_dir, FileStorage::new(base).await.unwrap())
}
