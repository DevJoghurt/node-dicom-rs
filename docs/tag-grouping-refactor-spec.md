# Specification: Simplified Tag Grouping API

## Overview
Simplify the tag extraction API by removing multiple grouping strategies and providing a clear, type-safe interface with consistent behavior across all events.

## Current Problems
1. Multiple optional tag fields (`tagsScoped`, `tagsFlat`, `tagsStudyLevel`) create confusion
2. Impossible to provide clean TypeScript types
3. Users must check multiple optional fields to find data
4. Inconsistent API between events

## New Design

### 1. Remove `groupingStrategy` Option
- **Remove from**: `StoreScp` constructor options
- **Remove from**: `DicomFile.extract()` method
- **Remove enum**: `GroupingStrategy`

### 2. OnFileStored Event - Flat Tags

**Structure:**
```typescript
interface FileStoredEvent {
  message: string;
  file: string;
  sopInstanceUid: string;
  studyInstanceUid: string;
  seriesInstanceUid: string;
  tags: Record<string, any>;  // Flat object with all extracted tags
}
```

**Example:**
```typescript
{
  message: "File stored successfully",
  file: "/path/to/file.dcm",
  sopInstanceUid: "1.2.3...",
  studyInstanceUid: "1.2.3...",
  seriesInstanceUid: "1.2.3...",
  tags: {
    PatientName: "DOE^JOHN",
    PatientID: "12345",
    StudyDate: "20231201",
    Modality: "CT",
    SeriesNumber: "1",
    InstanceNumber: "5",
    Manufacturer: "GE"
  }
}
```

**Rationale:**
- Simple, direct access: `event.tags.PatientName`
- UIDs provide hierarchy context
- Fast to use for single-file processing
- Clean TypeScript types

### 3. OnStudyCompleted Event - Hierarchical with Flat Tags at Each Level

**Structure:**
```typescript
interface StudyCompletedEvent {
  message: string;
  study: {
    studyInstanceUid: string;
    tags: Record<string, any>;  // Patient + Study level tags only
    series: Array<{
      seriesInstanceUid: string;
      tags: Record<string, any>;  // Series level tags only
      instances: Array<{
        sopInstanceUid: string;
        file: string;
        tags: Record<string, any>;  // Instance + Equipment level tags only
      }>;
    }>;
  };
}
```

**Tag Distribution by DICOM Scope:**

| Level | Contains | Example Tags |
|-------|----------|--------------|
| **Study** | Patient scope (0010,xxxx) + Study scope tags | PatientName, PatientID, StudyDate, StudyDescription, AccessionNumber |
| **Series** | Series scope tags only | Modality, SeriesNumber, SeriesDescription, SeriesDate, BodyPartExamined |
| **Instance** | Instance scope + Equipment scope (0018,1xxx) | InstanceNumber, SliceLocation, Manufacturer, SoftwareVersions |

**Example:**
```typescript
{
  message: "Study completed",
  study: {
    studyInstanceUid: "1.2.3...",
    tags: {
      PatientName: "DOE^JOHN",
      PatientID: "12345",
      StudyDate: "20231201",
      StudyDescription: "CT Chest"
    },
    series: [{
      seriesInstanceUid: "1.2.3.4...",
      tags: {
        Modality: "CT",
        SeriesNumber: "1",
        SeriesDescription: "Chest Routine"
      },
      instances: [{
        sopInstanceUid: "1.2.3.4.5...",
        file: "/path/to/file.dcm",
        tags: {
          InstanceNumber: "1",
          SliceThickness: "5.0",
          Manufacturer: "GE",
          ManufacturerModelName: "Discovery CT750"
        }
      }]
    }]
  }
}
```

**Rationale:**
- Hierarchy structure provides organization
- Flat tags at each level = simple access
- No data duplication (tags only at appropriate level)
- Clean TypeScript types
- Easy iteration

### 4. DicomFile.extract() Method

**Simplify to flat structure**:
- Remove `groupingStrategy` parameter
- Always return **flat** object with all requested tags
- Consistent with OnFileStored event for single-file processing

**New Signature:**
```typescript
extract(
  tags: string[],
  customTags?: Array<{ tag: string; name: string }>
): string  // Returns JSON with flat tag structure
```

**Example:**
```typescript
const json = file.extract(['PatientName', 'StudyDate', 'Modality', 'InstanceNumber']);
const data = JSON.parse(json);
// {
//   PatientName: "DOE^JOHN",
//   StudyDate: "20231201",
//   Modality: "CT",
//   InstanceNumber: "1"
// }
```

**Rationale:**
- Consistent with OnFileStored event (both are single-file operations)
- Simple, direct access: `data.PatientName`
- No unnecessary nesting for manual tag extraction
- Clean TypeScript types

## Implementation Tasks

### Rust Changes

1. **Remove from `src/storescp/mod.rs`:**
   - Remove `GroupingStrategy` enum
   - Remove `grouping_strategy` field from `StoreScpOptions`
   - Remove `grouping_strategy` field from `StoreScp` struct

2. **Update `src/storescp/mod.rs` event structures:**
   - Change `FileStoredEventData` to have single `tags: Option<serde_json::Value>` field
   - Change `StudyHierarchy` to have `tags: Option<serde_json::Value>` at each level
   - Remove `tags_scoped`, `tags_flat`, `tags_study_level` fields

