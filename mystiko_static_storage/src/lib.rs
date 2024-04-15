mod cache;
mod config;
mod file;
mod s3;

pub use cache::*;
pub use config::*;
pub use file::*;
pub use s3::*;

use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ListFoldersRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ListFoldersResponse {
    pub folders: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ListFilesRequest {
    pub path: PathBuf,
    #[builder(default = false)]
    pub non_recursively: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ListFilesResponse {
    #[builder(default)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ExistsRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ExistsResponse {
    #[builder(default = false)]
    pub exists: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct GetRequest {
    pub path: PathBuf,
    #[builder(default = false)]
    pub no_cache: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct GetResponse {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct PutRequest {
    pub path: PathBuf,
    #[builder(default)]
    pub data: Vec<u8>,
    #[builder(default = false)]
    pub overwrite: bool,
    #[builder(default, setter(strip_option))]
    pub content_type: Option<String>,
    #[builder(default, setter(strip_option))]
    pub cache_control: Option<String>,
    #[builder(default, setter(strip_option))]
    pub acl: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct PutResponse {}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFilesRequest {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFilesResponse {}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFileRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFileResponse {}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFolderRequest {
    pub path: PathBuf,
    #[builder(default = false)]
    pub non_recursively: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RemoveFolderResponse {}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse>;

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse>;

    async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse>;

    async fn get(&self, request: GetRequest) -> Result<GetResponse>;

    async fn put(&self, request: PutRequest) -> Result<PutResponse>;

    async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse>;

    async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse>;

    async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse>;
}

#[async_trait]
impl Storage for Box<dyn Storage> {
    async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse> {
        self.as_ref().list_folders(request).await
    }

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        self.as_ref().list_files(request).await
    }

    async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse> {
        self.as_ref().exists(request).await
    }

    async fn get(&self, request: GetRequest) -> Result<GetResponse> {
        self.as_ref().get(request).await
    }

    async fn put(&self, request: PutRequest) -> Result<PutResponse> {
        self.as_ref().put(request).await
    }

    async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse> {
        self.as_ref().remove_files(request).await
    }

    async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse> {
        self.as_ref().remove_file(request).await
    }

    async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse> {
        self.as_ref().remove_folder(request).await
    }
}

impl<P> From<P> for ListFoldersRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<P> From<P> for ListFilesRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<P> From<P> for ExistsRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<P> From<P> for GetRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<P> From<P> for PutRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<V> From<V> for RemoveFilesRequest
where
    V: Into<Vec<PathBuf>>,
{
    fn from(paths: V) -> Self {
        Self::builder().paths(paths).build()
    }
}

impl<P> From<P> for RemoveFileRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}

impl<P> From<P> for RemoveFolderRequest
where
    P: AsRef<Path> + Send + Clone,
{
    fn from(path: P) -> Self {
        Self::builder().path(path.as_ref().to_path_buf()).build()
    }
}
