use crate::{
    ExistsRequest, ExistsResponse, FileStorageConfig, GetRequest, GetResponse, ListFilesRequest, ListFilesResponse,
    ListFoldersRequest, ListFoldersResponse, PutRequest, PutResponse, RemoveFileRequest, RemoveFileResponse,
    RemoveFilesRequest, RemoveFilesResponse, RemoveFolderRequest, RemoveFolderResponse, Storage,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use typed_builder::TypedBuilder;

#[derive(Debug, Default, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct FileStorageOptions {
    #[builder(default)]
    base: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FileStorage {
    pub base: PathBuf,
}

#[async_trait]
impl Storage for FileStorage {
    async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse> {
        let mut paths = vec![];
        let mut dir = self.base.clone();
        dir.push(&request.path);
        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                paths.push(path.strip_prefix(&self.base)?.to_path_buf());
            }
        }
        Ok(ListFoldersResponse::builder().folders(paths).build())
    }

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        let mut paths = vec![];
        let mut dir = self.base.clone();
        dir.push(&request.path);
        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() && !request.non_recursively {
                let mut sub_files = self.list_files(path.into()).await?;
                paths.append(&mut sub_files.files);
            } else if !path.is_dir() {
                paths.push(path.strip_prefix(&self.base)?.to_path_buf());
            }
        }
        Ok(ListFilesResponse::builder().files(paths).build())
    }

    async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse> {
        let mut full_path = self.base.clone();
        full_path.push(&request.path);
        Ok(ExistsResponse::builder()
            .exists(fs::try_exists(full_path).await?)
            .build())
    }

    async fn get(&self, request: GetRequest) -> Result<GetResponse> {
        let mut full_path = self.base.clone();
        full_path.push(&request.path);
        Ok(GetResponse::builder().data(fs::read(full_path).await?).build())
    }

    async fn put(&self, request: PutRequest) -> Result<PutResponse> {
        let mut full_path = self.base.clone();
        full_path.push(&request.path);
        if let Some(parent) = full_path.parent() {
            if !fs::try_exists(parent).await? {
                fs::create_dir_all(parent).await?;
            }
        }
        let exists = self.exists(full_path.clone().into()).await?;
        if !request.overwrite && exists.exists {
            return Ok(PutResponse::builder().build());
        }
        fs::write(full_path, request.data).await?;
        Ok(PutResponse::builder().build())
    }

    async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse> {
        for path in request.paths.into_iter() {
            let mut full_path = self.base.clone();
            full_path.push(path.clone());
            fs::remove_file(full_path).await?;
        }
        Ok(RemoveFilesResponse::builder().build())
    }

    async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse> {
        self.remove_files(vec![request.path].into()).await?;
        Ok(RemoveFileResponse::builder().build())
    }

    async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse> {
        let mut full_path = self.base.clone();
        full_path.push(&request.path);
        if request.non_recursively {
            fs::remove_dir(full_path).await?;
        } else {
            fs::remove_dir_all(full_path).await?;
        }
        Ok(RemoveFolderResponse::builder().build())
    }
}

impl<P> From<P> for FileStorageOptions
where
    P: AsRef<Path>,
{
    fn from(path: P) -> Self {
        Self::builder().base(PathBuf::from(path.as_ref())).build()
    }
}

impl FileStorage {
    pub async fn new<O>(options: O) -> Result<Self>
    where
        O: Into<FileStorageOptions>,
    {
        let options: FileStorageOptions = options.into();
        if !fs::try_exists(options.base.as_path()).await? {
            fs::create_dir_all(options.base.as_path()).await?;
        }
        Ok(Self { base: options.base })
    }
}

impl FileStorage {
    pub async fn from_config(config: &FileStorageConfig) -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or(anyhow::anyhow!("cannot detect the data directory of current OS"))?
            .join("mystiko")
            .join("datapacker")
            .join("storage");
        let path = config.path.as_ref().map(PathBuf::from).unwrap_or(data_dir);
        Self::new(path).await
    }
}
