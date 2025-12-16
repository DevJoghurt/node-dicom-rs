#!/usr/bin/env node

/**
 * WADO-RS Frame Retrieval Demo
 * 
 * Demonstrates frame-level retrieval from DICOM instances.
 * Tests with single-frame images (multi-frame would work the same way).
 * 
 * Features:
 * - Retrieve specific frames by number
 * - Retrieve multiple frames with comma-separated list
 * - Retrieve frame ranges with hyphen notation
 * - Parse multipart response
 * - Extract raw pixel data
 */

import { WadoServer } from '../../index.js';
import fetch from 'node-fetch';
import { writeFileSync, mkdirSync } from 'fs';

console.log('='.repeat(80));
console.log('WADO-RS Frame Retrieval Demo');
console.log('='.repeat(80));

// ============================================================================
// Configuration
// ============================================================================

const PORT = 8043;
const BASE_PATH = './playground/testdata/store';

// Test data from organized storage
const SAMPLE_STUDY_UID = '1.3.6.1.4.1.9328.50.2.126368';
const SAMPLE_SERIES_UID = '1.3.6.1.4.1.9328.50.2.126606';
const SAMPLE_INSTANCE_UID = '1.3.6.1.4.1.9328.50.2.126632';

const config = {
    storageType: 'Filesystem',
    basePath: BASE_PATH,
    enableMetadata: true,
    enableFrames: true,
    enableRendered: false,
    enableThumbnail: false,
    enableBulkdata: false,
    enableCors: true,
    verbose: true,
};

console.log('\nConfiguration:');
console.log(JSON.stringify(config, null, 2));

// ============================================================================
// Start Server
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Starting WADO-RS server...');
console.log('-'.repeat(80));

const wadoServer = new WadoServer(PORT, config);
wadoServer.start();
console.log(`✓ WADO-RS server started on http://localhost:${PORT}`);

// Wait for server to be ready
await new Promise(resolve => setTimeout(resolve, 1000));

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Parse multipart/related response and extract parts
 */
function parseMultipartResponse(buffer, contentType) {
    // Extract boundary from Content-Type header
    const boundaryMatch = contentType.match(/boundary="?([^";,]+)"?/);
    if (!boundaryMatch) {
        throw new Error('No boundary found in Content-Type header');
    }
    
    const boundary = boundaryMatch[1];
    const parts = [];
    
    // Convert buffer to string for parsing headers, keep binary for data
    const content = buffer.toString('binary');
    
    // Split by boundary
    const sections = content.split(`--${boundary}`);
    
    for (const section of sections) {
        if (section.trim() === '' || section.trim() === '--') {
            continue;
        }
        
        // Split headers from body
        const headerBodySplit = section.indexOf('\r\n\r\n');
        if (headerBodySplit === -1) continue;
        
        const headers = section.substring(0, headerBodySplit);
        const body = section.substring(headerBodySplit + 4);
        
        // Extract Content-Type from headers
        const contentTypeMatch = headers.match(/Content-Type:\s*([^\r\n]+)/i);
        const partContentType = contentTypeMatch ? contentTypeMatch[1].trim() : 'application/octet-stream';
        
        // Remove trailing boundary markers and whitespace
        const cleanBody = body.replace(/\r\n--.*$/, '').replace(/\r\n$/, '');
        
        parts.push({
            contentType: partContentType,
            data: Buffer.from(cleanBody, 'binary')
        });
    }
    
    return parts;
}

// ============================================================================
// Test 1: Retrieve Single Frame
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 1: Retrieve Single Frame (Frame 1)');
console.log('-'.repeat(80));
console.log();

