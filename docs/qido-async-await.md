# QIDO-RS Async/Await Support

The QIDO-RS server in `node-dicom-rs` now supports both **synchronous** and **asynchronous** callback handlers, allowing you to integrate with databases, APIs, and other async operations seamlessly.

## Quick Start

```javascript
import { QidoServer } from 'node-dicom-rs'

const qido = new QidoServer(8888)

// Synchronous handler - must use Promise.resolve()
qido.onSearchForStudies((err, query) => {
  if (err) throw err
  const data = getFromCache(query)
  return Promise.resolve(JSON.stringify(data))
})

// Asynchronous handler - use async/await
qido.onSearchForSeries(async (err, query) => {
  if (err) throw err
  const data = await database.query('SELECT ...')
  return JSON.stringify(data)
})

qido.start()
```

## Callback Patterns

### Pattern 1: Synchronous with Promise.resolve()

Use when data is immediately available (in-memory cache, static data):

```javascript
qido.onSearchForStudies((err, query) => {
  if (err) throw err
  
  // Immediate data lookup
  const studies = memoryCache.get(query.patientId)
  
  // ✅ REQUIRED: Must wrap in Promise.resolve()
  return Promise.resolve(JSON.stringify(studies))
  
  // ❌ WRONG: This will fail with "InvalidArg" error
  // return JSON.stringify(studies)
})
```

**Why Promise.resolve()?** The Rust implementation uses NAPI-RS's `ThreadsafeFunction<T, Promise<R>>` type, which always expects a Promise return value. For synchronous callbacks, you must explicitly wrap the result.

### Pattern 2: Async/Await

Use when you need to perform asynchronous operations:

```javascript
qido.onSearchForSeries(async (err, query) => {
  if (err) throw err
  
  // Await database query
  const series = await db.query(
    'SELECT * FROM series WHERE study_uid = $1',
    [query.studyInstanceUid]
  )
  
  // Transform to DICOM JSON
  const dicomJson = series.map(s => ({
    "0020000E": { "vr": "UI", "Value": [s.series_uid] },
    "00080060": { "vr": "CS", "Value": [s.modality] }
  }))
  
  // Async functions automatically return a Promise
  return JSON.stringify(dicomJson)
})
```

### Pattern 3: Multiple Async Operations

Use `Promise.all()` for parallel async operations:

```javascript
qido.onSearchForStudyInstances(async (err, query) => {
  if (err) throw err
  
  // Run multiple queries in parallel
  const [instances, metadata, permissions] = await Promise.all([
    db.query('SELECT * FROM instances WHERE study_uid = $1', [query.studyInstanceUid]),
    cache.get(`metadata:${query.studyInstanceUid}`),
    checkUserPermissions(query.studyInstanceUid)
  ])
  
  // Process and return
  const filtered = instances.filter(inst => permissions.canAccess(inst.id))
  return JSON.stringify(filtered)
})
```

## Error Handling

Errors thrown in callbacks are caught by Rust and returned as HTTP 500 responses:

```javascript
qido.onSearchForStudies(async (err, query) => {
  if (err) throw err
  
  try {
    const data = await database.query(...)
    return JSON.stringify(data)
  } catch (dbError) {
    console.error('Database error:', dbError)
    // This error will be caught by Rust
    throw new Error(`Database query failed: ${dbError.message}`)
  }
})
```

The client will receive:

```json
{
  "error": "Promise rejected: Error { message: \"Database query failed: ...\" }"
}
```

## Real-World Example

Here's a complete example using a PostgreSQL database with connection pooling:

```javascript
import { QidoServer } from 'node-dicom-rs'
import pg from 'pg'

const pool = new pg.Pool({
  host: 'localhost',
  database: 'pacs',
  user: 'pacs_user',
  password: process.env.DB_PASSWORD,
  max: 20 // Connection pool size
})

const qido = new QidoServer(8080, {
  enableCors: true,
  corsAllowedOrigins: 'https://viewer.example.com'
})

// Search for Studies
qido.onSearchForStudies(async (err, query) => {
  if (err) throw err
  
  const client = await pool.connect()
  try {
    const result = await client.query(`
      SELECT 
        study_instance_uid,
        patient_name,
        patient_id,
        study_date,
        study_description,
        modalities_in_study
      FROM studies
      WHERE 
        ($1::text IS NULL OR patient_id ILIKE $1)
        AND ($2::text IS NULL OR patient_name ILIKE $2)
        AND ($3::text IS NULL OR study_date >= $3)
        AND ($4::text IS NULL OR study_date <= $4)
      ORDER BY study_date DESC
      LIMIT 100
    `, [
      query.patientId || null,
      query.patientName ? `%${query.patientName}%` : null,
      query.studyDateFrom || null,
      query.studyDateTo || null
    ])
    
    // Convert to DICOM JSON format
    const studies = result.rows.map(row => ({
      "0020000D": { "vr": "UI", "Value": [row.study_instance_uid] },
      "00100010": { "vr": "PN", "Value": [{ Alphabetic: row.patient_name }] },
      "00100020": { "vr": "LO", "Value": [row.patient_id] },
      "00080020": { "vr": "DA", "Value": [row.study_date] },
      "00081030": { "vr": "LO", "Value": [row.study_description] },
      "00080061": { "vr": "CS", "Value": row.modalities_in_study.split(',') }
    }))
    
    return JSON.stringify(studies)
    
  } finally {
    client.release()
  }
})

// Search for Series
qido.onSearchForSeries(async (err, query) => {
  if (err) throw err
  
  const client = await pool.connect()
  try {
    const result = await client.query(`
      SELECT 
        series_instance_uid,
        modality,
        series_number,
        series_description,
        number_of_instances
      FROM series
      WHERE study_instance_uid = $1
      ORDER BY series_number
    `, [query.studyInstanceUid])
    
    const series = result.rows.map(row => ({
      "0020000E": { "vr": "UI", "Value": [row.series_instance_uid] },
      "00080060": { "vr": "CS", "Value": [row.modality] },
      "00200011": { "vr": "IS", "Value": [row.series_number.toString()] },
      "0008103E": { "vr": "LO", "Value": [row.series_description] },
      "00201209": { "vr": "IS", "Value": [row.number_of_instances.toString()] }
    }))
    
    return JSON.stringify(series)
    
  } finally {
    client.release()
  }
})

qido.start()
console.log('QIDO-RS server with PostgreSQL backend running on port 8080')
```

