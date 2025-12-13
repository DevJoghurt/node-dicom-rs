# onBeforeStore Callback Feature

## Overview

The `onBeforeStore` callback feature allows you to intercept and modify DICOM tags **before** they are saved to disk. This enables powerful use cases like:
- **Anonymization**: Remove or replace patient identifying information
- **Validation**: Reject files that don't meet specific criteria
- **Tag normalization**: Standardize tag formats across different sources
- **Audit logging**: Track and record what's being stored

## How It Works

1. When a DICOM file is received by StoreScp
2. Tags are extracted (according to `extractTags` configuration)
3. **Before saving**, the `onBeforeStore` callback is invoked with the extracted tags
4. Your callback can modify the tags (synchronously)
5. The modified tags are applied back to the DICOM object
6. The updated DICOM file is saved to disk

## API

### TypeScript/JavaScript

```typescript
const scp = new StoreScp({
  port: 11115,
  outDir: './received-files',
  storeWithFileMeta: true, // Important: ensure full DICOM file with meta header
  extractTags: [
    'PatientName',
    'PatientID',
    'PatientBirthDate',
    'PatientSex',
    'StudyDescription'
  ]
});

// Register the callback
scp.onBeforeStore((tags) => {
  // Modify tags as needed
  return {
    ...tags,
    PatientName: 'ANONYMOUS^PATIENT',
    PatientID: generateAnonymousId(tags.PatientID)
  };
});

await scp.listen();
```

### Callback Signature

```typescript
type OnBeforeStoreCallback = (tags: Record<string, string>) => Record<string, string>;
```

**Parameters:**
- `tags`: Object containing the extracted DICOM tags as key-value pairs

**Returns:**
- Modified tags object with the same structure

**Important Notes:**
1. The callback is **synchronous** - it blocks file saving
2. Only tags specified in `extractTags` configuration are available
3. You must return a complete tags object (can include the same values if no modification needed)
4. If `extractTags` is empty or not configured, the callback won't be invoked

## Configuration Requirements

To use the `onBeforeStore` callback, you must:

1. **Configure `extractTags`**: Specify which tags to extract
   ```typescript
   extractTags: ['PatientName', 'PatientID', 'StudyDescription']
   ```

2. **Enable `storeWithFileMeta`**: Ensure files are saved with DICOM meta header
   ```typescript
   storeWithFileMeta: true
   ```

3. **Register the callback** before calling `listen()`

## Example Use Cases

### 1. Anonymization

```javascript
const patientMapping = new Map();
let anonymousCounter = 1000;

scp.onBeforeStore((tags) => {
  // Generate or retrieve anonymous ID
  let anonymousID = patientMapping.get(tags.PatientID);
  if (!anonymousID) {
    anonymousID = `ANON_${String(anonymousCounter++).padStart(4, '0')}`;
    patientMapping.set(tags.PatientID, anonymousID);
  }

  return {
    ...tags,
    PatientName: 'ANONYMOUS^PATIENT',
    PatientID: anonymousID,
    PatientBirthDate: '',
    PatientSex: '',
    StudyDescription: tags.StudyDescription 
      ? `ANONYMIZED - ${tags.StudyDescription}` 
      : 'ANONYMIZED STUDY'
  };
});
```

### 2. Validation

```javascript
scp.onBeforeStore((tags) => {
  // Validate required fields
  if (!tags.PatientID || !tags.StudyInstanceUID) {
    throw new Error('Missing required tags');
  }
  
  // Validate format
  if (!/^\d+$/.test(tags.PatientID)) {
    throw new Error('Invalid PatientID format');
  }
  
  return tags;
});
```

### 3. Tag Normalization

```javascript
scp.onBeforeStore((tags) => {
  return {
    ...tags,
    // Normalize patient name to uppercase
    PatientName: tags.PatientName?.toUpperCase() || '',
    // Ensure consistent date format
    PatientBirthDate: formatDate(tags.PatientBirthDate),
    // Standardize study description
    StudyDescription: normalizeDescription(tags.StudyDescription)
  };
});
```