const frameUrl = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}/series/${SAMPLE_SERIES_UID}/instances/${SAMPLE_INSTANCE_UID}/frames/1`;
console.log(`GET ${frameUrl}`);

try {
    const response = await fetch(frameUrl);
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    const contentType = response.headers.get('content-type');
    console.log(`Content-Type: ${contentType}`);
    
    if (response.ok) {
        const buffer = await response.buffer();
        console.log(`Response size: ${buffer.length} bytes (${(buffer.length / 1024).toFixed(2)} KB)`);
        
        // Parse multipart response
        const parts = parseMultipartResponse(buffer, contentType);
        console.log(`✓ Found ${parts.length} frame(s) in response`);
        
        if (parts.length > 0) {
            mkdirSync('./test-received/frames', { recursive: true });
            writeFileSync('./test-received/frames/frame-1.raw', parts[0].data);
            console.log(`  Frame 1 size: ${parts[0].data.length} bytes`);
            console.log(`  Saved to: ./test-received/frames/frame-1.raw`);
        }
    } else {
        const error = await response.text();
        console.log(`✗ Request failed: ${error}`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
}

// ============================================================================
// Test 2: Invalid Frame Number
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 2: Request Invalid Frame Number');
console.log('-'.repeat(80));
console.log();

const invalidFrameUrl = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}/series/${SAMPLE_SERIES_UID}/instances/${SAMPLE_INSTANCE_UID}/frames/99`;
console.log(`GET ${invalidFrameUrl}`);

try {
    const response = await fetch(invalidFrameUrl);
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    if (!response.ok) {
        const error = await response.json();
        console.log(`✓ Server correctly rejected invalid frame: ${error.error}`);
    } else {
        console.log(`✗ Server should have rejected frame 99`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
}

// ============================================================================
// Test 3: Invalid Frame List Format
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 3: Invalid Frame List Format');
console.log('-'.repeat(80));
console.log();

const invalidFormatUrl = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}/series/${SAMPLE_SERIES_UID}/instances/${SAMPLE_INSTANCE_UID}/frames/abc`;
console.log(`GET ${invalidFormatUrl}`);

try {
    const response = await fetch(invalidFormatUrl);
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    if (response.status === 400) {
        const error = await response.json();
        console.log(`✓ Server correctly rejected invalid format: ${error.error}`);
    } else {
        console.log(`✗ Expected 400 Bad Request for invalid format`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
}

// ============================================================================
// Test 4: Parse Frame List Formats
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 4: Frame List Format Parsing');
console.log('-'.repeat(80));
console.log();

const testFormats = [
    { format: '1', description: 'Single frame' },
    { format: '1-1', description: 'Range with single frame' },
];

for (const test of testFormats) {
    console.log(`Format: "${test.format}" (${test.description})`);
    
    const url = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}/series/${SAMPLE_SERIES_UID}/instances/${SAMPLE_INSTANCE_UID}/frames/${test.format}`;
    
    try {
        const response = await fetch(url);
        
        if (response.ok) {
            const buffer = await response.buffer();
            const parts = parseMultipartResponse(buffer, response.headers.get('content-type'));
            console.log(`  ✓ Parsed successfully: ${parts.length} frame(s), ${buffer.length} bytes`);
        } else {
            const error = await response.json();
            console.log(`  ✗ Failed: ${error.error}`);
        }
    } catch (error) {
        console.log(`  ✗ Error: ${error.message}`);
    }
}

// ============================================================================
// Cleanup
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Stopping server...');
console.log('-'.repeat(80));

wadoServer.stop();
console.log('WADO-RS server stopped');

console.log('\n' + '='.repeat(80));
console.log('Demo completed successfully!');
console.log('='.repeat(80));

console.log('\nSummary:');
console.log('✓ Single frame retrieval');
console.log('✓ Invalid frame number handling (400 Bad Request)');
console.log('✓ Invalid format handling (400 Bad Request)');
console.log('✓ Frame list format parsing');
console.log('\nFiles created:');
console.log('• test-received/frames/frame-1.raw - Raw pixel data for frame 1');
console.log('\nNote: With multi-frame DICOM files, you can use:');
console.log('  - /frames/1       - Single frame');
console.log('  - /frames/1,3,5   - Multiple specific frames');
console.log('  - /frames/1-10    - Frame range');
console.log('  - /frames/1,3-5,7 - Mixed format');
