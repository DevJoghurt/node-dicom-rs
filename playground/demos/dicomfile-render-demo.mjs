#!/usr/bin/env node

/**
 * DicomFile Image Rendering Demo
 * 
 * Demonstrates the enhanced image rendering capabilities of DicomFile using
 * the new image_processing utility module:
 * - PNG and JPEG output formats
 * - VOI LUT application from file
 * - Manual windowing with custom parameters
 * - Frame extraction from multi-frame images
 * - Automatic rescale (Hounsfield units for CT)
 */

import { DicomFile } from '../../index.js';
import { existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const testDataDir = join(__dirname, '../testdata/store');
const outputDir = join(__dirname, '../test-received');

// Ensure output directory exists
if (!existsSync(outputDir)) {
    mkdirSync(outputDir, { recursive: true });
}

// Test file paths
const testFile = join(testDataDir, '1.3.6.1.4.1.9328.50.2.124067/1.3.6.1.4.1.9328.50.2.125354/1.3.6.1.4.1.9328.50.2.125353.dcm');

console.log('=== DicomFile Image Rendering Demo ===\n');

async function runTests() {
    try {
        // Test 1: Default rendering to JPEG
        console.log('=== Test 1: Default JPEG Rendering ===');
        const file1 = new DicomFile();
        await file1.open(testFile);
        
        const info = file1.getPixelDataInfo();
        console.log(`Image info: ${info.width}x${info.height}, ${info.bitsAllocated}-bit, ${info.frames} frame(s)`);
        console.log(`Photometric: ${info.photometricInterpretation}`);
        if (info.windowCenter && info.windowWidth) {
            console.log(`Window from file: C=${info.windowCenter}, W=${info.windowWidth}`);
        }
        if (info.rescaleIntercept && info.rescaleSlope) {
            console.log(`Rescale: slope=${info.rescaleSlope}, intercept=${info.rescaleIntercept}`);
        }
        
        await file1.processPixelData({
            outputPath: join(outputDir, 'dicomfile-default.jpg'),
            format: 'Jpeg',
            decode: true,
        });
        console.log('✓ Rendered to dicomfile-default.jpg\n');
        file1.close();

        // Test 2: PNG rendering with VOI LUT from file
        console.log('=== Test 2: PNG with VOI LUT from File ===');
        const file2 = new DicomFile();
        await file2.open(testFile);
        
        await file2.processPixelData({
            outputPath: join(outputDir, 'dicomfile-voi-lut.png'),
            format: 'Png',
            decode: true,
            applyVoiLut: true,
        });
        console.log('✓ Rendered to dicomfile-voi-lut.png with VOI LUT from file\n');
        file2.close();

        // Test 3: Manual windowing (soft tissue window)
        console.log('=== Test 3: Manual Windowing (Soft Tissue) ===');
        const file3 = new DicomFile();
        await file3.open(testFile);
        
        await file3.processPixelData({
            outputPath: join(outputDir, 'dicomfile-soft-tissue.jpg'),
            format: 'Jpeg',
            decode: true,
            windowCenter: 40,
            windowWidth: 400,
        });
        console.log('✓ Rendered to dicomfile-soft-tissue.jpg (C=40, W=400)\n');
        file3.close();

        // Test 4: Different window (bone window)
        console.log('=== Test 4: Manual Windowing (Bone) ===');
        const file4 = new DicomFile();
        await file4.open(testFile);
        
        await file4.processPixelData({
            outputPath: join(outputDir, 'dicomfile-bone.jpg'),
            format: 'Jpeg',
            decode: true,
            windowCenter: 300,
            windowWidth: 1500,
        });
        console.log('✓ Rendered to dicomfile-bone.jpg (C=300, W=1500)\n');
        file4.close();

        // Test 5: Extract specific frame (if multi-frame)
        console.log('=== Test 5: Frame Extraction ===');
        const file5 = new DicomFile();
        await file5.open(testFile);
        
        const frameInfo = file5.getPixelDataInfo();
        if (frameInfo.frames > 1) {
            await file5.processPixelData({
                outputPath: join(outputDir, 'dicomfile-frame-0.png'),
                format: 'Png',
                decode: true,
                frameNumber: 0,
            });
            console.log(`✓ Extracted frame 0 to dicomfile-frame-0.png\n`);
        } else {
            console.log('Single-frame image, frame extraction not applicable\n');
        }
        file5.close();

        // Test 6: 8-bit conversion
        console.log('=== Test 6: 8-bit Conversion ===');
        const file6 = new DicomFile();
        await file6.open(testFile);
        
        await file6.processPixelData({
            outputPath: join(outputDir, 'dicomfile-8bit.jpg'),
            format: 'Jpeg',
            decode: true,
            convertTo8bit: true,
            applyVoiLut: true,
        });
        console.log('✓ Rendered to dicomfile-8bit.jpg with 8-bit conversion\n');
        file6.close();

        console.log('=== All Tests Complete ===\n');
        console.log('Generated images:');
        console.log('  - dicomfile-default.jpg (default rendering)');
        console.log('  - dicomfile-voi-lut.png (VOI LUT from file)');
        console.log('  - dicomfile-soft-tissue.jpg (C=40, W=400)');
        console.log('  - dicomfile-bone.jpg (C=300, W=1500)');
        if (frameInfo.frames > 1) {
            console.log('  - dicomfile-frame-0.png (frame extraction)');
        }
        console.log('  - dicomfile-8bit.jpg (8-bit conversion)');

    } catch (error) {
        console.error('Error:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

runTests();
