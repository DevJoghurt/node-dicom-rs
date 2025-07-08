use s3::{Bucket};
use s3::creds::Credentials;
use s3::Region;
use crate::storescp::S3Config;
use tracing::{info, error};

pub fn build_s3_bucket(config: &S3Config) -> Bucket {
    let endpoint = config.endpoint.clone().unwrap_or_else(|| "http://localhost:7070".to_string());
    let region = Region::Custom {
        region: "us-east-1".to_owned(),
        endpoint,
    };
    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    ).expect("Invalid S3 credentials");

    *Bucket::new(&config.bucket, region, credentials)
            .expect("Failed to create S3 bucket")
            .with_path_style()
}

pub async fn check_s3_connectivity(bucket: &Bucket) {
    match bucket.exists().await {
        Ok(_) => {
            info!("S3 connectivity check succeeded for bucket: {}", bucket.name());
        },
        Err(e) => {
            error!("S3 connectivity check failed for bucket: {}: {}", bucket.name(), e);
        }
    }
}

pub async fn s3_put_object(
    bucket: &Bucket,
    path: &str,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let response = bucket.put_object(path, data).await?;
    let code = response.status_code();
    if code == 200 || code == 201 {
        Ok(())
    } else {
        Err(format!("S3 put_object error: HTTP {}", code).into())
    }
}