## Performance Considerations

### 1. Connection Pooling
Always use connection pooling for database connections:

```javascript
// ✅ Good: Connection pool
const pool = new pg.Pool({ max: 20 })
const client = await pool.connect()
// ... use client ...
client.release()

// ❌ Bad: New connection per request
const client = new pg.Client()
await client.connect()
```

### 2. Async vs Sync Choice

| Use Synchronous | Use Asynchronous |
|----------------|------------------|
| In-memory cache | Database queries |
| Static data | File I/O |
| Pre-loaded config | External API calls |
| Simple transformations | Authentication checks |

### 3. Caching Strategy

```javascript
const cache = new Map()

qido.onSearchForStudies(async (err, query) => {
  if (err) throw err
  
  const cacheKey = JSON.stringify(query)
  
  // Check cache first
  if (cache.has(cacheKey)) {
    const cached = cache.get(cacheKey)
    return Promise.resolve(JSON.stringify(cached))
  }
  
  // Not in cache, query database
  const studies = await database.query(...)
  
  // Cache for 5 minutes
  cache.set(cacheKey, studies)
  setTimeout(() => cache.delete(cacheKey), 5 * 60 * 1000)
  
  return JSON.stringify(studies)
})
```

## Implementation Details

Under the hood, the QIDO-RS handlers use NAPI-RS's `ThreadsafeFunction` with async support:

```rust
// Rust implementation (simplified)
type SearchHandler = ThreadsafeFunction<Query, Promise<String>>;

async fn handle_request(query: Query, handler: Arc<SearchHandler>) {
    // Call JavaScript callback
    let promise = handler.call_async(Ok(query));
    
    // Await the Promise (works for both sync and async callbacks)
    match promise.await {
        Ok(json_future) => {
            match json_future.await {
                Ok(json_string) => { /* return response */ }
                Err(e) => { /* handle promise rejection */ }
            }
        }
        Err(e) => { /* handle callback error */ }
    }
}
```

This pattern automatically handles:
- **Synchronous callbacks**: JavaScript wraps return value in `Promise.resolve()`
- **Asynchronous callbacks**: JavaScript returns a `Promise` directly
- **Error handling**: Both JavaScript exceptions and Promise rejections are caught

## Migration from Synchronous

If you have existing synchronous handlers:

```javascript
// Old synchronous code
qido.onSearchForStudies((err, query) => {
  const data = getData(query)
  return JSON.stringify(data)  // ❌ This will now fail
})
```

Update to:

```javascript
// New code with Promise.resolve()
qido.onSearchForStudies((err, query) => {
  const data = getData(query)
  return Promise.resolve(JSON.stringify(data))  // ✅ Works
})
```

Or convert to async if you need database access:

```javascript
// Convert to async
qido.onSearchForStudies(async (err, query) => {
  const data = await database.query(...)
  return JSON.stringify(data)  // ✅ Works (async auto-wraps)
})
```

## Troubleshooting

### Error: "Call the PromiseRaw::then failed"

**Cause**: Callback returned a non-Promise value.

**Solution**: Wrap synchronous returns in `Promise.resolve()`:

```javascript
// ❌ Wrong
return JSON.stringify(data)

// ✅ Correct
return Promise.resolve(JSON.stringify(data))
```

### Error: "Promise rejected: ..."

**Cause**: JavaScript callback threw an error or Promise was rejected.

**Solution**: Check your error handling:

```javascript
qido.onSearchForStudies(async (err, query) => {
  if (err) throw err  // Handle QIDO errors
  
  try {
    const data = await database.query(...)
    return JSON.stringify(data)
  } catch (dbError) {
    console.error('Database error:', dbError)
    throw dbError  // Will be caught by Rust
  }
})
```

## Examples

See complete examples:
- [examples/qido-async-patterns.mjs](../examples/qido-async-patterns.mjs) - All patterns demonstrated
- [__test__/index.spec.ts](../__test__/index.spec.ts) - Test suite
- [playground/demos/storescp-validation-demo.mjs](../playground/demos/storescp-validation-demo.mjs) - Real-world usage

## References

- [NAPI-RS Function and Callbacks Guide](https://napi.rs/blog/function-and-callbacks)
- [DICOM QIDO-RS Standard](https://www.dicomstandard.org/using/dicomweb/query-qido-rs)
- [DICOM JSON Model](https://www.dicomstandard.org/using/dicomweb/dicom-json-format)
