# DICOM Node Library Demos

This directory contains demonstration scripts showing various features of the node-dicom library.

## Demos

### DICOMweb Server Demo

#### `dicomweb-server-demo.mjs`
**Complete QIDO-RS server with high-level typed API** - This is the main demo showing how to build a production-ready QIDO-RS server.

**Features:**
- All 4 QIDO-RS query handlers (Studies, Series, Study Instances, Series Instances)
- High-level builders (`QidoStudyResult`, `QidoSeriesResult`, `QidoInstanceResult`)
- No DICOM tags - use typed methods like `.patientName()`, `.studyDate()`
- Mock database with realistic test data
- Complete filtering and pagination support
- Graceful shutdown handling

**Usage:**
```bash
node playground/demos/dicomweb-server-demo.mjs
```

**Test queries:**
```bash
# Search all studies
curl http://localhost:8080/studies | jq .

# Filter by PatientID
curl "http://localhost:8080/studies?PatientID=12345" | jq .

# Get series in study
curl "http://localhost:8080/studies/1.2.840.113619.2.55.3.604688119.868.1234567890.1/series" | jq .

# Get instances in series
curl "http://localhost:8080/studies/1.2.840.../series/1.2.840.../instances" | jq .
```

### Tag Update Demos

#### `tag-update-demo.mjs`
Demonstrates the `updateTags()` method for modifying DICOM tags in memory and saving to a new file.

**Features:**
- Basic tag updates (PatientName, StudyDescription)
- Anonymization example
- Multiple tag modifications at once

**Usage:**
```bash
node playground/demos/tag-update-demo.mjs
```

#### `verify-updates.mjs`
Validates that tag updates were correctly applied by reading the modified DICOM file.

**Usage:**
```bash
node playground/demos/verify-updates.mjs
```

### Pixel Processing Demos

#### `pixel-processing-demo.mjs`
Demonstrates advanced pixel data processing with the `getProcessedPixelData()` method.

**Features:**
- Frame extraction from multi-frame files
- Windowing with CT presets (lung, bone, soft tissue)
- 8-bit conversion
- Memory-efficient processing

**Usage:**
```bash
node playground/demos/pixel-processing-demo.mjs
```

### StoreScp Demos

#### `storescp-anonymization-demo.mjs`
Demonstrates real-time DICOM file anonymization using the `onBeforeStore` callback.

**Features:**
- Automatic patient de-identification
- Patient ID mapping (maintains consistency across files)
- Removes PHI (Protected Health Information)
- Modifies tags BEFORE file storage

**Use Cases:**
- Research databases requiring de-identified data
- Secondary PACS for teaching/training
- GDPR/HIPAA compliance workflows
- Multi-institutional data sharing

**Usage:**
```bash
# Terminal 1: Start the anonymizing SCP server
node playground/demos/storescp-anonymization-demo.mjs

# Terminal 2: Send DICOM files to be anonymized
node playground/send.mjs
```

**Example Output:**
```
📋 Received DICOM file - Anonymizing...
Original tags: {
  PatientName: 'Doe^John',
  PatientID: '12345',
  PatientBirthDate: '19800101',
  StudyDescription: 'CT Chest'
}
✅ Anonymized tags: {
  PatientName: 'ANONYMOUS^PATIENT',
  PatientID: 'ANON_1000',
  PatientBirthDate: '',
  StudyDescription: 'ANONYMIZED - CT Chest'
}
```

#### `storescp-validation-demo.mjs`
Demonstrates tag validation and enrichment using the `onBeforeStore` callback.

**Features:**
- Tag validation before storage
- Automatic metadata enrichment
- Tag standardization (e.g., uppercase names)
- Institution-specific defaults

**Use Cases:**
- Ensuring data quality in PACS
- Adding missing institutional metadata
- Standardizing tag formats
- Quality control workflows

**Usage:**
```bash
# Terminal 1: Start the validating SCP server
node playground/demos/storescp-validation-demo.mjs

# Terminal 2: Send DICOM files to be validated
node playground/send.mjs
```

## onBeforeStore Callback

The `onBeforeStore` callback is a powerful feature that allows you to modify DICOM tags **synchronously before files are saved**. This is different from event listeners which fire after the file is stored.

### Callback Signature

```typescript
scp.onBeforeStore((tags: Record<string, string>) => Record<string, string>)
```

### Key Characteristics

1. **Synchronous Execution**: The callback blocks file storage until it returns
2. **Tag Extraction Required**: Only works with tags specified in `extractTags`
3. **Immutable UIDs**: Best practice is to preserve Study/Series/SOP Instance UIDs
4. **Pre-Storage**: Modifications are applied BEFORE writing to disk
5. **Error Handling**: If callback fails, original tags are used

### Common Patterns

#### Pattern 1: Anonymization
```javascript
scp.onBeforeStore((tags) => {
  return {
    ...tags,
    PatientName: 'ANONYMOUS',
    PatientID: generateAnonymousID(),
    PatientBirthDate: ''
  };
});
```

#### Pattern 2: Validation
```javascript
scp.onBeforeStore((tags) => {
  if (!tags.PatientID || !tags.PatientName) {
    console.warn('Missing required tags!');
  }
  return tags;
});
```

#### Pattern 3: Enrichment
```javascript
scp.onBeforeStore((tags) => {
  return {
    ...tags,
    InstitutionName: 'My Hospital',
    StudyDescription: `[${tags.Modality}] ${tags.StudyDescription}`
  };
});
```

## Requirements

- Node.js 18+
- DICOM test files (use `downloadTestData.sh` to get sample data)

## Test Data

Download test DICOM files:
```bash
cd playground
./downloadTestData.sh
```

This will download sample DICOM studies to `playground/testdata/`.

## Notes

- All demos use ES modules (`.mjs` extension)
- Demos create output directories automatically
- Server demos listen on different ports to avoid conflicts
- Use Ctrl+C to stop server demos
