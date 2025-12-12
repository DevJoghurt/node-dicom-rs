# DicomFile - Reading and Manipulating DICOM Files

The `DicomFile` class provides methods to read, parse, and extract data from DICOM files.

## Basic Usage

```typescript
import { DicomFile } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./scan.dcm');

// Extract metadata
const tags = file.extract(['PatientName', 'StudyDate'], undefined, 'Flat');
const data = JSON.parse(tags);

console.log('Patient:', data.PatientName);
console.log('Study Date:', data.StudyDate);

file.close();
```

## Opening Files

### From Filesystem

```typescript
const file = new DicomFile();
file.open('/path/to/scan.dcm');
```

### From S3

```typescript
const file = new DicomFile();
file.openFromS3({
    bucket: 'dicom-archive',
    key: 'studies/study1/series1/instance1.dcm',
    accessKey: 'YOUR_ACCESS_KEY',
    secretKey: 'YOUR_SECRET_KEY',
    endpoint: 'https://s3.amazonaws.com',
    region: 'us-east-1'
});
```

### Quick Check Without Opening

Check if a file is valid DICOM and get basic info without fully opening:

```typescript
const info = DicomFile.check('/path/to/scan.dcm');

if (info.isValid) {
    console.log('Valid DICOM file');
    console.log('SOP Instance UID:', info.sopInstanceUID);
    console.log('SOP Class UID:', info.sopClassUID);
} else {
    console.log('Not a valid DICOM file');
}
```

## Extracting Metadata

### Basic Tag Extraction

```typescript
const file = new DicomFile();
file.open('./scan.dcm');

const json = file.extract([
    'PatientName',
    'PatientID',
    'StudyDate',
    'Modality'
], undefined, 'Flat');

const data = JSON.parse(json);
console.log(data);
// { PatientName: "DOE^JOHN", PatientID: "12345", StudyDate: "20231201", Modality: "CT" }
```

### Grouped by DICOM Hierarchy

```typescript
const json = file.extract([
    'PatientName', 'PatientID',           // Patient level
    'StudyDate', 'StudyDescription',      // Study level
    'Modality', 'SeriesDescription',      // Series level
    'InstanceNumber', 'SliceThickness'    // Instance level
], undefined, 'ByScope');

const data = JSON.parse(json);
console.log(data);
// {
//   patient: { PatientName: "DOE^JOHN", PatientID: "12345" },
//   study: { StudyDate: "20231201", StudyDescription: "..." },
//   series: { Modality: "CT", SeriesDescription: "..." },
//   instance: { InstanceNumber: "1", SliceThickness: "5.0" }
// }
```

### Study-Level Grouping

```typescript
const json = file.extract([
    'PatientName', 'StudyDate',           // Study level
    'Modality', 'InstanceNumber'          // Instance level
], undefined, 'StudyLevel');

const data = JSON.parse(json);
// {
//   study_level: { PatientName: "...", StudyDate: "..." },
//   instance_level: { Modality: "...", InstanceNumber: "..." }
// }
```

### Custom Private Tags

Extract private/vendor-specific tags with custom names:

```typescript
const json = file.extract(
    ['PatientName', 'StudyDate'],
    [
        { tag: '00091001', name: 'VendorField1' },
        { tag: '00091002', name: 'VendorField2' }
    ],
    'Flat'
);

const data = JSON.parse(json);
// {
//   PatientName: "...",
//   StudyDate: "...",
//   VendorField1: "...",
//   VendorField2: "..."
// }
```

## TypeScript Autocomplete

Full autocomplete for 300+ standard DICOM tags:

```typescript
const json = file.extract([
    'PatientName',           // Auto-suggests all standard tags
    'PatientID',
    'PatientBirthDate',
    'StudyDate',
    'StudyTime',
    'StudyDescription',
    'Modality',
    'SeriesDescription',
    'SeriesNumber',
    'InstanceNumber',
    'SliceThickness',
    'ImageOrientationPatient',
    'ImagePositionPatient',
    // ... 300+ more tags with autocomplete
], undefined, 'ByScope');
```

