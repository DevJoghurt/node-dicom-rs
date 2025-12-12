# StoreScu - DICOM C-STORE SCU Client

The `StoreScu` class implements a DICOM C-STORE Service Class User (SCU) client that sends DICOM files to remote PACS systems.

## Basic Usage

```typescript
import { StoreScu } from '@nuxthealth/node-dicom';

const sender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'MY-SCU',
    calledAeTitle: 'REMOTE-SCP'
});

// Add files
sender.addFile('./scan.dcm');

// Send with optional callbacks
const result = await sender.send({
    onFileSent: (err, event) => {
        console.log('✓ File sent:', event.sopInstanceUid);
    }
});
console.log('Transfer complete:', result);
```

## Configuration Options

### Network Settings

```typescript
{
    addr: '192.168.1.100:104',     // Remote host:port (required)
    callingAeTitle: 'MY-SCU',      // Your AE Title (default: 'STORE-SCU')
    calledAeTitle: 'REMOTE-SCP',   // Remote AE Title (default: 'ANY-SCP')
    maxPduLength: 16384,           // Maximum PDU length (default: 16384)
    verbose: true                  // Enable verbose logging (default: false)
}
```

## Adding Files

### Single File

```typescript
sender.addFile('/path/to/file.dcm');
```

### Multiple Files

```typescript
sender.addFile('/path/to/scan1.dcm');
sender.addFile('/path/to/scan2.dcm');
sender.addFile('/path/to/scan3.dcm');
```

### Directory (Recursive)

```typescript
sender.addDirectory('/path/to/dicom/folder');
```

This recursively scans the directory and adds all `.dcm` files.

### From Memory/Buffer

```typescript
const dicomData = fs.readFileSync('./scan.dcm');
sender.addFileFromMemory(dicomData, 'scan.dcm');
```

## Sending Files

### Basic Send

```typescript
const result = await sender.send();

console.log('Summary:');
console.log(`  Successful: ${result.successful}`);
console.log(`  Failed: ${result.failed}`);
console.log(`  Warnings: ${result.warnings.length}`);
```

Result structure:
```typescript
{
    successful: number,    // Number of successfully sent files
    failed: number,        // Number of failed transfers
    warnings: string[],    // List of warning messages
    totalFiles: number     // Total files attempted
}
```

### With Progress Tracking

```typescript
const result = await sender.send({
    onFileSent: (err, event) => {
        console.log('✓ Sent:', event.sopInstanceUid);
    },
    onFileError: (err, event) => {
        console.error('✗ Failed:', event.message);
        console.error('  Error:', event.error);
    },
    onTransferCompleted: (err, event) => {
        console.log('All files transferred!', event.totalFiles);
    }
});
```

### Batch Processing with Limits

```typescript
// Process in batches of 10 concurrent transfers
for (let i = 0; i < files.length; i += 10) {
    const batch = files.slice(i, i + 10);
    const sender = new StoreScu({ /* config */ });
    
    batch.forEach(file => sender.addFile(file));
    
    const result = await sender.send();
    console.log(`Batch ${Math.floor(i/10) + 1}: ${result.successful}/${batch.length} sent`);
}
```

## Callbacks

Callbacks are passed to the `send()` method as an object with optional callback functions. All callbacks follow the Node.js error-first pattern: `(err: Error | null, event: EventType) => void`.

### onTransferStarted

Called once when the transfer operation begins (before any files are sent).

```typescript
await sender.send({
    onTransferStarted: (err, event) => {
        console.log(event.message); // "Transfer started"
        console.log('Total files:', event.totalFiles);
    }
});
```

Event data structure:
```typescript
{
    message: string,        // Human-readable message
    totalFiles: number      // Total number of files to transfer
}
```

### onFileSending

Called when a file is about to be sent.

```typescript
await sender.send({
    onFileSending: (err, event) => {
        console.log(`Sending: ${event.file}`);
        console.log('SOP Class:', event.sopClassUid);
        console.log('SOP Instance:', event.sopInstanceUid);
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    file: string,              // File path (local or S3)
    sopInstanceUid: string,    // SOP Instance UID
    sopClassUid: string        // SOP Class UID
}
```

### onFileSent

Called when a file is successfully sent.

