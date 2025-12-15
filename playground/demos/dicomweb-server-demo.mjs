#!/usr/bin/env node

/**
 * DICOMweb Server Example
 * 
 * This example demonstrates how to start QIDO-RS and WADO-RS servers
 * for querying and retrieving DICOM objects.
 */

const { QidoServer, WadoServer } = require('../index.js');

// Configuration
const QIDO_PORT = 8080;
const WADO_PORT = 8081;
const DICOM_STORAGE_PATH = './testdata'; // Path to DICOM files

async function main() {
  console.log('Starting DICOMweb servers...\n');

  // Initialize QIDO-RS server (Query service)
  console.log(`Initializing QIDO-RS server on port ${QIDO_PORT}...`);
  const qidoServer = new QidoServer(QIDO_PORT);
  
  try {
    qidoServer.start();
    console.log(`✓ QIDO-RS server listening on http://localhost:${QIDO_PORT}`);
    console.log('  Available endpoints:');
    console.log('    GET /studies - Search for studies');
    console.log('    GET /series - Search for series');
    console.log('    GET /instances - Search for instances\n');
  } catch (error) {
    console.error('✗ Failed to start QIDO server:', error.message);
    process.exit(1);
  }

  // Initialize WADO-RS server (Retrieval service)
  console.log(`Initializing WADO-RS server on port ${WADO_PORT}...`);
  const wadoConfig = {
    storageType: 'filesystem',
    basePath: DICOM_STORAGE_PATH
  };
  
  const wadoServer = new WadoServer(WADO_PORT, wadoConfig);
  
  try {
    wadoServer.start();
    console.log(`✓ WADO-RS server listening on http://localhost:${WADO_PORT}`);
    console.log(`  Storage: ${DICOM_STORAGE_PATH}`);
    console.log('  Available endpoints:');
    console.log('    GET /studies/{studyUID} - Retrieve study');
    console.log('    GET /studies/{studyUID}/series/{seriesUID} - Retrieve series');
    console.log('    GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID} - Retrieve instance');
    console.log('    GET /studies/{studyUID}/metadata - Retrieve study metadata\n');
  } catch (error) {
    console.error('✗ Failed to start WADO server:', error.message);
    qidoServer.stop();
    process.exit(1);
  }

  console.log('═'.repeat(60));
  console.log('DICOMweb servers are running!');
  console.log('═'.repeat(60));
  console.log('\nTest endpoints:');
  console.log(`  curl http://localhost:${QIDO_PORT}/studies`);
  console.log(`  curl http://localhost:${WADO_PORT}/studies/1.2.3/series/4.5.6/instances/7.8.9\n`);
  console.log('Press Ctrl+C to stop the servers\n');

  // Handle graceful shutdown
  const cleanup = () => {
    console.log('\nShutting down servers...');
    
    try {
      qidoServer.stop();
      console.log('✓ QIDO server stopped');
    } catch (error) {
      console.error('✗ Error stopping QIDO server:', error.message);
    }
    
    try {
      wadoServer.stop();
      console.log('✓ WADO server stopped');
    } catch (error) {
      console.error('✗ Error stopping WADO server:', error.message);
    }
    
    console.log('\nServers stopped successfully');
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);

  // Keep the process alive
  await new Promise(() => {});
}

// Run the example
main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
