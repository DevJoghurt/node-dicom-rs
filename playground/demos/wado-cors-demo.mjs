#!/usr/bin/env node
/**
 * WADO-RS CORS Configuration Demo
 * 
 * Demonstrates various CORS configurations for WADO-RS server:
 * 1. Internal network (no CORS)
 * 2. Development environment (all origins allowed)
 * 3. Production single origin
 * 4. Production multiple origins
 * 
 * Based on DICOM PS3.18 Section 8 (WADO-RS) with CORS support
 */

import { WadoServer } from '../../index.js';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testDataPath = path.resolve(__dirname, '../testdata');

console.log('='.repeat(80));
console.log('WADO-RS CORS Configuration Demo');
console.log('='.repeat(80));
console.log();

// ============================================================================
// Scenario 1: Internal Network (CORS Disabled)
// ============================================================================
console.log('Scenario 1: Internal Hospital Network');
console.log('-'.repeat(80));
console.log('Use Case: WADO server only accessible within hospital network');
console.log('Security: Network-level firewall protection');
console.log('CORS: Disabled (not needed for same-origin requests)');
console.log();

const wadoInternal = new WadoServer(8050, {
  storageType: 'Filesystem',
  basePath: testDataPath,
  enableCors: false,  // No CORS needed for internal network
  enableMetadata: true,
  enableFrames: true,
  enableRendered: true,
  enableThumbnail: true,
  verbose: true
});

wadoInternal.start();
console.log('✓ WADO server listening on http://0.0.0.0:8050 (no CORS)');
console.log('  Example: curl http://localhost:8050/studies/1.3.6.1.4.1.9328.50.2.125354/metadata');
console.log();

// ============================================================================
// Scenario 2: Development Environment (All Origins)
// ============================================================================
console.log('Scenario 2: Local Development Environment');
console.log('-'.repeat(80));
console.log('Use Case: Frontend (localhost:3000) + Backend (localhost:8051)');
console.log('Security: Development only - allows all origins for hot reload');
console.log('CORS: Enabled with wildcard (*) for convenience');
console.log();

const wadoDev = new WadoServer(8051, {
  storageType: 'Filesystem',
  basePath: testDataPath,
  enableCors: true,  // Enable CORS for development
  // corsAllowedOrigins not specified = allows all origins (*)
  enableMetadata: true,
  enableFrames: true,
  enableRendered: true,
  enableThumbnail: true,
  thumbnailOptions: {
    quality: 80,
    width: 200,
    height: 200
  },
  verbose: true
});

wadoDev.start();
console.log('✓ WADO server listening on http://0.0.0.0:8051');
console.log('  Test CORS:');
console.log('    curl -H "Origin: http://localhost:3000" \\');
console.log('         http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/metadata');
console.log();

// ============================================================================
// Scenario 3: Production Single Origin
// ============================================================================
console.log('Scenario 3: Production Single Viewer');
console.log('-'.repeat(80));
console.log('Use Case: Hospital has one official DICOM viewer application');
console.log('Security: Restrictive - only allows specific HTTPS origin');
console.log('CORS: Enabled with single origin');
console.log();

const wadoProdSingle = new WadoServer(8052, {
  storageType: 'Filesystem',
  basePath: testDataPath,
  enableCors: true,
  corsAllowedOrigins: 'https://viewer.hospital.com',  // Single origin
  enableMetadata: true,
  enableFrames: true,
  enableRendered: true,
  enableThumbnail: true,
  enableCompression: true,
  verbose: true
});

wadoProdSingle.start();
console.log('✓ WADO server listening on http://0.0.0.0:8052');
console.log('  Allowed origin: https://viewer.hospital.com');
console.log('  Test CORS:');
console.log('    curl -H "Origin: https://viewer.hospital.com" \\');
console.log('         http://localhost:8052/studies/1.3.6.1.4.1.9328.50.2.125354/metadata');
console.log();

// ============================================================================
// Scenario 4: Production Multiple Origins
// ============================================================================
console.log('Scenario 4: Production Multi-Department Setup');
console.log('-'.repeat(80));
console.log('Use Case: Hospital has multiple viewers in different departments');
console.log('Security: Restrictive - whitelisted HTTPS origins only');
console.log('CORS: Enabled with comma-separated origin list');
console.log();

const wadoProdMulti = new WadoServer(8053, {
  storageType: 'Filesystem',
  basePath: testDataPath,
  enableCors: true,
  corsAllowedOrigins: [
    'https://radiology.hospital.com',
    'https://cardiology.hospital.com',
    'https://oncology.hospital.com',
    'https://emergency.hospital.com'
  ].join(','),  // Comma-separated list
  enableMetadata: true,
  enableFrames: true,
  enableRendered: true,
  enableThumbnail: true,
  enableBulkdata: true,
  enableCompression: true,
  maxConnections: 100,
  verbose: true
});

wadoProdMulti.start();
console.log('✓ WADO server listening on http://0.0.0.0:8053');
console.log('  Allowed origins:');
console.log('    - https://radiology.hospital.com');
console.log('    - https://cardiology.hospital.com');
console.log('    - https://oncology.hospital.com');
console.log('    - https://emergency.hospital.com');
console.log();

// ============================================================================
// Testing Instructions
// ============================================================================
console.log('='.repeat(80));
console.log('Testing Instructions');
console.log('='.repeat(80));
console.log();
console.log('1. Test metadata endpoint:');
console.log('   curl http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/metadata');
console.log();
console.log('2. Test CORS headers:');
console.log('   curl -v -H "Origin: http://localhost:3000" \\');
console.log('        http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/metadata');
console.log('   Look for: access-control-allow-origin: *');
console.log();
console.log('3. Test instance retrieval:');
console.log('   curl -H "Accept: application/dicom" \\');
console.log('        http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/series/1.3.6.1.4.1.9328.50.2.126606/instances/1.3.6.1.4.1.9328.50.2.126608 \\');
console.log('        > instance.dcm');
console.log();
console.log('4. Test thumbnail endpoint:');
console.log('   curl -H "Accept: image/jpeg" \\');
console.log('        http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/series/1.3.6.1.4.1.9328.50.2.126606/instances/1.3.6.1.4.1.9328.50.2.126608/thumbnail \\');
console.log('        > thumbnail.jpg');
console.log();
console.log('5. Test in browser:');
console.log('   Open browser console and run:');
console.log('   fetch("http://localhost:8051/studies/1.3.6.1.4.1.9328.50.2.125354/metadata")');
console.log('     .then(res => res.json())');
console.log('     .then(data => console.log("Metadata:", data))');
console.log('     .catch(err => console.error("CORS Error:", err));');
console.log();
console.log('Press Ctrl+C to stop all servers');
console.log('='.repeat(80));
console.log();

// Keep the process alive
process.on('SIGINT', () => {
  console.log('\n\nShutting down WADO servers...');
  wadoInternal.stop();
  wadoDev.stop();
  wadoProdSingle.stop();
  wadoProdMulti.stop();
  console.log('✓ All servers stopped');
  process.exit(0);
});
