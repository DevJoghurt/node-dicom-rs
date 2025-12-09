use s3::{Bucket};
use s3::creds::Credentials;
use s3::Region;
use tracing::{info, error};

use super::S3Config;

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

pub async fn s3_get_object(
    bucket: &Bucket,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let response = bucket.get_object(path).await?;
    let code = response.status_code();
    if code == 200 {
        let bytes = response.bytes().to_vec();
        info!("Downloaded S3 object '{}': {} bytes, first 16 bytes: {:?}", 
              path, bytes.len(), &bytes.get(..16.min(bytes.len())));
        Ok(bytes)
    } else {
        Err(format!("S3 get_object error: HTTP {}", code).into())
    }
}

pub async fn s3_list_objects(
    bucket: &Bucket,
    prefix: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let results = bucket.list(prefix.to_string(), None).await?;
    
    let mut objects = Vec::new();
    for list in results {
        for obj in list.contents {
            objects.push(obj.key);
        }
    }
    
    Ok(objects)
}
