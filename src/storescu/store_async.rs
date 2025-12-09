use std::sync::Arc;

use dicom_dictionary_std::tags;
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{open_file, FileDicomObject, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{
    pdu::{PDataValue, PDataValueType},
    ClientAssociation, Pdu,
};
use indicatif::ProgressBar;
use snafu::{OptionExt, Report, ResultExt};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tracing::{debug, error, info, warn};

use crate::storescu::{
    check_presentation_contexts, into_ts, store_req_command, ConvertFieldSnafu, CreateCommandSnafu,
    DicomFile, Error, Event, EventData, FileSource, MissingAttributeSnafu, ReadDatasetSnafu, 
    ReadFilePathSnafu, ScuSnafu, StoreSCU, UnsupportedFileTransferSyntaxSnafu, WriteDatasetSnafu,
};



pub async fn send_file(
    mut scu: ClientAssociation<TcpStream>,
    file: DicomFile,
    s3_bucket: Option<&s3::Bucket>,
    message_id: u16,
    progress_bar: Option<&Arc<tokio::sync::Mutex<ProgressBar>>>,
    verbose: bool,
    fail_first: bool,
) -> Result<ClientAssociation<TcpStream>, Error>
{
    let start_time = std::time::Instant::now();
    
    if let (Some(pc_selected), Some(ts_uid_selected)) = (file.pc_selected, file.ts_selected) {
        // Emit OnFileSending event
        let file_path = match &file.source {
            FileSource::Local(path) => path.display().to_string(),
            FileSource::S3(key) => format!("s3://{}", key),
        };
        
        StoreSCU::emit_event(Event::OnFileSending, EventData {
            message: "Sending file".to_string(),
            data: Some(serde_json::json!({
                "file": file_path,
                "sopInstanceUid": file.sop_instance_uid,
                "sopClassUid": file.sop_class_uid,
                "transferSyntax": ts_uid_selected,
            }).to_string()),
        });
        let cmd = store_req_command(&file.sop_class_uid, &file.sop_instance_uid, message_id);

        let mut cmd_data = Vec::with_capacity(128);
        cmd.write_dataset_with_ts(
            &mut cmd_data,
            &dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN.erased(),
        )
        .map_err(Box::from)
        .context(CreateCommandSnafu)?;

        let mut object_data = Vec::with_capacity(2048);
        
        // Load DICOM file from source (local filesystem or S3)
        let dicom_file: FileDicomObject<InMemDicomObject> = match &file.source {
            FileSource::Local(path) => {
                open_file(path)
                    .map_err(Box::from)
                    .context(ReadFilePathSnafu {
                        path: path.display().to_string(),
                    })?
            }
            FileSource::S3(key) => {
                // Download S3 file on-demand to minimize memory usage
                use crate::storescu::s3_storage;
                let bucket = s3_bucket.expect("S3 bucket should be available for S3 files");
                let data = s3_storage::s3_get_object(bucket, key)
                    .await
                    .map_err(|e| Error::ReadFilePath {
                        path: format!("s3://{}", key),
                        source: Box::new(dicom_object::ReadError::ReadFile {
                            filename: format!("s3://{}", key).into(),
                            source: std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to download S3 file for sending: {}", e),
                            ),
                            backtrace: std::backtrace::Backtrace::capture(),
                        }),
                    })?;
                
                // Auto-detect file format by checking for DICM magic bytes
                let has_dicm_magic = data.len() > 132 && &data[128..132] == b"DICM";
                
                if !has_dicm_magic {
                    // Dataset-only file (no DICOM meta header) - read as InMemDicomObject and create meta
                    let obj = InMemDicomObject::read_dataset_with_ts(
                        &data[..],
                        &dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.erased(),
                    )
                    .or_else(|_| {
                        InMemDicomObject::read_dataset_with_ts(
                            &data[..],
                            &dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN.erased(),
                        )
                    })
                    .context(ReadDatasetSnafu)?;
                    
                    // Create file meta information from dataset attributes
                    use dicom_dictionary_std::tags;
                    use dicom_object::FileMetaTableBuilder;
                    
                    let sop_class_uid = obj.element(tags::SOP_CLASS_UID)
                        .context(MissingAttributeSnafu { tag: tags::SOP_CLASS_UID })?
                        .to_str()
                        .context(ConvertFieldSnafu { tag: tags::SOP_CLASS_UID })?
                        .trim()
                        .to_string();
                    let sop_instance_uid = obj.element(tags::SOP_INSTANCE_UID)
                        .context(MissingAttributeSnafu { tag: tags::SOP_INSTANCE_UID })?
                        .to_str()
                        .context(ConvertFieldSnafu { tag: tags::SOP_INSTANCE_UID })?
                        .trim()
                        .to_string();
                    
                    let meta = FileMetaTableBuilder::new()
                        .media_storage_sop_class_uid(&sop_class_uid)
                        .media_storage_sop_instance_uid(&sop_instance_uid)
                        .transfer_syntax(dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid())
                        .build()
                        .map_err(|e| Error::ReadDataset { 
                            source: dicom_object::ReadError::ParseMetaDataSet { source: e } 
                        })?;
                    
                    obj.with_exact_meta(meta)
                } else {
                    // Full DICOM file with meta header
                    dicom_object::from_reader(&data[..])
                        .context(ReadDatasetSnafu)?
                }
            }
        };
        
        let ts_selected = TransferSyntaxRegistry
            .get(&ts_uid_selected)
            .with_context(|| UnsupportedFileTransferSyntaxSnafu {
                uid: ts_uid_selected.to_string(),
            })?;

        // transcode file if necessary
        let dicom_file = into_ts(dicom_file, ts_selected, verbose)?;

        dicom_file
            .write_dataset_with_ts(&mut object_data, ts_selected)
            .map_err(Box::from)
            .context(WriteDatasetSnafu)?;

        let nbytes = cmd_data.len() + object_data.len();

        if verbose {
            let source_display = match &file.source {
                FileSource::Local(path) => path.display().to_string(),
                FileSource::S3(key) => format!("s3://{}", key),
            };
            info!(
                "Sending file {} (~ {} kB), uid={}, sop={}, ts={}",
                source_display,
                nbytes / 1_000,
                &file.sop_instance_uid,
                &file.sop_class_uid,
                ts_uid_selected,
            );
        }

        if nbytes < scu.acceptor_max_pdu_length().saturating_sub(100) as usize {
            let pdu = Pdu::PData {
                data: vec![
                    PDataValue {
                        presentation_context_id: pc_selected.id,
                        value_type: PDataValueType::Command,
                        is_last: true,
                        data: cmd_data,
                    },
                    PDataValue {
                        presentation_context_id: pc_selected.id,
                        value_type: PDataValueType::Data,
                        is_last: true,
                        data: object_data,
                    },
                ],
            };

            scu.send(&pdu).await.map_err(Box::from).context(ScuSnafu)?;
        } else {
            let pdu = Pdu::PData {
                data: vec![PDataValue {
                    presentation_context_id: pc_selected.id,
                    value_type: PDataValueType::Command,
                    is_last: true,
                    data: cmd_data,
                }],
            };

            scu.send(&pdu).await.map_err(Box::from).context(ScuSnafu)?;

            {
                let mut pdata = scu.send_pdata(pc_selected.id).await;
                pdata.write_all(&object_data).await.unwrap();
                //.whatever_context("Failed to send C-STORE-RQ P-Data")?;
            }
        }

        if verbose {
            debug!("Awaiting response...");
        }

        let rsp_pdu = scu.receive().await.map_err(Box::from).context(ScuSnafu)?;

        match rsp_pdu {
            Pdu::PData { data } => {
                let data_value = &data[0];

                let cmd_obj = InMemDicomObject::read_dataset_with_ts(
                    &data_value.data[..],
                    &dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN.erased(),
                )
                .context(ReadDatasetSnafu)?;
                if verbose {
                    debug!("Full response: {:?}", cmd_obj);
                }
                let status = cmd_obj
                    .element(tags::STATUS)
                    .context(MissingAttributeSnafu { tag: tags::STATUS })?
                    .to_int::<u16>()
                    .context(ConvertFieldSnafu { tag: tags::STATUS })?;
                let storage_sop_instance_uid = file
                    .sop_instance_uid
                    .trim_end_matches(|c: char| c.is_whitespace() || c == '\0');

                match status {
                    // Success
                    0 => {
                        let elapsed = start_time.elapsed();
                        if verbose {
                            info!(
                                "Successfully stored instance {} in {:.2}s",
                                storage_sop_instance_uid,
                                elapsed.as_secs_f64()
                            );
                        }
                        
                        // Emit OnFileSent event
                        let file_path = match &file.source {
                            FileSource::Local(path) => path.display().to_string(),
                            FileSource::S3(key) => format!("s3://{}", key),
                        };
                        
                        StoreSCU::emit_event(Event::OnFileSent, EventData {
                            message: "File sent successfully".to_string(),
                            data: Some(serde_json::json!({
                                "file": file_path,
                                "sopInstanceUid": storage_sop_instance_uid,
                                "sopClassUid": file.sop_class_uid,
                                "transferSyntax": ts_uid_selected,
                                "durationMs": elapsed.as_millis(),
                                "durationSeconds": elapsed.as_secs_f64(),
                                "status": "success",
                            }).to_string()),
                        });
                    }
                    // Warning
                    1 | 0x0107 | 0x0116 | 0xB000..=0xBFFF => {
                        warn!(
                            "Possible issue storing instance `{}` (status code {:04X}H)",
                            storage_sop_instance_uid, status
                        );
                    }
                    0xFF00 | 0xFF01 => {
                        warn!(
                            "Possible issue storing instance `{}`: status is pending (status code {:04X}H)",
                            storage_sop_instance_uid, status
                        );
                    }
                    0xFE00 => {
                        error!(
                            "Could not store instance `{}`: operation cancelled",
                            storage_sop_instance_uid
                        );
                        if fail_first {
                            let _ = scu.abort().await;
                            std::process::exit(-2);
                        }
                    }
                    _ => {
                        let elapsed = start_time.elapsed();
                        error!(
                            "Failed to store instance `{}` (status code {:04X}H)",
                            storage_sop_instance_uid, status
                        );
                        
                        // Emit OnFileError event
                        let file_path = match &file.source {
                            FileSource::Local(path) => path.display().to_string(),
                            FileSource::S3(key) => format!("s3://{}", key),
                        };
                        
                        StoreSCU::emit_event(Event::OnFileError, EventData {
                            message: format!("Failed to store file (status code {:04X}H)", status),
                            data: Some(serde_json::json!({
                                "file": file_path,
                                "sopInstanceUid": storage_sop_instance_uid,
                                "sopClassUid": &file.sop_class_uid,
                                "statusCode": format!("{:04X}H", status),
                                "durationMs": elapsed.as_millis(),
                                "error": format!("Status code {:04X}H", status),
                            }).to_string()),
                        });
                        
                        if fail_first {
                            let _ = scu.abort().await;
                            std::process::exit(-2);
                        }
                    }
                }
            }

            pdu @ Pdu::Unknown { .. }
            | pdu @ Pdu::AssociationRQ { .. }
            | pdu @ Pdu::AssociationAC { .. }
            | pdu @ Pdu::AssociationRJ { .. }
            | pdu @ Pdu::ReleaseRQ
            | pdu @ Pdu::ReleaseRP
            | pdu @ Pdu::AbortRQ { .. } => {
                error!("Unexpected SCP response: {:?}", pdu);
                let _ = scu.abort().await;
                std::process::exit(-2);
            }
        }
    }
    if let Some(pb) = progress_bar.as_ref() {
        pb.lock().await.inc(1)
    };
    Ok(scu)
}

