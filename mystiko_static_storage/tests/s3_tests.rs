use async_trait::async_trait;
use mockall::mock;
use mystiko_static_storage::{PutRequest, S3Storage, Storage};
use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::{ByteStream, RusotoError};
use rusoto_s3::*;
use std::path::PathBuf;

#[tokio::test]
async fn test_list_folders() {
    let mut client = MockS3Client::new();
    client
        .expect_list_objects_v2()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.prefix == Some("test-base/test-path".to_string())
                && request.delimiter == Some("/".to_string())
        })
        .times(1)
        .returning(|_| {
            Ok(ListObjectsV2Output {
                common_prefixes: Some(vec![
                    CommonPrefix {
                        prefix: Some("test-base/test-path/test-folder-1".to_string()),
                    },
                    CommonPrefix {
                        prefix: Some("test-base/test-path/test-folder-2".to_string()),
                    },
                ]),
                ..Default::default()
            })
        });
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    let result = storage
        .list_folders("test-path".into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(result, vec!["test-path/test-folder-1", "test-path/test-folder-2"]);
}

#[tokio::test]
async fn test_list_folders_continuation_token() {
    let mut client = MockS3Client::new();
    client
        .expect_list_objects_v2()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.prefix == Some("test-base/test-path".to_string())
                && request.delimiter == Some("/".to_string())
        })
        .times(2)
        .returning(|request| {
            if request.continuation_token.is_none() {
                Ok(ListObjectsV2Output {
                    common_prefixes: Some(vec![CommonPrefix {
                        prefix: Some("test-base/test-path/test-folder-1".to_string()),
                    }]),
                    continuation_token: Some("test-continuation-token".to_string()),
                    ..Default::default()
                })
            } else {
                Ok(ListObjectsV2Output {
                    common_prefixes: Some(vec![CommonPrefix {
                        prefix: Some("test-base/test-path/test-folder-2".to_string()),
                    }]),
                    ..Default::default()
                })
            }
        });
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    let result = storage
        .list_folders("test-path".into())
        .await
        .unwrap()
        .folders
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(result, vec!["test-path/test-folder-1", "test-path/test-folder-2"]);
}

