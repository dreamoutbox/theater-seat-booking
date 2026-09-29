use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::{Builder, Region},
    primitives::ByteStream,
};
use tracing::{error, info, warn};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket: String,
}

impl S3Service {
    pub async fn new(config: &AppConfig) -> Self {
        let creds = Credentials::new(
            config.s3_access_key.clone(),
            config.s3_secret_key.clone(),
            None,
            None,
            "static",
        );

        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.s3_region.clone()))
            .credentials_provider(creds)
            .endpoint_url(&config.s3_endpoint)
            .load()
            .await;

        let s3_config = Builder::from(&sdk_config).force_path_style(true).build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.s3_bucket.clone(),
        }
    }

    pub async fn ensure_bucket_exists(&self) -> Result<(), String> {
        info!("Checking MinIO S3 bucket '{}'...", self.bucket);
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => {
                info!(
                    "MinIO S3 bucket '{}' already exists and is ready.",
                    self.bucket
                );
                Ok(())
            }
            Err(_) => {
                info!(
                    "Bucket '{}' not accessible or does not exist. Creating...",
                    self.bucket
                );
                match self
                    .client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await
                {
                    Ok(_) => {
                        info!("Successfully created S3 bucket '{}'.", self.bucket);
                        Ok(())
                    }
                    Err(create_err) => {
                        warn!(
                            "Could not create bucket '{}': {:?}",
                            self.bucket, create_err
                        );
                        Err(format!("Failed to create bucket: {create_err}"))
                    }
                }
            }
        }
    }

    pub async fn upload_file(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        info!(
            "Uploading file to S3: bucket='{}', key='{}' ({} bytes)",
            self.bucket,
            key,
            data.len()
        );
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                error!("S3 upload error for key '{}': {:?}", key, e);
                format!("Failed to upload to S3: {e}")
            })?;

        Ok(key.to_string())
    }

    pub async fn get_file(&self, key: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("S3 get_object error for key '{}': {:?}", key, e);
                format!("Failed to download from S3: {e}")
            })?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| format!("Failed to read S3 byte stream: {e}"))?
            .into_bytes();

        Ok(bytes.to_vec())
    }

    pub async fn file_exists(&self, key: &str) -> bool {
        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }
}
