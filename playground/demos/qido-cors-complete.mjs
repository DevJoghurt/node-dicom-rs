#!/usr/bin/env node
/**
 * QIDO-RS CORS Configuration Examples
 * 
 * This demo shows various CORS configurations for QIDO-RS servers
 * based on different deployment scenarios.
 */

import { QidoServer, QidoStudyResult } from '../../index.js';

console.log('\n╔════════════════════════════════════════════════════════╗');
console.log('║   QIDO-RS CORS Configuration Examples                 ║');
console.log('╚════════════════════════════════════════════════════════╝\n');

// ============================================================================
// Scenario 1: Internal Network (No CORS)
// ============================================================================
console.log('📋 Scenario 1: Internal Hospital Network (CORS Disabled)');
console.log('   Use Case: PACS accessible only within hospital firewall');
console.log('   Security: Network-level isolation\n');

const internalQido = new QidoServer(8050, {
  enableCors: false,
  verbose: true
});

console.log('   Configuration:');
console.log('   {');
console.log('     enableCors: false,');
console.log('     verbose: true');
console.log('   }\n');

// ============================================================================
// Scenario 2: Development Environment (All Origins)
// ============================================================================
console.log('📋 Scenario 2: Development Environment (CORS All Origins)');
console.log('   Use Case: Local development with hot reload');
console.log('   Security: Development only - NOT for production\n');

const devQido = new QidoServer(8051, {
  enableCors: true,
  // No corsAllowedOrigins = allows all origins (*)
  verbose: true
});

// Register handler
devQido.onSearchForStudies((err, query) => {
  if (err) throw err;
  
  // Create sample study
  const result = new QidoStudyResult();
  result.studyInstanceUid('1.2.840.113619.2.55.3.1234567890.123');
  result.studyDate('20240116');
  result.studyTime('143022');
  result.patientName('TEST^PATIENT');
  result.patientId('PAT001');
  result.accessionNumber('ACC20240116001');
  result.studyDescription('CT CHEST WITH CONTRAST');
  result.modalitiesInStudy('CT');
  
  return JSON.stringify([result.getAttributes()]);
});

devQido.start();

console.log('   Configuration:');
console.log('   {');
console.log('     enableCors: true,');
console.log('     corsAllowedOrigins: undefined  // Allows all origins');
console.log('   }\n');

// ============================================================================
// Scenario 3: Production Single Origin
// ============================================================================
console.log('📋 Scenario 3: Production Single Viewer (Specific Origin)');
console.log('   Use Case: Hospital PACS with single web viewer');
console.log('   Security: Strict origin validation\n');

const prodSingleQido = new QidoServer(8052, {
  enableCors: true,
  corsAllowedOrigins: 'https://viewer.hospital.com',
  verbose: true
});

prodSingleQido.onSearchForStudies((err, query) => {
  if (err) throw err;
  const result = new QidoStudyResult();
  result.studyInstanceUid('1.2.840.113619.2.55.3.9999999999.999');
  result.studyDate('20240116');
  result.patientName('PROD^TEST');
  result.patientId('PROD001');
  return JSON.stringify([result.getAttributes()]);
});

prodSingleQido.start();

console.log('   Configuration:');
console.log('   {');
console.log('     enableCors: true,');
console.log('     corsAllowedOrigins: "https://viewer.hospital.com"');
console.log('   }\n');

// ============================================================================
// Scenario 4: Production Multi-Origin
// ============================================================================
console.log('📋 Scenario 4: Production Multi-Viewer (Multiple Origins)');
console.log('   Use Case: Multiple departments with separate viewers');
console.log('   Security: Whitelist of trusted origins\n');

const prodMultiQido = new QidoServer(8053, {
  enableCors: true,
  corsAllowedOrigins: 'https://radiology.hospital.com,https://cardiology.hospital.com,https://oncology.hospital.com',
  verbose: true
});

prodMultiQido.onSearchForStudies((err, query) => {
  if (err) throw err;
  const result = new QidoStudyResult();
  result.studyInstanceUid('1.2.840.113619.2.55.3.8888888888.888');
  result.studyDate('20240116');
  result.patientName('MULTI^DEPT');
  result.patientId('MULTI001');
  return JSON.stringify([result.getAttributes()]);
});

prodMultiQido.start();

console.log('   Configuration:');
console.log('   {');
console.log('     enableCors: true,');
console.log('     corsAllowedOrigins: "https://radiology.hospital.com,');
console.log('                          https://cardiology.hospital.com,');
console.log('                          https://oncology.hospital.com"');
console.log('   }\n');

// ============================================================================
// Test Instructions
// ============================================================================
console.log('╔════════════════════════════════════════════════════════╗');
console.log('║   Testing CORS Configuration                           ║');
console.log('╚════════════════════════════════════════════════════════╝\n');

console.log('🧪 Test with curl:\n');

console.log('# 1. No CORS (port 8050) - No Access-Control headers:');
console.log('curl -v http://localhost:8050/studies 2>&1 | grep -i access-control\n');

console.log('# 2. Dev CORS (port 8051) - Allows any origin:');
console.log('curl -v -H "Origin: http://localhost:3000" http://localhost:8051/studies 2>&1 | grep -i access-control\n');

console.log('# 3. Prod Single (port 8052) - Specific origin only:');
console.log('curl -v -H "Origin: https://viewer.hospital.com" http://localhost:8052/studies 2>&1 | grep -i access-control\n');

console.log('# 4. Prod Multi (port 8053) - Multiple allowed origins:');
console.log('curl -v -H "Origin: https://radiology.hospital.com" http://localhost:8053/studies 2>&1 | grep -i access-control\n');

console.log('🌐 Test from browser console:\n');
console.log('fetch("http://localhost:8051/studies")');
console.log('  .then(r => r.json())');
console.log('  .then(d => console.log("Studies:", d))');
console.log('  .catch(e => console.error("CORS Error:", e));\n');

console.log('📚 Documentation:');
console.log('   - DICOM PS3.18: https://dicom.nema.org/medical/dicom/current/output/html/part18.html');
console.log('   - MDN CORS: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS');
console.log('   - OHIF Viewer: https://docs.ohif.org/\n');

console.log('Press Ctrl+C to stop all servers...\n');

// Keep process alive
process.on('SIGINT', () => {
  console.log('\n\n⏹️  Stopping all servers...');
  devQido.stop();
  prodSingleQido.stop();
  prodMultiQido.stop();
  console.log('✅ All servers stopped\n');
  process.exit(0);
});