You can also use tag hex codes or (GGGG,EEEE) format:

```typescript
const json = file.extract([
    'PatientName',     // By name (autocomplete)
    '00100010',        // By hex
    '(0010,0010)',     // By (group,element)
], undefined, 'Flat');
```

## Using Tag Helper Functions

```typescript
import { getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const tags = getCommonTagSets();

// Extract patient + study basics
const json = file.extract(
    combineTags([tags.patientBasic, tags.studyBasic]),
    undefined,
    'ByScope'
);

// Extract all CT-related tags
const ctJson = file.extract(tags.ct, undefined, 'Flat');

// Combine multiple sets
const allTags = combineTags([
    tags.patientBasic,
    tags.studyBasic,
    tags.ct,
    tags.imagePixel
]);
```

Available tag sets:
- `patientBasic` - Patient name, ID, birth date, sex
- `patientExtended` - Additional patient info
- `studyBasic` - Study date, time, description, ID
- `studyExtended` - Referring physician, accession number
- `seriesBasic` - Series number, description, modality
- `instanceBasic` - Instance number, creation date/time
- `imagePixel` - Rows, columns, bits allocated, photometric interpretation
- `imageGeometry` - Pixel spacing, slice thickness, orientation
- `ct` - CT-specific tags (KVP, exposure, reconstruction)
- `mr` - MR-specific tags (echo time, repetition time, flip angle)
- `us` - Ultrasound-specific tags
- `pet` - PET-specific tags
- `equipment` - Manufacturer, model, software versions

## Pixel Data Operations

### Get Pixel Data Info

```typescript
const info = file.getPixelDataInfo();

console.log('Dimensions:', info.width, 'x', info.height);
console.log('Frames:', info.frames);
console.log('Bits Allocated:', info.bitsAllocated);
console.log('Bits Stored:', info.bitsStored);
console.log('Samples per Pixel:', info.samplesPerPixel);
console.log('Photometric Interpretation:', info.photometricInterpretation);
console.log('Is Compressed:', info.isCompressed);
console.log('Transfer Syntax:', info.transferSyntaxUID);

if (info.windowCenter && info.windowWidth) {
    console.log('Window Center/Width:', info.windowCenter, '/', info.windowWidth);
}
```

### Save Raw Pixel Data

Extract the raw pixel data bytes:

```typescript
// Save to file
file.saveRawPixelData('./output/pixels.raw');

// Or use static method
import { saveRawPixelData } from '@nuxthealth/node-dicom';
saveRawPixelData('./scan.dcm', './output/pixels.raw');
```

This saves the pixel data as-is (may be compressed). For uncompressed data, you'll need to decode it according to the transfer syntax.

### Decode Pixel Data (if supported)

```typescript
// For uncompressed images
const pixelInfo = file.getPixelDataInfo();

if (!pixelInfo.isCompressed) {
    file.saveRawPixelData('./pixels.raw');
    
    // Now you can process the raw bytes according to:
    // - width, height, frames
    // - bitsAllocated, bitsStored
    // - samplesPerPixel
    // - photometricInterpretation
}
```

## File Information

### Get All Elements

Get a list of all DICOM elements in the file:

```typescript
const elements = file.getElements();

elements.forEach(element => {
    console.log(`${element.tag}: ${element.vr} = ${element.value}`);
});
```

Element structure:
```typescript
{
    tag: string,           // Tag in (GGGG,EEEE) format
    vr: string,            // Value Representation
    value: string,         // String representation of value
    name?: string          // Tag name from dictionary (if available)
}
```

### Dump to JSON

Get the entire DICOM file as a JSON structure:

```typescript
const json = file.dump();
const obj = JSON.parse(json);

// Access any tag
console.log(obj['(0010,0010)']);  // Patient Name
```

