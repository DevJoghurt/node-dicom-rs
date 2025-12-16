#!/usr/bin/env node
/**
 * WADO-RS Metadata Retrieval Demo
 * Tests study and series metadata endpoints
 */

import { WadoServer } from '../../index.js';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Test study and series UIDs
const STUDY_UID = '1.3.6.1.4.1.9328.50.2.124067';
const SERIES_UID = '1.3.6.1.4.1.9328.50.2.125354';
const INSTANCE_UID = '1.3.6.1.4.1.9328.50.2.125353';

console.log('=== WADO-RS Metadata Retrieval Demo ===\n');

// Start server
const storageDir = path.join(__dirname, '../testdata/store');
console.log(`Starting WADO-RS server with storage: ${storageDir}\n`);

const server = new WadoServer(8043, {
    storageType: 'Filesystem',
    basePath: storageDir,
    enableMetadata: true,
    verbose: true
});

try {
    await server.start();
    console.log('Server started on http://localhost:8043\n');
    
    // Wait for server to be ready
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test 1: Retrieve single instance metadata
    console.log('=== Test 1: Single Instance Metadata ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/metadata`);
    
    const instanceResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/metadata`,
        { headers: { 'Accept': 'application/dicom+json' } }
    );
    
    console.log(`Status: ${instanceResponse.status} ${instanceResponse.statusText}`);
    console.log(`Content-Type: ${instanceResponse.headers.get('Content-Type')}`);
    
    if (instanceResponse.ok) {
        const metadata = await instanceResponse.json();
        console.log(`Response is array: ${Array.isArray(metadata)}`);
        console.log(`Number of objects: ${metadata.length}`);
        
        if (metadata.length > 0) {
            const tags = Object.keys(metadata[0]);
            console.log(`Number of tags: ${tags.length}`);
            console.log('Sample tags:', tags.slice(0, 5));
            
            // Check for key DICOM tags
            const hasPatientName = metadata[0]['00100010'] !== undefined;
            const hasStudyUID = metadata[0]['0020000D'] !== undefined;
            const hasSeriesUID = metadata[0]['0020000E'] !== undefined;
            console.log(`Has PatientName: ${hasPatientName}`);
            console.log(`Has StudyInstanceUID: ${hasStudyUID}`);
            console.log(`Has SeriesInstanceUID: ${hasSeriesUID}`);
        }
    } else {
        console.log('Error:', await instanceResponse.text());
    }
    
    console.log('\n');
    
    // Test 2: Retrieve series metadata (all instances in series)
    console.log('=== Test 2: Series Metadata (All Instances) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/metadata`);
    
    const seriesResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/metadata`,
        { headers: { 'Accept': 'application/dicom+json' } }
    );
    
    console.log(`Status: ${seriesResponse.status} ${seriesResponse.statusText}`);
    console.log(`Content-Type: ${seriesResponse.headers.get('Content-Type')}`);
    
    if (seriesResponse.ok) {
        const metadata = await seriesResponse.json();
        console.log(`Response is array: ${Array.isArray(metadata)}`);
        console.log(`Number of instances: ${metadata.length}`);
        
        if (metadata.length > 0) {
            const tags = Object.keys(metadata[0]);
            console.log(`Tags per instance: ${tags.length}`);
            
            // Verify all have same SeriesInstanceUID
            const seriesUIDs = new Set(
                metadata
                    .filter(m => m['0020000E'])
                    .map(m => m['0020000E'].Value[0])
            );
            console.log(`Unique SeriesInstanceUIDs: ${seriesUIDs.size}`);
            console.log(`All same series: ${seriesUIDs.size === 1}`);
            
            // Show instance numbers
            const instanceNumbers = metadata
                .filter(m => m['00200013'])
                .map(m => m['00200013'].Value[0])
                .sort((a, b) => a - b);
            console.log(`Instance numbers: ${instanceNumbers.join(', ')}`);
        }
    } else {
        console.log('Error:', await seriesResponse.text());
    }
    
    console.log('\n');
    
    // Test 3: Retrieve study metadata (all instances in study)
    console.log('=== Test 3: Study Metadata (All Instances) ===');
    console.log(`GET /studies/${STUDY_UID}/metadata`);
    
    const studyResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/metadata`,
        { headers: { 'Accept': 'application/dicom+json' } }
    );
    
    console.log(`Status: ${studyResponse.status} ${studyResponse.statusText}`);
    console.log(`Content-Type: ${studyResponse.headers.get('Content-Type')}`);
    
    if (studyResponse.ok) {
        const metadata = await studyResponse.json();
        console.log(`Response is array: ${Array.isArray(metadata)}`);
        console.log(`Number of instances: ${metadata.length}`);
        
        if (metadata.length > 0) {
            // Verify all have same StudyInstanceUID
            const studyUIDs = new Set(
                metadata
                    .filter(m => m['0020000D'])
                    .map(m => m['0020000D'].Value[0])
            );
            console.log(`Unique StudyInstanceUIDs: ${studyUIDs.size}`);
            console.log(`All same study: ${studyUIDs.size === 1}`);
            
            // Count series
            const seriesUIDs = new Set(
                metadata
                    .filter(m => m['0020000E'])
                    .map(m => m['0020000E'].Value[0])
            );
            console.log(`Number of series: ${seriesUIDs.size}`);
            
            // Show series breakdown
            const seriesCounts = {};
            metadata.forEach(m => {
                if (m['0020000E']) {
                    const seriesUID = m['0020000E'].Value[0];
                    seriesCounts[seriesUID] = (seriesCounts[seriesUID] || 0) + 1;
                }
            });
            console.log('Instances per series:');
            Object.entries(seriesCounts).forEach(([uid, count]) => {
                console.log(`  ${uid.substring(uid.length - 15)}: ${count} instances`);
            });
        }
    } else {
        console.log('Error:', await studyResponse.text());
    }
    
    console.log('\n');
    
    // Test 4: Error handling - non-existent study
    console.log('=== Test 4: Error Handling (Non-existent Study) ===');
    console.log('GET /studies/1.2.3.4.5.6.7.8.9/metadata');
    
    const errorResponse = await fetch(
        'http://localhost:8043/studies/1.2.3.4.5.6.7.8.9/metadata',
        { headers: { 'Accept': 'application/dicom+json' } }
    );
    
    console.log(`Status: ${errorResponse.status} ${errorResponse.statusText}`);
    if (!errorResponse.ok) {
        const error = await errorResponse.json();
        console.log('Error message:', error.error);
    }
    
    console.log('\n=== All Tests Complete ===');
    
} catch (error) {
    console.error('Error during testing:', error);
} finally {
    console.log('\nStopping server...');
    await server.stop();
    console.log('Server stopped');
}
