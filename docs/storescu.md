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
        console.log('✓ File sent:', event.data?.sopInstanceUid);
    }
});
console.log('Transfer complete:', result);
```

## Configuration Options

The `StoreScu` constructor accepts a configuration object with the following options:

### Required Options

#### addr

**Type:** `string` (required)

The network address of the remote DICOM SCP (receiver) in the format `host:port`.

```typescript
// IPv4 address
addr: '192.168.1.100:104'

// Hostname
addr: 'pacs.hospital.org:104'

// IPv6 address (if supported)
addr: '[2001:db8::1]:104'

// Localhost
addr: '127.0.0.1:11112'
```

**Default Port:** DICOM standard port is `104`, but many systems use `11112` or other ports.

**Common Issues:**
- **Connection refused**: Remote SCP not running or firewall blocking
- **Timeout**: Wrong IP address or routing issues
- **Port in use**: If running local SCP on same port

### Optional Options

#### callingAeTitle

**Type:** `string` (optional)  
**Default:** `'STORE-SCU'`

Your Application Entity (AE) Title - identifies this SCU client to the remote SCP.

```typescript
callingAeTitle: 'HOSPITAL-SCU'
callingAeTitle: 'CT-SCANNER-1'
callingAeTitle: 'WORKSTATION-42'
```

**Constraints:**
- Maximum 16 characters
- Usually uppercase
- No spaces (use hyphens or underscores)
- Must match what the remote SCP expects (if configured for specific AE titles)

**Use Cases:**
- Remote SCP may route files differently based on calling AE title
- Some SCPs require pre-configured AE titles for security
- Helpful for logging and tracking on remote system

#### calledAeTitle

**Type:** `string` (optional)  
**Default:** `'ANY-SCP'`

The AE Title of the remote SCP you're connecting to.

```typescript
calledAeTitle: 'PACS'
calledAeTitle: 'ORTHANC'
calledAeTitle: 'DCM4CHEE'
calledAeTitle: 'REMOTE-SCP'
```

**Constraints:**
- Maximum 16 characters
- Usually uppercase
- Must match the remote SCP's configured AE title

**Common Issues:**
- **Association rejected**: Wrong called AE title - check remote SCP configuration
- Case sensitive on some systems

#### maxPduLength

**Type:** `number` (optional)  
**Default:** `16384` (16 KB)

Maximum Protocol Data Unit (PDU) size in bytes for DICOM network communication.

```typescript
// Small PDU for slow/unreliable networks
maxPduLength: 16384    // 16 KB (default)

// Medium PDU for normal networks
maxPduLength: 32768    // 32 KB

// Large PDU for fast local networks
maxPduLength: 65536    // 64 KB

// Maximum PDU size
maxPduLength: 131072   // 128 KB
```

**Range:** Typically `16384` to `131072` bytes

**Guidelines:**
- **Slow/unstable networks**: Use smaller PDU (16 KB - 32 KB)
- **Fast local networks**: Use larger PDU (64 KB - 128 KB)
- **Internet transfers**: Stick to default or 32 KB
- Larger PDU = faster transfer (if network can handle it)
- Some older systems may not support large PDUs

**Performance Impact:**
```typescript
// Transfer speed examples (1000 files):
maxPduLength: 16384   // ~45 seconds
maxPduLength: 32768   // ~30 seconds
maxPduLength: 65536   // ~20 seconds (on LAN)
```

#### verbose

**Type:** `boolean` (optional)  
**Default:** `false`

Enable detailed logging of DICOM protocol operations.

```typescript
// Production: minimal logging
verbose: false