```typescript
await sender.send({
    onFileSent: (err, event) => {
        console.log(event.message); // "File sent successfully"
        console.log('File:', event.file);
        console.log('SOP Instance UID:', event.sopInstanceUid);
        console.log('SOP Class UID:', event.sopClassUid);
        console.log('Transfer Syntax:', event.transferSyntax);
        console.log('Duration:', event.durationSeconds, 'seconds');
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    file: string,              // File path (local or S3)
    sopInstanceUid: string,    // SOP Instance UID
    sopClassUid: string,       // SOP Class UID
    transferSyntax: string,    // Transfer Syntax UID used
    durationSeconds: number    // Transfer duration in seconds
}
```

### onFileError

Called when a file transfer fails.

```typescript
await sender.send({
    onFileError: (err, event) => {
        console.error('Error:', event.message);
        console.error('File:', event.file);
        console.error('Details:', event.error);
        if (event.sopInstanceUid) {
            console.error('SOP Instance UID:', event.sopInstanceUid);
            console.error('SOP Class UID:', event.sopClassUid);
            console.error('File Transfer Syntax:', event.fileTransferSyntax);
        }
    }
});
```

Event data structure:
```typescript
{
    message: string,               // Error message
    file: string,                  // File path that failed
    error: string,                 // Detailed error information
    sopInstanceUid?: string,       // SOP Instance UID (if available)
    sopClassUid?: string,          // SOP Class UID (if available)
    fileTransferSyntax?: string    // Original file transfer syntax (if available)
}
```

### onTransferCompleted

Called once when all files have been transferred.

```typescript
await sender.send({
    onTransferCompleted: (err, event) => {
        console.log('All files transferred!');
        console.log(`Total: ${event.totalFiles} files`);
        console.log(`Successful: ${event.successful} files`);
        console.log(`Failed: ${event.failed} files`);
        console.log(`Duration: ${event.durationSeconds.toFixed(2)}s`);
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    totalFiles: number,        // Total number of files attempted
    successful: number,        // Number of successfully transferred files
    failed: number,            // Number of failed transfers
    durationSeconds: number    // Total transfer duration in seconds
}
```

## Transfer Syntax Selection

### Auto-Select (Default)

By default, StoreScu uses the original transfer syntax from each file.

```typescript
const sender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'MY-SCU'
    // Will use original transfer syntax from each file
});
```

### Specify Transfer Syntax

Force all files to be sent with a specific transfer syntax:

```typescript
const sender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'MY-SCU',
    transferSyntax: 'ImplicitVRLittleEndian'  // Force uncompressed
});
```

Common transfer syntaxes:
- `'ImplicitVRLittleEndian'` - Uncompressed (1.2.840.10008.1.2)
- `'ExplicitVRLittleEndian'` - Uncompressed (1.2.840.10008.1.2.1)
- `'JPEGBaseline'` - JPEG lossy (1.2.840.10008.1.2.4.50)
- `'JPEG2000Lossless'` - JPEG 2000 lossless (1.2.840.10008.1.2.4.90)

Or use UIDs directly:
```typescript
transferSyntax: '1.2.840.10008.1.2'  // Implicit VR Little Endian
```

## Error Handling

### Connection Errors

```typescript
try {
    const result = await sender.send();
} catch (error) {
    if (error.message.includes('could not establish association')) {
        console.error('Cannot connect to remote SCP');
        console.error('Check: IP address, port, firewall, SCP is running');
    } else if (error.message.includes('rejected')) {
        console.error('Association rejected by remote SCP');
        console.error('Check: AE titles, SCP configuration');
    } else {
        console.error('Transfer failed:', error.message);
    }
}
```

### Individual File Failures

```typescript
const failures = [];

const result = await sender.send({
    onFileError: (err, event) => {
        failures.push({
            file: event.file,
            error: event.error
        });
    }
});

if (failures.length > 0) {
    console.error('Failed transfers:');
    failures.forEach(f => {
        console.error(`  ${f.file}: ${f.error}`);
    });
}
```

## Complete Example

