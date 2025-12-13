import { StoreScu } from '../../index.js'
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Send to anonymization server on port 11115
const sender = new StoreScu({
    callingAeTitle: 'DEMO-SCU',
    calledAeTitle: 'STORE-SCP',
    addr: '127.0.0.1:11115',
    verbose: true
});

// Add just a few files from test data (go up to playground directory)
const testDataDir = join(__dirname, '../testdata/1.3.6.1.4.1.9328.50.2.125354');
console.log('Adding files from:', testDataDir);

// Add first 5 DICOM files
for (let i = 1; i <= 5; i++) {
    const filename = `${String(i).padStart(8, '0')}.dcm`;
    const filepath = join(testDataDir, filename);
    try {
        sender.addFile(filepath);
        console.log(`Added: ${filename}`);
    } catch (err) {
        console.error(`Failed to add ${filename}:`, err.message);
    }
}

console.log('\nSending files to anonymization server...\n');

sender.send({
    onTransferStarted: (err, event) => {
        console.log('📤 Transfer started');
        console.log('   Total files:', event.data?.totalFiles);
        console.log();
    },
    onFileSent: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log(`✅ File sent: ${data.file}`);
        console.log(`   Duration: ${data.durationSeconds.toFixed(3)}s`);
    },
    onFileError: (err, event) => {
        console.error('❌ Error:', event.message);
    },
    onTransferCompleted: (err, event) => {
        const data = event.data;
        if (!data) return;
        
        console.log('\n' + '='.repeat(60));
        console.log('📊 Transfer Complete!');
        console.log('='.repeat(60));
        console.log(`✓ Successful: ${data.successful}/${data.totalFiles} files`);
        console.log(`✗ Failed: ${data.failed} files`);
        console.log(`⏱️  Duration: ${data.durationSeconds.toFixed(2)} seconds`);
        console.log('='.repeat(60));
        
        console.log('\nCheck the anonymization server output to see the tag modifications!');
        process.exit(0);
    }
});
