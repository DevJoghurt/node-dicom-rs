# StoreScp - DICOM C-STORE SCP Server

The `StoreScp` class implements a DICOM C-STORE Service Class Provider (SCP) server that receives DICOM files over the network.

## Basic Usage

```typescript
import { StoreScp } from '@nuxthealth/node-dicom';

const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './dicom-storage',
    verbose: true
});

receiver.addEventListener('OnFileStored', (data) => {
    console.log('File received:', data.file);
});

receiver.listen();
```

## Configuration Options

### Network Settings

```typescript
{
    port: 4446,                    // TCP port to listen on (required)
    callingAeTitle: 'STORE-SCP',   // Application Entity title (default: 'STORE-SCP')
    strict: false,                 // Enforce strict PDU length limits (default: false)
    maxPduLength: 16384            // Maximum PDU length in bytes (default: 16384)
}
```

### Storage Settings

```typescript
{
    outDir: './dicom-storage',     // Output directory for filesystem storage
    storageBackend: 'Filesystem',  // 'Filesystem' | 'S3' (default: 'Filesystem')
    storeWithFileMeta: false       // Include file meta header (default: false)
}
```

### Tag Extraction

```typescript
{
    extractTags: [                 // DICOM tags to extract from received files
        'PatientName',
        'PatientID',
        'StudyDate',
        'Modality',
        'SeriesDescription'
    ],
    extractCustomTags: [           // Custom/private tags with user-defined names
        { tag: '00091001', name: 'CustomField' }
    ],
    groupingStrategy: 'ByScope'    // 'ByScope' | 'Flat' | 'StudyLevel' | 'Custom'
}
```

### SOP Class Configuration

Control which types of DICOM objects your SCP accepts:

```typescript
import { getCommonSopClasses } from '@nuxthealth/node-dicom';

const sopClasses = getCommonSopClasses();

{
    abstractSyntaxMode: 'Custom',  // 'AllStorage' | 'All' | 'Custom'
    abstractSyntaxes: [            // When mode is 'Custom'
        ...sopClasses.ct,          // CT Image Storage
        ...sopClasses.mr,          // MR Image Storage
        ...sopClasses.ultrasound   // Ultrasound Image Storage
    ]
}
```

Available SOP Class categories:
- `ct` - CT imaging (2 classes)
- `mr` - MR imaging (2 classes)
- `ultrasound` - Ultrasound (2 classes)
- `pet` - PET imaging (3 classes)
- `xray` - X-Ray (3 classes)
- `mammography` - Mammography (5 classes)
- `secondaryCapture` - Secondary Capture (4 classes)
- `radiationTherapy` - RT objects (4 classes)
- `documents` - Encapsulated documents (4 classes)
- `structuredReports` - SR documents (3 classes)
- `allImaging` - All imaging types (17 classes)
- `all` - Everything (33 classes)

### Transfer Syntax Configuration

Control which transfer syntaxes (compression formats) are accepted:

```typescript
import { getCommonTransferSyntaxes } from '@nuxthealth/node-dicom';

const transferSyntaxes = getCommonTransferSyntaxes();

{
    transferSyntaxMode: 'Custom',  // 'All' | 'UncompressedOnly' | 'Custom'
    transferSyntaxes: [            // When mode is 'Custom'
        ...transferSyntaxes.uncompressed,  // Uncompressed formats
        ...transferSyntaxes.jpeg           // JPEG compression
    ]
}
```

Available transfer syntax categories:
- `uncompressed` - Implicit/Explicit VR Little Endian (3 syntaxes)
- `jpeg` - JPEG variants (4 syntaxes)
- `jpegLs` - JPEG-LS (2 syntaxes)
- `jpeg2000` - JPEG 2000 (2 syntaxes)
- `rle` - RLE Lossless (1 syntax)
- `mpeg` - MPEG video (4 syntaxes)
- `allCompressed` - All compressed (13 syntaxes)
- `all` - Everything (17 syntaxes)

### Study Completion

```typescript
{
    studyTimeout: 30  // Seconds to wait before OnStudyCompleted event (default: 30)
}
```

## Events

### OnServerStarted

Triggered when the server starts listening.

```typescript
receiver.addEventListener('OnServerStarted', (data) => {
    console.log(`Server started on port ${data.port}`);
});
```

### OnFileStored

Triggered when each DICOM file is received and stored.

```typescript
receiver.addEventListener('OnFileStored', (data) => {
    console.log('File:', data.file);
    console.log('SOP Instance UID:', data.sopInstanceUID);
    console.log('SOP Class UID:', data.sopClassUID);
    console.log('Transfer Syntax:', data.transferSyntaxUID);
    console.log('Study UID:', data.studyInstanceUID);
    console.log('Series UID:', data.seriesInstanceUID);
    console.log('Extracted tags:', data.tags);
});
```

Event data structure depends on `groupingStrategy`:

**ByScope** (default):
```typescript
{
    file: "path/to/file.dcm",
    sopInstanceUID: "1.2.3...",
    sopClassUID: "1.2.840...",
    transferSyntaxUID: "1.2.840...",
    studyInstanceUID: "1.2.3...",
    seriesInstanceUID: "1.2.3...",
    tags: {
        patient: {
            PatientName: "DOE^JOHN",
            PatientID: "12345"
        },
        study: {
            StudyDate: "20231201",
            StudyDescription: "CT Chest"
        },
        series: {
            Modality: "CT",
            SeriesDescription: "Chest with contrast"
        },
        instance: {
            InstanceNumber: "1",
            SliceThickness: "5.0"
        }
    }
}
```

