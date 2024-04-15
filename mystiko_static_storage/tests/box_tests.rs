use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use mystiko_static_storage::{
    ExistsRequest, ExistsResponse, GetRequest, GetResponse, ListFilesRequest, ListFilesResponse, ListFoldersRequest,
    ListFoldersResponse, PutRequest, PutResponse, RemoveFileRequest, RemoveFileResponse, RemoveFilesRequest,
    RemoveFilesResponse, RemoveFolderRequest, RemoveFolderResponse, Storage as StorageTrait,
};
use std::sync::Arc;

mock! {
    #[derive(Debug)]
    Storage {}

    #[async_trait]
    impl StorageTrait for Storage {
        async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse>;
        async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse>;
        async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse>;
        async fn get(&self, request: GetRequest) -> Result<GetResponse>;
        async fn put(&self, request: PutRequest) -> Result<PutResponse>;
        async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse>;
        async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse>;
        async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse>;
    }
}

#[tokio::test]
async fn test_box_impl() {
    let mut storage = MockStorage::new();
    storage
        .expect_list_folders()
        .times(1)
        .returning(|_| Ok(ListFoldersResponse::default()));
    storage
        .expect_list_files()
        .times(1)
        .returning(|_| Ok(ListFilesResponse::default()));
    storage
        .expect_exists()
        .times(1)
        .returning(|_| Ok(ExistsResponse::default()));
    storage.expect_get().times(1).returning(|_| Ok(GetResponse::default()));
    storage.expect_put().times(1).returning(|_| Ok(PutResponse::default()));
    storage
        .expect_remove_files()
        .times(1)
        .returning(|_| Ok(RemoveFilesResponse::default()));
    storage
        .expect_remove_file()
        .times(1)
        .returning(|_| Ok(RemoveFileResponse::default()));
    storage
        .expect_remove_folder()
        .times(1)
        .returning(|_| Ok(RemoveFolderResponse::default()));
    let storage = Arc::new(Box::new(storage) as Box<dyn StorageTrait>);
    assert_eq!(
        storage.list_folders(ListFoldersRequest::default()).await.unwrap(),
        ListFoldersResponse::default()
    );
    assert_eq!(
        storage.list_files(ListFilesRequest::default()).await.unwrap(),
        ListFilesResponse::default()
    );
    assert_eq!(
        storage.exists(ExistsRequest::default()).await.unwrap(),
        ExistsResponse::default()
    );
    assert_eq!(
        storage.get(GetRequest::default()).await.unwrap(),
        GetResponse::default()
    );
    assert_eq!(
        storage.put(PutRequest::default()).await.unwrap(),
        PutResponse::default()
    );
    assert_eq!(
        storage.remove_files(RemoveFilesRequest::default()).await.unwrap(),
        RemoveFilesResponse::default()
    );
    assert_eq!(
        storage.remove_file(RemoveFileRequest::default()).await.unwrap(),
        RemoveFileResponse::default()
    );
    assert_eq!(
        storage.remove_folder(RemoveFolderRequest::default()).await.unwrap(),
        RemoveFolderResponse::default()
    );
}
