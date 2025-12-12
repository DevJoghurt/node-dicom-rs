import { StoreScu } from '../index.js'

const sender = new StoreScu({
    callingAeTitle: 'MY-SCU',
    calledAeTitle: 'STORE-SCP',
    addr: '127.0.0.1:4446',
    s3Config: {
        accessKey: 'user',
        secretKey: 'password',
        bucket: 'dicom',
        endpoint: 'http://localhost:7070'
    },
    concurrency: 4,
    verbose: false
});

sender.addFolder('1.3.6.1.4.1.9328.50.2.124067');

console.log('Starting transfer with callback-based events...');

sender.send({
    onTransferStarted: (err, event) => {
        console.log('onTransferStarted:', event.message);
        console.log('  → Total Files:', event.totalFiles);
    },
    onFileSending: (err, event) => {
        console.log('onFileSending:', event.message);
        console.log('  → File:', event.file);
        console.log('  → SOP Instance UID:', event.sopInstanceUid);
        console.log('  → SOP Class UID:', event.sopClassUid);
    },
    onFileSent: (err, event) => {
        console.log('onFileSent:', event.message);
        console.log('  ✓ File:', event.file);
        console.log('  ✓ SOP Instance UID:', event.sopInstanceUid);
        console.log('  ✓ Transfer Syntax:', event.transferSyntax);
        console.log('  ✓ Duration:', event.durationSeconds.toFixed(2), 'seconds');
    },
    onFileError: (err, event) => {
        console.error('onFileError:', event.message);
        console.error('  ✗ File:', event.file);
        console.error('  ✗ Error:', event.error);
        if (event.sopInstanceUid) {
            console.error('  ✗ SOP Instance UID:', event.sopInstanceUid);
            console.error('  ✗ SOP Class UID:', event.sopClassUid);
            console.error('  ✗ File Transfer Syntax:', event.fileTransferSyntax);
        }
    },
    onTransferCompleted: (err, event) => {
        console.log('\n=== Transfer Complete ===');
        console.log('  Message:', event.message);
        console.log('  Total Files:', event.totalFiles);
        console.log('  Successful:', event.successful);
        console.log('  Failed:', event.failed);
        console.log('  Duration:', event.durationSeconds.toFixed(2), 'seconds');
        console.log('  Rate:', (event.successful / event.durationSeconds).toFixed(2), 'files/sec');
    }
})
    .then((result) => {
        console.log(`\n=== Transfer Complete ===`);
        console.log(`  Result:`, result);
    })
    .catch((error) => {
        console.error('Transfer failed:', error);
        process.exit(1);
    });
