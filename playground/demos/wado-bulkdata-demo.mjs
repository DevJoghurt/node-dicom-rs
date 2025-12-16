#!/usr/bin/env node
/**
 * WADO-RS Bulkdata Retrieval Demo
 * Tests retrieving specific DICOM attributes by tag
 */

import { WadoServer } from '../../index.js';
import path from 'path';
import { fileURLToPath } from 'url';
import { writeFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Test study and series UIDs
const STUDY_UID = '1.3.6.1.4.1.9328.50.2.124067';
const SERIES_UID = '1.3.6.1.4.1.9328.50.2.125354';
const INSTANCE_UID = '1.3.6.1.4.1.9328.50.2.125353';

console.log('=== WADO-RS Bulkdata Retrieval Demo ===\n');

// Start server
const storageDir = path.join(__dirname, '../testdata/store');
console.log(`Starting WADO-RS server with storage: ${storageDir}\n`);

const server = new WadoServer(8043, {
    storageType: 'Filesystem',
    basePath: storageDir,
    enableBulkdata: true,
    verbose: true
});

try {
    await server.start();
    console.log('Server started on http://localhost:8043\n');
    
    // Wait for server to be ready
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test 1: Retrieve PixelData (7FE00010)
    console.log('=== Test 1: Retrieve PixelData ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/7FE00010`);
    
    const pixelDataResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/7FE00010`
    );
    
    console.log(`Status: ${pixelDataResponse.status} ${pixelDataResponse.statusText}`);
    console.log(`Content-Type: ${pixelDataResponse.headers.get('Content-Type')}`);
    
    if (pixelDataResponse.ok) {
        const buffer = await pixelDataResponse.arrayBuffer();
        console.log(`PixelData size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        // Save to file
        const outputPath = path.join(__dirname, '../test-received/bulkdata-pixeldata.bin');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await pixelDataResponse.text());
    }
    
    console.log('\n');
    
    // Test 2: Retrieve PatientName (00100010)
    console.log('=== Test 2: Retrieve PatientName ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/00100010`);
    
    const patientNameResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/00100010`
    );
    
    console.log(`Status: ${patientNameResponse.status} ${patientNameResponse.statusText}`);
    console.log(`Content-Type: ${patientNameResponse.headers.get('Content-Type')}`);
    
    if (patientNameResponse.ok) {
        const buffer = await patientNameResponse.arrayBuffer();
        const text = new TextDecoder().decode(buffer);
        console.log(`PatientName size: ${buffer.byteLength} bytes`);
        console.log(`PatientName value: "${text}"`);
    } else {
        console.log('Error:', await patientNameResponse.text());
    }
    
    console.log('\n');
    
    // Test 3: Retrieve StudyInstanceUID (0020000D)
    console.log('=== Test 3: Retrieve StudyInstanceUID ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/0020000D`);
    
    const studyUidResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/0020000D`
    );
    
    console.log(`Status: ${studyUidResponse.status} ${studyUidResponse.statusText}`);
    console.log(`Content-Type: ${studyUidResponse.headers.get('Content-Type')}`);
    
    if (studyUidResponse.ok) {
        const buffer = await studyUidResponse.arrayBuffer();
        const text = new TextDecoder().decode(buffer);
        console.log(`StudyInstanceUID size: ${buffer.byteLength} bytes`);
        console.log(`StudyInstanceUID value: "${text}"`);
        console.log(`Matches expected: ${text.includes(STUDY_UID)}`);
    } else {
        console.log('Error:', await studyUidResponse.text());
    }
    
    console.log('\n');
    
    // Test 4: Error handling - non-existent attribute
    console.log('=== Test 4: Error Handling (Non-existent Attribute) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/99999999`);
    
    const errorResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/99999999`
    );
    
    console.log(`Status: ${errorResponse.status} ${errorResponse.statusText}`);
    if (!errorResponse.ok) {
        const error = await errorResponse.json();
        console.log('Error message:', error.error);
    }
    
    console.log('\n');
    
    // Test 5: Error handling - invalid tag format
    console.log('=== Test 5: Error Handling (Invalid Tag Format) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/INVALID`);
    
    const invalidTagResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/INVALID`
    );
    
    console.log(`Status: ${invalidTagResponse.status} ${invalidTagResponse.statusText}`);
    if (!invalidTagResponse.ok) {
        const error = await invalidTagResponse.json();
        console.log('Error message:', error.error);
    }
    
    console.log('\n');
    
    // Test 6: Feature flag (disabled)
    console.log('=== Test 6: Feature Flag (Disabled) ===');
    console.log('Stopping current server and starting with bulkdata disabled...');
    await server.stop();
    
    const server2 = new WadoServer(8043, {
        storageType: 'Filesystem',
        basePath: storageDir,
        enableBulkdata: false,
        verbose: false
    });
    
    await server2.start();
    await new Promise(resolve => setTimeout(resolve, 500));
    
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/7FE00010`);
    
    const disabledResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/bulkdata/7FE00010`
    );
    
    console.log(`Status: ${disabledResponse.status} ${disabledResponse.statusText}`);
    console.log('Bulkdata correctly disabled when feature flag is false');
    
    await server2.stop();
    
    console.log('\n=== All Tests Complete ===');
    
} catch (error) {
    console.error('Error during testing:', error);
} finally {
    console.log('\nStopping server...');
    await server.stop();
    console.log('Server stopped');
}
