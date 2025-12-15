use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use warp::{Filter, Reply, reply::Response};

/// Storage backend configuration for WADO-RS
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WadoStorageConfig {
    /// Storage type: "filesystem" or "s3"
    pub storage_type: String,
    
    /// Base path for filesystem storage
    pub base_path: Option<String>,
    
    /// S3 bucket name
    pub s3_bucket: Option<String>,
    
    /// S3 region
    pub s3_region: Option<String>,
    
    /// S3 endpoint (for MinIO or custom endpoints)
    pub s3_endpoint: Option<String>,
    
    /// S3 access key
    pub s3_access_key: Option<String>,
    
    /// S3 secret key
    pub s3_secret_key: Option<String>,
}

/// WADO-RS Server
#[napi]
pub struct WadoServer {
    port: u16,
    config: WadoStorageConfig,
    runtime: Option<Runtime>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[napi]
impl WadoServer {
    #[napi(constructor)]
    pub fn new(port: u16, config: WadoStorageConfig) -> Result<Self> {
        Ok(Self {
            port,
            config,
            runtime: None,
            shutdown_tx: None,
        })
    }

    /// Start the WADO server
    #[napi]
    pub fn start(&mut self) -> Result<()> {
        let runtime = Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        let config = self.config.clone();
        let port = self.port;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        runtime.spawn(async move {
            // Route: /studies/{study_uid}
            let retrieve_study = warp::path!("studies" / String)
                .and(warp::get())
                .map(move |study_uid: String| {
                    warp::reply::with_status(
                        format!("Retrieve study: {}", study_uid),
                        warp::http::StatusCode::NOT_IMPLEMENTED,
                    )
                });

            // Route: /studies/{study_uid}/series/{series_uid}
            let retrieve_series = warp::path!("studies" / String / "series" / String)
                .and(warp::get())
                .map(move |study_uid: String, series_uid: String| {
                    warp::reply::with_status(
                        format!("Retrieve series: {}/{}", study_uid, series_uid),
                        warp::http::StatusCode::NOT_IMPLEMENTED,
                    )
                });

            // Route: /studies/{study_uid}/series/{series_uid}/instances/{instance_uid}
            let config_clone = config.clone();
            let retrieve_instance = warp::path!("studies" / String / "series" / String / "instances" / String)
                .and(warp::get())
                .map(move |study_uid: String, series_uid: String, instance_uid: String| {
                    retrieve_instance_sync(study_uid, series_uid, instance_uid, config_clone.clone())
                });

            // Route: /studies/{study_uid}/metadata
            let retrieve_study_metadata = warp::path!("studies" / String / "metadata")
                .and(warp::get())
                .map(move |study_uid: String| {
                    warp::reply::with_status(
                        format!("Retrieve study metadata: {}", study_uid),
                        warp::http::StatusCode::NOT_IMPLEMENTED,
                    )
                });

            let routes = retrieve_study
                .or(retrieve_series)
                .or(retrieve_instance)
                .or(retrieve_study_metadata);

            let (_, server) = warp::serve(routes)
                .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
                    shutdown_rx.await.ok();
                });

            server.await;
        });

        self.runtime = Some(runtime);
        self.shutdown_tx = Some(shutdown_tx);
        
        Ok(())
    }

    /// Stop the WADO server
    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.runtime = None;
        Ok(())
    }
}

// Handler functions
fn retrieve_instance_sync(
    study_uid: String,
    series_uid: String,
    instance_uid: String,
    config: WadoStorageConfig,
) -> Response {
    match config.storage_type.as_str() {
        "filesystem" => {
            if let Some(base_path) = config.base_path {
                let file_path = PathBuf::from(base_path)
                    .join(&study_uid)
                    .join(&series_uid)
                    .join(format!("{}.dcm", instance_uid));

                if file_path.exists() {
                    match std::fs::read(&file_path) {
                        Ok(data) => {
                            return warp::reply::with_header(
                                data,
                                "Content-Type",
                                "application/dicom",
                            )
                            .into_response();
                        }
                        Err(_) => {
                            return warp::reply::with_status(
                                "Failed to read file",
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into_response();
                        }
                    }
                }
            }
            warp::reply::with_status(
                "Instance not found",
                warp::http::StatusCode::NOT_FOUND,
            )
            .into_response()
        }
        "s3" => {
            warp::reply::with_status(
                "S3 retrieval not yet implemented",
                warp::http::StatusCode::NOT_IMPLEMENTED,
            )
            .into_response()
        }
        _ => warp::reply::with_status(
            "Invalid storage type",
            warp::http::StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}
