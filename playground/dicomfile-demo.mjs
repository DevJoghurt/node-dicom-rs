#!/usr/bin/env node
/**
 * DicomFile Demo - Read and manipulate DICOM files
 * 
 * Run: node playground/dicomfile-demo.mjs
 * Prerequisites: Run ./playground/downloadTestData.sh first
 */

import { DicomFile } from '../index.js';

console.log('🔬 DicomFile Demo\n');

const file = new DicomFile();

// Open a DICOM file from testdata
await file.open('./playground/testdata/1.3.6.1.4.1.9328.50.2.125354/00000001.dcm');

// Extract common tags
const tags = file.extract([
  'PatientName',
  'PatientID', 
  'StudyDate',
  'Modality',
  'SeriesDescription',
  'InstanceNumber'
]);

console.log('📋 Extracted Tags:');
console.log('  Patient:', tags.PatientName);
console.log('  Patient ID:', tags.PatientID);
console.log('  Study Date:', tags.StudyDate);
console.log('  Modality:', tags.Modality);
console.log('  Series:', tags.SeriesDescription);
console.log('  Instance:', tags.InstanceNumber);

// Get pixel data info
const pixelInfo = file.getPixelDataInfo();
console.log('\n🖼️  Pixel Data Info:');
console.log('  Dimensions:', `${pixelInfo.width}x${pixelInfo.height}`);
console.log('  Frames:', pixelInfo.frames);
console.log('  Bits Allocated:', pixelInfo.bitsAllocated);
console.log('  Compressed:', pixelInfo.isCompressed);

// Get processed pixel data for display
const processedPixels = file.getProcessedPixelData({
  applyVoiLut: true,
  convertTo8bit: true
});

console.log('\n✨ Processed Pixel Data:');
console.log('  Size:', processedPixels.length, 'bytes');
console.log('  Format: 8-bit grayscale, ready for display');

// Update tags (anonymization example)
file.updateTags({
  PatientName: 'ANONYMOUS',
  PatientID: 'ANON123'
});

// Save anonymized file
await file.saveAsDicom('./playground/test-received/anonymized.dcm');
console.log('\n💾 Saved anonymized file to test-received/anonymized.dcm');

file.close();
console.log('\n✅ Demo complete!');