**Flat**:
```typescript
{
    file: "path/to/file.dcm",
    // ... UIDs ...
    tags: {
        PatientName: "DOE^JOHN",
        PatientID: "12345",
        StudyDate: "20231201",
        Modality: "CT"
    }
}
```

**StudyLevel**:
```typescript
{
    file: "path/to/file.dcm",
    // ... UIDs ...
    tags: {
        study_level: {
            PatientName: "DOE^JOHN",
            StudyDate: "20231201"
        },
        instance_level: {
            Modality: "CT",
            InstanceNumber: "1"
        }
    }
}
```

### OnStudyCompleted

Triggered when no new files are received for a study after the timeout period.

```typescript
receiver.addEventListener('OnStudyCompleted', (study) => {
    console.log(`Study ${study.study_instance_uid} completed`);
    console.log(`${study.series.length} series`);
    
    for (const series of study.series) {
        console.log(`  Series ${series.series_instance_uid}: ${series.instances.length} instances`);
        
        for (const instance of series.instances) {
            console.log(`    ${instance.file}`);
        }
    }
});
```

Event data structure (with `ByScope` grouping):
```typescript
{
    study_instance_uid: "1.2.3...",
    tags: {                        // Study-level tags (Patient + Study)
        PatientName: "DOE^JOHN",
        StudyDate: "20231201"
    },
    series: [
        {
            series_instance_uid: "1.2.3...",
            tags: {                // Series-level tags
                Modality: "CT",
                SeriesDescription: "..."
            },
            instances: [
                {
                    sop_instance_uid: "1.2.3...",
                    sop_class_uid: "1.2.840...",
                    transfer_syntax_uid: "1.2.840...",
                    file: "path/to/file.dcm",
                    tags: {        // Instance-level tags
                        InstanceNumber: "1",
                        SliceThickness: "5.0"
                    }
                }
            ]
        }
    ]
}
```

**Note:** With `Flat` grouping, all data goes to `instance.tags` only. With `StudyLevel`, data is split between `study.tags` and `instance.tags`.

## Storage Backends

### Filesystem Storage

```typescript
const receiver = new StoreScp({
    port: 4446,
    storageBackend: 'Filesystem',
    outDir: './dicom-storage'
});
```

Files are stored in hierarchy: `{outDir}/{StudyInstanceUID}/{SeriesInstanceUID}/{SOPInstanceUID}.dcm`

### S3 Storage

```typescript
const receiver = new StoreScp({
    port: 4446,
    storageBackend: 'S3',
    s3Config: {
        bucket: 'dicom-archive',
        accessKey: 'YOUR_ACCESS_KEY',
        secretKey: 'YOUR_SECRET_KEY',
        endpoint: 'https://s3.amazonaws.com',  // Or MinIO/other S3-compatible
        region: 'us-east-1'                    // Optional
    }
});
```

Files are stored with the same path structure in the S3 bucket.

## Complete Example

```typescript
import { StoreScp, getCommonSopClasses, getCommonTransferSyntaxes } from '@nuxthealth/node-dicom';

const sopClasses = getCommonSopClasses();
const transferSyntaxes = getCommonTransferSyntaxes();

const receiver = new StoreScp({
    // Network
    port: 4446,
    callingAeTitle: 'HOSPITAL-SCP',
    maxPduLength: 32768,
    
    // Storage
    storageBackend: 'Filesystem',
    outDir: './dicom-archive',
    storeWithFileMeta: false,
    
    // Tag Extraction
    extractTags: [
        'PatientName', 'PatientID', 'PatientBirthDate',
        'StudyDate', 'StudyDescription', 'AccessionNumber',
        'Modality', 'SeriesDescription', 'SeriesNumber',
        'InstanceNumber', 'SliceThickness'
    ],
    groupingStrategy: 'ByScope',
    
    // SOP Classes (only accept CT and MR)
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: [...sopClasses.ct, ...sopClasses.mr],
    
    // Transfer Syntaxes (accept all)
    transferSyntaxMode: 'All',
    
    // Study completion
    studyTimeout: 60,
    
    verbose: true
});

receiver.addEventListener('OnServerStarted', (data) => {
    console.log(`✓ SCP Server listening on port ${data.port}`);
});

receiver.addEventListener('OnFileStored', (data) => {
    console.log(`✓ Received: ${data.file}`);
    console.log(`  Patient: ${data.tags.patient?.PatientName}`);
    console.log(`  Study: ${data.tags.study?.StudyDescription}`);
});

receiver.addEventListener('OnStudyCompleted', (study) => {
    const totalInstances = study.series.reduce((sum, s) => sum + s.instances.length, 0);
    console.log(`✓ Study ${study.study_instance_uid} completed`);
    console.log(`  ${study.series.length} series, ${totalInstances} instances`);
});

receiver.listen();
```

## Tips

1. **Choose the right grouping strategy**: Use `ByScope` for proper hierarchy, `Flat` for simple key-value extraction
2. **Configure SOP classes**: Limit to only the modalities you need for better control
3. **Set appropriate timeout**: Adjust `studyTimeout` based on your typical scan duration
4. **Use S3 for production**: Filesystem is good for development, S3 for scalable storage
5. **Extract only needed tags**: Don't extract unnecessary tags to reduce memory usage
