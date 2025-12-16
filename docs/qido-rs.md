# QIDO-RS - Query based on ID for DICOM Objects

QIDO-RS is the query service of DICOMweb, providing RESTful HTTP endpoints for searching DICOM studies, series, and instances. This implementation follows the [DICOM Part 18](http://dicom.nema.org/medical/dicom/current/output/html/part18.html) specification.

## Overview

The QIDO-RS server provides a **high-level, developer-friendly API** that completely hides DICOM JSON complexity. You never need to deal with DICOM tag numbers or VR codes!

### Key Features

✅ **No DICOM Tags** - Use typed methods like `.patientName()`, `.studyDate()`  
✅ **No JSON Complexity** - Builders handle DICOM JSON format automatically  
✅ **Full Type Safety** - TypeScript auto-completion for all attributes  
✅ **DICOM Compliant** - Automatic tag numbers, VR codes, proper formatting  
✅ **Separate Handlers** - One handler per query level (Studies, Series, Instances)  
✅ **CORS Support** - Built-in CORS configuration for web applications

## Quick Start

```typescript
import { 
  QidoServer, 
  QidoStudyResult, 
  createQidoStudiesResponse,
  createQidoEmptyResponse
} from '@nuxthealth/node-dicom';

const server = new QidoServer(8042, {
  enableCors: true,
  corsAllowedOrigins: 'http://localhost:3000',
  verbose: true
});

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

## Configuration

```typescript
const config = {
    // CORS Configuration
    enableCors?: boolean,              // Enable CORS headers (default: false)
    corsAllowedOrigins?: string,       // Comma-separated list of origins
    
    // Logging
    verbose?: boolean,                 // Enable verbose logging (default: false)
};

const server = new QidoServer(port, config);
```

### CORS Configuration

CORS (Cross-Origin Resource Sharing) allows web applications from different origins to access the QIDO-RS server.

**When to Enable CORS:**
- Web-based DICOM viewers (OHIF, Cornerstone-based apps)
- Single-page applications (SPAs) accessing PACS from different domain
- Development environments (frontend: localhost:3000, backend: localhost:8042)
- Mobile apps using WebView

**Production Security:**
```typescript
// ✅ GOOD: Specific origins in production
const qido = new QidoServer(8042, {
    enableCors: true,
    corsAllowedOrigins: 'https://viewer.hospital.com,https://app.hospital.com',
    verbose: false
});

// ❌ BAD: Allow all origins in production
const qido = new QidoServer(8042, {
    enableCors: true,
    // No corsAllowedOrigins = allows all origins (*)
    verbose: false
});
```

**Development Setup:**
```typescript
// Development: Allow localhost
const qido = new QidoServer(8042, {
    enableCors: true,
    corsAllowedOrigins: 'http://localhost:3000,http://localhost:5173',
    verbose: true
});
```

## Builder Classes

The API provides three builder classes for constructing DICOM responses without dealing with tag numbers.

### QidoStudyResult

Builder for Study-level responses (GET /studies).

**Patient Module:**
```typescript
const study = new QidoStudyResult();
study.patientName('Doe^John');           // (0010,0010) PN
study.patientId('12345');                 // (0010,0020) LO
study.patientBirthDate('19800101');       // (0010,0030) DA
study.patientSex('M');                    // (0010,0040) CS
```

**Study Module:**
```typescript
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

### QidoSeriesResult

Builder for Series-level responses (GET /studies/{uid}/series).

```typescript
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

### QidoInstanceResult

Builder for Instance-level responses (GET /studies/{uid}/instances, GET /studies/{uid}/series/{uid}/instances).

```typescript
const instance = new QidoInstanceResult();
instance.sopInstanceUid('1.2.3.4.5.1.1'); // (0008,0018) UI
instance.sopClassUid('1.2.840...');        // (0008,0016) UI
instance.instanceNumber('1');              // (0020,0013) IS
instance.rows('512');                      // (0028,0010) US
instance.columns('512');                   // (0028,0011) US
instance.bitsAllocated('16');              // (0028,0100) US
instance.numberOfFrames('1');              // (0028,0008) IS
```

## Response Helper Functions

```typescript
// Convert builder arrays to DICOM JSON strings
createQidoStudiesResponse(results: QidoStudyResult[]): string
createQidoSeriesResponse(results: QidoSeriesResult[]): string
createQidoInstancesResponse(results: QidoInstanceResult[]): string
createQidoEmptyResponse(): string  // Returns "[]" for error cases
```

## Handler Registration

The QIDO-RS API provides four separate handlers, one for each query level defined in DICOM PS3.18:

```typescript
import { 
  QidoServer,
  QidoStudyResult,
  QidoSeriesResult,
  QidoInstanceResult,
  createQidoStudiesResponse,
  createQidoSeriesResponse,
  createQidoInstancesResponse,
  createQidoEmptyResponse
} from '@nuxthealth/node-dicom';

const server = new QidoServer(8042, {
  enableCors: true,
  corsAllowedOrigins: 'http://localhost:3000',
  verbose: true
});

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

## Query Parameters

All query handlers receive a typed query object with standard DICOM parameters:

### SearchForStudiesQuery

```typescript
interface SearchForStudiesQuery {
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

### SearchForSeriesQuery

```typescript
interface SearchForSeriesQuery {
  studyInstanceUid: string;  // From URL path (required)
  
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

### SearchForStudyInstancesQuery

```typescript
interface SearchForStudyInstancesQuery {
  studyInstanceUid: string;  // From URL path (required)
  
  limit?: number;
  offset?: number;
  fuzzymatching?: boolean;
  includefield?: string;
  
  sopClassUid?: string;
  sopInstanceUid?: string;
  instanceNumber?: string;
}
```

### SearchForSeriesInstancesQuery

```typescript
interface SearchForSeriesInstancesQuery {
  studyInstanceUid: string;       // From URL path (required)
  seriesInstanceUid: string;      // From URL path (required)
  
  limit?: number;
  offset?: number;
  fuzzymatching?: boolean;
  includefield?: string;
  
  sopClassUid?: string;
  sopInstanceUid?: string;
  instanceNumber?: string;
}
```

## QIDO-RS Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/studies` | `onSearchForStudies` | Search for studies |
| GET | `/studies/{uid}/series` | `onSearchForSeries` | Search for series in a study |
| GET | `/studies/{uid}/instances` | `onSearchForStudyInstances` | Search for instances in a study |
| GET | `/studies/{uid}/series/{uid}/instances` | `onSearchForSeriesInstances` | Search for instances in a series |

## Complete Example

```typescript
import { 
  QidoServer,
  QidoStudyResult,
  QidoSeriesResult,
  QidoInstanceResult,
  createQidoStudiesResponse,
  createQidoSeriesResponse,
  createQidoInstancesResponse,
  createQidoEmptyResponse
} from '@nuxthealth/node-dicom';

// Mock database (replace with your actual database)
const database = {
  findStudies: (query) => [
    {
      patientName: 'Doe^John',
      patientId: '12345',
      studyUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      studyDate: '20240101',
      studyTime: '120000',
      accessionNumber: 'ACC001',
      studyDescription: 'CT Chest',
      modalitiesInStudy: 'CT',
      numberOfSeries: '5',
      numberOfInstances: '100'
    }
  ],
  
  getSeriesByStudy: (studyUid) => [
    {
      seriesUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.2',
      modality: 'CT',
      seriesNumber: '1',
      seriesDescription: 'Chest Axial',
      numberOfInstances: '20'
    }
  ],
  
  getInstancesByStudy: (studyUid) => [
    {
      sopInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.3',
      sopClassUid: '1.2.840.10008.5.1.4.1.1.2',
      instanceNumber: '1'
    }
  ],
  
  getInstancesBySeries: (studyUid, seriesUid) => [
    {
      sopInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.3',
      sopClassUid: '1.2.840.10008.5.1.4.1.1.2',
      instanceNumber: '1',
      rows: '512',
      columns: '512'
    }
  ]
};

// Create QIDO-RS server
const qido = new QidoServer(8042, {
  enableCors: true,
  corsAllowedOrigins: 'http://localhost:3000',
  verbose: true
});

// Register handlers
qido.onSearchForStudies((err, query) => {
  if (err) {
    console.error('Search for studies error:', err);
    return createQidoEmptyResponse();
  }
  
  console.log('Searching studies with query:', query);
  const studies = database.findStudies(query);
  
  const results = studies.map(s => {
    const result = new QidoStudyResult();
    result.patientName(s.patientName);
    result.patientId(s.patientId);
    result.studyInstanceUid(s.studyUid);
    result.studyDate(s.studyDate);
    result.studyTime(s.studyTime);
    result.accessionNumber(s.accessionNumber);
    result.studyDescription(s.studyDescription);
    result.modalitiesInStudy(s.modalitiesInStudy);
    result.numberOfStudyRelatedSeries(s.numberOfSeries);
    result.numberOfStudyRelatedInstances(s.numberOfInstances);
    return result;
  });
  
  return createQidoStudiesResponse(results);
});

qido.onSearchForSeries((err, query) => {
  if (err) {
    console.error('Search for series error:', err);
    return createQidoEmptyResponse();
  }
  
  console.log('Searching series for study:', query.studyInstanceUid);
  const series = database.getSeriesByStudy(query.studyInstanceUid);
  
  const results = series.map(s => {
    const result = new QidoSeriesResult();
    result.seriesInstanceUid(s.seriesUid);
    result.modality(s.modality);
    result.seriesNumber(s.seriesNumber);
    result.seriesDescription(s.seriesDescription);
    result.numberOfSeriesRelatedInstances(s.numberOfInstances);
    return result;
  });
  
  return createQidoSeriesResponse(results);
});

qido.onSearchForStudyInstances((err, query) => {
  if (err) {
    console.error('Search for study instances error:', err);
    return createQidoEmptyResponse();
  }
  
  console.log('Searching instances for study:', query.studyInstanceUid);
  const instances = database.getInstancesByStudy(query.studyInstanceUid);
  
  const results = instances.map(i => {
    const result = new QidoInstanceResult();
    result.sopInstanceUid(i.sopInstanceUid);
    result.sopClassUid(i.sopClassUid);
    result.instanceNumber(i.instanceNumber);
    return result;
  });
  
  return createQidoInstancesResponse(results);
});

qido.onSearchForSeriesInstances((err, query) => {
  if (err) {
    console.error('Search for series instances error:', err);
    return createQidoEmptyResponse();
  }
  
  console.log('Searching instances for series:', query.seriesInstanceUid);
  const instances = database.getInstancesBySeries(
    query.studyInstanceUid,
    query.seriesInstanceUid
  );
  
  const results = instances.map(i => {
    const result = new QidoInstanceResult();
    result.sopInstanceUid(i.sopInstanceUid);
    result.sopClassUid(i.sopClassUid);
    result.instanceNumber(i.instanceNumber);
    result.rows(i.rows);
    result.columns(i.columns);
    return result;
  });
  
  return createQidoInstancesResponse(results);
});

// Start server
qido.start();
console.log('QIDO-RS server listening on http://0.0.0.0:8042');

// Cleanup on shutdown
process.on('SIGINT', () => {
  console.log('Shutting down QIDO-RS server...');
  qido.stop();
  process.exit(0);
});
```

## Benefits

**Developer Friendly**: No DICOM expertise needed - use semantic method names like `.patientName()` instead of tag numbers

**Type Safe**: Full TypeScript support with auto-completion for all attributes

**Less Error Prone**: Can't use wrong tag numbers or VR codes - the builder handles it

**Maintainable**: Readable code that's easy to understand and modify

**DICOM Compliant**: Proper tags, VRs, and JSON format guaranteed

## Example: Before vs After

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

## Testing

### Using curl

```bash
# Search for all studies
curl "http://localhost:8042/studies" | jq .

# Search with filters
curl "http://localhost:8042/studies?PatientID=12345&StudyDate=20240101" | jq .

# Get series in a study
curl "http://localhost:8042/studies/1.2.840.113619.2.55.3.604688119.868.1234567890.1/series" | jq .

# Get instances in a series
curl "http://localhost:8042/studies/1.2.840.../series/1.2.840.../instances" | jq .
```

### Test CORS

```bash
# Test preflight OPTIONS request
curl -X OPTIONS \
     -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: Content-Type" \
     -v http://localhost:8042/studies

# Test actual GET request with CORS
curl -H "Origin: http://localhost:3000" \
     -v http://localhost:8042/studies?limit=10

# Check for Access-Control-Allow-Origin header in response
```

### Using Browser

```javascript
// This will fail if CORS is not properly configured
fetch('http://localhost:8042/studies?limit=10')
  .then(res => res.json())
  .then(data => console.log('QIDO Studies:', data))
  .catch(err => console.error('CORS Error:', err));
```

## Integration with OHIF Viewer

The QIDO-RS server can be integrated with OHIF Viewer for medical image viewing:

```typescript
// QIDO server for OHIF
const qido = new QidoServer(8042, {
  enableCors: true,
  corsAllowedOrigins: 'http://localhost:3000',  // OHIF dev server
  verbose: true
});

// Register handlers to query your PACS/database
qido.onSearchForStudies((err, query) => {
  if (err) return createQidoEmptyResponse();
  const studies = pacsDatabase.searchStudies(query);
  return createQidoStudiesResponse(studies);
});

qido.start();
```

**OHIF Configuration:**
```json
{
  "dataSources": [
    {
      "namespace": "@ohif/extension-default.dataSourcesModule.dicomweb",
      "sourceName": "dicomweb",
      "configuration": {
        "friendlyName": "Hospital PACS",
        "name": "PACS",
        "wadoUriRoot": "http://localhost:8043",
        "qidoRoot": "http://localhost:8042",
        "wadoRoot": "http://localhost:8043",
        "qidoSupportsIncludeField": true,
        "supportsReject": false
      }
    }
  ]
}
```

## Troubleshooting

### CORS Issues

**Error: "No 'Access-Control-Allow-Origin' header"**

Enable CORS:
```typescript
const qido = new QidoServer(8042, {
  enableCors: true
});
```

**Error: "Origin not allowed"**

Add your origin to allowed list:
```typescript
const qido = new QidoServer(8042, {
  enableCors: true,
  corsAllowedOrigins: 'http://localhost:3000'
});
```

### Empty Results

Check handler registration and database queries:
```typescript
qido.onSearchForStudies((err, query) => {
  console.log('Query received:', query);  // Debug
  const results = database.findStudies(query);
  console.log('Results found:', results.length);  // Debug
  return createQidoStudiesResponse(results);
});
```

## Architecture

```
┌─────────────────┐
│   Node.js App   │
│  (Your Code)    │
└────────┬────────┘
         │
         │ Handlers
         ▼
┌─────────────────┐
│  QIDO-RS Server │
│   (Rust/Warp)   │
└────────┬────────┘
         │
         ├─────────────┐
         │             │
         ▼             ▼
┌─────────────┐  ┌──────────┐
│  Database   │  │   PACS   │
│   Query     │  │  Query   │
└─────────────┘  └──────────┘
```

## See Also

- **WADO-RS Documentation**: `docs/wado-rs.md` - Retrieval service
- **StoreSCP Documentation**: `docs/storescp.md` - DICOM C-STORE receiver
- **DICOM Standard**: [PS3.18 DICOMweb](https://dicom.nema.org/medical/dicom/current/output/html/part18.html)
- **OHIF Viewer**: [https://ohif.org](https://ohif.org)