// Development/debugging: detailed logging
verbose: true
```

**When enabled, logs include:**
- Association negotiation details
- Presentation context acceptance/rejection
- Transfer syntax negotiation
- PDU exchanges
- File transfer progress
- Error details

**Example output:**
```
[StoreScu] Connecting to 192.168.1.100:104
[StoreScu] Association established
[StoreScu] Presentation context accepted: CT Image Storage
[StoreScu] Transfer syntax: Implicit VR Little Endian
[StoreScu] Sending file: scan001.dcm
[StoreScu] C-STORE response: Success
```

**Use Cases:**
- Development and testing
- Troubleshooting connection issues
- Understanding transfer syntax negotiation
- Debugging file transfer failures

**Note:** Verbose output may contain sensitive information (AE titles, file names). Don't enable in production unless necessary.

#### transferSyntax

**Type:** `string` (optional)  
**Default:** Uses original transfer syntax from each file

Force all files to be sent with a specific transfer syntax, regardless of their original encoding.

```typescript
// Use UIDs directly
transferSyntax: '1.2.840.10008.1.2'         // Implicit VR Little Endian
transferSyntax: '1.2.840.10008.1.2.1'       // Explicit VR Little Endian
transferSyntax: '1.2.840.10008.1.2.4.50'    // JPEG Baseline (lossy)
transferSyntax: '1.2.840.10008.1.2.4.90'    // JPEG 2000 Lossless

// Or use helper names (if supported)
transferSyntax: 'ImplicitVRLittleEndian'
transferSyntax: 'ExplicitVRLittleEndian'
```

**Common Transfer Syntaxes:**

| Name | UID | Description | Use Case |
|------|-----|-------------|----------|
| Implicit VR Little Endian | 1.2.840.10008.1.2 | Uncompressed, implicit | Default, widest compatibility |
| Explicit VR Little Endian | 1.2.840.10008.1.2.1 | Uncompressed, explicit | Standard uncompressed |
| Deflated Explicit VR Little Endian | 1.2.840.10008.1.2.1.99 | ZIP compression | Lossless compression |
| JPEG Baseline (Process 1) | 1.2.840.10008.1.2.4.50 | JPEG lossy 8-bit | Web/preview images |
| JPEG Lossless | 1.2.840.10008.1.2.4.70 | JPEG lossless | Diagnostic quality |
| JPEG 2000 Lossless | 1.2.840.10008.1.2.4.90 | JP2 lossless | High quality, smaller |
| JPEG 2000 Lossy | 1.2.840.10008.1.2.4.91 | JP2 lossy | Compressed archives |
| RLE Lossless | 1.2.840.10008.1.2.5 | RLE compression | Medical images |

**When to specify:**
- Remote SCP only accepts specific transfer syntaxes
- Need to compress/decompress during transfer
- Standardizing encoding across mixed files
- Converting legacy encodings

**Important Notes:**
- Not all SCPs support all transfer syntaxes
- Some transfer syntaxes require transcoding (slow)
- Lossy compression should not be used for diagnostic images (without medical approval)
- Leave unset to use original file encoding (fastest)

### Complete Configuration Example

```typescript
import { StoreScu } from '@nuxthealth/node-dicom';

// Minimal configuration
const minimalSender = new StoreScu({
    addr: '192.168.1.100:104'
});

// Standard configuration
const standardSender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'MY-SCU',
    calledAeTitle: 'PACS'
});

// High-performance configuration (fast LAN)
const fastSender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'HIGH-PERF-SCU',
    calledAeTitle: 'PACS',
    maxPduLength: 131072,  // 128 KB for maximum speed
    concurrency: 8,         // Send 8 files simultaneously
    verbose: false
});

// Development/debugging configuration
const debugSender = new StoreScu({
    addr: '127.0.0.1:11112',
    callingAeTitle: 'DEBUG-SCU',
    calledAeTitle: 'LOCAL-SCP',
    maxPduLength: 16384,
    verbose: true  // See all protocol details
});

// Force specific encoding
const uncompressedSender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'UNCOMP-SCU',
    calledAeTitle: 'PACS',
    transferSyntax: '1.2.840.10008.1.2.1'  // Force uncompressed
});
// Internet/WAN configuration (conservative settings)
const wanSender = new StoreScu({
    addr: 'pacs.remote-site.org:104',
    callingAeTitle: 'SITE-A-SCU',
    calledAeTitle: 'SITE-B-PACS',
    maxPduLength: 32768,  // Moderate PDU for reliability
    concurrency: 2,       // Low concurrency for stability
    verbose: false
});

