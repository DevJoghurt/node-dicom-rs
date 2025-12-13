# Helper Functions Guide

Comprehensive guide to utility functions and helpers in node-dicom-rs for DICOM workflows.

## Table of Contents

- [Overview](#overview)
- [Tag Helper Functions](#tag-helper-functions)
- [SOP Class Helpers](#sop-class-helpers)
- [Transfer Syntax Helpers](#transfer-syntax-helpers)
- [Custom Tag Creation](#custom-tag-creation)
- [Best Practices](#best-practices)

## Overview

node-dicom-rs provides a rich set of helper functions to simplify common DICOM workflows. These utilities eliminate boilerplate code and provide type-safe, well-tested implementations for:

- Managing tag sets and tag combinations
- Configuring SOP classes for different modalities
- Selecting transfer syntaxes
- Creating custom tag mappings
- Validating and discovering available tags

## Tag Helper Functions

### getCommonTagSets()

Get predefined sets of commonly used DICOM tags organized by category.

**Returns:** `CommonTagSets` object containing 13 different tag sets

**Available Sets:**
- `patientBasic` - 7 essential patient demographics tags
- `studyBasic` - 7 study-level metadata tags
- `seriesBasic` - 8 series-level metadata tags
- `instanceBasic` - 5 instance identifiers
- `imagePixelInfo` - 9 image dimensions and pixel characteristics
- `equipment` - 6 device and institution tags
- `ct` - 6 CT-specific imaging parameters
- `mr` - 6 MR-specific parameters
- `ultrasound` - 6 ultrasound-specific tags
- `petNm` - 11 PET/Nuclear Medicine tags
- `xa` - 8 X-Ray Angiography tags
- `rt` - 11 Radiation Therapy tags
- `default` - 42 comprehensive tags (combines patient, study, series, instance, pixel, equipment)

**Example:**

```typescript
import { getCommonTagSets, DicomFile } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();

// Extract patient demographics
const file = new DicomFile();
file.open('scan.dcm');
const patientData = file.extract(tagSets.patientBasic);
console.log('Patient:', patientData.PatientName);
console.log('ID:', patientData.PatientID);
console.log('Birth Date:', patientData.PatientBirthDate);

// Extract study metadata
const studyData = file.extract(tagSets.studyBasic);
console.log('Study:', studyData.StudyDescription);
console.log('Date:', studyData.StudyDate);

// Extract comprehensive metadata
const allData = file.extract(tagSets.default);
// 42 common tags available

file.close();
```

**Modality-Specific Workflows:**

```typescript
const tagSets = getCommonTagSets();

// CT workflow
const ctTags = [...tagSets.default, ...tagSets.ct];
const ctData = file.extract(ctTags);
console.log('CT Parameters:');
console.log('  kVp:', ctData.KVP);
console.log('  Exposure Time:', ctData.ExposureTime);
console.log('  Tube Current:', ctData.XRayTubeCurrent);
console.log('  Convolution Kernel:', ctData.ConvolutionKernel);

// MR workflow
const mrTags = [...tagSets.default, ...tagSets.mr];
const mrData = file.extract(mrTags);
console.log('MR Parameters:');
console.log('  TR:', mrData.RepetitionTime);
console.log('  TE:', mrData.EchoTime);
console.log('  Flip Angle:', mrData.FlipAngle);
console.log('  Field Strength:', mrData.MagneticFieldStrength);

// PET workflow
const petTags = [...tagSets.default, ...tagSets.petNm];
const petData = file.extract(petTags);
console.log('PET Parameters:');
console.log('  Units:', petData.Units);
console.log('  Decay Correction:', petData.DecayCorrection);
console.log('  Radionuclide:', petData.RadionuclideTotalDose);
```

**StoreScp Configuration:**

```typescript
import { StoreScp, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();

// Configure SCP to extract modality-specific tags
const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './received',
    extractTags: combineTags([
        tagSets.default,
        tagSets.ct,
        tagSets.mr
    ])
});

receiver.onFileStored((err, event) => {
    const tags = event.data?.tags;
    if (tags) {
        console.log('Common:', tags.PatientName, tags.Modality);
        // CT/MR specific tags available if present
        if (tags.KVP) console.log('CT kVp:', tags.KVP);
        if (tags.RepetitionTime) console.log('MR TR:', tags.RepetitionTime);
    }
});
```

### combineTags()

Combine multiple tag arrays into a single deduplicated array.

**Parameters:**
- `tagArrays: Vec<Vec<String>>` - Array of tag name arrays to combine

**Returns:** `Vec<String>` - Single array containing all unique tag names

**Features:**
- Removes duplicates automatically
- Preserves order of first appearance
- Works with any combination of tag arrays

**Example:**

```typescript
import { getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();

// Combine predefined sets
const combined = combineTags([
    tagSets.patientBasic,
    tagSets.studyBasic,
    tagSets.seriesBasic,
    tagSets.imagePixelInfo
]);

console.log(`Combined ${combined.length} unique tags`);
// Duplicates automatically removed
```

**Mix Predefined and Custom Tags:**

```typescript
// Build custom workflow tags
const workflowTags = combineTags([
    tagSets.patientBasic,
    tagSets.studyBasic,
    ['WindowCenter', 'WindowWidth'],           // Display params
    ['RescaleIntercept', 'RescaleSlope'],     // Rescale params
    tagSets.ct,
    ['SliceThickness', 'SpacingBetweenSlices'] // Geometry
]);

// Any overlaps are automatically removed
const data = file.extract(workflowTags);
```

**Modality-Agnostic Extraction:**

```typescript
// Create universal tag set covering all modalities
const universalTags = combineTags([
    tagSets.default,
    tagSets.ct,
    tagSets.mr,
    tagSets.ultrasound,
    tagSets.petNm,
    tagSets.xa,
    tagSets.rt
]);

console.log(`Universal tag set: ${universalTags.length} tags`);
// Use for processing mixed-modality archives
```

**Build Reusable Configurations:**

```typescript
// Define once, use everywhere
export const ANONYMIZATION_TAGS = combineTags([
    tagSets.studyBasic,
    tagSets.seriesBasic,
    tagSets.instanceBasic
    // Excludes patient demographics
]);

export const ROUTING_TAGS = combineTags([
    ['StudyInstanceUID', 'SeriesInstanceUID', 'SOPInstanceUID'],
    ['Modality', 'StationName'],
    ['StudyDate', 'StudyTime']
]);

export const QA_TAGS = combineTags([
    tagSets.imagePixelInfo,
    ['WindowCenter', 'WindowWidth', 'ImageType'],
    ['BurnedInAnnotation', 'LossyImageCompression']
]);

// Use in your application
const qaData = file.extract(QA_TAGS);
```

### getAvailableTagNames()

Get a comprehensive list of 300+ commonly used DICOM tag names.

**Returns:** `Array<string>` - Array of standard DICOM tag names

**Coverage:**
- Patient demographics and identification
- Study, series, and instance metadata
- Image characteristics and pixel data
- Equipment and institution information
- Modality-specific parameters (CT, MR, US, PET, XA, RT)
- Timing and temporal information
- Geometry and spatial information
- Display and presentation parameters
- Technical acquisition parameters
- Overlays, graphics, and waveforms
- Multi-frame and cine sequences

**Example:**

```typescript
import { getAvailableTagNames } from '@nuxthealth/node-dicom';

// Get all available tags
const allTags = getAvailableTagNames();
console.log(`Total available: ${allTags.length} tags`);
// Output: Total available: 300+ tags

// Check if specific tag is available
const hasTag = allTags.includes('WindowCenter');
console.log('WindowCenter available:', hasTag); // true

// Validate user input
const userTags = ['PatientName', 'InvalidTag', 'StudyDate'];
const validTags = userTags.filter(tag => allTags.includes(tag));
console.log('Valid tags:', validTags); // ['PatientName', 'StudyDate']
```

**Search for Tags:**

```typescript
const allTags = getAvailableTagNames();

// Find patient-related tags
const patientTags = allTags.filter(tag => 
    tag.toLowerCase().includes('patient')
);
console.log('Patient tags:', patientTags);
// ['PatientName', 'PatientID', 'PatientBirthDate', ...]

// Find all UID tags
const uidTags = allTags.filter(tag => tag.endsWith('UID'));
console.log('UID tags:', uidTags);
// ['StudyInstanceUID', 'SeriesInstanceUID', 'SOPInstanceUID', ...]

// Find timing tags
const timingTags = allTags.filter(tag => 
    tag.includes('Time') || tag.includes('Date')
);
console.log('Timing tags:', timingTags);
```

**Build Tag Category Groups:**

```typescript
const allTags = getAvailableTagNames();

// Categorize tags
const categories = {
    patient: allTags.filter(t => t.includes('Patient')),
    study: allTags.filter(t => t.includes('Study')),
    series: allTags.filter(t => t.includes('Series')),
    instance: allTags.filter(t => t.includes('Instance')),
    image: allTags.filter(t => t.includes('Image') || t.includes('Pixel')),
    equipment: allTags.filter(t => 
        t.includes('Manufacturer') || 
        t.includes('Device') || 
        t.includes('Station')
    ),
    modality: allTags.filter(t => t.includes('Modality'))
};

// Display category sizes
Object.entries(categories).forEach(([name, tags]) => {
    console.log(`${name}: ${tags.length} tags`);
});
```

**Tag Validation Function:**

```typescript
function validateAndCleanTags(userTags: string[]): {
    valid: string[],
    invalid: string[]
} {
    const available = getAvailableTagNames();
    
    return {
        valid: userTags.filter(tag => available.includes(tag)),
        invalid: userTags.filter(tag => !available.includes(tag))
    };
}

// Usage
const userInput = [
    'PatientName',
    'StudyDate',
    'InvalidTag',
    'Modality',
    'NotATag'
];

const result = validateAndCleanTags(userInput);
console.log('Valid:', result.valid);     // ['PatientName', 'StudyDate', 'Modality']
console.log('Invalid:', result.invalid); // ['InvalidTag', 'NotATag']
```

## SOP Class Helpers

### getCommonSopClasses()

Get predefined sets of SOP Class UIDs organized by modality and image type.

**Returns:** `CommonSopClasses` object with modality-specific SOP class arrays

**Available Sets:**
- `ct` - CT Image Storage
- `mr` - MR Image Storage
- `us` - Ultrasound Image Storage (Standard, Enhanced, Multi-frame)
- `pet` - PET Image Storage
- `nm` - Nuclear Medicine Image Storage
- `xa` - X-Ray Angiographic Image Storage
- `xrf` - X-Ray Radiofluoroscopic Image Storage
- `dx` - Digital X-Ray Image Storage
- `mg` - Digital Mammography X-Ray Image Storage
- `sc` - Secondary Capture Image Storage
- `rt` - RT Image Storage, RT Dose Storage, RT Structure Set Storage, RT Plan Storage
- `all` - Comprehensive array of all above SOP classes

**Example:**

```typescript
import { StoreScp, getCommonSopClasses } from '@nuxthealth/node-dicom';

const sopClasses = getCommonSopClasses();

// Accept only CT images
const ctReceiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'CT-SCP',
    outDir: './ct-storage',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: sopClasses.ct
});

// Accept CT and MR
const crossModalityReceiver = new StoreScp({
    port: 4447,
    callingAeTitle: 'CROSS-SCP',
    outDir: './multimodal-storage',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: [...sopClasses.ct, ...sopClasses.mr]
});

// Accept all common types
const universalReceiver = new StoreScp({
    port: 4448,
    callingAeTitle: 'UNIVERSAL-SCP',
    outDir: './universal-storage',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: sopClasses.all
});
```

**Specialized Receivers:**

```typescript
// Radiation Therapy only
const rtReceiver = new StoreScp({
    port: 4449,
    callingAeTitle: 'RT-SCP',
    outDir: './rt-storage',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: sopClasses.rt
});

// Imaging modalities (no RT)
const imagingReceiver = new StoreScp({
    port: 4450,
    callingAeTitle: 'IMAGING-SCP',
    outDir: './imaging-storage',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: [
        ...sopClasses.ct,
        ...sopClasses.mr,
        ...sopClasses.us,
        ...sopClasses.pet,
        ...sopClasses.nm,
        ...sopClasses.xa,
        ...sopClasses.dx,
        ...sopClasses.mg
    ]
});

// Secondary Capture and screenshots
const scReceiver = new StoreScp({
    port: 4451,
    callingAeTitle: 'SC-SCP',
    outDir: './screenshots',
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: sopClasses.sc
});
```

## Transfer Syntax Helpers

### getCommonTransferSyntaxes()

Get predefined sets of Transfer Syntax UIDs for compression and encoding.

**Returns:** `CommonTransferSyntaxes` object with compression-specific arrays

**Available Sets:**
- `uncompressed` - Explicit/Implicit VR Little/Big Endian (3 syntaxes)
- `jpegLossy` - JPEG Lossy compressions, various quality levels (7 syntaxes)
- `jpegLossless` - JPEG Lossless compressions (4 syntaxes)
- `jpeg2000Lossy` - JPEG 2000 Lossy (1 syntax)
- `jpeg2000Lossless` - JPEG 2000 Lossless (1 syntax)
- `rle` - RLE Lossless (1 syntax)
- `all` - All common transfer syntaxes (17 syntaxes)

**Example:**

```typescript
import { StoreScp, getCommonTransferSyntaxes } from '@nuxthealth/node-dicom';

const transferSyntaxes = getCommonTransferSyntaxes();

// Accept only uncompressed images
const uncompressedReceiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'UNCOMP-SCP',
    outDir: './uncompressed',
    transferSyntaxMode: 'Custom',
    transferSyntaxes: transferSyntaxes.uncompressed
});

// Accept any JPEG (lossy or lossless)
const jpegReceiver = new StoreScp({
    port: 4447,
    callingAeTitle: 'JPEG-SCP',
    outDir: './jpeg-storage',
    transferSyntaxMode: 'Custom',
    transferSyntaxes: [
        ...transferSyntaxes.jpegLossy,
        ...transferSyntaxes.jpegLossless
    ]
});

// Accept lossless only (JPEG Lossless, JPEG 2000 Lossless, RLE)
const losslessReceiver = new StoreScp({
    port: 4448,
    callingAeTitle: 'LOSSLESS-SCP',
    outDir: './lossless-storage',
    transferSyntaxMode: 'Custom',
    transferSyntaxes: [
        ...transferSyntaxes.uncompressed,
        ...transferSyntaxes.jpegLossless,
        ...transferSyntaxes.jpeg2000Lossless,
        ...transferSyntaxes.rle
    ]
});

// Accept everything
const universalReceiver = new StoreScp({
    port: 4449,
    callingAeTitle: 'UNIVERSAL-SCP',
    outDir: './all-storage',
    transferSyntaxMode: 'Custom',
    transferSyntaxes: transferSyntaxes.all
});
```

**Mode-Based Configuration:**

```typescript
// Simpler: use predefined modes
const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './storage',
    
    // Mode-based (no need for transfer syntax arrays)
    transferSyntaxMode: 'UncompressedOnly'  // or 'All' or 'Custom'
});
```

## Custom Tag Creation

### createCustomTag()

Create custom tag specifications for private or vendor-specific DICOM tags.

**Parameters:**
- `tag: string` - DICOM tag in hex format (e.g., "00091001" or "(0009,1001)")
- `name: string` - Human-readable name for this tag

**Returns:** `CustomTag` object for use in extraction functions

**Example:**

```typescript
import { DicomFile, createCustomTag } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('scan-with-private-tags.dcm');

// Define custom tags
const customTag1 = createCustomTag('00091001', 'VendorSpecificID');
const customTag2 = createCustomTag('00431027', 'ScannerMode');
const customTag3 = createCustomTag('(0019,100A)', 'ProcessingFlags');

// Extract with custom tags
const json = file.extract(
    ['PatientName', 'StudyDate', 'Modality'],
    [customTag1, customTag2, customTag3]
);

const data = JSON.parse(json);
console.log('Standard:', data.PatientName, data.Modality);
console.log('Custom:', data.VendorSpecificID, data.ScannerMode);

file.close();
```

**Vendor-Specific Tag Libraries:**

```typescript
// GE private tags
const geTags = [
    createCustomTag('00091001', 'GE_PrivateCreator'),
    createCustomTag('00091027', 'GE_ScanOptions'),
    createCustomTag('00431001', 'GE_ImageFiltering'),
    createCustomTag('00431010', 'GE_ReconstructionParams')
];

// Siemens private tags
const siemensTags = [
    createCustomTag('00191008', 'Siemens_ImagingMode'),
    createCustomTag('00191009', 'Siemens_SequenceInfo'),
    createCustomTag('00191010', 'Siemens_CoilID'),
    createCustomTag('0029100C', 'Siemens_CoilString')
];

// Philips private tags
const philipsTags = [
    createCustomTag('20011001', 'Philips_ScanMode'),
    createCustomTag('20011003', 'Philips_ContrastEnhancement'),
    createCustomTag('20051080', 'Philips_ReconstructionParams'),
    createCustomTag('20051081', 'Philips_ImageEnhancements')
];

// Vendor tag library
const vendorTagLibrary = {
    'GE MEDICAL SYSTEMS': geTags,
    'SIEMENS': siemensTags,
    'Philips Medical Systems': philipsTags
};
```

**Dynamic Vendor Tag Selection:**

```typescript
import { getCommonTagSets, createCustomTag } from '@nuxthealth/node-dicom';

const tagSets = getCommonTagSets();
const file = new DicomFile();
file.open('scan.dcm');

// Detect manufacturer
const mfgData = file.extract(['Manufacturer'])
const manufacturer = mfgData.Manufacturer;

// Select appropriate vendor tags
const vendorTags = vendorTagLibrary[manufacturer] || [];

// Extract all data including vendor-specific tags
const allTags = file.extract(
    tagSets.default,
    vendorTags
);

console.log('Manufacturer:', allTags.Manufacturer);
if (manufacturer.includes('GE')) {
    console.log('GE Scan Options:', allTags.GE_ScanOptions);
} else if (manufacturer.includes('SIEMENS')) {
    console.log('Siemens Sequence:', allTags.Siemens_SequenceInfo);
}

file.close();
```

**StoreScp with Custom Tags:**

```typescript
import { StoreScp, getCommonTagSets, createCustomTag } from '@nuxthealth/node-dicom';

const customTags = [
    createCustomTag('00091001', 'VendorID'),
    createCustomTag('00091010', 'ScanProtocol'),
    createCustomTag('00431001', 'QualityMetrics')
];

const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './received',
    extractTags: getCommonTagSets().default,
    extractCustomTags: customTags
});

receiver.onFileStored((err, event) => {
    const tags = event.data?.tags;
    if (tags) {
        console.log('Standard:', tags.PatientName, tags.Modality);
        console.log('Custom:', tags.VendorID, tags.ScanProtocol);
        
        // Store vendor-specific metrics
        if (tags.QualityMetrics) {
            storeQualityMetrics(tags.SOPInstanceUID, tags.QualityMetrics);
        }
    }
});

receiver.listen();
```

## Best Practices

### 1. Use Predefined Helper Functions

Leverage built-in helpers instead of hardcoding values:

```typescript
// Good ✓
const tagSets = getCommonTagSets();
const tags = file.extract(tagSets.default);

const sopClasses = getCommonSopClasses();
const receiver = new StoreScp({
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: sopClasses.ct
});

// Avoid ✗
const tags = file.extract([
    'PatientName', 'PatientID', /* ... 40 more tags */
]);

const receiver = new StoreScp({
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: [
        '1.2.840.10008.5.1.4.1.1.2',  // Hard to maintain
        '1.2.840.10008.5.1.4.1.1.4'
    ]
});
```

### 2. Create Reusable Configurations

Define configuration objects once and reuse:

```typescript
// config.ts
import { getCommonTagSets, getCommonSopClasses, combineTags } from '@nuxthealth/node-dicom';

export const TAG_CONFIGS = {
    anonymization: combineTags([
        getCommonTagSets().studyBasic,
        getCommonTagSets().seriesBasic,
        getCommonTagSets().instanceBasic
    ]),
    
    routing: [
        'StudyInstanceUID',
        'SeriesInstanceUID',
        'SOPInstanceUID',
        'Modality'
    ],
    
    qualityCheck: combineTags([
        getCommonTagSets().imagePixelInfo,
        ['ImageType', 'BurnedInAnnotation', 'LossyImageCompression']
    ])
};

export const SCP_CONFIGS = {
    ctOnly: {
        abstractSyntaxes: getCommonSopClasses().ct,
        transferSyntaxes: getCommonTransferSyntaxes().uncompressed
    },
    
    imaging: {
        abstractSyntaxes: [
            ...getCommonSopClasses().ct,
            ...getCommonSopClasses().mr,
            ...getCommonSopClasses().us
        ],
        transferSyntaxes: getCommonTransferSyntaxes().all
    }
};

// Use in your application
import { TAG_CONFIGS, SCP_CONFIGS } from './config';

const qaData = file.extract(TAG_CONFIGS.qualityCheck);

const receiver = new StoreScp({
    port: 4446,
    ...SCP_CONFIGS.imaging
});
```

### 3. Validate and Handle Missing Tags

Not all tags are present in all files:

```typescript
const tagSets = getCommonTagSets();
const data = file.extract(tagSets.default);

// Safe access with defaults
const patientName = data.PatientName || 'Unknown';
const studyDesc = data.StudyDescription || 'No description';

// Conditional logic
if (data.WindowCenter && data.WindowWidth) {
    applyWindowing(data.WindowCenter, data.WindowWidth);
}

// Validation function
function hasRequiredTags(data: any, required: string[]): boolean {
    return required.every(tag => data[tag] != null);
}

if (!hasRequiredTags(data, ['PatientID', 'StudyInstanceUID'])) {
    console.error('Missing required tags');
}
```

### 4. Build Modality-Aware Workflows

Adapt tag extraction and processing based on modality:

```typescript
const tagSets = getCommonTagSets();
const modalityData = file.extract(['Modality']);
const modality = modalityData.Modality;

// Select appropriate tag set
let tags = tagSets.default;
switch (modality) {
    case 'CT':
        tags = combineTags([tagSets.default, tagSets.ct]);
        break;
    case 'MR':
        tags = combineTags([tagSets.default, tagSets.mr]);
        break;
    case 'US':
        tags = combineTags([tagSets.default, tagSets.ultrasound]);
        break;
    case 'PT':
        tags = combineTags([tagSets.default, tagSets.petNm]);
        break;
}

const fullData = file.extract(tags);
processModalitySpecific(modality, fullData);
```

### 5. Organize Custom Tags by Vendor

Create structured libraries for vendor-specific tags:

```typescript
// vendor-tags.ts
import { createCustomTag } from '@nuxthealth/node-dicom';

export const VENDOR_TAGS = {
    ge: {
        privateCreator: createCustomTag('00091001', 'GE_PrivateCreator'),
        scanOptions: createCustomTag('00091027', 'GE_ScanOptions'),
        filtering: createCustomTag('00431001', 'GE_ImageFiltering')
    },
    
    siemens: {
        imagingMode: createCustomTag('00191008', 'Siemens_ImagingMode'),
        sequenceInfo: createCustomTag('00191009', 'Siemens_SequenceInfo'),
        coilString: createCustomTag('0029100C', 'Siemens_CoilString')
    },
    
    philips: {
        scanMode: createCustomTag('20011001', 'Philips_ScanMode'),
        contrast: createCustomTag('20011003', 'Philips_Contrast'),
        recon: createCustomTag('20051080', 'Philips_ReconParams')
    }
};

// Get vendor tags as array
export function getVendorTags(manufacturer: string): CustomTag[] {
    const vendor = manufacturer.toLowerCase();
    
    if (vendor.includes('ge')) {
        return Object.values(VENDOR_TAGS.ge);
    } else if (vendor.includes('siemens')) {
        return Object.values(VENDOR_TAGS.siemens);
    } else if (vendor.includes('philips')) {
        return Object.values(VENDOR_TAGS.philips);
    }
    
    return [];
}
```

### 6. Document Tag Usage

Add clear documentation for tag configurations:

```typescript
/**
 * Quality assurance tag set
 * Extracts tags needed for automated QA checks:
 * - Image dimensions and bit depth
 * - Window/level settings
 * - Compression status
 * - Burned-in annotation flag
 */
export const QA_TAGS = combineTags([
    tagSets.imagePixelInfo,
    ['WindowCenter', 'WindowWidth'],
    ['ImageType', 'BurnedInAnnotation', 'LossyImageCompression']
]);

/**
 * PACS routing tag set
 * Minimal tags required for routing decisions:
 * - Study/Series/Instance UIDs
 * - Modality and station
 * - Study date for archive tier selection
 */
export const ROUTING_TAGS = [
    'StudyInstanceUID',
    'SeriesInstanceUID',
    'SOPInstanceUID',
    'Modality',
    'StationName',
    'StudyDate'
];
```

### 7. Test Tag Availability

Check available tags before use:

```typescript
const allTags = getAvailableTagNames();

// Validate configuration
function validateTagConfig(configTags: string[]): boolean {
    const unavailable = configTags.filter(tag => !allTags.includes(tag));
    
    if (unavailable.length > 0) {
        console.warn('Unavailable tags:', unavailable);
        return false;
    }
    
    return true;
}

const myTags = ['PatientName', 'StudyDate', 'InvalidTag'];
if (validateTagConfig(myTags)) {
    // Proceed with extraction
} else {
    // Fix configuration
}
```

### 8. Performance Optimization

Optimize tag extraction for performance:

```typescript
// Extract once, use many times
const data = file.extract(tagSets.default);

// Good ✓
for (let i = 0; i < 1000; i++) {
    processPatient(data.PatientName);
    processStudy(data.StudyInstanceUID);
}

// Bad ✗ - Extracts 1000 times
for (let i = 0; i < 1000; i++) {
    const data = file.extract(['PatientName']);
    processPatient(data.PatientName);
}
```

### 9. Error Handling

Always handle errors gracefully:

```typescript
try {
    const tagSets = getCommonTagSets();
    const tags = file.extract(tagSets.default);
    const data = JSON.parse(tags);
    
    if (!validateRequiredTags(data)) {
        throw new Error('Missing required tags');
    }
    
    processData(data);
    
} catch (error) {
    console.error('Tag extraction failed:', error);
    // Handle error appropriately
}
```

### 10. Logging and Monitoring

Log helper usage for debugging and monitoring:

```typescript
const tagSets = getCommonTagSets();
const sopClasses = getCommonSopClasses();
const transferSyntaxes = getCommonTransferSyntaxes();

console.log('Configuration:');
console.log(`  Default tags: ${tagSets.default.length}`);
console.log(`  CT tags: ${tagSets.ct.length}`);
console.log(`  MR tags: ${tagSets.mr.length}`);
console.log(`  CT SOP classes: ${sopClasses.ct.length}`);
console.log(`  Uncompressed syntaxes: ${transferSyntaxes.uncompressed.length}`);
console.log(`  Total available tags: ${getAvailableTagNames().length}`);
```

## Summary

Helper functions in node-dicom-rs provide:

- **Simplified Configuration**: Predefined tag sets, SOP classes, and transfer syntaxes
- **Type Safety**: Full TypeScript support with autocomplete
- **Maintainability**: Centralized definitions reduce duplication
- **Flexibility**: Combine and customize for specific workflows
- **Validation**: Check tag availability and validate inputs
- **Vendor Support**: Easy integration of custom/private tags

Use these helpers to build robust, maintainable DICOM workflows with minimal boilerplate code.
