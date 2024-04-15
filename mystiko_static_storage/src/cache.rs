use crate::{
    ExistsRequest, ExistsResponse, FileStorage, GetRequest, GetResponse, ListFilesRequest, ListFilesResponse,
    ListFoldersRequest, ListFoldersResponse, PutRequest, PutResponse, RemoveFileRequest, RemoveFileResponse,
    RemoveFilesRequest, RemoveFilesResponse, RemoveFolderRequest, RemoveFolderResponse, Storage, StorageCacheConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct CachedStorageOptions<P: AsRef<Path> + Send + Clone, S: Storage = Box<dyn Storage>> {
    pub cache_folder: P,
    pub raw: Arc<S>,
}

#[derive(TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct CachedStorage<S: Storage = Box<dyn Storage>> {
    cache: FileStorage,
    raw: Arc<S>,
}

impl<S> CachedStorage<S>
where
    S: Storage,
{
    pub async fn new<P, O>(options: O) -> Result<Self>
    where
        P: AsRef<Path> + Send + Clone,
        O: Into<CachedStorageOptions<P, S>>,
    {
        let options = options.into();
        let cache = FileStorage::new(options.cache_folder).await?;
        Ok(Self {
            cache,
            raw: options.raw,
        })
    }
}

#[async_trait]
impl<S> Storage for CachedStorage<S>
where
    S: Storage,
{
    async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse> {
        self.raw.list_folders(request).await
    }

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        self.raw.list_files(request).await
    }

    async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse> {
        self.raw.exists(request).await
    }

    async fn get(&self, request: GetRequest) -> Result<GetResponse> {
        if request.no_cache {
            self.raw.get(request).await
        } else {
            let exists_request: ExistsRequest = request.path.clone().into();
            let exists_response = self.cache.exists(exists_request).await?;
            if exists_response.exists {
                self.cache.get(request).await
            } else {
                let path = request.path.clone();
                let raw_response = self.raw.get(request).await?;
                let put_request: PutRequest = PutRequest::builder()
                    .path(path)
                    .data(raw_response.data.clone())
                    .overwrite(true)
                    .build();
                self.cache.put(put_request).await?;
                Ok(raw_response)
            }
        }
    }

    async fn put(&self, request: PutRequest) -> Result<PutResponse> {
        let mut cache_put = request.clone();
        cache_put.overwrite = true;
        self.cache.put(cache_put).await?;
        self.raw.put(request).await
    }

    async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse> {
        let paths = request.paths.clone();
        let mut existing_cached_files = vec![];
        for path in paths.into_iter() {
            let exists_request: ExistsRequest = path.clone().into();
            let exists_response = self.cache.exists(exists_request).await?;
            if exists_response.exists {
                existing_cached_files.push(path);
            }
        }
        if !existing_cached_files.is_empty() {
            self.cache.remove_files(existing_cached_files.into()).await?;
        }
        self.raw.remove_files(request).await
    }

    async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse> {
        let path = request.path.clone();
        let exists_request: ExistsRequest = path.clone().into();
        let exists_response = self.cache.exists(exists_request).await?;
        if exists_response.exists {
            self.cache.remove_file(path.into()).await?;
        }
        self.raw.remove_file(request).await
    }

    async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse> {
        let path = request.path.clone();
        let exists_request: ExistsRequest = path.clone().into();
        let exists_response = self.cache.exists(exists_request).await?;
        if exists_response.exists {
            self.cache.remove_folder(path.into()).await?;
        }
        self.raw.remove_folder(request).await
    }
}

impl<S> CachedStorage<S>
where
    S: Storage,
{
    pub async fn from_config(config: &StorageCacheConfig, raw: S) -> Result<Self> {
        let cache_dir = dirs::data_dir()
            .ok_or(anyhow::anyhow!("cannot detect the data directory of current OS"))?
            .join("mystiko")
            .join("datapacker")
            .join("cache");
        let cache_dir = config.path.as_ref().map(PathBuf::from).unwrap_or(cache_dir);
        let options = CachedStorageOptions::<PathBuf, S>::builder()
            .raw(raw)
            .cache_folder(cache_dir)
            .build();
        Self::new(options).await
    }
}
