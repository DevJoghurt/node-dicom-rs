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