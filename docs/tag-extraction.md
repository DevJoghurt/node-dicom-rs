# Tag Extraction Guide

This guide covers the flexible tag extraction system used across DicomFile and StoreSCP.

## Grouping Strategies

The library supports four different grouping strategies for organizing extracted DICOM tags:

### ByScope (Default)

Groups tags according to DICOM hierarchy levels based on their scope in the DICOM standard:

```typescript
const json = file.extract([
    'PatientName', 'PatientID',           // Patient scope
    'StudyDate', 'StudyDescription',      // Study scope
    'Modality', 'SeriesDescription',      // Series scope
    'InstanceNumber', 'SliceThickness'    // Instance scope
], undefined, 'ByScope');

const data = JSON.parse(json);
// {
//   patient: { PatientName: "...", PatientID: "..." },
//   study: { StudyDate: "...", StudyDescription: "..." },
//   series: { Modality: "...", SeriesDescription: "..." },
//   instance: { InstanceNumber: "...", SliceThickness: "..." },
//   equipment: { ... }  // If equipment tags are extracted
// }
```

**Tag Scope Classification:**
- **Patient**: All tags in group (0010,xxxx) - Patient demographics
- **Study**: Study-related tags (StudyDate, StudyDescription, AccessionNumber, etc.)
- **Series**: Series-related tags (Modality, SeriesNumber, SeriesDescription, etc.)
- **Instance**: Image instance tags (InstanceNumber, SliceLocation, ImagePosition, etc.)
- **Equipment**: Equipment/acquisition tags in group (0018,1xxx)

**Use when:** You want proper DICOM hierarchy organization

### Flat

All tags in a single flat object with no grouping:

```typescript
const json = file.extract([
    'PatientName',
    'StudyDate',
    'Modality',
    'InstanceNumber'
], undefined, 'Flat');

const data = JSON.parse(json);
// {
//   PatientName: "DOE^JOHN",
//   StudyDate: "20231201",
//   Modality: "CT",
//   InstanceNumber: "1"
// }
```

**Use when:** You want simple key-value access without hierarchy

### StudyLevel

Two-level grouping: study-level data (Patient + Study) and instance-level data (everything else):

```typescript
const json = file.extract([
    'PatientName', 'StudyDate',           // Study level
    'Modality', 'InstanceNumber'          // Instance level
], undefined, 'StudyLevel');

const data = JSON.parse(json);
// {
//   study_level: {
//     PatientName: "...",
//     StudyDate: "..."
//   },
//   instance_level: {
//     Modality: "...",
//     InstanceNumber: "..."
//   }
// }
```

**Use when:** You want to separate "per-study" data from "per-instance" data

### Custom

Currently behaves the same as `ByScope`. Can be extended for user-defined grouping rules in the future.

## How Grouping Affects Events

The grouping strategy affects both `DicomFile.extract()` and `StoreSCP` events:

### DicomFile.extract()

The grouping strategy controls the structure of the returned JSON:

```typescript
const file = new DicomFile();
file.open('./scan.dcm');

// ByScope: Hierarchical
const scoped = JSON.parse(file.extract(tags, undefined, 'ByScope'));
console.log(scoped.patient.PatientName);

// Flat: Direct access
const flat = JSON.parse(file.extract(tags, undefined, 'Flat'));
console.log(flat.PatientName);

// StudyLevel: Two-level
const studyLevel = JSON.parse(file.extract(tags, undefined, 'StudyLevel'));
console.log(studyLevel.study_level.PatientName);
```

### StoreScp OnFileStored Event

The `data` field structure matches the grouping strategy:

```typescript
const receiver = new StoreScp({
    extractTags: ['PatientName', 'StudyDate', 'Modality'],
    groupingStrategy: 'ByScope'  // or 'Flat', 'StudyLevel'
});

receiver.addEventListener('OnFileStored', (event) => {
    // With ByScope:
    console.log(event.tags.patient?.PatientName);
    
    // With Flat:
    console.log(event.tags.PatientName);
    
    // With StudyLevel:
    console.log(event.tags.study_level?.PatientName);
});
```

