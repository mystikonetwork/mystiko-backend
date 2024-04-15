use crate::{
    ExistsRequest, ExistsResponse, GetRequest, GetResponse, ListFilesRequest, ListFilesResponse, ListFoldersRequest,
    ListFoldersResponse, PutRequest, PutResponse, RemoveFileRequest, RemoveFileResponse, RemoveFilesRequest,
    RemoveFilesResponse, RemoveFolderRequest, RemoveFolderResponse, S3StorageConfig, Storage,
};
use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{
    DeleteObjectRequest, GetObjectRequest, HeadObjectError, HeadObjectRequest, ListObjectsV2Request, ObjectIdentifier,
    PutObjectRequest, S3Client, S3,
};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::io::AsyncReadExt;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct S3Storage<C: S3 + Send + Sync = S3Client> {
    pub client: C,
    #[builder(default = default_bucket())]
    pub s3_bucket: String,
    #[builder(default = default_base())]
    pub base: PathBuf,
}

#[async_trait]
impl<C> Storage for S3Storage<C>
where
    C: S3 + Send + Sync,
{
    async fn list_folders(&self, request: ListFoldersRequest) -> Result<ListFoldersResponse> {
        let mut paths = vec![];
        let mut continuation_token: Option<String> = None;
        loop {
            let s3_request = ListObjectsV2Request {
                bucket: self.s3_bucket.clone(),
                prefix: Some(self.base.join(&request.path).to_string_lossy().to_string()),
                continuation_token: continuation_token.clone(),
                delimiter: Some("/".to_string()),
                ..Default::default()
            };
            let result = self.client.list_objects_v2(s3_request).await?;
            if let Some(common_prefixes) = result.common_prefixes {
                for prefix in common_prefixes {
                    if let Some(prefix) = prefix.prefix {
                        paths.push(Path::new(&prefix).strip_prefix(&self.base)?.to_path_buf());
                    }
                }
            }
            if result.continuation_token.is_none() {
                break;
            } else {
                continuation_token = result.continuation_token;
            }
        }
        Ok(ListFoldersResponse::builder().folders(paths).build())
    }

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        let mut paths = vec![];
        let mut continuation_token: Option<String> = None;
        loop {
            let request = ListObjectsV2Request {
                bucket: self.s3_bucket.clone(),
                prefix: Some(self.base.join(&request.path).to_string_lossy().to_string()),
                continuation_token: continuation_token.clone(),
                delimiter: request.non_recursively.then(|| "/".to_string()),
                ..Default::default()
            };
            let result = self.client.list_objects_v2(request).await?;
            if let Some(contents) = result.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        paths.push(Path::new(&key).strip_prefix(&self.base)?.to_path_buf());
                    }
                }
            }
            if result.continuation_token.is_none() {
                break;
            } else {
                continuation_token = result.continuation_token;
            }
        }
        Ok(ListFilesResponse::builder().files(paths).build())
    }

    async fn exists(&self, request: ExistsRequest) -> Result<ExistsResponse> {
        let s3_request = HeadObjectRequest {
            bucket: self.s3_bucket.clone(),
            key: self.base.join(request.path).to_string_lossy().to_string(),
            ..Default::default()
        };
        let exists = match self.client.head_object(s3_request).await {
            Ok(_) => Ok(true),
            Err(RusotoError::Service(HeadObjectError::NoSuchKey(_))) => Ok(false),
            Err(RusotoError::Unknown(http_resp)) => {
                if http_resp.status.as_u16() == 404 {
                    Ok(false)
                } else {
                    Err(RusotoError::<HeadObjectError>::Unknown(http_resp).into())
                }
            }
            Err(e) => Err(e.into()),
        };
        exists.map(|exists| ExistsResponse::builder().exists(exists).build())
    }

    async fn get(&self, request: GetRequest) -> Result<GetResponse> {
        let s3_request = GetObjectRequest {
            bucket: self.s3_bucket.clone(),
            key: self.base.join(request.path).to_string_lossy().to_string(),
            ..Default::default()
        };
        let result = self.client.get_object(s3_request).await?;
        let data = if let Some(body) = result.body {
            let mut bytes = vec![];
            body.into_async_read().read_to_end(&mut bytes).await?;
            bytes
        } else {
            vec![]
        };
        Ok(GetResponse::builder().data(data).build())
    }

    async fn put(&self, request: PutRequest) -> Result<PutResponse> {
        let path = self.base.join(&request.path);
        let key = path.to_string_lossy().to_string();
        let exists = self.exists(request.path.clone().into()).await?;
        if !exists.exists || request.overwrite {
            let s3_request = PutObjectRequest {
                bucket: self.s3_bucket.clone(),
                key,
                body: Some(request.data.into()),
                content_type: request.content_type,
                cache_control: request.cache_control,
                acl: request.acl,
                ..Default::default()
            };
            self.client.put_object(s3_request).await?;
        }
        Ok(PutResponse::builder().build())
    }

    async fn remove_files(&self, request: RemoveFilesRequest) -> Result<RemoveFilesResponse> {
        let mut objects = vec![];
        for path in request.paths.into_iter() {
            objects.push(ObjectIdentifier {
                key: self.base.join(path).to_string_lossy().to_string(),
                ..Default::default()
            });
        }
        self.client
            .delete_objects(rusoto_s3::DeleteObjectsRequest {
                bucket: self.s3_bucket.clone(),
                delete: rusoto_s3::Delete {
                    objects,
                    quiet: Some(true),
                },
                ..Default::default()
            })
            .await?;
        Ok(RemoveFilesResponse::builder().build())
    }

    async fn remove_file(&self, request: RemoveFileRequest) -> Result<RemoveFileResponse> {
        let s3_request = DeleteObjectRequest {
            bucket: self.s3_bucket.clone(),
            key: self.base.join(request.path).to_string_lossy().to_string(),
            ..Default::default()
        };
        self.client.delete_object(s3_request).await?;
        Ok(RemoveFileResponse::builder().build())
    }

    async fn remove_folder(&self, request: RemoveFolderRequest) -> Result<RemoveFolderResponse> {
        let objects = self.list_files(request.path.into()).await?;
        self.remove_files(objects.files.into()).await?;
        Ok(RemoveFolderResponse::builder().build())
    }
}

impl S3Storage<S3Client> {
    pub fn from_config(config: &S3StorageConfig) -> Result<Self> {
        let client = S3Client::new(Region::from_str(&config.region)?);
        Ok(S3Storage::builder()
            .client(client)
            .s3_bucket(config.bucket.clone())
            .base(PathBuf::from(&config.base))
            .build())
    }
}

fn default_bucket() -> String {
    String::from("static.mystiko.network")
}

fn default_base() -> PathBuf {
    PathBuf::default()
}
