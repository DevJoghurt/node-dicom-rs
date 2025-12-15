# DICOMweb Implementation

This document describes the DICOMweb server implementation in node-dicom-rs, which provides QIDO-RS (Query based on ID for DICOM Objects) and WADO-RS (Web Access to DICOM Objects) services.

## Overview

The DICOMweb implementation is based on the [dicomweb-rs](https://github.com/feliwir/dicomweb-rs) project and provides two main services:

1. **QIDO-RS**: Query service for searching DICOM studies, series, and instances
2. **WADO-RS**: Retrieval service for accessing DICOM files

## QIDO-RS Server

QIDO-RS allows clients to search for DICOM objects using standardized query parameters.

### Basic Usage

```javascript
const { QidoServer } = require('node-dicom-rs');

// Create a QIDO server on port 8080
const qidoServer = new QidoServer(8080);

// Start the server
qidoServer.start();

// Stop the server when done
qidoServer.stop();
```

### Query Parameters

#### Study Level Queries
- `limit`: Maximum number of results
- `offset`: Number of results to skip
- `fuzzymatching`: Enable fuzzy matching
- `patientName`: Patient name filter
- `patientId`: Patient ID filter
- `studyDate`: Study date filter
- `studyInstanceUid`: Study Instance UID filter
- `accessionNumber`: Accession number filter

#### Series Level Queries
- `limit`: Maximum number of results
- `offset`: Number of results to skip
- `modality`: Modality filter
- `seriesInstanceUid`: Series Instance UID filter
- `seriesNumber`: Series number filter

#### Instance Level Queries
- `limit`: Maximum number of results
- `offset`: Number of results to skip
- `sopInstanceUid`: SOP Instance UID filter
- `instanceNumber`: Instance number filter

### Endpoints

- `GET /studies` - Search for studies
- `GET /studies/{studyUID}/series` - Search for series in a study
- `GET /studies/{studyUID}/series/{seriesUID}/instances` - Search for instances in a series

### Callback Mechanism (Future Implementation)

The callback mechanism for handling queries will be implemented in a future version, similar to the StoreSCP pattern:

```javascript
// Future implementation
qidoServer.onQuery((level, studyUid, seriesUid) => {
  // Query your database
  // Return array of DICOM JSON strings
  return queryResults;
});
```

## WADO-RS Server

WADO-RS provides standardized retrieval of DICOM objects via HTTP.

### Basic Usage

```javascript
const { WadoServer } = require('node-dicom-rs');

// Configure storage backend
const config = {
  storageType: 'filesystem',
  basePath: '/path/to/dicom/files'
};

// Create a WADO server on port 8081
const wadoServer = new WadoServer(8081, config);

// Start the server
wadoServer.start();

// Stop the server when done
wadoServer.stop();
```

### Storage Configuration

#### Filesystem Storage

```javascript
const config = {
  storageType: 'filesystem',
  basePath: '/data/dicom'
};
```

Files should be organized as: `{basePath}/{studyUID}/{seriesUID}/{instanceUID}.dcm`

#### S3 Storage (Future Implementation)

```javascript
const config = {
  storageType: 's3',
  s3Bucket: 'my-dicom-bucket',
  s3Region: 'us-east-1',
  s3Endpoint: 'https://s3.amazonaws.com', // Optional for MinIO/custom
  s3AccessKey: 'ACCESS_KEY',
  s3SecretKey: 'SECRET_KEY'
};
```

### Endpoints

- `GET /studies/{studyUID}` - Retrieve all instances in a study
- `GET /studies/{studyUID}/series/{seriesUID}` - Retrieve all instances in a series
- `GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}` - Retrieve a specific instance
- `GET /studies/{studyUID}/metadata` - Retrieve metadata for a study

### Response Format

WADO-RS returns DICOM files with the content type `application/dicom` or `multipart/related` for multiple instances.

## Architecture

The DICOMweb implementation uses:

- **Warp**: Fast, composable web framework for Rust
- **Tokio**: Async runtime for handling concurrent requests
- **NAPI-RS**: Bindings to expose Rust functionality to Node.js

## Compliance

The implementation follows the DICOM PS3.18 standard for DICOMweb services:
- [QIDO-RS Specification](https://www.dicomstandard.org/using/dicomweb/query-qido-rs)
- [WADO-RS Specification](https://www.dicomstandard.org/using/dicomweb/retrieve-wado-rs-and-wado-uri)

## Future Enhancements

- [ ] Implement callback mechanism for QIDO-RS queries
- [ ] Complete S3 storage backend for WADO-RS
- [ ] Add support for STOW-RS (Store over the Web)
- [ ] Add metadata-only retrieval endpoints
- [ ] Implement frame-level retrieval
- [ ] Add support for rendered media types
- [ ] Implement DICOM JSON metadata responses
- [ ] Add authentication and authorization
- [ ] Implement bulk data retrieval

## Example Integration

```javascript
const { QidoServer, WadoServer } = require('node-dicom-rs');

// Start QIDO server
const qidoServer = new QidoServer(8080);
qidoServer.start();
console.log('QIDO server listening on port 8080');

// Start WADO server
const wadoConfig = {
  storageType: 'filesystem',
  basePath: '/data/dicom'
};
const wadoServer = new WadoServer(8081, wadoConfig);
wadoServer.start();
console.log('WADO server listening on port 8081');

// Use with OHIF Viewer or other DICOMweb clients
// Configure your viewer to use:
// - QIDO URL: http://localhost:8080
// - WADO URL: http://localhost:8081

// Cleanup on shutdown
process.on('SIGINT', () => {
  qidoServer.stop();
  wadoServer.stop();
  process.exit(0);
});
```

## Testing

Test the servers using curl or a DICOMweb client:

```bash
# Query for studies
curl http://localhost:8080/studies

# Retrieve an instance
curl http://localhost:8081/studies/1.2.3.4/series/5.6.7.8/instances/9.10.11.12 \
  -o instance.dcm
```
