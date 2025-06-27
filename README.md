# node-dicom-rs

Nodejs bindings for dicom-rs.

## Installation

To install the package, run the following command:

```sh
npm install node-dicom-rs
```

## First steps

### Receiving DICOM files

To receive DICOM files using the `StoreSCP` class, you can use the following example:

```javascript
import { StoreSCP } from 'node-dicom-rs';

const receiver = new StoreSCP({
    verbose: true,
    calling_ae_title: 'STORE-SCP',
    strict: false,
    uncompressed_only: false,
    promiscuous: false,
    max_pdu_length: 16384,
    out_dir: './tmp',
    port: 4446
});

receiver.addEventListener('OnFileStored', (eventData) => {
    console.log('File stored:', eventData);
});

receiver.listen();
```

#### Events

The `StoreSCP` class emits the following events that you can listen to:

- **OnServerStarted**: Triggered when the StoreSCP server has started and is listening for connections.

- **OnFileStored**: Triggered when a DICOM file has been successfully received and stored.
    - **Event data**: An object containing information about the stored file, such as the file path and metadata.

- **OnStudyCompleted**: Triggered when all files for a study have been received and the study is considered complete.
    - **Event data**: An object containing information about the stored study, such as series, instances and metadata.

You can add event listeners using the `addEventListener` method:

```javascript
receiver.addEventListener('OnFileStored', (eventData) => {
        console.log('File stored:', eventData);
});

receiver.addEventListener('OnStudyCompleted', (eventData) => {
        console.error('Full study stored:', eventData);
});
```



#### Storage Backends
The `StoreSCP` class supports multiple storage backends for received DICOM files. Currently, the following backends are available:

- **Filesystem** (default): Stores files on the local disk.
- **S3**: Stores files in an Amazon S3 bucket.

You can specify the backend using the `storage_backend` option when creating a `StoreSCP` instance. For example:

```javascript
const receiver = new StoreSCP({
    storageBackend: 'S3', // default: Filesystem
    s3Config: {
        bucket: 'your-s3-bucket-name',
        access_key_id: 'your-access-key-id',
        secret_access_key: 'your-secret-access-key',
        endpoint: 'http://localhost:7070'
    },
    // ...other options
});
```

If no `storage_backend` is specified, files are stored on the local filesystem by default.

### Sending DICOM files

To send DICOM files using the `StoreScu` class, you can use the following example:

```javascript
import { StoreScu } from 'node-dicom-rs';

const sender = new StoreScu({
    addr: '127.0.0.1:4446',
    verbose: true
});

sender.addFile('./__test__/fixtures/test.dcm');

const result = await sender.send();

console.log(result);
```

### Working with DICOM files

To work with DICOM files using the `DicomFile` class, you can use the following example:

```javascript
import { DicomFile, saveRawPixelData } from 'node-dicom-rs';

const file = new DicomFile();

file.open('./__test__/fixtures/test.dcm');

file.saveRawPixelData('./tmp/raw_pixel_data.jpg');

saveRawPixelData('./__test__/fixtures/test.dcm', './tmp/raw_pixel_data_2.jpg');

console.log(file.getElements());

file.close();
```


## Credits
- Code is based on the rust lib [dicom-rs](https://github.com/Enet4/dicom-rs) by Eduardo Pinho [Enet4](https://github.com/Enet4)