// Batch processing configuration
const batchSender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'BATCH-SCU',
    calledAeTitle: 'PACS',
    maxPduLength: 65536,
    concurrency: 4,       // Moderate concurrency for reliability
    verbose: false
}); verbose: false
});
```

### Configuration Best Practices

1. **Start with defaults**
   ```typescript
   // Good for most use cases
   const sender = new StoreScu({
       addr: '192.168.1.100:104',
       callingAeTitle: 'MY-SCU',
3. **Tune PDU length and concurrency for your network**
   ```typescript
   // Fast LAN: larger PDU + high concurrency
   maxPduLength: 65536
   concurrency: 8
   
   // WAN: smaller PDU + low concurrency
   maxPduLength: 32768
   concurrency: 2
   
   // Slow network: conservative settings
   maxPduLength: 16384
   concurrency: 1
   ```lingAeTitle: 'HOSPITAL-SCU',  // Must be pre-configured
   calledAeTitle: 'PACS'            // Must match exactly
   ```

3. **Tune PDU length for your network**
   ```typescript
   // LAN: use larger PDU
   maxPduLength: 65536
   
   // WAN: use smaller PDU
   maxPduLength: 32768
   ```

4. **Enable verbose mode only when needed**
   ```typescript
   // Production
   verbose: false
   
   // Development/troubleshooting
   verbose: true
   ```

5. **Don't specify transferSyntax unless required**
   ```typescript
   // Let files use their original encoding (fastest)
   // Only specify if remote SCP has restrictions
   ```

6. **Test configuration before production**
   ```typescript
   // Send single test file first
   sender.addFile('test.dcm');
   await sender.send();
   
   // Then proceed with full batch
   sender.clean();
   sender.addDirectory('./all-files');
   await sender.send();
   ```

### Troubleshooting Configuration Issues

**Connection Refused**
```typescript
// Check: IP address, port, firewall, SCP running
addr: '192.168.1.100:104'  // Verify correct
```

**Association Rejected**
**Slow Transfers**
```typescript
// Try increasing PDU length and concurrency (if on LAN)
maxPduLength: 65536    // or 131072
concurrency: 4         // or higher for many files
```

**Slow Transfers**
```typescript
// Try increasing PDU length (if on LAN)
maxPduLength: 65536  // or 131072
```

**Transfer Syntax Not Supported**
```typescript
// Remove transferSyntax to use original
// Or check what SCP accepts:
verbose: true  // See negotiation details
```

**Intermittent Failures**
```typescript
// Use smaller PDU for unstable networks
maxPduLength: 16384
```

#### concurrency

**Type:** `number` (optional)  
**Default:** `1` (sequential transfer)

Number of files to send concurrently. This can significantly speed up transfers when sending many files.

```typescript
// Sequential transfer (default)
concurrency: 1    // Send one file at a time

// Low concurrency
concurrency: 2    // Send 2 files simultaneously

// Moderate concurrency
concurrency: 4    // Send 4 files simultaneously

// High concurrency
concurrency: 8    // Send 8 files simultaneously

// Very high concurrency
concurrency: 16   // Send 16 files simultaneously
```

**Range:** Typically `1` to `16` (or higher depending on your needs)

**Performance Impact:**

Concurrency dramatically improves transfer speed, especially with many small files:

```typescript
// Example: 1000 files, 100 KB each, to remote PACS

concurrency: 1    // ~120 seconds (sequential)
concurrency: 2    // ~65 seconds  (1.8x faster)
concurrency: 4    // ~35 seconds  (3.4x faster)
concurrency: 8    // ~20 seconds  (6x faster)
concurrency: 16   // ~15 seconds  (8x faster)
```

**How It Works:**

- Opens multiple DICOM associations simultaneously
- Each association handles its own file transfers
- Files are distributed across available associations
- All associations run in parallel

**When to Use High Concurrency:**

✅ **Good for:**
- Many small/medium files (< 10 MB each)
- Fast, reliable networks (LAN)
- High-bandwidth connections
- Remote SCP with good performance
- Sending to cloud PACS systems

❌ **Avoid for:**
- Very large files (> 100 MB each)
- Slow networks or high latency
- Bandwidth-limited connections
- Remote SCP with limited resources
- Single large file transfers

**Guidelines by Network Type:**

```typescript
// Gigabit LAN (1000 Mbps) - Local PACS
concurrency: 8    // or higher

// Fast LAN (100 Mbps)
concurrency: 4

// WAN / Internet connection
concurrency: 2    // Conservative for reliability

// Slow or unreliable network
concurrency: 1    // Sequential, most reliable
```

**Guidelines by File Characteristics:**

```typescript
// Many tiny files (< 1 MB each)
concurrency: 16   // Maximum parallelization

// Small to medium files (1-10 MB)
concurrency: 8    // Good balance

// Large files (10-50 MB)
concurrency: 4    // Moderate concurrency

// Very large files (> 50 MB)
concurrency: 2    // Low concurrency, focus on bandwidth

// Single huge file or few large files
concurrency: 1    // Sequential transfer
```

**Resource Considerations:**

Higher concurrency uses more resources:

- **Memory**: Each concurrent association uses memory for buffers
- **CPU**: More associations = more processing overhead
- **Network**: More connections = more network overhead
- **Remote SCP**: May have connection limits

**Recommended Settings:**

```typescript
// Default/safe - works everywhere
concurrency: 1

// Balanced - good for most cases
concurrency: 4

// High-performance - fast LAN to capable PACS
concurrency: 8

// Maximum throughput - optimal conditions
concurrency: 16
```

**Testing Concurrency:**

Always test to find optimal concurrency for your specific setup:

```typescript
async function testConcurrency() {
    const testFiles = ['test1.dcm', 'test2.dcm', /* ... 100 files */];
    const concurrencyLevels = [1, 2, 4, 8, 16];
    
    for (const concurrency of concurrencyLevels) {
        const sender = new StoreScu({
            addr: '192.168.1.100:104',
            callingAeTitle: 'TEST-SCU',
            calledAeTitle: 'PACS',
            concurrency
        });
        
        testFiles.forEach(f => sender.addFile(f));
        
        const startTime = Date.now();
        const result = await sender.send();
        const duration = (Date.now() - startTime) / 1000;
        
        console.log(`Concurrency ${concurrency}: ${duration.toFixed(2)}s`);
        console.log(`  Rate: ${(result.successful / duration).toFixed(2)} files/sec`);
        
        sender.clean();
    }
}
```

**Important Notes:**

1. **Remote SCP Limits**: Some PACS systems limit concurrent associations
   ```typescript
   // If remote SCP rejects associations, reduce concurrency
   concurrency: 2  // Start low and increase gradually
   ```

2. **Network Bandwidth**: Don't exceed available bandwidth
   ```typescript
   // Monitor network usage and adjust accordingly
   // High concurrency won't help if network is saturated
   ```

3. **File Size Matters**: Large files benefit less from concurrency
   ```typescript
   // 10 files × 1 GB each
   concurrency: 1   // Better: focus bandwidth on each file
   
   // 10000 files × 100 KB each
   concurrency: 8   // Better: parallelize small files
   ```

4. **Error Handling**: More concurrency = more complexity
   ```typescript
   // Track failures with callbacks
   const failures = [];
   await sender.send({
       onFileError: (err, event) => {
           failures.push(event.data?.file);
       }
   });
   ```

**Common Patterns:**

```typescript
// Adaptive concurrency based on file count
function getConcurrency(fileCount: number): number {
    if (fileCount < 10) return 1;
    if (fileCount < 50) return 2;
    if (fileCount < 200) return 4;
    if (fileCount < 1000) return 8;
    return 16;
}

const sender = new StoreScu({
    addr: '192.168.1.100:104',
    concurrency: getConcurrency(files.length)
});

// Adaptive concurrency based on network type
const concurrencyByNetwork = {
    'lan': 8,
    'wan': 2,
    'internet': 2,
    'slow': 1
};

const sender = new StoreScu({
    addr: remoteAddress,
    concurrency: concurrencyByNetwork[networkType]
});
```

**Troubleshooting:**

If experiencing issues with concurrency:

1. **Reduce concurrency**: Start at 1 and increase gradually
2. **Check remote SCP logs**: Look for connection limit errors
3. **Monitor network**: Ensure bandwidth isn't saturated
4. **Increase maxPduLength**: May help with concurrent transfers
5. **Use verbose mode**: See detailed association information

```typescript
// Debug configuration
const sender = new StoreScu({
    addr: '192.168.1.100:104',
    concurrency: 4,
    verbose: true,  // See what's happening
    maxPduLength: 65536  // Larger PDU may help
});
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
        console.log('✓ Sent:', event.data?.sopInstanceUid);
    },
    onFileError: (err, event) => {
        console.error('✗ Failed:', event.message);
        console.error('  Error:', event.data?.error);
    },
    onTransferCompleted: (err, event) => {
        console.log('All files transferred!', event.data?.totalFiles);
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
        console.log('Total files:', event.data?.totalFiles);
    }
});
```

Event data structure:
```typescript
{
    message: string,        // Human-readable message
    data?: {
        totalFiles: number  // Total number of files to transfer
    }
}
```

### onFileSending

Called when a file is about to be sent.

```typescript
await sender.send({
    onFileSending: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log(`Sending: ${data.file}`);
        console.log('SOP Class:', data.sopClassUid);
        console.log('SOP Instance:', data.sopInstanceUid);
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    data?: {
        file: string,              // File path (local or S3)
        sopInstanceUid: string,    // SOP Instance UID
        sopClassUid: string        // SOP Class UID
    }
}
```

### onFileSent

Called when a file is successfully sent.

```typescript
await sender.send({
    onFileSent: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log(event.message); // "File sent successfully"
        console.log('File:', data.file);
        console.log('SOP Instance UID:', data.sopInstanceUid);
        console.log('SOP Class UID:', data.sopClassUid);
        console.log('Transfer Syntax:', data.transferSyntax);
        console.log('Duration:', data.durationSeconds, 'seconds');
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    data?: {
        file: string,              // File path (local or S3)
        sopInstanceUid: string,    // SOP Instance UID
        sopClassUid: string,       // SOP Class UID
        transferSyntax: string,    // Transfer Syntax UID used
        durationSeconds: number    // Transfer duration in seconds
    }
}
```

### onFileError

Called when a file transfer fails.

```typescript
await sender.send({
    onFileError: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.error('Error:', event.message);
        console.error('File:', data.file);
        console.error('Details:', data.error);
        if (data.sopInstanceUid) {
            console.error('SOP Instance UID:', data.sopInstanceUid);
            console.error('SOP Class UID:', data.sopClassUid);
            console.error('File Transfer Syntax:', data.fileTransferSyntax);
        }
    }
});
```

Event data structure:
```typescript
{
    message: string,               // Error message
    data?: {
        file: string,                  // File path that failed
        error: string,                 // Detailed error information
        sopInstanceUid?: string,       // SOP Instance UID (if available)
        sopClassUid?: string,          // SOP Class UID (if available)
        fileTransferSyntax?: string    // Original file transfer syntax (if available)
    }
}
```

### onTransferCompleted

Called once when all files have been transferred.

```typescript
await sender.send({
    onTransferCompleted: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log('All files transferred!');
        console.log(`Total: ${data.totalFiles} files`);
        console.log(`Successful: ${data.successful} files`);
        console.log(`Failed: ${data.failed} files`);
        console.log(`Duration: ${data.durationSeconds.toFixed(2)}s`);
    }
});
```

Event data structure:
```typescript
{
    message: string,           // Human-readable message
    data?: {
        totalFiles: number,        // Total number of files attempted
        successful: number,        // Number of successfully transferred files
        failed: number,            // Number of failed transfers
        durationSeconds: number    // Total transfer duration in seconds
    }
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
        const data = event.data;
        if (!data) return;
        
        failures.push({
            file: data.file,
            error: data.error
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
    const sender = new StoreScu({
        addr: remoteAddress,
        callingAeTitle: 'HOSPITAL-SCU',
        calledAeTitle: 'PACS',
        maxPduLength: 32768,
        concurrency: 4,
        verbose: true
    });unction sendStudy(studyPath: string, remoteAddress: string) {
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
            const data = event.data;
            if (data) {
                console.log(`✓ [${sent}] ${path.basename(data.file)}`);
            }
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
                const data = event.data;
                if (data) {
                    failedFiles.push(data.file);
                    console.error(`✗ Failed: ${data.file}`);
                }
            },
            onFileSent: (err, event) => {
                const data = event.data;
                if (data) {
                    console.log(`✓ Sent: ${data.file}`);
                }
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
