use s3::{Bucket, Region};
use s3::creds::Credentials;
use tracing::{info, error};

/// S3 storage configuration
#[derive(Debug, Clone)]
#[napi(object)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// AWS access key ID
    pub access_key: String,
    /// AWS secret access key
    pub secret_key: String,
    /// S3 endpoint (e.g., "http://localhost:9000" for MinIO)
    pub endpoint: Option<String>,
}

/// Build an S3 bucket instance from configuration
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

/// Check S3 connectivity
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

/// Get object from S3
pub async fn s3_get_object(
    bucket: &Bucket,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let response = match bucket.get_object(path).await {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };
    let code = response.status_code();
    if code == 200 {
        let bytes = response.bytes().to_vec();
        info!("Downloaded S3 object '{}': {} bytes", path, bytes.len());
        Ok(bytes)
    } else {
        Err(format!("S3 get_object error: HTTP {}", code).into())
    }
}

/// Put object to S3
pub async fn s3_put_object(
    bucket: &Bucket,
    path: &str,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = match bucket.put_object(path, data).await {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };
    let code = response.status_code();
    if code == 200 || code == 201 {
        Ok(())
    } else {
        Err(format!("S3 put_object error: HTTP {}", code).into())
    }
}

/// List objects in S3 with given prefix
pub async fn s3_list_objects(
    bucket: &Bucket,
    prefix: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let results = match bucket.list(prefix.to_string(), None).await {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };
    
    let mut objects = Vec::new();
    for list in results {
        for obj in list.contents {
            objects.push(obj.key);
        }
    }
    
    Ok(objects)
}
