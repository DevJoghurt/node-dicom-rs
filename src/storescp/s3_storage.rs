use minio::s3::client::{Client, ClientBuilder};
use minio::s3::builders::ObjectContent;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use crate::storescp::S3Config;
use minio::s3::types::S3Api;
use tracing::{info, error};


pub fn build_s3_client(config: &S3Config) -> Client {
    let endpoint_str = config.endpoint.clone().unwrap_or_else(|| "http://localhost:7070".to_string());
    let endpoint = endpoint_str.parse::<BaseUrl>().expect("Invalid S3 endpoint URL");
    let static_provider = StaticProvider::new(
        &config.access_key,
        &config.secret_key,
        None,
    );
    let client = ClientBuilder::new(endpoint)
        .provider(Some(Box::new(static_provider)))
        .build().expect("Failed to build S3 client");
    client
}

pub async fn check_s3_connectivity(client: &Client, bucket: &str) {
    match client.bucket_exists(bucket).send().await {
        Ok(_) => {
            info!("S3 connectivity check succeeded for bucket: {}", bucket);
        },
        Err(e) => {
            error!("S3 connectivity check failed for bucket: {}: {}", bucket, e);
        }
    }
}

pub async fn s3_put_object(
    client: &Client,
    bucket: &str,
    path: &str,
    data: &Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = ObjectContent::from(data.clone());
    client.put_object_content(bucket, path, content).send().await
        .map_err(|e| format!("MinIO put_object error: {}", e))?;
    Ok(())
}