### StoreScp OnStudyCompleted Event

The grouping strategy controls where data appears in the hierarchy:

**ByScope**: Data is distributed across hierarchy levels

```typescript
{
    study_instance_uid: "...",
    tags: {                        // Study + Patient tags only
        PatientName: "...",
        StudyDate: "..."
    },
    series: [{
        series_instance_uid: "...",
        tags: {                    // Series tags only
            Modality: "CT",
            SeriesDescription: "..."
        },
        instances: [{
            sop_instance_uid: "...",
            file: "...",
            tags: {                // Instance + Equipment tags only
                InstanceNumber: "1",
                SliceThickness: "5.0"
            }
        }]
    }]
}
```

**Flat**: All data at instance level only

```typescript
{
    study_instance_uid: "...",
    tags: {},                      // Empty
    series: [{
        series_instance_uid: "...",
        tags: {},                  // Empty
        instances: [{
            sop_instance_uid: "...",
            file: "...",
            tags: {                // ALL tags here
                PatientName: "...",
                StudyDate: "...",
                Modality: "CT",
                InstanceNumber: "1"
            }
        }]
    }]
}
```

**StudyLevel**: Study-level at study, rest at instance

```typescript
{
    study_instance_uid: "...",
    tags: {                        // Patient + Study tags
        PatientName: "...",
        StudyDate: "..."
    },
    series: [{
        series_instance_uid: "...",
        tags: {},                  // Empty
        instances: [{
            sop_instance_uid: "...",
            file: "...",
            tags: {                // Series + Instance + Equipment
                Modality: "CT",
                InstanceNumber: "1",
                SliceThickness: "5.0"
            }
        }]
    }]
}
```

## Custom Tags

Extract private or vendor-specific tags with user-defined names:

```typescript
const json = file.extract(
    ['PatientName', 'StudyDate'],  // Standard tags
    [                              // Custom tags
        { tag: '00091001', name: 'VendorID' },
        { tag: '00091002', name: 'ScannerType' },
        { tag: '(0009,1010)', name: 'CustomField' }  // Also supports (GGGG,EEEE) format
    ],
    'ByScope'
);

const data = JSON.parse(json);
// {
//   patient: { PatientName: "..." },
//   study: { StudyDate: "..." },
//   custom: {
//     VendorID: "...",
//     ScannerType: "...",
//     CustomField: "..."
//   }
// }
```

Custom tags always appear in a separate `custom` group (for `ByScope`/`StudyLevel`), or mixed with standard tags (for `Flat`).

### In StoreScp

```typescript
const receiver = new StoreScp({
    extractTags: ['PatientName', 'StudyDate'],
    extractCustomTags: [
        { tag: '00091001', name: 'VendorID' }
    ],
    groupingStrategy: 'ByScope'
});
```

## Helper Functions

### getCommonTagSets()

Get predefined sets of commonly used DICOM tags:

```typescript
import { getCommonTagSets } from '@nuxthealth/node-dicom';

const tags = getCommonTagSets();

// Available sets:
tags.patientBasic      // Patient name, ID, birth date, sex (4 tags)
tags.patientExtended   // Additional patient info (6 tags)
tags.studyBasic        // Study date, time, description, ID (5 tags)
tags.studyExtended     // Accession number, referring physician (4 tags)
tags.seriesBasic       // Series number, description, modality (5 tags)
tags.instanceBasic     // Instance number, creation date/time (3 tags)
tags.imagePixel        // Rows, columns, bits, photometric interpretation (8 tags)
tags.imageGeometry     // Pixel spacing, orientation, slice thickness (6 tags)
tags.ct                // CT-specific: KVP, exposure, kernel (8 tags)
tags.mr                // MR-specific: echo time, TR, flip angle (10 tags)
tags.us                // Ultrasound-specific (5 tags)
tags.pet               // PET-specific: radiopharmaceutical, units (6 tags)
tags.equipment         // Manufacturer, model, software (4 tags)

// Use in extraction
const json = file.extract(tags.patientBasic, undefined, 'Flat');
```

