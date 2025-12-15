# DICOMweb Implementation

This document describes the DICOMweb server implementation in node-dicom-rs, which provides QIDO-RS (Query based on ID for DICOM Objects) and WADO-RS (Web Access to DICOM Objects) services.

## Overview

The DICOMweb implementation is based on the [dicomweb-rs](https://github.com/feliwir/dicomweb-rs) project and provides two main services:

1. **QIDO-RS**: Query service for searching DICOM studies, series, and instances
2. **WADO-RS**: Retrieval service for accessing DICOM files

## QIDO-RS Server

QIDO-RS provides a **high-level, developer-friendly API** that completely hides DICOM JSON complexity. You never need to deal with DICOM tag numbers or VR codes!

### Key Features

✅ **No DICOM Tags** - Use typed methods like `.patientName()`, `.studyDate()`  
✅ **No JSON Complexity** - Builders handle DICOM JSON format automatically  
✅ **Full Type Safety** - TypeScript auto-completion for all attributes  
✅ **DICOM Compliant** - Automatic tag numbers, VR codes, proper formatting  
✅ **Separate Handlers** - One handler per query level (Studies, Series, Instances)

### Quick Start

```javascript
import { 
  QidoServer, 
  QidoStudyResult, 
  createQidoStudiesResponse 
} from 'node-dicom-rs';

const server = new QidoServer(8080);

server.onSearchForStudies((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  // Filter your database by query.patientId, query.studyDate, etc.
  const studies = database.findStudies(query);
  
  // Build results - NO DICOM TAGS!
  const results = studies.map(study => {
    const result = new QidoStudyResult();
    result.patientName(study.patientName);
    result.patientId(study.patientId);
    result.studyInstanceUid(study.studyUid);
    result.studyDate(study.date);
    result.accessionNumber(study.accNum);
    return result;
  });
  
  return createQidoStudiesResponse(results);
});

server.start();
```

### Builder Classes

The API provides three builder classes for constructing DICOM responses without dealing with tag numbers:

#### QidoStudyResult

Builder for Study-level responses (GET /studies).

**Patient Module:**
```javascript
const study = new QidoStudyResult();
study.patientName('Doe^John');           // (0010,0010) PN
study.patientId('12345');                 // (0010,0020) LO
study.patientBirthDate('19800101');       // (0010,0030) DA
study.patientSex('M');                    // (0010,0040) CS
```

**Study Module:**
```javascript
study.studyInstanceUid('1.2.3.4.5');      // (0020,000D) UI
study.studyDate('20240101');              // (0008,0020) DA
study.studyTime('120000');                // (0008,0030) TM
study.accessionNumber('ACC001');          // (0008,0050) SH
study.studyDescription('CT Chest');       // (0008,1030) LO
study.studyId('STU001');                  // (0020,0010) SH
study.referringPhysicianName('Smith^J');  // (0008,0090) PN
study.modalitiesInStudy('CT');            // (0008,0061) CS
study.numberOfStudyRelatedSeries('5');    // (0020,1206) IS
study.numberOfStudyRelatedInstances('100'); // (0020,1208) IS
```

#### QidoSeriesResult

Builder for Series-level responses (GET /studies/{uid}/series).

```javascript
const series = new QidoSeriesResult();
series.seriesInstanceUid('1.2.3.4.5.1');  // (0020,000E) UI
series.modality('CT');                     // (0008,0060) CS
series.seriesNumber('1');                  // (0020,0011) IS
series.seriesDescription('Axial');         // (0008,103E) LO
series.seriesDate('20240101');             // (0008,0021) DA
series.seriesTime('120000');               // (0008,0031) TM
series.performingPhysicianName('Doe^J');   // (0008,1050) PN
series.numberOfSeriesRelatedInstances('20'); // (0020,1209) IS
series.bodyPartExamined('CHEST');          // (0018,0015) CS
series.protocolName('Standard');           // (0018,1030) LO
```

#### QidoInstanceResult

Builder for Instance-level responses (GET /studies/{uid}/instances, GET /studies/{uid}/series/{uid}/instances).

```javascript
const instance = new QidoInstanceResult();
instance.sopInstanceUid('1.2.3.4.5.1.1'); // (0008,0018) UI
instance.sopClassUid('1.2.840...');        // (0008,0016) UI
instance.instanceNumber('1');              // (0020,0013) IS
instance.rows('512');                      // (0028,0010) US
instance.columns('512');                   // (0028,0011) US
instance.bitsAllocated('16');              // (0028,0100) US
instance.numberOfFrames('1');              // (0028,0008) IS
```

### Response Helper Functions

```javascript
// Convert builder arrays to DICOM JSON strings
createQidoStudiesResponse(results: QidoStudyResult[]): string
createQidoSeriesResponse(results: QidoSeriesResult[]): string
createQidoInstancesResponse(results: QidoInstanceResult[]): string
createQidoEmptyResponse(): string  // Returns "[]" for error cases
```

### Handler Registration

The QIDO-RS API provides four separate handlers, one for each query level defined in DICOM PS3.18:

```javascript
import { 
  QidoServer,
  QidoStudyResult,
  QidoSeriesResult,
  QidoInstanceResult,
  createQidoStudiesResponse,
  createQidoSeriesResponse,
  createQidoInstancesResponse,
  createQidoEmptyResponse
} from 'node-dicom-rs';

const server = new QidoServer(8080);

// Handler 1: Search for Studies (GET /studies)
server.onSearchForStudies((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  // query.patientId, query.studyDate, etc. are properly typed
  const studies = database.findStudies(query);
  
  const results = studies.map(s => {
    const result = new QidoStudyResult();
    result.patientName(s.patientName);
    result.patientId(s.patientId);
    result.studyInstanceUid(s.studyUid);
    result.studyDate(s.studyDate);
    return result;
  });
  
  return createQidoStudiesResponse(results);
});

// Handler 2: Search for Series (GET /studies/{uid}/series)
server.onSearchForSeries((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  // query.studyInstanceUid is extracted from URL path
  const series = database.getSeriesByStudy(query.studyInstanceUid);
  
  const results = series.map(s => {
    const result = new QidoSeriesResult();
    result.seriesInstanceUid(s.seriesUid);
    result.modality(s.modality);
    result.seriesNumber(s.seriesNumber);
    return result;
  });
  
  return createQidoSeriesResponse(results);
});

// Handler 3: Search for Instances in Study (GET /studies/{uid}/instances)
server.onSearchForStudyInstances((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  const instances = database.getInstancesByStudy(query.studyInstanceUid);
  
  const results = instances.map(i => {
    const result = new QidoInstanceResult();
    result.sopInstanceUid(i.sopInstanceUid);
    result.instanceNumber(i.instanceNumber);
    return result;
  });
  
  return createQidoInstancesResponse(results);
});

// Handler 4: Search for Instances in Series (GET /studies/{uid}/series/{uid}/instances)
server.onSearchForSeriesInstances((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  // Both UIDs extracted from URL path
  const instances = database.getInstancesBySeries(
    query.studyInstanceUid,
    query.seriesInstanceUid
  );
  
  const results = instances.map(i => {
    const result = new QidoInstanceResult();
    result.sopInstanceUid(i.sopInstanceUid);
    result.instanceNumber(i.instanceNumber);
    return result;
  });
  
  return createQidoInstancesResponse(results);
});

server.start();
```

### Query Parameters

All query handlers receive a typed query object with standard DICOM parameters:

#### SearchForStudiesQuery
```typescript
{
  // Pagination
  limit?: number;
  offset?: number;
  fuzzymatching?: boolean;
  includefield?: string;
  
  // Study-level filters
  studyDate?: string;
  studyTime?: string;
  accessionNumber?: string;
  modalitiesInStudy?: string;
  referringPhysicianName?: string;
  patientName?: string;
  patientId?: string;
  studyInstanceUid?: string;
  studyId?: string;
}
```

#### SearchForSeriesQuery
```typescript
{
  studyInstanceUid: string;  // From URL path
  
  limit?: number;
  offset?: number;
  fuzzymatching?: boolean;
  includefield?: string;
  
  modality?: string;
  seriesInstanceUid?: string;
  seriesNumber?: string;
  performedProcedureStepStartDate?: string;
  performedProcedureStepStartTime?: string;
}
```

#### SearchForStudyInstancesQuery / SearchForSeriesInstancesQuery
```typescript
{
  studyInstanceUid: string;       // From URL path
  seriesInstanceUid?: string;     // From URL path (series instances only)
  
  limit?: number;
  offset?: number;
  fuzzymatching?: boolean;
  includefield?: string;
  
  sopClassUid?: string;
  sopInstanceUid?: string;
  instanceNumber?: string;
}
```

### QIDO-RS Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/studies` | `onSearchForStudies` | Search for studies |
| GET | `/studies/{uid}/series` | `onSearchForSeries` | Search for series in a study |
| GET | `/studies/{uid}/instances` | `onSearchForStudyInstances` | Search for instances in a study |
| GET | `/studies/{uid}/series/{uid}/instances` | `onSearchForSeriesInstances` | Search for instances in a series |

### Benefits

**Developer Friendly**: No DICOM expertise needed - use semantic method names like `.patientName()` instead of tag numbers

**Type Safe**: Full TypeScript support with auto-completion for all attributes

**Less Error Prone**: Can't use wrong tag numbers or VR codes - the builder handles it

**Maintainable**: Readable code that's easy to understand and modify

**DICOM Compliant**: Proper tags, VRs, and JSON format guaranteed

### Example: Before vs After

**Before (manual DICOM JSON):**
```javascript
// Complex, error-prone
return JSON.stringify([{
  "00100010": { "vr": "PN", "Value": ["Doe^John"] },
  "00100020": { "vr": "LO", "Value": ["12345"] },
  "0020000D": { "vr": "UI", "Value": ["1.2.3.4.5"] }
}]);
```

**After (high-level builders):**
```javascript
// Clean, readable, type-safe
const result = new QidoStudyResult();
result.patientName('Doe^John');
result.patientId('12345');
result.studyInstanceUid('1.2.3.4.5');
return createQidoStudiesResponse([result]);
```

### Demo

See `playground/demos/dicomweb-server-demo.mjs` for a complete working example with all four handlers.

```bash
# Run the demo
node playground/demos/dicomweb-server-demo.mjs

# Test queries
curl "http://localhost:8080/studies?PatientID=12345" | jq .
curl "http://localhost:8080/studies/1.2.840.../series" | jq .
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

All QIDO-RS responses follow proper DICOM JSON format (PS3.18 Section F.2):
- Correct tag numbering (e.g., "00100010")
- Proper Value Representations (VR)
- Value arrays (always arrays, even for single values)

## Future Enhancements

- [ ] Complete S3 storage backend for WADO-RS
- [ ] Add support for STOW-RS (Store over the Web)
- [ ] Add metadata-only retrieval endpoints
- [ ] Implement frame-level retrieval
- [ ] Add support for rendered media types
- [ ] Implement DICOM JSON metadata responses
- [ ] Add authentication and authorization
- [ ] Implement bulk data retrieval

## Complete Integration Example

```javascript
import { 
  QidoServer, 
  WadoServer,
  QidoStudyResult,
  QidoSeriesResult,
  createQidoStudiesResponse,
  createQidoSeriesResponse
} from 'node-dicom-rs';

// ==================== QIDO-RS Setup ====================
const qidoServer = new QidoServer(8080);

qidoServer.onSearchForStudies((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  // Query your database
  const studies = database.findStudies({
    patientId: query.patientId,
    studyDate: query.studyDate
  });
  
  // Build responses with high-level builders
  const results = studies.map(s => {
    const result = new QidoStudyResult();
    result.patientName(s.patientName);
    result.patientId(s.patientId);
    result.studyInstanceUid(s.studyUid);
    result.studyDate(s.studyDate);
    return result;
  });
  
  return createQidoStudiesResponse(results);
});

qidoServer.onSearchForSeries((err, query) => {
  if (err) return createQidoEmptyResponse();
  
  const series = database.getSeriesByStudy(query.studyInstanceUid);
  
  const results = series.map(s => {
    const result = new QidoSeriesResult();
    result.seriesInstanceUid(s.seriesUid);
    result.modality(s.modality);
    result.seriesNumber(s.seriesNumber);
    return result;
  });
  
  return createQidoSeriesResponse(results);
});

qidoServer.start();
console.log('QIDO-RS server listening on http://0.0.0.0:8080');

// ==================== WADO-RS Setup ====================
const wadoConfig = {
  storageType: 'filesystem',
  basePath: '/data/dicom'
};
const wadoServer = new WadoServer(8081, wadoConfig);
wadoServer.start();
console.log('WADO-RS server listening on http://0.0.0.0:8081');

// ==================== Use with OHIF Viewer ====================
// Configure your DICOM viewer to use:
// - QIDO URL: http://localhost:8080
// - WADO URL: http://localhost:8081

// Cleanup on shutdown
process.on('SIGINT', () => {
  console.log('Shutting down servers...');
  qidoServer.stop();
  wadoServer.stop();
  process.exit(0);
});
```

## Testing

### Test QIDO-RS

```bash
# Search for all studies
curl http://localhost:8080/studies | jq .

# Search with filters
curl "http://localhost:8080/studies?PatientID=12345&StudyDate=20240101" | jq .

# Get series in a study
curl "http://localhost:8080/studies/1.2.840.113619.2.55.3.604688119.868.1234567890.1/series" | jq .

# Get instances in a series
curl "http://localhost:8080/studies/1.2.840.../series/1.2.840.../instances" | jq .
```

### Test WADO-RS

```bash
# Retrieve a specific instance
curl http://localhost:8081/studies/1.2.3.4/series/5.6.7.8/instances/9.10.11.12 \
  -o instance.dcm

# Retrieve metadata
curl http://localhost:8081/studies/1.2.3.4/metadata | jq .
```

## See Also

- **Complete Demo:** `playground/demos/dicomweb-server-demo.mjs`
- **Simple Example:** `playground/demos/dicomweb-server-demo.mjs`
- **DICOM Standard:** [PS3.18 DICOMweb](https://www.dicomstandard.org/current)
