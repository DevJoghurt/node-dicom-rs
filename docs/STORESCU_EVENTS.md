# StoreSCU Event System

The StoreSCU now provides real-time visibility into file transfer operations through an event-driven API.

## Events

### OnTransferStarted
Fired when the association is established and transfer begins.

**Event Data:**
```json
{
  "address": "localhost:4446",
  "callingAeTitle": "STORE-SCU",
  "totalFiles": 10
}
```

### OnFileSending
Fired before each file is sent.

**Event Data:**
```json
{
  "file": "s3://bucket/path/file.dcm",
  "sopInstanceUid": "1.2.3.4.5.6...",
  "sopClassUid": "1.2.840.10008.5.1.4.1.1.2",
  "transferSyntax": "1.2.840.10008.1.2.1"
}
```

### OnFileSent
Fired after a file is successfully sent.

**Event Data:**
```json
{
  "file": "s3://bucket/path/file.dcm",
  "sopInstanceUid": "1.2.3.4.5.6...",
  "sopClassUid": "1.2.840.10008.5.1.4.1.1.2",
  "transferSyntax": "1.2.840.10008.1.2.1",
  "durationMs": 245,
  "durationSeconds": 0.245,
  "status": "success"
}
```

### OnFileError
Fired when a file fails to send.

**Event Data:**
```json
{
  "file": "s3://bucket/path/file.dcm",
  "sopInstanceUid": "1.2.3.4.5.6...",
  "sopClassUid": "1.2.840.10008.5.1.4.1.1.2",
  "statusCode": "C000H",
  "durationMs": 125,
  "error": "Status code C000H"
}
```

### OnTransferCompleted
Fired when all files have been processed.

**Event Data:**
```json
{
  "totalFiles": 10,
  "status": "completed"
}
```

### OnError
Fired for general errors during the transfer process.

**Event Data:**
```json
{
  "message": "Error description",
  "data": "Optional additional error information"
}
```

## Usage Example

```javascript
import { StoreScu } from 'node-dicom-rs'

const scu = new StoreScu({
    addr: 'STORE-SCP@localhost:4446',
    verbose: false,
    concurrency: 2
})

// Monitor transfer progress
scu.addEventListener('OnTransferStarted', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`Starting transfer of ${data.totalFiles} files to ${data.address}`)
})

scu.addEventListener('OnFileSending', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`Sending: ${data.file}`)
})

scu.addEventListener('OnFileSent', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`✓ Sent in ${data.durationSeconds.toFixed(3)}s`)
})

scu.addEventListener('OnFileError', (error, event) => {
    const data = JSON.parse(event.data)
    console.error(`✗ Failed: ${data.file} - ${data.error}`)
})

scu.addEventListener('OnTransferCompleted', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`Transfer completed: ${data.totalFiles} files`)
})

// Add files and send
scu.addFolder('./dicom-files')
await scu.send()
```

## S3 Support with Events

The event system works seamlessly with S3 storage:

```javascript
scu.s3Config = {
    bucket: 'my-dicom-bucket',
    accessKey: 'access-key',
    secretKey: 'secret-key',
    endpoint: 'http://localhost:7070'
}

scu.addFolder('./study-folder')  // S3 prefix

// Events will include s3:// URIs in file paths
scu.addEventListener('OnFileSent', (error, event) => {
    const data = JSON.parse(event.data)
    // data.file will be like "s3://my-dicom-bucket/study-folder/file.dcm"
})

await scu.send()
```

## Performance Tracking

Use the timing information from events to track performance:

```javascript
const timings = []

scu.addEventListener('OnFileSent', (error, event) => {
    const data = JSON.parse(event.data)
    timings.push(data.durationMs)
})

scu.addEventListener('OnTransferCompleted', (error, event) => {
    const avg = timings.reduce((a, b) => a + b, 0) / timings.length
    const max = Math.max(...timings)
    const min = Math.min(...timings)
    
    console.log(`Average: ${avg.toFixed(2)}ms`)
    console.log(`Min: ${min}ms, Max: ${max}ms`)
})

await scu.send()
```
