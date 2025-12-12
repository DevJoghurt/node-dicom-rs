# Tag Extraction Guide

Complete guide to extracting DICOM metadata tags across all components of node-dicom-rs.

## Table of Contents

- [Overview](#overview)
- [DicomFile Tag Extraction](#dicomfile-tag-extraction)
- [StoreScp Tag Extraction](#storescp-tag-extraction)
- [Tag Name Formats](#tag-name-formats)
- [Custom Tags for Private Data](#custom-tags-for-private-data)
- [Common Tag Sets](#common-tag-sets)
- [Best Practices](#best-practices)

## Overview

Tag extraction allows you to retrieve specific DICOM metadata from files. The node-dicom-rs library provides consistent, type-safe tag extraction across three main components:

- **DicomFile**: Extract tags from files on disk or S3
- **StoreScp**: Extract tags when receiving files over the network
- **StoreScu**: Access file metadata before sending

All extraction methods return **flat key-value structures** for simple, direct access to tag values.

## DicomFile Tag Extraction

### Basic Extraction

Extract specific tags from a DICOM file:

```typescript
import { DicomFile } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./scan.dcm');

// Extract tags - always returns flat structure
const data = file.extract(['PatientName', 'StudyDate', 'Modality']);

console.log('Patient:', data.PatientName);     // "DOE^JOHN"
console.log('Study Date:', data.StudyDate);    // "20240101"
console.log('Modality:', data.Modality);       // "CT"

file.close();
```

### Multiple Tags

Extract many tags at once:

```typescript
const data = file.extract([
    'PatientName',
    'PatientID',
    'PatientBirthDate',
    'PatientSex',
    'StudyInstanceUID',
    'StudyDate',
    'StudyDescription',
    'Modality',
    'SeriesDescription',
    'SeriesNumber',
    'InstanceNumber',
    'SOPInstanceUID'
]);

// All tags in flat structure: { PatientName: "...", PatientID: "...", ... }
```

### Using Predefined Tag Sets

Instead of manually listing tags, use predefined tag sets:

```typescript
import { DicomFile, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./scan.dcm');

const tagSets = getCommonTagSets();

// Extract patient demographics only
const patientData = file.extract(tagSets.patientBasic);
// { PatientName: "...", PatientID: "...", PatientBirthDate: "...", ... }

// Extract study information
const studyData = file.extract(tagSets.studyBasic);

// Extract comprehensive metadata
const allData = file.extract(tagSets.default);
// 42 common tags covering patient, study, series, instance, pixel info, and equipment

file.close();
```

### Modality-Specific Extraction

Extract modality-specific parameters:

```typescript
const tagSets = getCommonTagSets();

// CT-specific tags
const ctTags = combineTags([
    tagSets.default,
    tagSets.ct
]);
const ctData = file.extract(ctTags);
console.log('kVp:', ctData.KVP);
console.log('Kernel:', ctData.ConvolutionKernel);

// MR-specific tags
const mrTags = combineTags([
    tagSets.default,
    tagSets.mr
]);
const mrData = file.extract(mrTags);
console.log('TR:', mrData.RepetitionTime);
console.log('TE:', mrData.EchoTime);
console.log('Field Strength:', mrData.MagneticFieldStrength);

// Ultrasound-specific tags
const usTags = combineTags([
    tagSets.default,
    tagSets.ultrasound
]);
const usData = file.extract(usTags);
console.log('Transducer:', usData.TransducerType);
console.log('Frequency:', usData.TransducerFrequency);

// PET/Nuclear Medicine
const petTags = combineTags([
    tagSets.default,
    tagSets.petNm
]);
const petData = file.extract(petTags);
console.log('Units:', petData.Units);
console.log('Decay Correction:', petData.DecayCorrection);
```

### S3 File Extraction

Extract tags from DICOM files stored in S3:

```typescript
const file = new DicomFile();

// Open S3 file
file.openS3({
    bucket: 'dicom-archive',
    key: 'studies/2024/CT-12345.dcm',
    accessKey: 'YOUR_KEY',
    secretKey: 'YOUR_SECRET',
    endpoint: 'https://s3.amazonaws.com',
    region: 'us-east-1'
});

// Extract tags (same API as local files)
const data = file.extract(tagSets.default);

file.close();
```

## StoreScp Tag Extraction

StoreScp automatically extracts tags as files are received over the network.

### OnFileStored Event - Flat Tags

Extract tags for each received file:

```typescript
import { StoreScp, getCommonTagSets } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();

const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './received',
    verbose: true,
    // Specify tags to extract
    extractTags: tagSets.default,
    extractCustomTags: [] // Optional: custom tags
});

receiver.onFileStored((err, event) => {
    if (err) return console.error('Error:', err);
    
    const data = event.data;
    if (!data || !data.tags) return;
    
    // Tags are flat key-value pairs
    console.log('Received file:', data.file);
    console.log('Patient:', data.tags.PatientName);
    console.log('Study:', data.tags.StudyDescription);
    console.log('Modality:', data.tags.Modality);
    console.log('Instance:', data.tags.InstanceNumber);
    
    // Direct access to any extracted tag
    if (data.tags.SeriesDescription) {
        console.log('Series:', data.tags.SeriesDescription);
    }
});

receiver.listen();
```

### OnStudyCompleted Event - Hierarchical Tags

When a study is complete, get hierarchical organization with flat tags at each level:

```typescript
receiver.onStudyCompleted((err, event) => {
    if (err) return console.error('Error:', err);
    
    const study = event.data?.study;
    if (!study) return;
    
    console.log('Study completed:', study.studyInstanceUid);
    console.log('Total series:', study.series.length);
    
    // Study-level tags (Patient + Study scope)
    if (study.tags) {
        console.log('Patient Name:', study.tags.PatientName);
        console.log('Study Date:', study.tags.StudyDate);
        console.log('Study Description:', study.tags.StudyDescription);
    }
    
    // Series-level tags
    study.series.forEach(series => {
        console.log(`Series ${series.seriesInstanceUid}:`);
        console.log(`  Modality: ${series.tags?.Modality}`);
        console.log(`  Series Description: ${series.tags?.SeriesDescription}`);
        console.log(`  Instances: ${series.instances.length}`);
        
        // Instance-level tags
        series.instances.forEach(instance => {
            console.log(`    Instance ${instance.sopInstanceUid}`);
            console.log(`      Number: ${instance.tags?.InstanceNumber}`);
            console.log(`      File: ${instance.file}`);
            // Access any extracted tag at instance level
            if (instance.tags?.ImageType) {
                console.log(`      Type: ${instance.tags.ImageType}`);
            }
        });
    });
});
```

### Modality-Specific SCP Configuration

Configure SCP to extract modality-specific tags:

```typescript
const tagSets = getCommonTagSets();

// CT receiver - extract CT-specific parameters
const ctReceiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'CT-SCP',
    outDir: './ct-storage',
    extractTags: combineTags([
        tagSets.default,
        tagSets.ct
    ])
});

ctReceiver.onFileStored((err, event) => {
    const tags = event.data?.tags;
    if (tags) {
        console.log('CT Parameters:');
        console.log('  kVp:', tags.KVP);
        console.log('  Tube Current:', tags.XRayTubeCurrent);
        console.log('  Kernel:', tags.ConvolutionKernel);
        console.log('  Slice Thickness:', tags.SliceThickness);
    }
});

// MR receiver - extract MR-specific parameters
const mrReceiver = new StoreScp({
    port: 4447,
    callingAeTitle: 'MR-SCP',
    outDir: './mr-storage',
    extractTags: combineTags([
        tagSets.default,
        tagSets.mr
    ])
});

mrReceiver.onFileStored((err, event) => {
    const tags = event.data?.tags;
    if (tags) {
        console.log('MR Parameters:');
        console.log('  TR:', tags.RepetitionTime);
        console.log('  TE:', tags.EchoTime);
        console.log('  Field Strength:', tags.MagneticFieldStrength);
        console.log('  Flip Angle:', tags.FlipAngle);
    }
});
```

## Tag Name Formats

Tags can be specified in multiple formats:

### Standard Names

Use standard DICOM tag names (recommended):

```typescript
file.extract([
    'PatientName',
    'StudyDate',
    'Modality',
    'SeriesDescription'
]);
```

Benefits:
- Full TypeScript autocomplete support (300+ tags)
- Clear and readable
- IDE validation

### Hex Format

Use 8-digit hex format (without separators):

```typescript
file.extract([
    '00100010',  // PatientName
    '00080020',  // StudyDate
    '00080060',  // Modality
    '0008103E'   // SeriesDescription
]);
```

### Parenthesized Format

Use (GGGG,EEEE) format:

```typescript
file.extract([
    '(0010,0010)',  // PatientName
    '(0008,0020)',  // StudyDate
    '(0008,0060)',  // Modality
    '(0008,103E)'   // SeriesDescription
]);
```

### Mixed Formats

All formats can be mixed:

```typescript
file.extract([
    'PatientName',      // Standard name
    '00080020',         // Hex
    '(0008,0060)',      // Parenthesized
    'SeriesDescription' // Standard name
]);
```

## Custom Tags for Private Data

Extract private or vendor-specific tags using custom tag definitions.

### Creating Custom Tags

```typescript
import { DicomFile, createCustomTag } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./scan-with-private-tags.dcm');

// Define custom tags
const customTags = [
    createCustomTag('00091001', 'VendorID'),
    createCustomTag('00091010', 'ScannerMode'),
    createCustomTag('00431001', 'ImageQuality')
];

// Extract with custom tags
const data = file.extract(
    ['PatientName', 'StudyDate', 'Modality'],
    customTags
);
console.log('Standard tags:', data.PatientName, data.StudyDate);
console.log('Custom tags:', data.VendorID, data.ScannerMode);

file.close();
```

### Vendor-Specific Tags

Create libraries of vendor-specific tags:

```typescript
// GE private tags
const geTags = [
    createCustomTag('00091001', 'GE_PrivateCreator'),
    createCustomTag('00091027', 'GE_ScanOptions'),
    createCustomTag('00431001', 'GE_ImageFiltering')
];

// Siemens private tags
const siemensTags = [
    createCustomTag('00191008', 'Siemens_ImagingMode'),
    createCustomTag('00191009', 'Siemens_SequenceInfo'),
    createCustomTag('0029100C', 'Siemens_CoilString')
];

// Philips private tags
const philipsTags = [
    createCustomTag('20011001', 'Philips_ScanMode'),
    createCustomTag('20011003', 'Philips_Contrast'),
    createCustomTag('20051080', 'Philips_ReconParams')
];

// Dynamic selection based on manufacturer
const manufacturerData = file.extract(['Manufacturer']);
const manufacturer = manufacturerData.Manufacturer;

const vendorTags = {
    'GE': geTags,
    'SIEMENS': siemensTags,
    'Philips': philipsTags
}[manufacturer] || [];

const data = file.extract(tagSets.default, vendorTags);
```

### StoreScp with Custom Tags

Extract custom tags in StoreScp:

```typescript
const customTags = [
    createCustomTag('00091001', 'VendorID'),
    createCustomTag('00431001', 'ProcessingFlag')
];

const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './received',
    extractTags: tagSets.default,
    extractCustomTags: customTags
});

receiver.onFileStored((err, event) => {
    const tags = event.data?.tags;
    if (tags) {
        console.log('Standard:', tags.PatientName);
        console.log('Custom:', tags.VendorID, tags.ProcessingFlag);
    }
});
```

## Common Tag Sets

The library provides predefined tag sets for common use cases.

### Available Tag Sets

```typescript
import { getCommonTagSets } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();

// Patient demographics (7 tags)
tagSets.patientBasic
// ['PatientName', 'PatientID', 'PatientBirthDate', 'PatientSex', ...]

// Study metadata (7 tags)
tagSets.studyBasic
// ['StudyInstanceUID', 'StudyDate', 'StudyTime', 'StudyDescription', ...]

// Series metadata (8 tags)
tagSets.seriesBasic
// ['SeriesInstanceUID', 'SeriesNumber', 'SeriesDescription', 'Modality', ...]

// Instance identifiers (5 tags)
tagSets.instanceBasic
// ['SOPInstanceUID', 'SOPClassUID', 'InstanceNumber', ...]

// Image pixel info (9 tags)
tagSets.imagePixelInfo
// ['Rows', 'Columns', 'BitsAllocated', 'PixelSpacing', ...]

// Equipment info (6 tags)
tagSets.equipment
// ['Manufacturer', 'ManufacturerModelName', 'DeviceSerialNumber', ...]

// Modality-specific sets
tagSets.ct          // CT parameters (6 tags)
tagSets.mr          // MR parameters (6 tags)
tagSets.ultrasound  // Ultrasound parameters (6 tags)
tagSets.petNm       // PET/NM parameters (11 tags)
tagSets.xa          // X-Ray Angiography (8 tags)
tagSets.rt          // Radiation Therapy (11 tags)

// Comprehensive default (42 tags)
tagSets.default
// Combines: patient, study, series, instance, pixel info, equipment
```

### Combining Tag Sets

```typescript
import { combineTags } from '@nuxthealth/node-dicom';

// Combine multiple sets
const comprehensiveTags = combineTags([
    tagSets.patientBasic,
    tagSets.studyBasic,
    tagSets.seriesBasic,
    tagSets.imagePixelInfo,
    tagSets.ct
]);

// Add custom tags to predefined sets
const customWorkflow = combineTags([
    tagSets.default,
    ['WindowCenter', 'WindowWidth', 'RescaleIntercept', 'RescaleSlope'],
    tagSets.ct
]);

// No duplicates - combineTags deduplicates automatically
const tags = file.extract(comprehensiveTags);
```

## Best Practices

### 1. Use Predefined Tag Sets

Prefer predefined tag sets over manual lists:

```typescript
// Good ✓
const tags = file.extract(tagSets.default);

// Avoid ✗
const tags = file.extract([
    'PatientName', 'PatientID', 'PatientBirthDate', // ... 40 more tags
]);
```

### 2. Extract Only What You Need

Don't extract all tags if you only need a few:

```typescript
// Minimal extraction
const basicInfo = file.extract(['PatientName', 'StudyDate', 'Modality']);

// Better performance and clearer intent
```

### 3. Handle Missing Tags

Not all files contain all tags:

```typescript
const data = file.extract(tagSets.default);

// Safe access
if (data.PatientName) {
    console.log('Patient:', data.PatientName);
}

// With defaults
const patientName = data.PatientName || 'Unknown';
const studyDescription = data.StudyDescription || 'No description';
```

### 4. Use TypeScript for Tag Safety

TypeScript provides autocomplete and validation:

```typescript
// TypeScript knows about 300+ tags
const tags = file.extract([
    'PatientName',  // ✓ Autocomplete suggests this
    'StudyDate',    // ✓ Validated
    'InvalidTag'    // ✗ TypeScript warning (still allowed as string)
]);
```

### 5. Reuse Tag Sets

Create reusable tag configurations:

```typescript
// config.ts
export const ANONYMIZATION_TAGS = combineTags([
    tagSets.studyBasic,
    tagSets.seriesBasic,
    tagSets.instanceBasic,
    // Exclude patient demographics
]);

export const ROUTING_TAGS = [
    'StudyInstanceUID',
    'SeriesInstanceUID',
    'SOPInstanceUID',
    'Modality',
    'StationName'
];

export const QUALITY_CHECK_TAGS = combineTags([
    tagSets.imagePixelInfo,
    ['WindowCenter', 'WindowWidth', 'ImageType']
]);

// Use across your application
const data = file.extract(QUALITY_CHECK_TAGS);
```

### 6. Modality-Aware Extraction

Extract based on modality:

```typescript
// Detect modality first
const basicData = file.extract(['Modality']);
const modality = basicData.Modality;

// Extract modality-specific tags
let modalityTags = tagSets.default;

switch (modality) {
    case 'CT':
        modalityTags = combineTags([tagSets.default, tagSets.ct]);
        break;
    case 'MR':
        modalityTags = combineTags([tagSets.default, tagSets.mr]);
        break;
    case 'US':
        modalityTags = combineTags([tagSets.default, tagSets.ultrasound]);
        break;
    case 'PT':
        modalityTags = combineTags([tagSets.default, tagSets.petNm]);
        break;
}

const fullData = file.extract(modalityTags);
```

### 7. Batch Processing

Process multiple files efficiently:

```typescript
import { readdirSync } from 'fs';
import { join } from 'path';

const dicomDir = './dicom-files';
const files = readdirSync(dicomDir).filter(f => f.endsWith('.dcm'));

const file = new DicomFile();
const results = [];

for (const filename of files) {
    try {
        file.open(join(dicomDir, filename));
        const data = file.extract(tagSets.default);
        results.push({
            file: filename,
            data: data
        });
        file.close();
    } catch (error) {
        console.error(`Error processing ${filename}:`, error);
    }
}

console.log(`Processed ${results.length} files`);
```

### 8. Validation and Error Handling

Always validate extracted data:

```typescript
function validateStudyData(data: any): boolean {
    const required = ['PatientID', 'StudyInstanceUID', 'Modality'];
    return required.every(tag => data[tag] != null);
}

try {
    const data = file.extract(tagSets.default);
    
    if (!validateStudyData(data)) {
        console.error('Missing required tags');
        return;
    }
    
    // Process valid data
    processStudy(data);
    
} catch (error) {
    console.error('Extraction failed:', error);
}
```

### 9. Performance Considerations

- Extract tags once and reuse the data
- Don't extract tags in tight loops
- Close files after use to free resources

```typescript
// Good ✓
const data = file.extract(tagSets.default);
for (let i = 0; i < 1000; i++) {
    processPatient(data.PatientName);
}

// Bad ✗
for (let i = 0; i < 1000; i++) {
    const data = file.extract(['PatientName']);
    processPatient(data.PatientName);
}
```

### 10. Logging and Debugging

Log extracted tags for debugging:

```typescript
const data = file.extract(tagSets.default);

// Structured logging
console.log('Extracted tags:', {
    patient: {
        name: data.PatientName,
        id: data.PatientID
    },
    study: {
        uid: data.StudyInstanceUID,
        date: data.StudyDate,
        description: data.StudyDescription
    },
    modality: data.Modality
});
```

## Summary

Tag extraction in node-dicom-rs is:

- **Simple**: Flat key-value structures for direct access
- **Consistent**: Same API across DicomFile and StoreScp
- **Type-safe**: Full TypeScript support with 300+ tag autocomplete
- **Flexible**: Standard tags, custom tags, hex formats all supported
- **Efficient**: Extract only what you need
- **Comprehensive**: Predefined sets for all modalities and use cases

Use predefined tag sets, handle missing values gracefully, and extract only the tags you need for optimal performance and maintainability.
