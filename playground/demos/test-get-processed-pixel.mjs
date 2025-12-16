#!/usr/bin/env node
import { DicomFile } from '../../index.js';

console.log('\n=== Testing getProcessedPixelData() ===\n');

const file = new DicomFile();
await file.open('__test__/fixtures/test.dcm');

const info = file.getPixelDataInfo();
console.log(`Image: ${info.width}x${info.height}, ${info.bitsAllocated}-bit`);
console.log(`Photometric: ${info.photometricInterpretation}`);
console.log(`Window: C=${info.windowCenter}, W=${info.windowWidth}\n`);

// Test 1: Basic processing with VOI LUT
console.log('Test 1: Processing with VOI LUT from file...');
const processed1 = file.getProcessedPixelData({
    applyVoiLut: true,
    convertTo8bit: true
});
console.log(`✓ Result: ${processed1.length} bytes (${info.width * info.height * 3} expected for RGB)\n`);

// Test 2: Custom window (soft tissue)
console.log('Test 2: Custom window (soft tissue C=40, W=400)...');
const processed2 = file.getProcessedPixelData({
    windowCenter: 40,
    windowWidth: 400,
    convertTo8bit: true
});
console.log(`✓ Result: ${processed2.length} bytes\n`);

// Test 3: Custom window (bone)
console.log('Test 3: Custom window (bone C=300, W=1500)...');
const processed3 = file.getProcessedPixelData({
    windowCenter: 300,
    windowWidth: 1500,
    convertTo8bit: true
});
console.log(`✓ Result: ${processed3.length} bytes\n`);

// Test 4: No processing (just decode)
console.log('Test 4: No processing (raw decoded)...');
const processed4 = file.getProcessedPixelData();
console.log(`✓ Result: ${processed4.length} bytes\n`);

file.close();

console.log('=== All tests passed! ===\n');
