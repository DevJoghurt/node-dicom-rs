#!/usr/bin/env node

/**
 * WADO-RS Multipart Response Demo
 * 
 * Demonstrates bulk retrieval of DICOM instances at study and series level
 * with multipart/related responses.
 * 
 * Features:
 * - Retrieve all instances in a series as multipart response
 * - Parse multipart boundaries and extract individual parts
 * - Save individual DICOM files from multipart response
 * - Count instances returned
 */

import { WadoServer } from '../../index.js';
import fetch from 'node-fetch';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.log('='.repeat(80));
console.log('WADO-RS Multipart Response Demo');
console.log('='.repeat(80));

// ============================================================================
// Configuration
// ============================================================================

const PORT = 8043;
const BASE_PATH = './playground/testdata/store';

// Test data from organized storage
const SAMPLE_STUDY_UID = '1.3.6.1.4.1.9328.50.2.126368';
const SAMPLE_SERIES_UID = '1.3.6.1.4.1.9328.50.2.126606';

const config = {
    storageType: 'Filesystem',
    basePath: BASE_PATH,
    enableMetadata: true,
    enableFrames: false,
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
    
    // Convert buffer to string for parsing
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
// Test 1: Retrieve Series (Multipart DICOM)
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 1: Retrieve All Instances in Series (Multipart DICOM)');
console.log('-'.repeat(80));
console.log();

const seriesUrl = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}/series/${SAMPLE_SERIES_UID}`;
console.log(`GET ${seriesUrl}`);
console.log('Accept: application/dicom');

try {
    const response = await fetch(seriesUrl, {
        headers: {
            'Accept': 'application/dicom'
        }
    });
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    const contentType = response.headers.get('content-type');
    console.log(`Content-Type: ${contentType}`);
    
    if (response.ok) {
        const buffer = await response.buffer();
        console.log(`Response size: ${buffer.length} bytes`);
        
        // Parse multipart response
        const parts = parseMultipartResponse(buffer, contentType);
        console.log(`✓ Found ${parts.length} instances in multipart response`);
        
        // Save first 3 instances as examples
        mkdirSync('./test-received/multipart', { recursive: true });
        const samplesToSave = Math.min(3, parts.length);
        
        for (let i = 0; i < samplesToSave; i++) {
            const filename = `./test-received/multipart/series-instance-${i + 1}.dcm`;
            writeFileSync(filename, parts[i].data);
            console.log(`  Saved instance ${i + 1} to: ${filename} (${parts[i].data.length} bytes)`);
        }
        
        if (parts.length > samplesToSave) {
            console.log(`  ... and ${parts.length - samplesToSave} more instances`);
        }
    } else {
        console.log(`✗ Request failed: ${response.statusText}`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
}

// ============================================================================
// Test 2: Retrieve Series (Multipart DICOM JSON)
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 2: Retrieve Series Metadata (Multipart DICOM JSON)');
console.log('-'.repeat(80));
console.log();

console.log(`GET ${seriesUrl}`);
console.log('Accept: application/dicom+json');

try {
    const response = await fetch(seriesUrl, {
        headers: {
            'Accept': 'application/dicom+json'
        }
    });
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    const contentType = response.headers.get('content-type');
    console.log(`Content-Type: ${contentType}`);
    
    if (response.ok) {
        const buffer = await response.buffer();
        console.log(`Response size: ${buffer.length} bytes`);
        
        // Parse multipart response
        const parts = parseMultipartResponse(buffer, contentType);
        console.log(`✓ Found ${parts.length} metadata objects in multipart response`);
        
        // Parse first metadata object as example
        if (parts.length > 0) {
            const metadata = JSON.parse(parts[0].data.toString());
            console.log('\nFirst instance tags:');
            const tagCount = Object.keys(metadata).length;
            console.log(`  Total tags: ${tagCount}`);
            
            // Save all metadata
            const allMetadata = parts.map(p => JSON.parse(p.data.toString()));
            writeFileSync('./test-received/multipart/series-metadata.json', 
                JSON.stringify(allMetadata, null, 2));
            console.log(`✓ Saved metadata for ${parts.length} instances to: ./test-received/multipart/series-metadata.json`);
        }
    } else {
        console.log(`✗ Request failed: ${response.statusText}`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
}

// ============================================================================
// Test 3: Retrieve Study (All Series)
// ============================================================================

console.log('\n' + '-'.repeat(80));
console.log('Test 3: Retrieve All Instances in Study (All Series)');
console.log('-'.repeat(80));
console.log();

const studyUrl = `http://localhost:${PORT}/studies/${SAMPLE_STUDY_UID}`;
console.log(`GET ${studyUrl}`);
console.log('Accept: application/dicom');

try {
    const response = await fetch(studyUrl, {
        headers: {
            'Accept': 'application/dicom'
        }
    });
    
    console.log(`Status: ${response.status} ${response.statusText}`);
    
    const contentType = response.headers.get('content-type');
    console.log(`Content-Type: ${contentType}`);
    
    if (response.ok) {
        const buffer = await response.buffer();
        console.log(`Response size: ${buffer.length} bytes (${(buffer.length / 1024 / 1024).toFixed(2)} MB)`);
        
        // Parse multipart response
        const parts = parseMultipartResponse(buffer, contentType);
        console.log(`✓ Found ${parts.length} instances across all series in study`);
        
        // Calculate statistics
        const totalSize = parts.reduce((sum, p) => sum + p.data.length, 0);
        const avgSize = totalSize / parts.length;
        console.log(`  Average instance size: ${(avgSize / 1024).toFixed(2)} KB`);
        console.log(`  Total data size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);
    } else {
        console.log(`✗ Request failed: ${response.statusText}`);
    }
} catch (error) {
    console.error('✗ Error:', error.message);
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
console.log('✓ Series bulk retrieval with multipart/related response');
console.log('✓ Study bulk retrieval (all series)');
console.log('✓ Multipart parsing and extraction');
console.log('✓ Both DICOM and DICOM JSON formats');
console.log('\nFiles created:');
console.log('• test-received/multipart/series-instance-*.dcm - Sample DICOM instances');
console.log('• test-received/multipart/series-metadata.json - All instance metadata');
