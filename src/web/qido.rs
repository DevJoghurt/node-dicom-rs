use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use warp::Filter;

/// QIDO-RS Query Parameters for Study Level
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QidoStudyQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub fuzzymatching: Option<bool>,
    #[serde(rename = "PatientName")]
    pub patient_name: Option<String>,
    #[serde(rename = "PatientID")]
    pub patient_id: Option<String>,
    #[serde(rename = "StudyDate")]
    pub study_date: Option<String>,
    #[serde(rename = "StudyInstanceUID")]
    pub study_instance_uid: Option<String>,
    #[serde(rename = "AccessionNumber")]
    pub accession_number: Option<String>,
}

/// QIDO-RS Query Parameters for Series Level
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QidoSeriesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "Modality")]
    pub modality: Option<String>,
    #[serde(rename = "SeriesInstanceUID")]
    pub series_instance_uid: Option<String>,
    #[serde(rename = "SeriesNumber")]
    pub series_number: Option<String>,
}

/// QIDO-RS Query Parameters for Instance Level
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QidoInstanceQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "SOPInstanceUID")]
    pub sop_instance_uid: Option<String>,
    #[serde(rename = "InstanceNumber")]
    pub instance_number: Option<String>,
}

/// QIDO-RS Query Response - representing a DICOM JSON dataset
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QidoResponse {
    /// JSON representation of DICOM dataset
    pub data: String,
}

/// QIDO-RS Server
#[napi]
pub struct QidoServer {
    port: u16,
    runtime: Option<Runtime>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[napi]
impl QidoServer {
    #[napi(constructor)]
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            runtime: None,
            shutdown_tx: None,
        })
    }

    /// Start the QIDO server
    /// Note: The callback mechanism will be implemented in a future version
    #[napi]
    pub fn start(&mut self) -> Result<()> {
        let runtime = Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        let port = self.port;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        runtime.spawn(async move {
            let search_studies = warp::path!("studies")
                .and(warp::get())
                .and(warp::query::<QidoStudyQuery>())
                .map(|_query: QidoStudyQuery| {
                    // Placeholder response
                    warp::reply::with_header(
                        "[]",
                        "Content-Type",
                        "application/dicom+json",
                    )
                });

            let routes = search_studies;

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

    /// Stop the QIDO server
    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.runtime = None;
        Ok(())
    }
}