#[tokio::test]
async fn test_list_files() {
    let mut client = MockS3Client::new();
    client
        .expect_list_objects_v2()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.prefix == Some("test-base/test-path".to_string())
                && request.delimiter.is_none()
        })
        .times(1)
        .returning(|_| {
            Ok(ListObjectsV2Output {
                contents: Some(vec![
                    Object {
                        key: Some("test-base/test-path/test-file-1".to_string()),
                        ..Default::default()
                    },
                    Object {
                        key: Some("test-base/test-path/test-file-2".to_string()),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            })
        });
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    let result = storage
        .list_files("test-path".into())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(result, vec!["test-path/test-file-1", "test-path/test-file-2"]);
}

#[tokio::test]
async fn test_list_files_continuation_token() {
    let mut client = MockS3Client::new();
    client
        .expect_list_objects_v2()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.prefix == Some("test-base/test-path".to_string())
                && request.delimiter.is_none()
        })
        .times(2)
        .returning(|request| {
            if request.continuation_token.is_none() {
                Ok(ListObjectsV2Output {
                    contents: Some(vec![Object {
                        key: Some("test-base/test-path/test-file-1".to_string()),
                        ..Default::default()
                    }]),
                    continuation_token: Some("test-continuation-token".to_string()),
                    ..Default::default()
                })
            } else {
                Ok(ListObjectsV2Output {
                    contents: Some(vec![Object {
                        key: Some("test-base/test-path/test-file-2".to_string()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                })
            }
        });
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    let result = storage
        .list_files("test-path".into())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(result, vec!["test-path/test-file-1", "test-path/test-file-2"]);
}

#[tokio::test]
async fn test_exists() {
    let mut client = MockS3Client::new();
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file1")
        .times(1)
        .returning(|_| Ok(HeadObjectOutput::default()));
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file2")
        .times(1)
        .returning(|_| {
            Err(RusotoError::Service(HeadObjectError::NoSuchKey(
                "test-error".to_string(),
            )))
        });
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file3")
        .times(1)
        .returning(|_| {
            Err(RusotoError::Unknown(BufferedHttpResponse {
                status: http::status::StatusCode::NOT_FOUND,
                body: bytes::Bytes::default(),
                headers: http::HeaderMap::default(),
            }))
        });
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file4")
        .times(1)
        .returning(|_| {
            Err(RusotoError::Unknown(BufferedHttpResponse {
                status: http::status::StatusCode::SERVICE_UNAVAILABLE,
                body: bytes::Bytes::default(),
                headers: http::HeaderMap::default(),
            }))
        });
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file5")
        .times(1)
        .returning(|_| Err(RusotoError::Validation("test-error".to_string())));
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    assert!(storage.exists("test-path/test-file1".into()).await.unwrap().exists);
    assert!(!storage.exists("test-path/test-file2".into()).await.unwrap().exists);
    assert!(!storage.exists("test-path/test-file3".into()).await.unwrap().exists);
    assert!(storage.exists("test-path/test-file4".into()).await.is_err());
    assert!(storage.exists("test-path/test-file5".into()).await.is_err());
}

#[tokio::test]
async fn test_get() {
    let mut client = MockS3Client::new();
    client
        .expect_get_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file1")
        .times(1)
        .returning(|_| {
            Ok(GetObjectOutput {
                body: Some(ByteStream::from("test-data1".as_bytes().to_vec())),
                ..Default::default()
            })
        });
    client
        .expect_get_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file2")
        .times(1)
        .returning(|_| {
            Ok(GetObjectOutput {
                body: None,
                ..Default::default()
            })
        });
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    assert_eq!(
        String::from_utf8_lossy(&storage.get("test-path/test-file1".into()).await.unwrap().data),
        "test-data1"
    );
    assert!(storage
        .get("test-path/test-file2".into())
        .await
        .unwrap()
        .data
        .is_empty());
}

#[tokio::test]
async fn test_put() {
    let mut client = MockS3Client::new();
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file1")
        .times(1)
        .returning(|_| {
            Err(RusotoError::Service(HeadObjectError::NoSuchKey(
                "test-error".to_string(),
            )))
        });
    client
        .expect_put_object()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.key == "test-base/test-path/test-file1"
                && request.content_type.clone().unwrap() == "application/json"
                && request.cache_control.clone().unwrap() == "no-cache"
                && request.acl.clone().unwrap() == "access"
        })
        .times(1)
        .returning(|_| Ok(PutObjectOutput::default()));
    client
        .expect_head_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file2")
        .times(2)
        .returning(|_| Ok(HeadObjectOutput::default()));
    client
        .expect_put_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file2")
        .times(1)
        .returning(|_| Ok(PutObjectOutput::default()));
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    storage
        .put(
            PutRequest::builder()
                .path("test-path/test-file1")
                .content_type("application/json")
                .cache_control("no-cache")
                .acl("access")
                .build(),
        )
        .await
        .unwrap();
    storage
        .put(PutRequest::builder().path("test-path/test-file2").build())
        .await
        .unwrap();
    storage
        .put(
            PutRequest::builder()
                .path("test-path/test-file2")
                .overwrite(true)
                .build(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_remove_files() {
    let mut client = MockS3Client::new();
    client
        .expect_delete_objects()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.delete.objects.len() == 2
                && request
                    .delete
                    .objects
                    .iter()
                    .any(|o| o.key == "test-base/test-path/test-file1")
                && request
                    .delete
                    .objects
                    .iter()
                    .any(|o| o.key == "test-base/test-path/test-file2")
        })
        .times(1)
        .returning(|_| Ok(DeleteObjectsOutput::default()));
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    let files: Vec<PathBuf> = vec![
        PathBuf::from("test-path/test-file1"),
        PathBuf::from("test-path/test-file2"),
    ];
    storage.remove_files(files.into()).await.unwrap();
}

#[tokio::test]
async fn test_remove_file() {
    let mut client = MockS3Client::new();
    client
        .expect_delete_object()
        .withf(|request| request.bucket == "test-bucket" && request.key == "test-base/test-path/test-file1")
        .times(1)
        .returning(|_| Ok(DeleteObjectOutput::default()));
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    storage.remove_file("test-path/test-file1".into()).await.unwrap();
}

#[tokio::test]
async fn test_remove_folder() {
    let mut client = MockS3Client::new();
    client
        .expect_list_objects_v2()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.prefix == Some("test-base/test-path".to_string())
                && request.delimiter.is_none()
        })
        .times(1)
        .returning(|_| {
            Ok(ListObjectsV2Output {
                contents: Some(vec![
                    Object {
                        key: Some("test-base/test-path/test-file1".to_string()),
                        ..Default::default()
                    },
                    Object {
                        key: Some("test-base/test-path/test-file2".to_string()),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            })
        });
    client
        .expect_delete_objects()
        .withf(|request| {
            request.bucket == "test-bucket"
                && request.delete.objects.len() == 2
                && request
                    .delete
                    .objects
                    .iter()
                    .any(|o| o.key == "test-base/test-path/test-file1")
                && request
                    .delete
                    .objects
                    .iter()
                    .any(|o| o.key == "test-base/test-path/test-file2")
        })
        .times(1)
        .returning(|_| Ok(DeleteObjectsOutput::default()));
    let storage = S3Storage::<MockS3Client>::builder()
        .base("test-base")
        .s3_bucket("test-bucket")
        .client(client)
        .build();
    storage.remove_folder("test-path".into()).await.unwrap();
}

#[tokio::test]
async fn test_with_default() {
    let mut client = MockS3Client::new();
    client
        .expect_delete_object()
        .withf(|request| request.bucket == "static.mystiko.network" && request.key == "test-path/test-file1")
        .times(1)
        .returning(|_| Ok(DeleteObjectOutput::default()));
    let storage = S3Storage::<MockS3Client>::builder().client(client).build();
    storage.remove_file("test-path/test-file1".into()).await.unwrap();
}

mock! {
    #[derive(Debug)]
    S3Client {}

    #[async_trait]
    impl S3 for S3Client {
        async fn abort_multipart_upload(
            &self,
            input: AbortMultipartUploadRequest,
        ) -> Result<AbortMultipartUploadOutput, RusotoError<AbortMultipartUploadError>>;
        async fn complete_multipart_upload(
            &self,
            input: CompleteMultipartUploadRequest,
        ) -> Result<CompleteMultipartUploadOutput, RusotoError<CompleteMultipartUploadError>>;
        async fn copy_object(&self, input: CopyObjectRequest) -> Result<CopyObjectOutput, RusotoError<CopyObjectError>>;
        async fn create_bucket(
            &self,
            input: CreateBucketRequest,
        ) -> Result<CreateBucketOutput, RusotoError<CreateBucketError>>;
        async fn create_multipart_upload(
            &self,
            input: CreateMultipartUploadRequest,
        ) -> Result<CreateMultipartUploadOutput, RusotoError<CreateMultipartUploadError>>;
        async fn delete_bucket(&self, input: DeleteBucketRequest) -> Result<(), RusotoError<DeleteBucketError>>;
        async fn delete_bucket_analytics_configuration(
            &self,
            input: DeleteBucketAnalyticsConfigurationRequest,
        ) -> Result<(), RusotoError<DeleteBucketAnalyticsConfigurationError>>;
        async fn delete_bucket_cors(
            &self,
            input: DeleteBucketCorsRequest,
        ) -> Result<(), RusotoError<DeleteBucketCorsError>>;
        async fn delete_bucket_encryption(
            &self,
            input: DeleteBucketEncryptionRequest,
        ) -> Result<(), RusotoError<DeleteBucketEncryptionError>>;
        async fn delete_bucket_intelligent_tiering_configuration(
            &self,
            input: DeleteBucketIntelligentTieringConfigurationRequest,
        ) -> Result<(), RusotoError<DeleteBucketIntelligentTieringConfigurationError>>;
        async fn delete_bucket_inventory_configuration(
            &self,
            input: DeleteBucketInventoryConfigurationRequest,
        ) -> Result<(), RusotoError<DeleteBucketInventoryConfigurationError>>;
        async fn delete_bucket_lifecycle(
            &self,
            input: DeleteBucketLifecycleRequest,
        ) -> Result<(), RusotoError<DeleteBucketLifecycleError>>;
        async fn delete_bucket_metrics_configuration(
            &self,
            input: DeleteBucketMetricsConfigurationRequest,
        ) -> Result<(), RusotoError<DeleteBucketMetricsConfigurationError>>;
        async fn delete_bucket_ownership_controls(
            &self,
            input: DeleteBucketOwnershipControlsRequest,
        ) -> Result<(), RusotoError<DeleteBucketOwnershipControlsError>>;
        async fn delete_bucket_policy(
            &self,
            input: DeleteBucketPolicyRequest,
        ) -> Result<(), RusotoError<DeleteBucketPolicyError>>;
        async fn delete_bucket_replication(
            &self,
            input: DeleteBucketReplicationRequest,
        ) -> Result<(), RusotoError<DeleteBucketReplicationError>>;
        async fn delete_bucket_tagging(
            &self,
            input: DeleteBucketTaggingRequest,
        ) -> Result<(), RusotoError<DeleteBucketTaggingError>>;
        async fn delete_bucket_website(
            &self,
            input: DeleteBucketWebsiteRequest,
        ) -> Result<(), RusotoError<DeleteBucketWebsiteError>>;
        async fn delete_object(
            &self,
            input: DeleteObjectRequest,
        ) -> Result<DeleteObjectOutput, RusotoError<DeleteObjectError>>;
        async fn delete_object_tagging(
            &self,
            input: DeleteObjectTaggingRequest,
        ) -> Result<DeleteObjectTaggingOutput, RusotoError<DeleteObjectTaggingError>>;
        async fn delete_objects(
            &self,
            input: DeleteObjectsRequest,
        ) -> Result<DeleteObjectsOutput, RusotoError<DeleteObjectsError>>;
        async fn delete_public_access_block(
            &self,
            input: DeletePublicAccessBlockRequest,
        ) -> Result<(), RusotoError<DeletePublicAccessBlockError>>;
        async fn get_bucket_accelerate_configuration(
            &self,
            input: GetBucketAccelerateConfigurationRequest,
        ) -> Result<GetBucketAccelerateConfigurationOutput, RusotoError<GetBucketAccelerateConfigurationError>>;
        async fn get_bucket_acl(
            &self,
            input: GetBucketAclRequest,
        ) -> Result<GetBucketAclOutput, RusotoError<GetBucketAclError>>;
        async fn get_bucket_analytics_configuration(
            &self,
            input: GetBucketAnalyticsConfigurationRequest,
        ) -> Result<GetBucketAnalyticsConfigurationOutput, RusotoError<GetBucketAnalyticsConfigurationError>>;
        async fn get_bucket_cors(
            &self,
            input: GetBucketCorsRequest,
        ) -> Result<GetBucketCorsOutput, RusotoError<GetBucketCorsError>>;
        async fn get_bucket_encryption(
            &self,
            input: GetBucketEncryptionRequest,
        ) -> Result<GetBucketEncryptionOutput, RusotoError<GetBucketEncryptionError>>;
        async fn get_bucket_intelligent_tiering_configuration(
            &self,
            input: GetBucketIntelligentTieringConfigurationRequest,
        ) -> Result<
            GetBucketIntelligentTieringConfigurationOutput,
            RusotoError<GetBucketIntelligentTieringConfigurationError>,
        >;
        async fn get_bucket_inventory_configuration(
            &self,
            input: GetBucketInventoryConfigurationRequest,
        ) -> Result<GetBucketInventoryConfigurationOutput, RusotoError<GetBucketInventoryConfigurationError>>;
        async fn get_bucket_lifecycle(
            &self,
            input: GetBucketLifecycleRequest,
        ) -> Result<GetBucketLifecycleOutput, RusotoError<GetBucketLifecycleError>>;
        async fn get_bucket_lifecycle_configuration(
            &self,
            input: GetBucketLifecycleConfigurationRequest,
        ) -> Result<GetBucketLifecycleConfigurationOutput, RusotoError<GetBucketLifecycleConfigurationError>>;
        async fn get_bucket_location(
            &self,
            input: GetBucketLocationRequest,
        ) -> Result<GetBucketLocationOutput, RusotoError<GetBucketLocationError>>;
        async fn get_bucket_logging(
            &self,
            input: GetBucketLoggingRequest,
        ) -> Result<GetBucketLoggingOutput, RusotoError<GetBucketLoggingError>>;
        async fn get_bucket_metrics_configuration(
            &self,
            input: GetBucketMetricsConfigurationRequest,
        ) -> Result<GetBucketMetricsConfigurationOutput, RusotoError<GetBucketMetricsConfigurationError>>;
        async fn get_bucket_notification(
            &self,
            input: GetBucketNotificationConfigurationRequest,
        ) -> Result<NotificationConfigurationDeprecated, RusotoError<GetBucketNotificationError>>;
        async fn get_bucket_notification_configuration(
            &self,
            input: GetBucketNotificationConfigurationRequest,
        ) -> Result<NotificationConfiguration, RusotoError<GetBucketNotificationConfigurationError>>;
        async fn get_bucket_ownership_controls(
            &self,
            input: GetBucketOwnershipControlsRequest,
        ) -> Result<GetBucketOwnershipControlsOutput, RusotoError<GetBucketOwnershipControlsError>>;
        async fn get_bucket_policy(
            &self,
            input: GetBucketPolicyRequest,
        ) -> Result<GetBucketPolicyOutput, RusotoError<GetBucketPolicyError>>;
        async fn get_bucket_policy_status(
            &self,
            input: GetBucketPolicyStatusRequest,
        ) -> Result<GetBucketPolicyStatusOutput, RusotoError<GetBucketPolicyStatusError>>;
        async fn get_bucket_replication(
            &self,
            input: GetBucketReplicationRequest,
        ) -> Result<GetBucketReplicationOutput, RusotoError<GetBucketReplicationError>>;
        async fn get_bucket_request_payment(
            &self,
            input: GetBucketRequestPaymentRequest,
        ) -> Result<GetBucketRequestPaymentOutput, RusotoError<GetBucketRequestPaymentError>>;
        async fn get_bucket_tagging(
            &self,
            input: GetBucketTaggingRequest,
        ) -> Result<GetBucketTaggingOutput, RusotoError<GetBucketTaggingError>>;
        async fn get_bucket_versioning(
            &self,
            input: GetBucketVersioningRequest,
        ) -> Result<GetBucketVersioningOutput, RusotoError<GetBucketVersioningError>>;
        async fn get_bucket_website(
            &self,
            input: GetBucketWebsiteRequest,
        ) -> Result<GetBucketWebsiteOutput, RusotoError<GetBucketWebsiteError>>;
        async fn get_object(&self, input: GetObjectRequest) -> Result<GetObjectOutput, RusotoError<GetObjectError>>;
        async fn get_object_acl(
            &self,
            input: GetObjectAclRequest,
        ) -> Result<GetObjectAclOutput, RusotoError<GetObjectAclError>>;
        async fn get_object_legal_hold(
            &self,
            input: GetObjectLegalHoldRequest,
        ) -> Result<GetObjectLegalHoldOutput, RusotoError<GetObjectLegalHoldError>>;
        async fn get_object_lock_configuration(
            &self,
            input: GetObjectLockConfigurationRequest,
        ) -> Result<GetObjectLockConfigurationOutput, RusotoError<GetObjectLockConfigurationError>>;
        async fn get_object_retention(
            &self,
            input: GetObjectRetentionRequest,
        ) -> Result<GetObjectRetentionOutput, RusotoError<GetObjectRetentionError>>;
        async fn get_object_tagging(
            &self,
            input: GetObjectTaggingRequest,
        ) -> Result<GetObjectTaggingOutput, RusotoError<GetObjectTaggingError>>;
        async fn get_object_torrent(
            &self,
            input: GetObjectTorrentRequest,
        ) -> Result<GetObjectTorrentOutput, RusotoError<GetObjectTorrentError>>;
        async fn get_public_access_block(
            &self,
            input: GetPublicAccessBlockRequest,
        ) -> Result<GetPublicAccessBlockOutput, RusotoError<GetPublicAccessBlockError>>;
        async fn head_bucket(&self, input: HeadBucketRequest) -> Result<(), RusotoError<HeadBucketError>>;
        async fn head_object(&self, input: HeadObjectRequest) -> Result<HeadObjectOutput, RusotoError<HeadObjectError>>;
        async fn list_bucket_analytics_configurations(
            &self,
            input: ListBucketAnalyticsConfigurationsRequest,
        ) -> Result<ListBucketAnalyticsConfigurationsOutput, RusotoError<ListBucketAnalyticsConfigurationsError>>;
        async fn list_bucket_intelligent_tiering_configurations(
            &self,
            input: ListBucketIntelligentTieringConfigurationsRequest,
        ) -> Result<
            ListBucketIntelligentTieringConfigurationsOutput,
            RusotoError<ListBucketIntelligentTieringConfigurationsError>,
        >;
        async fn list_bucket_inventory_configurations(
            &self,
            input: ListBucketInventoryConfigurationsRequest,
        ) -> Result<ListBucketInventoryConfigurationsOutput, RusotoError<ListBucketInventoryConfigurationsError>>;
        async fn list_bucket_metrics_configurations(
            &self,
            input: ListBucketMetricsConfigurationsRequest,
        ) -> Result<ListBucketMetricsConfigurationsOutput, RusotoError<ListBucketMetricsConfigurationsError>>;
        async fn list_buckets(&self) -> Result<ListBucketsOutput, RusotoError<ListBucketsError>>;
        async fn list_multipart_uploads(
            &self,
            input: ListMultipartUploadsRequest,
        ) -> Result<ListMultipartUploadsOutput, RusotoError<ListMultipartUploadsError>>;
        async fn list_object_versions(
            &self,
            input: ListObjectVersionsRequest,
        ) -> Result<ListObjectVersionsOutput, RusotoError<ListObjectVersionsError>>;
        async fn list_objects(&self, input: ListObjectsRequest)
                              -> Result<ListObjectsOutput, RusotoError<ListObjectsError>>;
        async fn list_objects_v2(
            &self,
            input: ListObjectsV2Request,
        ) -> Result<ListObjectsV2Output, RusotoError<ListObjectsV2Error>>;
        async fn list_parts(&self, input: ListPartsRequest) -> Result<ListPartsOutput, RusotoError<ListPartsError>>;
        async fn put_bucket_accelerate_configuration(
            &self,
            input: PutBucketAccelerateConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketAccelerateConfigurationError>>;
        async fn put_bucket_acl(&self, input: PutBucketAclRequest) -> Result<(), RusotoError<PutBucketAclError>>;
        async fn put_bucket_analytics_configuration(
            &self,
            input: PutBucketAnalyticsConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketAnalyticsConfigurationError>>;
        async fn put_bucket_cors(&self, input: PutBucketCorsRequest) -> Result<(), RusotoError<PutBucketCorsError>>;
        async fn put_bucket_encryption(
            &self,
            input: PutBucketEncryptionRequest,
        ) -> Result<(), RusotoError<PutBucketEncryptionError>>;
        async fn put_bucket_intelligent_tiering_configuration(
            &self,
            input: PutBucketIntelligentTieringConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketIntelligentTieringConfigurationError>>;
        async fn put_bucket_inventory_configuration(
            &self,
            input: PutBucketInventoryConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketInventoryConfigurationError>>;
        async fn put_bucket_lifecycle(
            &self,
            input: PutBucketLifecycleRequest,
        ) -> Result<(), RusotoError<PutBucketLifecycleError>>;
        async fn put_bucket_lifecycle_configuration(
            &self,
            input: PutBucketLifecycleConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketLifecycleConfigurationError>>;
        async fn put_bucket_logging(
            &self,
            input: PutBucketLoggingRequest,
        ) -> Result<(), RusotoError<PutBucketLoggingError>>;
        async fn put_bucket_metrics_configuration(
            &self,
            input: PutBucketMetricsConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketMetricsConfigurationError>>;
        async fn put_bucket_notification(
            &self,
            input: PutBucketNotificationRequest,
        ) -> Result<(), RusotoError<PutBucketNotificationError>>;
        async fn put_bucket_notification_configuration(
            &self,
            input: PutBucketNotificationConfigurationRequest,
        ) -> Result<(), RusotoError<PutBucketNotificationConfigurationError>>;
        async fn put_bucket_ownership_controls(
            &self,
            input: PutBucketOwnershipControlsRequest,
        ) -> Result<(), RusotoError<PutBucketOwnershipControlsError>>;
        async fn put_bucket_policy(&self, input: PutBucketPolicyRequest) -> Result<(), RusotoError<PutBucketPolicyError>>;
        async fn put_bucket_replication(
            &self,
            input: PutBucketReplicationRequest,
        ) -> Result<(), RusotoError<PutBucketReplicationError>>;
        async fn put_bucket_request_payment(
            &self,
            input: PutBucketRequestPaymentRequest,
        ) -> Result<(), RusotoError<PutBucketRequestPaymentError>>;
        async fn put_bucket_tagging(
            &self,
            input: PutBucketTaggingRequest,
        ) -> Result<(), RusotoError<PutBucketTaggingError>>;
        async fn put_bucket_versioning(
            &self,
            input: PutBucketVersioningRequest,
        ) -> Result<(), RusotoError<PutBucketVersioningError>>;
        async fn put_bucket_website(
            &self,
            input: PutBucketWebsiteRequest,
        ) -> Result<(), RusotoError<PutBucketWebsiteError>>;
        async fn put_object(&self, input: PutObjectRequest) -> Result<PutObjectOutput, RusotoError<PutObjectError>>;
        async fn put_object_acl(
            &self,
            input: PutObjectAclRequest,
        ) -> Result<PutObjectAclOutput, RusotoError<PutObjectAclError>>;
        async fn put_object_legal_hold(
            &self,
            input: PutObjectLegalHoldRequest,
        ) -> Result<PutObjectLegalHoldOutput, RusotoError<PutObjectLegalHoldError>>;
        async fn put_object_lock_configuration(
            &self,
            input: PutObjectLockConfigurationRequest,
        ) -> Result<PutObjectLockConfigurationOutput, RusotoError<PutObjectLockConfigurationError>>;
        async fn put_object_retention(
            &self,
            input: PutObjectRetentionRequest,
        ) -> Result<PutObjectRetentionOutput, RusotoError<PutObjectRetentionError>>;
        async fn put_object_tagging(
            &self,
            input: PutObjectTaggingRequest,
        ) -> Result<PutObjectTaggingOutput, RusotoError<PutObjectTaggingError>>;
        async fn put_public_access_block(
            &self,
            input: PutPublicAccessBlockRequest,
        ) -> Result<(), RusotoError<PutPublicAccessBlockError>>;
        async fn restore_object(
            &self,
            input: RestoreObjectRequest,
        ) -> Result<RestoreObjectOutput, RusotoError<RestoreObjectError>>;
        async fn select_object_content(
            &self,
            input: SelectObjectContentRequest,
        ) -> Result<SelectObjectContentOutput, RusotoError<SelectObjectContentError>>;
        async fn upload_part(&self, input: UploadPartRequest) -> Result<UploadPartOutput, RusotoError<UploadPartError>>;
        async fn upload_part_copy(
            &self,
            input: UploadPartCopyRequest,
        ) -> Result<UploadPartCopyOutput, RusotoError<UploadPartCopyError>>;
        async fn write_get_object_response(
            &self,
            input: WriteGetObjectResponseRequest,
        ) -> Result<(), RusotoError<WriteGetObjectResponseError>>;
    }

}