3. **Update `src/object/mod.rs`:**
   - Change `extract()` method to remove `strategy` parameter
   - Always return flat structure (all tags in single object)
   - Update JSDoc comments

4. **Update tag extraction logic:**
   - For `OnFileStored`: Extract all tags into flat structure
   - For `OnStudyCompleted`: Distribute tags by scope:
     - Study level: Patient (0010,xxxx) + Study tags
     - Series level: Series tags
     - Instance level: Instance + Equipment (0018,1xxx) tags

### TypeScript Changes

1. **Update `index.d.ts`:**
   - Remove `GroupingStrategy` enum export
   - Update `StoreScpOptions` interface (remove `groupingStrategy`)
   - Update `FileStoredEventData` interface
   - Update `StudyHierarchy`, `SeriesHierarchy`, `InstanceHierarchy` interfaces
   - Update `DicomFile.extract()` signature

### Documentation Changes

1. **Update docs/storescp.md:**
   - Remove grouping strategy section
   - Update event examples
   - Show new simpler API

2. **Update docs/tag-extraction.md:**
   - Remove grouping strategy comparison
   - Focus on scope-based tag classification
   - Update examples

3. **Update docs/dicomfile.md:**
   - Update `extract()` method documentation
   - Remove strategy parameter examples

4. **Update README.md:**
   - Update example code
   - Simplify quick start examples

5. **Update playground/server.mjs:**
   - Remove grouping strategy option
   - Update event handlers to use new structure

## Benefits

1. **Clean TypeScript Types**: Single tag field at each level, properly typed
2. **Simpler API**: No need to check multiple optional fields
3. **Better DX**: Autocomplete works perfectly
4. **Consistent**: Flat tags for single-file operations (DicomFile.extract() and OnFileStored), hierarchical only where it adds value (OnStudyCompleted)
5. **DICOM-aligned**: OnStudyCompleted follows natural DICOM hierarchy
6. **No Duplication**: Tags only at appropriate level in hierarchy
7. **Easy to Use**: Direct access for single files, organized structure for multi-file studies

## Migration Guide for Users

**Before:**
```typescript
const scp = new StoreScp({
  groupingStrategy: 'ByScope'
});

scp.onFileStored((err, event) => {
  console.log(event.data.tagsScoped?.patient?.PatientName);
});
```

**After:**
```typescript
const scp = new StoreScp({
  // No groupingStrategy option
});

scp.onFileStored((err, event) => {
  console.log(event.tags.PatientName);  // Simple, direct access
});
```

**OnStudyCompleted Before:**
```typescript
scp.onStudyCompleted((err, event) => {
  const study = event.data?.study;
  // Had to check tagsScoped, tagsFlat, or tagsStudyLevel
  console.log(study?.tagsScoped?.patient?.PatientName);
  
  for (const series of study?.series || []) {
    console.log(series.tagsScoped?.series?.Modality);
  }
});
```

**OnStudyCompleted After:**
```typescript
scp.onStudyCompleted((err, event) => {
  const study = event.study;
  // Simple, always in same place
  console.log(study.tags.PatientName);
  
  for (const series of study.series) {
    console.log(series.tags.Modality);
    
    for (const instance of series.instances) {
      console.log(instance.tags.InstanceNumber);
      console.log(instance.tags.Manufacturer);  // Equipment tags here
    }
  }
});
```

**DicomFile.extract() Before:**
```typescript
const json = file.extract(['PatientName', 'StudyDate'], undefined, 'ByScope');
const data = JSON.parse(json);
// { patient: { PatientName: "..." }, study: { StudyDate: "..." } }
```

**DicomFile.extract() After:**
```typescript
const json = file.extract(['PatientName', 'StudyDate']);  // No strategy parameter
const data = JSON.parse(json);  // Always flat structure
// { PatientName: "DOE^JOHN", StudyDate: "20231201" }

// Simple, direct access - consistent with OnFileStored
console.log(data.PatientName);
```

## Testing Requirements

1. **Unit Tests**:
   - Verify flat tag extraction for OnFileStored
   - Verify hierarchical tag distribution for OnStudyCompleted
   - Verify DicomFile.extract() always returns flat structure
   - Test with various tag combinations across all scopes

2. **Integration Tests**:
   - Test complete SCP workflow with new event structure
   - Test study completion with multiple series/instances
   - Verify no data duplication in hierarchy
   - Test custom tags still work correctly

3. **Type Tests**:
   - Verify TypeScript types compile correctly
   - Verify autocomplete works in IDE
   - No optional chaining needed for tag access

## Timeline

1. **Phase 1**: Rust implementation (2-3 days)
   - Remove enum and options
   - Update event structures
   - Update tag extraction logic
   
2. **Phase 2**: TypeScript definitions (1 day)
   - Update index.d.ts
   - Ensure clean types
   
3. **Phase 3**: Documentation (1 day)
   - Update all docs
   - Create migration guide
   
4. **Phase 4**: Testing (1-2 days)
   - Write comprehensive tests
   - Update playground examples
   
**Total estimate**: 5-7 days
