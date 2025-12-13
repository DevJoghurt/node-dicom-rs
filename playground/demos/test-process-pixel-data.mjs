import { DicomFile } from '../../index.js';

console.log('Testing processPixelData with uncompressed file...\n');

const file = new DicomFile();
await file.open('__test__/fixtures/test.dcm');

const info = file.getPixelDataInfo();
console.log(`File info: ${info.width}x${info.height}`);
console.log(`Compressed: ${info.isCompressed}`);
console.log(`Transfer Syntax: ${info.transferSyntaxUid}\n`);

// Test with decode=true on uncompressed file (this would hang before the fix)
console.log('Testing decode=true on uncompressed file...');
try {
    const result = await file.processPixelData({
        outputPath: 'test-received/decoded-test.raw',
        format: 'Raw',
        decode: true
    });
    console.log('✅ SUCCESS:', result);
} catch (err) {
    console.log('❌ FAILED:', err.message);
}

file.close();

console.log('\n✨ Test completed without hanging!');
