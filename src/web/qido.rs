use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use warp::Filter;

lazy_static::lazy_static! {
    // Global tokio runtime - CRITICAL: Same pattern as StoreSCP for ThreadsafeFunction to work
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

// ============================================================================
// DICOM JSON Model (PS3.18 Section F.2)
// ============================================================================

/// DICOM JSON Value representation (PS3.18 Section F.2.2)
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomJsonValue {
    /// Value Representation (e.g., "PN", "DA", "TM", "UI", "LO", "SH")
    pub vr: String,
    /// Array of values - always an array even for single values
    #[serde(rename = "Value")]
    pub value: Option<Vec<String>>,
}

/// DICOM JSON Attribute - a single tag with its value
/// Tag is the key in the parent object (e.g., "00100010")
pub type DicomJsonAttributes = HashMap<String, DicomJsonValue>;

// ============================================================================
// QIDO-RS Query Parameters (PS3.18 Table 10.6.1-2)
// ============================================================================

/// Search for Studies - All Studies
/// Endpoint: /studies
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchForStudiesQuery {
    // Standard query parameters
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub fuzzymatching: Option<bool>,
    pub includefield: Option<String>,
    
    // Study-level matching attributes (Table 10.6.1-2)
    #[serde(rename = "StudyDate")]
    pub study_date: Option<String>,
    #[serde(rename = "StudyTime")]
    pub study_time: Option<String>,
    #[serde(rename = "AccessionNumber")]
    pub accession_number: Option<String>,
    #[serde(rename = "ModalitiesInStudy")]
    pub modalities_in_study: Option<String>,
    #[serde(rename = "ReferringPhysicianName")]
    pub referring_physician_name: Option<String>,
    #[serde(rename = "PatientName")]
    pub patient_name: Option<String>,
    #[serde(rename = "PatientID")]
    pub patient_id: Option<String>,
    #[serde(rename = "StudyInstanceUID")]
    pub study_instance_uid: Option<String>,
    #[serde(rename = "StudyID")]
    pub study_id: Option<String>,
}

/// Search for Series - All Series in a Study
/// Endpoint: /studies/{StudyInstanceUID}/series
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchForSeriesQuery {
    // Path parameter
    #[serde(rename = "StudyInstanceUID")]
    pub study_instance_uid: String,
    
    // Standard query parameters
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub fuzzymatching: Option<bool>,
    pub includefield: Option<String>,
    
    // Series-level matching attributes
    #[serde(rename = "Modality")]
    pub modality: Option<String>,
    #[serde(rename = "SeriesInstanceUID")]
    pub series_instance_uid: Option<String>,
    #[serde(rename = "SeriesNumber")]
    pub series_number: Option<String>,
    #[serde(rename = "PerformedProcedureStepStartDate")]
    pub performed_procedure_step_start_date: Option<String>,
    #[serde(rename = "PerformedProcedureStepStartTime")]
    pub performed_procedure_step_start_time: Option<String>,
    #[serde(rename = "RequestAttributeSequence.ScheduledProcedureStepID")]
    pub scheduled_procedure_step_id: Option<String>,
    #[serde(rename = "RequestAttributeSequence.RequestedProcedureID")]
    pub requested_procedure_id: Option<String>,
}

/// Search for Instances - All Instances in a Study
/// Endpoint: /studies/{StudyInstanceUID}/instances
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchForStudyInstancesQuery {
    // Path parameter
    #[serde(rename = "StudyInstanceUID")]
    pub study_instance_uid: String,
    
    // Standard query parameters
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub fuzzymatching: Option<bool>,
    pub includefield: Option<String>,
    
    // Instance-level matching attributes
    #[serde(rename = "SOPClassUID")]
    pub sop_class_uid: Option<String>,
    #[serde(rename = "SOPInstanceUID")]
    pub sop_instance_uid: Option<String>,
    #[serde(rename = "InstanceNumber")]
    pub instance_number: Option<String>,
}

/// Search for Instances - All Instances in a Series
/// Endpoint: /studies/{StudyInstanceUID}/series/{SeriesInstanceUID}/instances
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchForSeriesInstancesQuery {
    // Path parameters
    #[serde(rename = "StudyInstanceUID")]
    pub study_instance_uid: String,
    #[serde(rename = "SeriesInstanceUID")]
    pub series_instance_uid: String,
    
    // Standard query parameters
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub fuzzymatching: Option<bool>,
    pub includefield: Option<String>,
    
    // Instance-level matching attributes
    #[serde(rename = "SOPClassUID")]
    pub sop_class_uid: Option<String>,
    #[serde(rename = "SOPInstanceUID")]
    pub sop_instance_uid: Option<String>,
    #[serde(rename = "InstanceNumber")]
    pub instance_number: Option<String>,
}

// ============================================================================
// Response Types - Properly typed for each query level
// ============================================================================

/// Study-level attributes returned by Search for Studies
/// Contains all Study-level tags as per PS3.18 Table 10.6.1-2
pub type StudyAttributes = DicomJsonAttributes;

/// Series-level attributes returned by Search for Series
/// Contains all Series-level tags
pub type SeriesAttributes = DicomJsonAttributes;

/// Instance-level attributes returned by Search for Instances
/// Contains all Instance-level tags
pub type InstanceAttributes = DicomJsonAttributes;

// ============================================================================
// Callback Types for Each Query Level
// ============================================================================

type SearchForStudiesHandler = ThreadsafeFunction<SearchForStudiesQuery, String>;
type SearchForSeriesHandler = ThreadsafeFunction<SearchForSeriesQuery, String>;
type SearchForStudyInstancesHandler = ThreadsafeFunction<SearchForStudyInstancesQuery, String>;
type SearchForSeriesInstancesHandler = ThreadsafeFunction<SearchForSeriesInstancesQuery, String>;

// ============================================================================
// High-Level Builder APIs - Hide DICOM JSON complexity
// ============================================================================

/// Builder for creating Study-level DICOM JSON responses
/// Handles all the DICOM tags and VR types automatically
#[napi]
pub struct QidoStudyResult {
    attributes: HashMap<String, DicomJsonValue>,
}

#[napi]
impl QidoStudyResult {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    // Patient Module
    #[napi]
    pub fn patient_name(&mut self, value: String) -> &Self {
        self.attributes.insert("00100010".to_string(), DicomJsonValue { vr: "PN".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn patient_id(&mut self, value: String) -> &Self {
        self.attributes.insert("00100020".to_string(), DicomJsonValue { vr: "LO".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn patient_birth_date(&mut self, value: String) -> &Self {
        self.attributes.insert("00100030".to_string(), DicomJsonValue { vr: "DA".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn patient_sex(&mut self, value: String) -> &Self {
        self.attributes.insert("00100040".to_string(), DicomJsonValue { vr: "CS".to_string(), value: Some(vec![value]) });
        self
    }

    // Study Module
    #[napi]
    pub fn study_instance_uid(&mut self, value: String) -> &Self {
        self.attributes.insert("0020000D".to_string(), DicomJsonValue { vr: "UI".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn study_date(&mut self, value: String) -> &Self {
        self.attributes.insert("00080020".to_string(), DicomJsonValue { vr: "DA".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn study_time(&mut self, value: String) -> &Self {
        self.attributes.insert("00080030".to_string(), DicomJsonValue { vr: "TM".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn accession_number(&mut self, value: String) -> &Self {
        self.attributes.insert("00080050".to_string(), DicomJsonValue { vr: "SH".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn study_description(&mut self, value: String) -> &Self {
        self.attributes.insert("00081030".to_string(), DicomJsonValue { vr: "LO".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn study_id(&mut self, value: String) -> &Self {
        self.attributes.insert("00200010".to_string(), DicomJsonValue { vr: "SH".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn referring_physician_name(&mut self, value: String) -> &Self {
        self.attributes.insert("00080090".to_string(), DicomJsonValue { vr: "PN".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn modalities_in_study(&mut self, value: String) -> &Self {
        self.attributes.insert("00080061".to_string(), DicomJsonValue { vr: "CS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn number_of_study_related_series(&mut self, value: String) -> &Self {
        self.attributes.insert("00201206".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn number_of_study_related_instances(&mut self, value: String) -> &Self {
        self.attributes.insert("00201208".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    /// Internal method to get attributes
    pub fn get_attributes(&self) -> HashMap<String, DicomJsonValue> {
        self.attributes.clone()
    }
}

/// Builder for creating Series-level DICOM JSON responses
#[napi]
pub struct QidoSeriesResult {
    attributes: HashMap<String, DicomJsonValue>,
}

#[napi]
impl QidoSeriesResult {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    #[napi]
    pub fn series_instance_uid(&mut self, value: String) -> &Self {
        self.attributes.insert("0020000E".to_string(), DicomJsonValue { vr: "UI".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn modality(&mut self, value: String) -> &Self {
        self.attributes.insert("00080060".to_string(), DicomJsonValue { vr: "CS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn series_number(&mut self, value: String) -> &Self {
        self.attributes.insert("00200011".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn series_description(&mut self, value: String) -> &Self {
        self.attributes.insert("0008103E".to_string(), DicomJsonValue { vr: "LO".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn series_date(&mut self, value: String) -> &Self {
        self.attributes.insert("00080021".to_string(), DicomJsonValue { vr: "DA".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn series_time(&mut self, value: String) -> &Self {
        self.attributes.insert("00080031".to_string(), DicomJsonValue { vr: "TM".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn performing_physician_name(&mut self, value: String) -> &Self {
        self.attributes.insert("00081050".to_string(), DicomJsonValue { vr: "PN".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn number_of_series_related_instances(&mut self, value: String) -> &Self {
        self.attributes.insert("00201209".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn body_part_examined(&mut self, value: String) -> &Self {
        self.attributes.insert("00180015".to_string(), DicomJsonValue { vr: "CS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn protocol_name(&mut self, value: String) -> &Self {
        self.attributes.insert("00181030".to_string(), DicomJsonValue { vr: "LO".to_string(), value: Some(vec![value]) });
        self
    }

    pub fn get_attributes(&self) -> HashMap<String, DicomJsonValue> {
        self.attributes.clone()
    }
}

/// Builder for creating Instance-level DICOM JSON responses
#[napi]
pub struct QidoInstanceResult {
    attributes: HashMap<String, DicomJsonValue>,
}

#[napi]
impl QidoInstanceResult {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    #[napi]
    pub fn sop_instance_uid(&mut self, value: String) -> &Self {
        self.attributes.insert("00080018".to_string(), DicomJsonValue { vr: "UI".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn sop_class_uid(&mut self, value: String) -> &Self {
        self.attributes.insert("00080016".to_string(), DicomJsonValue { vr: "UI".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn instance_number(&mut self, value: String) -> &Self {
        self.attributes.insert("00200013".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn rows(&mut self, value: String) -> &Self {
        self.attributes.insert("00280010".to_string(), DicomJsonValue { vr: "US".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn columns(&mut self, value: String) -> &Self {
        self.attributes.insert("00280011".to_string(), DicomJsonValue { vr: "US".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn bits_allocated(&mut self, value: String) -> &Self {
        self.attributes.insert("00280100".to_string(), DicomJsonValue { vr: "US".to_string(), value: Some(vec![value]) });
        self
    }

    #[napi]
    pub fn number_of_frames(&mut self, value: String) -> &Self {
        self.attributes.insert("00280008".to_string(), DicomJsonValue { vr: "IS".to_string(), value: Some(vec![value]) });
        self
    }

    pub fn get_attributes(&self) -> HashMap<String, DicomJsonValue> {
        self.attributes.clone()
    }
}

/// Create final JSON response from Study results
#[napi]
pub fn create_qido_studies_response(studies: Vec<&QidoStudyResult>) -> String {
    let json_array: Vec<HashMap<String, DicomJsonValue>> = studies
        .iter()
        .map(|s| s.get_attributes())
        .collect();
    serde_json::to_string(&json_array).unwrap_or_else(|_| "[]".to_string())
}

/// Create final JSON response from Series results
#[napi]
pub fn create_qido_series_response(series: Vec<&QidoSeriesResult>) -> String {
    let json_array: Vec<HashMap<String, DicomJsonValue>> = series
        .iter()
        .map(|s| s.get_attributes())
        .collect();
    serde_json::to_string(&json_array).unwrap_or_else(|_| "[]".to_string())
}

/// Create final JSON response from Instance results
#[napi]
pub fn create_qido_instances_response(instances: Vec<&QidoInstanceResult>) -> String {
    let json_array: Vec<HashMap<String, DicomJsonValue>> = instances
        .iter()
        .map(|i| i.get_attributes())
        .collect();
    serde_json::to_string(&json_array).unwrap_or_else(|_| "[]".to_string())
}

/// Helper to create empty response array
#[napi]
pub fn create_qido_empty_response() -> String {
    "[]".to_string()
}

// ============================================================================
// QIDO-RS Server with Typed Handlers
// ============================================================================

/// QIDO-RS Server (using warp + RUNTIME pattern like StoreSCP)
#[napi]
pub struct QidoServer {
    port: u16,
    search_for_studies_handler: Arc<RwLock<Option<Arc<SearchForStudiesHandler>>>>,
    search_for_series_handler: Arc<RwLock<Option<Arc<SearchForSeriesHandler>>>>,
    search_for_study_instances_handler: Arc<RwLock<Option<Arc<SearchForStudyInstancesHandler>>>>,
    search_for_series_instances_handler: Arc<RwLock<Option<Arc<SearchForSeriesInstancesHandler>>>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[napi]
impl QidoServer {
    #[napi(constructor)]
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            search_for_studies_handler: Arc::new(RwLock::new(None)),
            search_for_series_handler: Arc::new(RwLock::new(None)),
            search_for_study_instances_handler: Arc::new(RwLock::new(None)),
            search_for_series_instances_handler: Arc::new(RwLock::new(None)),
            shutdown_tx: None,
        })
    }

    /// Register handler for "Search for Studies" query (GET /studies)
    /// Callback receives SearchForStudiesQuery and returns JSON string array
    #[napi(ts_args_type = "callback: (err: Error | null, query: SearchForStudiesQuery) => string")]
    pub fn on_search_for_studies(&mut self, callback: SearchForStudiesHandler) -> Result<()> {
        RUNTIME.block_on(async {
            let mut handler = self.search_for_studies_handler.write().await;
            *handler = Some(Arc::new(callback));
        });
        Ok(())
    }

    /// Register handler for "Search for Series" query (GET /studies/{uid}/series)
    /// Callback receives SearchForSeriesQuery and returns JSON string array
    #[napi(ts_args_type = "callback: (err: Error | null, query: SearchForSeriesQuery) => string")]
    pub fn on_search_for_series(&mut self, callback: SearchForSeriesHandler) -> Result<()> {
        RUNTIME.block_on(async {
            let mut handler = self.search_for_series_handler.write().await;
            *handler = Some(Arc::new(callback));
        });
        Ok(())
    }

    /// Register handler for "Search for Instances" in a Study (GET /studies/{uid}/instances)
    /// Callback receives SearchForStudyInstancesQuery and returns JSON string array
    #[napi(ts_args_type = "callback: (err: Error | null, query: SearchForStudyInstancesQuery) => string")]
    pub fn on_search_for_study_instances(&mut self, callback: SearchForStudyInstancesHandler) -> Result<()> {
        RUNTIME.block_on(async {
            let mut handler = self.search_for_study_instances_handler.write().await;
            *handler = Some(Arc::new(callback));
        });
        Ok(())
    }

    /// Register handler for "Search for Instances" in a Series (GET /studies/{uid}/series/{uid}/instances)
    /// Callback receives SearchForSeriesInstancesQuery and returns JSON string array
    #[napi(ts_args_type = "callback: (err: Error | null, query: SearchForSeriesInstancesQuery) => string")]
    pub fn on_search_for_series_instances(&mut self, callback: SearchForSeriesInstancesHandler) -> Result<()> {
        RUNTIME.block_on(async {
            let mut handler = self.search_for_series_instances_handler.write().await;
            *handler = Some(Arc::new(callback));
        });
        Ok(())
    }

    /// Start the QIDO server using RUNTIME pattern like StoreSCP
    #[napi]
    pub fn start(&mut self) -> Result<()> {
        let port = self.port;
        let studies_handler = self.search_for_studies_handler.clone();
        let series_handler = self.search_for_series_handler.clone();
        let study_instances_handler = self.search_for_study_instances_handler.clone();
        let series_instances_handler = self.search_for_series_instances_handler.clone();
        
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);
        
        eprintln!("Starting QIDO server on port {}...", port);
        
        // Spawn server task in RUNTIME (same pattern as StoreSCP)
        RUNTIME.spawn(async move {
            // GET /studies - Search for Studies
            let studies_route = warp::path!("studies")
                .and(warp::get())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::any().map(move || studies_handler.clone()))
                .and_then(handle_search_for_studies);
            
            // GET /studies/{StudyInstanceUID}/series - Search for Series
            let series_route = warp::path!("studies" / String / "series")
                .and(warp::get())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::any().map(move || series_handler.clone()))
                .and_then(handle_search_for_series);
            
            // GET /studies/{StudyInstanceUID}/instances - Search for Instances in Study
            let study_instances_route = warp::path!("studies" / String / "instances")
                .and(warp::get())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::any().map(move || study_instances_handler.clone()))
                .and_then(handle_search_for_study_instances);
            
            // GET /studies/{StudyInstanceUID}/series/{SeriesInstanceUID}/instances - Search for Instances in Series
            let series_instances_route = warp::path!("studies" / String / "series" / String / "instances")
                .and(warp::get())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::any().map(move || series_instances_handler.clone()))
                .and_then(handle_search_for_series_instances);
            
            let routes = studies_route
                .or(series_route)
                .or(study_instances_route)
                .or(series_instances_route);
            
            let (_addr, server) = warp::serve(routes)
                .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
                    shutdown_rx.await.ok();
                });
            
            eprintln!("✓ QIDO server listening on http://0.0.0.0:{}", port);
            server.await;
        });
        
        Ok(())
    }

    /// Stop the QIDO server
    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        eprintln!("Stopping QIDO server...");
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

// ============================================================================
// Route Handlers - Each properly typed for their query level
// ============================================================================

/// Handler for GET /studies - Search for Studies
async fn handle_search_for_studies(
    params: HashMap<String, String>,
    handler: Arc<RwLock<Option<Arc<SearchForStudiesHandler>>>>,
) -> std::result::Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    let handler_lock = handler.read().await;
    let handler_arc = match &*handler_lock {
        Some(h) => h.clone(),
        None => {
            drop(handler_lock);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "No handler registered for Search for Studies"})),
                warp::http::StatusCode::NOT_IMPLEMENTED,
            ));
        }
    };
    drop(handler_lock);
    
    // Parse query parameters into SearchForStudiesQuery
    let query: SearchForStudiesQuery = serde_json::from_value(serde_json::to_value(&params).unwrap())
        .unwrap_or_default();
    
    // Create channel for response
    let (tx, rx) = tokio::sync::oneshot::channel::<std::result::Result<String, String>>();
    
    // Call JS callback
    let _status = handler_arc.call_with_return_value(
        Ok(query),
        ThreadsafeFunctionCallMode::Blocking,
        move |result: std::result::Result<String, _>, _env| {
            match result {
                Ok(json_string) => { let _ = tx.send(Ok(json_string)); }
                Err(e) => { let _ = tx.send(Err(format!("Callback error: {:?}", e))); }
            }
            Ok(())
        }
    );
    
    handle_response(rx).await
}

/// Handler for GET /studies/{StudyInstanceUID}/series - Search for Series
async fn handle_search_for_series(
    study_uid: String,
    params: HashMap<String, String>,
    handler: Arc<RwLock<Option<Arc<SearchForSeriesHandler>>>>,
) -> std::result::Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    let handler_lock = handler.read().await;
    let handler_arc = match &*handler_lock {
        Some(h) => h.clone(),
        None => {
            drop(handler_lock);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "No handler registered for Search for Series"})),
                warp::http::StatusCode::NOT_IMPLEMENTED,
            ));
        }
    };
    drop(handler_lock);
    
    // Parse query with StudyInstanceUID from path
    let mut all_params = params.clone();
    all_params.insert("StudyInstanceUID".to_string(), study_uid);
    let query: SearchForSeriesQuery = serde_json::from_value(serde_json::to_value(&all_params).unwrap())
        .unwrap_or_else(|_| {
            let mut q = SearchForSeriesQuery::default();
            q.study_instance_uid = all_params.get("StudyInstanceUID").unwrap().clone();
            q
        });
    
    let (tx, rx) = tokio::sync::oneshot::channel::<std::result::Result<String, String>>();
    
    let _status = handler_arc.call_with_return_value(
        Ok(query),
        ThreadsafeFunctionCallMode::Blocking,
        move |result: std::result::Result<String, _>, _env| {
            match result {
                Ok(json_string) => { let _ = tx.send(Ok(json_string)); }
                Err(e) => { let _ = tx.send(Err(format!("Callback error: {:?}", e))); }
            }
            Ok(())
        }
    );
    
    handle_response(rx).await
}

/// Handler for GET /studies/{StudyInstanceUID}/instances - Search for Instances in Study
async fn handle_search_for_study_instances(
    study_uid: String,
    params: HashMap<String, String>,
    handler: Arc<RwLock<Option<Arc<SearchForStudyInstancesHandler>>>>,
) -> std::result::Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    let handler_lock = handler.read().await;
    let handler_arc = match &*handler_lock {
        Some(h) => h.clone(),
        None => {
            drop(handler_lock);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "No handler registered for Search for Study Instances"})),
                warp::http::StatusCode::NOT_IMPLEMENTED,
            ));
        }
    };
    drop(handler_lock);
    
    let mut all_params = params.clone();
    all_params.insert("StudyInstanceUID".to_string(), study_uid);
    let query: SearchForStudyInstancesQuery = serde_json::from_value(serde_json::to_value(&all_params).unwrap())
        .unwrap_or_else(|_| {
            let mut q = SearchForStudyInstancesQuery::default();
            q.study_instance_uid = all_params.get("StudyInstanceUID").unwrap().clone();
            q
        });
    
    let (tx, rx) = tokio::sync::oneshot::channel::<std::result::Result<String, String>>();
    
    let _status = handler_arc.call_with_return_value(
        Ok(query),
        ThreadsafeFunctionCallMode::Blocking,
        move |result: std::result::Result<String, _>, _env| {
            match result {
                Ok(json_string) => { let _ = tx.send(Ok(json_string)); }
                Err(e) => { let _ = tx.send(Err(format!("Callback error: {:?}", e))); }
            }
            Ok(())
        }
    );
    
    handle_response(rx).await
}

/// Handler for GET /studies/{StudyInstanceUID}/series/{SeriesInstanceUID}/instances
async fn handle_search_for_series_instances(
    study_uid: String,
    series_uid: String,
    params: HashMap<String, String>,
    handler: Arc<RwLock<Option<Arc<SearchForSeriesInstancesHandler>>>>,
) -> std::result::Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    let handler_lock = handler.read().await;
    let handler_arc = match &*handler_lock {
        Some(h) => h.clone(),
        None => {
            drop(handler_lock);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "No handler registered for Search for Series Instances"})),
                warp::http::StatusCode::NOT_IMPLEMENTED,
            ));
        }
    };
    drop(handler_lock);
    
    let mut all_params = params.clone();
    all_params.insert("StudyInstanceUID".to_string(), study_uid);
    all_params.insert("SeriesInstanceUID".to_string(), series_uid);
    let query: SearchForSeriesInstancesQuery = serde_json::from_value(serde_json::to_value(&all_params).unwrap())
        .unwrap_or_else(|_| {
            let mut q = SearchForSeriesInstancesQuery::default();
            q.study_instance_uid = all_params.get("StudyInstanceUID").unwrap().clone();
            q.series_instance_uid = all_params.get("SeriesInstanceUID").unwrap().clone();
            q
        });
    
    let (tx, rx) = tokio::sync::oneshot::channel::<std::result::Result<String, String>>();
    
    let _status = handler_arc.call_with_return_value(
        Ok(query),
        ThreadsafeFunctionCallMode::Blocking,
        move |result: std::result::Result<String, _>, _env| {
            match result {
                Ok(json_string) => { let _ = tx.send(Ok(json_string)); }
                Err(e) => { let _ = tx.send(Err(format!("Callback error: {:?}", e))); }
            }
            Ok(())
        }
    );
    
    handle_response(rx).await
}

/// Shared response handler - converts String response to warp Reply
async fn handle_response(
    rx: tokio::sync::oneshot::Receiver<std::result::Result<String, String>>,
) -> std::result::Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    let response_json = match tokio::time::timeout(tokio::time::Duration::from_secs(5), rx).await {
        Ok(Ok(Ok(json))) => json,
        Ok(Ok(Err(e))) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": e})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        Ok(Err(_)) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Channel closed"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Timeout"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    
    // Parse and return as DICOM JSON
    match serde_json::from_str::<serde_json::Value>(&response_json) {
        Ok(json) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json),
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": format!("Invalid JSON: {}", e)})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}