### combineTags()

Combine multiple tag sets into one array:

```typescript
import { getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const tags = getCommonTagSets();

const allBasic = combineTags([
    tags.patientBasic,
    tags.studyBasic,
    tags.seriesBasic,
    tags.instanceBasic
]);

const ctComplete = combineTags([
    tags.patientBasic,
    tags.studyBasic,
    tags.ct,
    tags.imagePixel,
    tags.imageGeometry
]);

const json = file.extract(ctComplete, undefined, 'ByScope');
```

### Using with StoreScp

```typescript
import { StoreScp, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const tags = getCommonTagSets();

const receiver = new StoreScp({
    port: 4446,
    extractTags: combineTags([
        tags.patientBasic,
        tags.studyBasic,
        tags.ct
    ]),
    groupingStrategy: 'ByScope'
});
```

## Complete Example: Different Strategies

```typescript
import { DicomFile, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./ct_scan.dcm');

const tags = getCommonTagSets();
const extractTags = combineTags([
    tags.patientBasic,
    tags.studyBasic,
    tags.seriesBasic,
    tags.ct
]);

console.log('=== ByScope Strategy ===');
const byScope = JSON.parse(file.extract(extractTags, undefined, 'ByScope'));
console.log('Patient:', byScope.patient);
console.log('Study:', byScope.study);
console.log('Series:', byScope.series);
console.log('Equipment:', byScope.equipment);

console.log('\n=== Flat Strategy ===');
const flat = JSON.parse(file.extract(extractTags, undefined, 'Flat'));
console.log('All tags:', Object.keys(flat).length);
console.log('PatientName:', flat.PatientName);
console.log('Modality:', flat.Modality);

console.log('\n=== StudyLevel Strategy ===');
const studyLevel = JSON.parse(file.extract(extractTags, undefined, 'StudyLevel'));
console.log('Study Level:', Object.keys(studyLevel.study_level || {}));
console.log('Instance Level:', Object.keys(studyLevel.instance_level || {}));

file.close();
```

## Best Practices

1. **Choose the right strategy:**
   - Use `ByScope` for proper DICOM hierarchy and when data will be stored hierarchically
   - Use `Flat` for simple queries or when you need direct key-value access
   - Use `StudyLevel` when you want to separate per-study metadata from per-instance data

2. **Extract only what you need:**
   - Don't extract all tags if you only need a few
   - Use helper functions for predefined sets

3. **Handle missing tags gracefully:**
   ```typescript
   const data = JSON.parse(json);
   const patientName = data.patient?.PatientName || 'Unknown';
   ```

4. **Consider storage implications:**
   - `ByScope` in OnStudyCompleted avoids data duplication
   - `Flat` duplicates study-level data across all instances

5. **Use TypeScript autocomplete:**
   - Leverage the 300+ tag name suggestions
   - Less error-prone than manual hex codes

6. **Custom tags for vendor data:**
   - Use `extractCustomTags` for private tags
   - Give them meaningful names for easier access

## Tag Scope Reference

How tags are classified by the scope system:

| Scope | DICOM Standard | Examples |
|-------|---------------|----------|
| **Patient** | Patient Module (0010,xxxx) | PatientName, PatientID, PatientBirthDate, PatientSex |
| **Study** | Study Module | StudyDate, StudyTime, AccessionNumber, StudyDescription, ReferringPhysicianName |
| **Series** | Series Module | Modality, SeriesNumber, SeriesDescription, SeriesDate, BodyPartExamined |
| **Instance** | Image/Instance Module | InstanceNumber, ImagePosition, ImageOrientation, SliceLocation, WindowCenter |
| **Equipment** | Equipment Module (0018,1xxx) | Manufacturer, ManufacturerModelName, SoftwareVersions, StationName |

The system handles all 6000+ tags in the DICOM standard by using group-based patterns, so even rare tags are correctly classified.