pub async fn inner(
    mut scu: ClientAssociation<TcpStream>,
    d_files: Arc<Mutex<Vec<DicomFile>>>,
    s3_bucket: Option<Arc<s3::Bucket>>,
    progress_bar: Option<&Arc<tokio::sync::Mutex<ProgressBar>>>,
    fail_first: bool,
    verbose: bool,
    never_transcode: bool,
    ignore_sop_class: bool,
) -> Result<(), Error>
{
    let mut message_id = 1;
    loop {
        let file = {
            let mut files = d_files.lock().await;
            files.pop()
        };
        let mut file = match file {
            Some(file) => file,
            None => break,
        };
        let r: Result<_, Error> = check_presentation_contexts(
            &file,
            scu.presentation_contexts(),
            ignore_sop_class,
            never_transcode,
        );
        match r {
            Ok((pc, ts)) => {
                if verbose {
                    let source_display = match &file.source {
                        FileSource::Local(path) => path.display().to_string(),
                        FileSource::S3(key) => format!("s3://{}", key),
                    };
                    debug!(
                        "{}: Selected presentation context: {:?}",
                        source_display,
                        pc
                    );
                }
                file.pc_selected = Some(pc);
                file.ts_selected = Some(ts);
            }
            Err(e) => {
                error!("{}", Report::from_error(e));
                if fail_first {
                    let _ = scu.abort().await;
                    std::process::exit(-2);
                }
            }
        }
        scu = send_file(scu, file, s3_bucket.as_deref(), message_id, progress_bar, verbose, fail_first).await?;
        message_id += 1;
    }
    let _ = scu.release().await;
    Ok(())
}