## Demo Scripts

Three demo scripts are included:

### 1. Anonymization Demo
**File:** `playground/demos/storescp-anonymization-demo.mjs`

Demonstrates real-time anonymization of incoming DICOM files.

```bash
node playground/demos/storescp-anonymization-demo.mjs
```

### 2. Validation Demo  
**File:** `playground/demos/storescp-validation-demo.mjs`

Shows how to validate and reject files based on tag criteria.

```bash
node playground/demos/storescp-validation-demo.mjs
```

### 3. Send Test Files
**File:** `playground/demos/send-to-anon.mjs`

Client script to send test files to the demo servers.

```bash
node playground/demos/send-to-anon.mjs
```

### 4. Verify Anonymization
**File:** `playground/demos/verify-anonymization.mjs`

Reads stored files to verify anonymization was applied.

```bash
node playground/demos/verify-anonymization.mjs
```

### 5. Automated Test
**File:** `playground/demos/final-test.sh`

Complete automated test: start server → send files → verify results.

```bash
bash playground/demos/final-test.sh
```

## Implementation Details

### Rust Side

The callback is implemented as a `ThreadsafeFunction` that can be safely called from the Rust async runtime:

```rust
pub struct StoreScp {
    // ... other fields
    on_before_store: Option<Arc<ThreadsafeFunction<HashMap<String, String>, HashMap<String, String>>>>,
}

#[napi]
impl StoreScp {
    #[napi]
    pub fn onBeforeStore(&mut self, callback: JsFunction) -> NapiResult<()> {
        let tsfn: ThreadsafeFunction<HashMap<String, String>, HashMap<String, String>> = 
            callback.create_threadsafe_function(0, |ctx| {
                let tags = ctx.value;
                Ok(vec![tags])
            })?;
        self.on_before_store = Some(Arc::new(tsfn));
        Ok(())
    }
}
```

### Synchronous Invocation

The callback is invoked synchronously using a oneshot channel:

```rust
let (tx, rx) = oneshot::channel();
let _ = callback_arc.call_with_return_value(
    extracted_tags.clone(),
    ThreadsafeFunctionCallMode::Blocking,
    move |modified_tags: HashMap<String, String>| {
        tx.send(modified_tags).ok();
        Ok(())
    },
);

// Wait for modified tags
let modified_tags = rx.await?;
```

## Performance Considerations

1. **Synchronous blocking**: The callback blocks file saving, so keep operations fast
2. **Thread safety**: The callback is called from async Rust context via ThreadsafeFunction
3. **Tag extraction overhead**: Only extract tags you need to modify
4. **Memory**: Each callback invocation clones the tags HashMap

## Troubleshooting

### Callback Not Being Invoked

1. **Check `extractTags` configuration**: If empty, callback won't run
2. **Verify callback registration**: Must call `onBeforeStore()` before `listen()`
3. **Enable logging**: Set `RUST_LOG=info` to see callback invocation logs

### Tags Not Being Modified

1. **Check return value**: Must return a complete tags object
2. **Verify tag names**: Use exact DICOM tag names (case-sensitive)
3. **Check `storeWithFileMeta`**: Must be `true` for proper file saving

### Errors in Callback

If the callback throws an error, the file will not be saved and the client will receive an error response.

## Testing

Run the complete test suite:

```bash
# Clean previous results
rm -rf playground/test-received-anon

# Run automated test
bash playground/demos/final-test.sh

# Manually verify files
node playground/demos/verify-anonymization.mjs
```

Expected output:
```
✅ SUCCESS! File was anonymized!
PatientName: ANONYMOUS^PATIENT
PatientID: ANON_1000
PatientBirthDate: 
PatientSex: 
StudyDescription: ANONYMIZED - CT/CH/ABD/PEL(PANC)
```

## Future Enhancements

Potential improvements:
- Async callback support for database lookups
- Batch callback invocation for performance
- Callback for metadata only (without tag modification)
- Pre-save validation hooks separate from modification
- Support for binary tag modifications (not just strings)
