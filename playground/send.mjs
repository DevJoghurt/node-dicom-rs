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
        console.log('  → Total Files:', event.data?.totalFiles);
    },
    onFileSending: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log('onFileSending:', event.message);
        console.log('  → File:', data.file);
        console.log('  → SOP Instance UID:', data.sopInstanceUid);
        console.log('  → SOP Class UID:', data.sopClassUid);
    },
    onFileSent: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log('onFileSent:', event.message);
        console.log('  ✓ File:', data.file);
        console.log('  ✓ SOP Instance UID:', data.sopInstanceUid);
        console.log('  ✓ Transfer Syntax:', data.transferSyntax);
        console.log('  ✓ Duration:', data.durationSeconds.toFixed(2), 'seconds');
    },
    onFileError: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.error('onFileError:', event.message);
        console.error('  ✗ File:', data.file);
        console.error('  ✗ Error:', data.error);
        if (data.sopInstanceUid) {
            console.error('  ✗ SOP Instance UID:', data.sopInstanceUid);
            console.error('  ✗ SOP Class UID:', data.sopClassUid);
            console.error('  ✗ File Transfer Syntax:', data.fileTransferSyntax);
        }
    },
    onTransferCompleted: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log('\n=== Transfer Complete ===');
        console.log('  Message:', event.message);
        console.log('  Total Files:', data.totalFiles);
        console.log('  Successful:', data.successful);
        console.log('  Failed:', data.failed);
        console.log('  Duration:', data.durationSeconds.toFixed(2), 'seconds');
        console.log('  Rate:', (data.successful / data.durationSeconds).toFixed(2), 'files/sec');
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