```typescript
import { StoreScu } from '@nuxthealth/node-dicom';
import * as fs from 'fs';
import * as path from 'path';

async function sendStudy(studyPath: string, remoteAddress: string) {
    const sender = new StoreScu({
        addr: remoteAddress,
        callingAeTitle: 'HOSPITAL-SCU',
        calledAeTitle: 'PACS',
        maxPduLength: 32768,
        verbose: true
    });

    // Track progress
    let sent = 0;
    let failed = 0;
    const startTime = Date.now();

    const callbacks = {
        onFileSent: (err, event) => {
            sent++;
            console.log(`✓ [${sent}] ${path.basename(event.file)}`);
        },
        onFileError: (err, event) => {
            failed++;
            console.error(`✗ [${failed}] ${event.message}`);
        }
    };

    // Add all DICOM files from directory
    sender.addDirectory(studyPath);

    console.log(`Sending study from: ${studyPath}`);
    console.log(`Target: ${remoteAddress}`);
    console.log('---');

    try {
        const result = await sender.send(callbacks);
        const duration = ((Date.now() - startTime) / 1000).toFixed(2);

        console.log('---');
        console.log('Transfer Summary:');
        console.log(`  Successful: ${result.successful}`);
        console.log(`  Failed: ${result.failed}`);
        console.log(`  Total: ${result.totalFiles}`);
        console.log(`  Duration: ${duration}s`);
        console.log(`  Rate: ${(result.successful / parseFloat(duration)).toFixed(2)} files/sec`);

        return result;
    } catch (error) {
        console.error('Transfer failed:', error.message);
        throw error;
    }
}

// Usage
sendStudy('./studies/CT-Chest-001', '192.168.1.100:104')
    .then(() => console.log('Done!'))
    .catch(err => console.error('Error:', err));
```

## Batch Transfer with Retry

Use the `clean()` method to reset the file queue and retry only failed files:

```typescript
async function sendWithRetry(files: string[], remoteAddress: string, maxRetries = 3) {
    // Create sender once and reuse it
    const scu = new StoreScu({
        addr: remoteAddress,
        callingAeTitle: 'MY-SCU'
        concurrency: 4
    });

    let remainingFiles = [...files];
    let attempt = 0;

    while (attempt < maxRetries && remainingFiles.length > 0) {
        attempt++;
        console.log(`Attempt ${attempt}/${maxRetries} - Sending ${remainingFiles.length} files`);

        const failedFiles: string[] = [];

        // Clear previous files and add current batch
        scu.clean();
        remainingFiles.forEach(file => scu.addFile(file));

        // Send with callbacks to track failures
        await scu.send({
            onFileError: (err, event) => {
                failedFiles.push(event.file);
                console.error(`✗ Failed: ${event.file}`);
            },
            onFileSent: (err, event) => {
                console.log(`✓ Sent: ${event.file}`);
            }
        });

        console.log(`Attempt ${attempt} complete`);

        // Update remaining files for next retry
        remainingFiles = failedFiles;

        if (failedFiles.length === 0) {
            console.log('✓ All files sent successfully!');
            return { success: true, attempts: attempt };
        }

        if (attempt < maxRetries && failedFiles.length > 0) {
            console.log(`Waiting 2s before retry... (${failedFiles.length} files remaining)`);
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
    }

    console.error(`✗ Failed to send ${remainingFiles.length} files after ${maxRetries} attempts`);
    return { success: false, failed: remainingFiles, attempts: attempt };
}

// Usage
const files = ['file1.dcm', 'file2.dcm', 'file3.dcm'];
const result = await sendWithRetry(files, '192.168.1.100:104', 3);
```

Alternatively, for large batches, split files into chunks:

```typescript
async function sendInBatches(files: string[], remoteAddress: string, batchSize = 100) {
    const scu = new StoreScu({
        addr: remoteAddress,
        callingAeTitle: 'MY-SCU'
    });

    const results = {
        successful: 0,
        failed: 0,
        batches: 0
    };

    // Process files in batches
    for (let i = 0; i < files.length; i += batchSize) {
        const batch = files.slice(i, i + batchSize);
        results.batches++;
        
        console.log(`\nBatch ${results.batches}: ${batch.length} files`);

        // Clear and add new batch
        scu.clean();
        batch.forEach(file => scu.addFile(file));

        let batchSuccess = 0;
        let batchFailed = 0;

        await scu.send({
            onFileSent: () => batchSuccess++,
            onFileError: () => batchFailed++
        });

        results.successful += batchSuccess;
        results.failed += batchFailed;

        console.log(`Batch ${results.batches}: ${batchSuccess} sent, ${batchFailed} failed`);
    }

    return results;
}
```
```

## Tips

1. **Test connection first**: Send a single test file before batch operations
2. **Use callbacks for progress**: Implement `onFileSent` and `onFileError` for large transfers
3. **Handle failures gracefully**: Some files may fail due to encoding issues - track them with `onFileError`
4. **Batch large transfers**: Split thousands of files into smaller batches
5. **Set appropriate PDU length**: Larger PDU = faster transfer (if network supports it)
6. **Verify SCP settings**: Ensure remote SCP accepts your AE title and transfer syntaxes
7. **Use verbose mode**: Enable during development to see detailed DICOM protocol messages
8. **Callbacks are optional**: Only provide callbacks you need - all are optional
