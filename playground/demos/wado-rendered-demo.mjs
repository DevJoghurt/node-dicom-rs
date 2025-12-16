#!/usr/bin/env node
/**
 * WADO-RS Rendered/Thumbnail Demo
 * Tests image rendering and thumbnail generation
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

console.log('=== WADO-RS Rendered/Thumbnail Demo ===\n');

// Start server
const storageDir = path.join(__dirname, '../testdata/store');
console.log(`Starting WADO-RS server with storage: ${storageDir}\n`);

const server = new WadoServer(8043, {
    storageType: 'Filesystem',
    basePath: storageDir,
    enableRendered: true,
    enableThumbnail: true,
    thumbnailOptions: {
        width: 200,
        height: 200,
        quality: 80
    },
    verbose: true
});

try {
    await server.start();
    console.log('Server started on http://localhost:8043\n');
    
    // Wait for server to be ready
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test 1: Retrieve rendered image (default size, JPEG)
    console.log('=== Test 1: Rendered Image (Default) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered`);
    
    const renderedDefaultResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered`,
        { headers: { 'Accept': 'image/jpeg' } }
    );
    
    console.log(`Status: ${renderedDefaultResponse.status} ${renderedDefaultResponse.statusText}`);
    console.log(`Content-Type: ${renderedDefaultResponse.headers.get('Content-Type')}`);
    
    if (renderedDefaultResponse.ok) {
        const buffer = await renderedDefaultResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/rendered-default.jpg');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await renderedDefaultResponse.text());
    }
    
    console.log('\n');
    
    // Test 2: Rendered with viewport (resized)
    console.log('=== Test 2: Rendered Image (Viewport 512x512) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=512,512`);
    
    const renderedViewportResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=512,512`,
        { headers: { 'Accept': 'image/jpeg' } }
    );
    
    console.log(`Status: ${renderedViewportResponse.status} ${renderedViewportResponse.statusText}`);
    console.log(`Content-Type: ${renderedViewportResponse.headers.get('Content-Type')}`);
    
    if (renderedViewportResponse.ok) {
        const buffer = await renderedViewportResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/rendered-512x512.jpg');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await renderedViewportResponse.text());
    }
    
    console.log('\n');
    
    // Test 3: Rendered with custom quality
    console.log('=== Test 3: Rendered Image (Quality 50) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=256,256&quality=50`);
    
    const renderedQualityResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=256,256&quality=50`,
        { headers: { 'Accept': 'image/jpeg' } }
    );
    
    console.log(`Status: ${renderedQualityResponse.status} ${renderedQualityResponse.statusText}`);
    console.log(`Content-Type: ${renderedQualityResponse.headers.get('Content-Type')}`);
    
    if (renderedQualityResponse.ok) {
        const buffer = await renderedQualityResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/rendered-quality50.jpg');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await renderedQualityResponse.text());
    }
    
    console.log('\n');
    
    // Test 4: Rendered as PNG
    console.log('=== Test 4: Rendered Image (PNG Format) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=256,256`);
    
    const renderedPngResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/rendered?viewport=256,256`,
        { headers: { 'Accept': 'image/png' } }
    );
    
    console.log(`Status: ${renderedPngResponse.status} ${renderedPngResponse.statusText}`);
    console.log(`Content-Type: ${renderedPngResponse.headers.get('Content-Type')}`);
    
    if (renderedPngResponse.ok) {
        const buffer = await renderedPngResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/rendered-png.png');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await renderedPngResponse.text());
    }
    
    console.log('\n');
    
    // Test 5: Thumbnail (default size from config)
    console.log('=== Test 5: Thumbnail (Default from Config) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/thumbnail`);
    
    const thumbnailResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/thumbnail`,
        { headers: { 'Accept': 'image/jpeg' } }
    );
    
    console.log(`Status: ${thumbnailResponse.status} ${thumbnailResponse.statusText}`);
    console.log(`Content-Type: ${thumbnailResponse.headers.get('Content-Type')}`);
    
    if (thumbnailResponse.ok) {
        const buffer = await thumbnailResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/thumbnail-default.jpg');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await thumbnailResponse.text());
    }
    
    console.log('\n');
    
    // Test 6: Thumbnail with custom viewport
    console.log('=== Test 6: Thumbnail (Custom 100x100) ===');
    console.log(`GET /studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/thumbnail?viewport=100,100`);
    
    const thumbnailCustomResponse = await fetch(
        `http://localhost:8043/studies/${STUDY_UID}/series/${SERIES_UID}/instances/${INSTANCE_UID}/thumbnail?viewport=100,100`,
        { headers: { 'Accept': 'image/jpeg' } }
    );
    
    console.log(`Status: ${thumbnailCustomResponse.status} ${thumbnailCustomResponse.statusText}`);
    console.log(`Content-Type: ${thumbnailCustomResponse.headers.get('Content-Type')}`);
    
    if (thumbnailCustomResponse.ok) {
        const buffer = await thumbnailCustomResponse.arrayBuffer();
        console.log(`Image size: ${buffer.byteLength} bytes (${(buffer.byteLength / 1024).toFixed(2)} KB)`);
        
        const outputPath = path.join(__dirname, '../test-received/thumbnail-100x100.jpg');
        writeFileSync(outputPath, Buffer.from(buffer));
        console.log(`Saved to: ${outputPath}`);
    } else {
        console.log('Error:', await thumbnailCustomResponse.text());
    }
    
    console.log('\n=== All Tests Complete ===');
    console.log('\nGenerated images:');
    console.log('  - rendered-default.jpg (original size)');
    console.log('  - rendered-512x512.jpg (resized)');
    console.log('  - rendered-quality50.jpg (lower quality)');
    console.log('  - rendered-png.png (PNG format)');
    console.log('  - thumbnail-default.jpg (200x200 from config)');
    console.log('  - thumbnail-100x100.jpg (custom size)');
    
} catch (error) {
    console.error('Error during testing:', error);
} finally {
    console.log('\nStopping server...');
    await server.stop();
    console.log('Server stopped');
}
