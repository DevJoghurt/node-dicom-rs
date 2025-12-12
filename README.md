# node-dicom-rs

High-performance Node.js bindings for DICOM (Digital Imaging and Communications in Medicine) operations, powered by Rust and [dicom-rs](https://github.com/Enet4/dicom-rs).

## Features

- **StoreScp**: Receive DICOM files over the network with C-STORE SCP server
- **StoreScu**: Send DICOM files to remote PACS systems
- **DicomFile**: Read, parse, and manipulate DICOM files with full metadata extraction
- **Storage Backends**: Filesystem and S3-compatible object storage support
- **TypeScript Support**: Full TypeScript definitions with autocomplete for 300+ DICOM tags
- **Flexible Tag Extraction**: Extract DICOM metadata with multiple grouping strategies

## Installation

```bash
npm install @nuxthealth/node-dicom
```

## Quick Start

### Receiving DICOM Files (StoreScp)

```typescript
import { StoreScp } from '@nuxthealth/node-dicom';

const receiver = new StoreScp({
    port: 4446,
    callingAeTitle: 'MY-SCP',
    outDir: './dicom-storage',
    verbose: true,
    extractTags: ['PatientName', 'StudyDate', 'Modality'],
    groupingStrategy: 'ByScope'
});

receiver.addEventListener('OnFileStored', (data) => {
    console.log('File received:', data.file);
    console.log('Patient data:', data.tags.patient);
});

receiver.addEventListener('OnStudyCompleted', (study) => {
    console.log(`Study ${study.study_instance_uid} complete`);
    console.log(`${study.series.length} series, total instances: ${study.series.reduce((sum, s) => sum + s.instances.length, 0)}`);
});

receiver.listen();
```

### Sending DICOM Files (StoreScu)

```typescript
import { StoreScu } from '@nuxthealth/node-dicom';

const sender = new StoreScu({
    addr: '192.168.1.100:104',
    callingAeTitle: 'MY-SCU',
    calledAeTitle: 'REMOTE-SCP',
    verbose: true
});

// Add files
sender.addFile('./path/to/file.dcm');
sender.addDirectory('./dicom-folder');

// Send with progress tracking
const result = await sender.send({
    onFileSent: (err, event) => {
        console.log('✓ File sent:', event.sopInstanceUid);
    },
    onFileError: (err, event) => {
        console.error('✗ Error:', event.message, event.error);
    },
    onTransferCompleted: (err, event) => {
        console.log(`Transfer complete! ${event.successful}/${event.totalFiles} files in ${event.durationSeconds.toFixed(2)}s`);
    }
});

console.log('Result:', result);
```
```

### Working with DICOM Files

```typescript
import { DicomFile } from '@nuxthealth/node-dicom';

const file = new DicomFile();
file.open('./scan.dcm');

// Extract specific tags
const tags = file.extract(['PatientName', 'StudyDate', 'Modality'], undefined, 'ByScope');
const data = JSON.parse(tags);
console.log('Patient:', data.patient?.PatientName);
console.log('Study:', data.study?.StudyDate);

// Get pixel data info
const pixelInfo = file.getPixelDataInfo();
console.log(`Image: ${pixelInfo.width}x${pixelInfo.height}, ${pixelInfo.frames} frames`);

// Save raw pixel data
file.saveRawPixelData('./output.raw');

file.close();
```

## Documentation

For detailed documentation, see:

- **[StoreScp Guide](./docs/storescp.md)** - Receiving DICOM files, tag extraction, storage backends
- **[StoreScu Guide](./docs/storescu.md)** - Sending DICOM files, transfer syntaxes, batch operations
- **[DicomFile Guide](./docs/dicomfile.md)** - Reading files, extracting metadata, pixel data operations
- **[Tag Extraction Guide](./docs/tag-extraction.md)** - Grouping strategies, custom tags, helper functions

## Key Features

### Flexible Tag Extraction

Extract DICOM metadata with multiple grouping strategies:

```typescript
// Grouped by DICOM hierarchy (Patient, Study, Series, Instance)
const scoped = file.extract(['PatientName', 'StudyDate', 'Modality'], undefined, 'ByScope');

// Flat structure
const flat = file.extract(['PatientName', 'StudyDate'], undefined, 'Flat');

// Study-level grouping
const studyLevel = file.extract(tags, undefined, 'StudyLevel');
```

### TypeScript Autocomplete

Full autocomplete support for 300+ DICOM tags:

```typescript
const data = file.extract([
    'PatientName',      // Autocomplete suggests all standard tags
    'StudyDate',
    'Modality',
    'SeriesDescription'
], undefined, 'ByScope');
```

### Storage Backends

Store received DICOM files to filesystem or S3:

```typescript
// S3 Storage
const receiver = new StoreScp({
    port: 4446,
    storageBackend: 'S3',
    s3Config: {
        bucket: 'dicom-archive',
        accessKey: 'YOUR_KEY',
        secretKey: 'YOUR_SECRET',
        endpoint: 'https://s3.amazonaws.com'
    }
});
```

### Configurable SCP Acceptance

Control which DICOM types your SCP accepts:

```typescript
import { getCommonSopClasses, getCommonTransferSyntaxes } from '@nuxthealth/node-dicom';

const sopClasses = getCommonSopClasses();
const transferSyntaxes = getCommonTransferSyntaxes();

const receiver = new StoreScp({
    port: 4446,
    abstractSyntaxMode: 'Custom',
    abstractSyntaxes: [...sopClasses.ct, ...sopClasses.mr], // Only CT and MR
    transferSyntaxMode: 'UncompressedOnly' // Only uncompressed
});
```

## Examples

Check the `playground/` directory for more examples:

- Basic SCP receiver
- SCU sender with batch processing
- File metadata extraction
- S3 storage integration
- Custom tag extraction

## Performance

Built with Rust for maximum performance:
- Fast DICOM parsing and validation
- Efficient memory usage for large files
- Native async/await support
- Zero-copy operations where possible

## Credits

- Built on [dicom-rs](https://github.com/Enet4/dicom-rs) by Eduardo Pinho [@Enet4](https://github.com/Enet4)
- Uses [napi-rs](https://napi.rs/) for Rust ↔ Node.js bindings

## License

See LICENSE file for details.