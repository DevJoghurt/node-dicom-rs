#!/usr/bin/env node
/**
 * DICOMweb Demo - QIDO-RS (Query) and WADO-RS (Retrieve) servers
 * 
 * Run: node playground/dicomweb-demo.mjs
 * Prerequisites: Run ./playground/downloadTestData.sh first
 * 
 * Then test with:
 * curl http://localhost:8042/studies
 * curl http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}
 */

import { QidoServer, WadoServer } from '../index.js';

console.log('🌐 DICOMweb Demo - QIDO-RS + WADO-RS Servers\n');

// Start QIDO-RS server (Query)
const qidoServer = new QidoServer(8042);
qidoServer.start();
console.log('✅ QIDO-RS Server started on http://localhost:8042');
console.log('   Query studies: http://localhost:8042/studies');
console.log('   Query series:  http://localhost:8042/series');
console.log('   Query instances: http://localhost:8042/instances');

// Start WADO-RS server (Retrieve)
const wadoConfig = {
  storageType: 'filesystem',
  basePath: './playground/testdata'
};

const wadoServer = new WadoServer(8043, wadoConfig);
wadoServer.start();
console.log('\n✅ WADO-RS Server started on http://localhost:8043');
console.log('   Retrieve studies: http://localhost:8043/studies/{studyUID}');
console.log('   Retrieve series:  http://localhost:8043/studies/{studyUID}/series/{seriesUID}');
console.log('   Retrieve instance: http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}');
console.log('   Get metadata: http://localhost:8043/studies/{studyUID}/metadata');

console.log('\n📖 Example queries:');
console.log('   curl http://localhost:8042/studies');
console.log('   curl http://localhost:8043/studies/1.3.6.1.4.1.9328.50.2.125354');

console.log('\n🛑 Press Ctrl+C to stop servers\n');

// Handle Ctrl+C
process.on('SIGINT', () => {
  console.log('\n\n🛑 Stopping servers...');
  qidoServer.stop();
  wadoServer.stop();
  console.log('✅ Servers stopped');
  process.exit(0);
});

// Keep process running
process.stdin.resume();