## Saving Files

### Save as DICOM

```typescript
// Save to filesystem
file.saveAsDicom('./output/modified.dcm');

// Save to S3
file.saveAsDicomToS3({
    bucket: 'dicom-archive',
    key: 'output/modified.dcm',
    accessKey: 'YOUR_ACCESS_KEY',
    secretKey: 'YOUR_SECRET_KEY',
    endpoint: 'https://s3.amazonaws.com'
});
```

## Complete Example: DICOM Metadata Extractor

```typescript
import { DicomFile, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';
import * as fs from 'fs';
import * as path from 'path';

interface DicomMetadata {
    file: string;
    patient: any;
    study: any;
    series: any;
    instance: any;
    pixelData?: any;
}

function extractMetadata(filePath: string): DicomMetadata | null {
    const file = new DicomFile();
    
    try {
        // Check if valid first
        const info = DicomFile.check(filePath);
        if (!info.isValid) {
            console.error(`Not a valid DICOM file: ${filePath}`);
            return null;
        }

        file.open(filePath);

        // Get comprehensive tag set
        const tags = getCommonTagSets();
        const allTags = combineTags([
            tags.patientBasic,
            tags.studyBasic,
            tags.seriesBasic,
            tags.instanceBasic,
            tags.imagePixel,
            tags.imageGeometry
        ]);

        // Extract metadata
        const json = file.extract(allTags, undefined, 'ByScope');
        const data = JSON.parse(json);

        // Get pixel data info if available
        let pixelData;
        try {
            pixelData = file.getPixelDataInfo();
        } catch {
            // No pixel data
        }

        const metadata: DicomMetadata = {
            file: filePath,
            patient: data.patient || {},
            study: data.study || {},
            series: data.series || {},
            instance: data.instance || {},
            pixelData: pixelData
        };

        return metadata;
    } catch (error) {
        console.error(`Error processing ${filePath}:`, error.message);
        return null;
    } finally {
        file.close();
    }
}

function processDirectory(dirPath: string): DicomMetadata[] {
    const results: DicomMetadata[] = [];
    
    const files = fs.readdirSync(dirPath, { withFileTypes: true });
    
    for (const file of files) {
        const fullPath = path.join(dirPath, file.name);
        
        if (file.isDirectory()) {
            results.push(...processDirectory(fullPath));
        } else if (file.name.endsWith('.dcm')) {
            const metadata = extractMetadata(fullPath);
            if (metadata) {
                results.push(metadata);
            }
        }
    }
    
    return results;
}

// Usage
const metadata = processDirectory('./dicom-studies');

// Group by study
const studies = new Map<string, DicomMetadata[]>();
metadata.forEach(m => {
    const studyUID = m.study.StudyInstanceUID;
    if (!studies.has(studyUID)) {
        studies.set(studyUID, []);
    }
    studies.get(studyUID)!.push(m);
});

console.log(`Found ${studies.size} studies:`);
studies.forEach((instances, studyUID) => {
    const first = instances[0];
    console.log(`  Study: ${first.study.StudyDescription || 'Unknown'}`);
    console.log(`    Patient: ${first.patient.PatientName}`);
    console.log(`    Date: ${first.study.StudyDate}`);
    console.log(`    Instances: ${instances.length}`);
});

// Save to JSON
fs.writeFileSync('./metadata.json', JSON.stringify([...studies.entries()], null, 2));
```

## Tips

1. **Always close files**: Call `close()` when done, especially in loops
2. **Use check() first**: Verify files before opening for better error handling
3. **Choose right grouping**: `ByScope` for hierarchy, `Flat` for simple extraction
4. **Use helper functions**: `getCommonTagSets()` for predefined tag lists
5. **Handle missing tags**: Not all files have all tags, check before accessing
6. **Pixel data is complex**: Use `getPixelDataInfo()` to understand format before processing
7. **Large files**: Be mindful of memory with multi-frame or large matrix files
