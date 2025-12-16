#!/usr/bin/env node
import { QidoServer, QidoStudyResult } from '../../index.js';

console.log('\n=== QIDO-RS CORS Configuration Demo ===\n');

// Example 1: Basic server without CORS (restrictive)
console.log('1. Creating QIDO server WITHOUT CORS (internal network only):');
const qidoBasic = new QidoServer(8043, {
  verbose: true
});
console.log('   ✓ Server configured for internal access only\n');

// Example 2: Development server with CORS enabled (all origins)
console.log('2. Creating QIDO server WITH CORS (allows all origins):');
const qidoDev = new QidoServer(8044, {
  enableCors: true,
  verbose: true
});

// Register a simple handler
qidoDev.onSearchForStudies((err, query) => {
  if (err) {
    console.error('Error:', err);
    return JSON.stringify([]);
  }
  
  console.log('   Received query:', {
    patientName: query.PatientName,
    studyDate: query.StudyDate,
    limit: query.limit
  });
  
  // Create sample response
  const result = new QidoStudyResult();
  result.studyInstanceUid('1.2.3.4.5');
  result.studyDate('20240116');
  result.patientName('DOE^JOHN');
  result.patientId('12345');
  result.accessionNumber('ACC001');
  result.studyDescription('CT CHEST');
  
  return JSON.stringify([result.getAttributes()]);
});

qidoDev.start();
console.log('   ✓ Server started with CORS: * (all origins)\n');

// Example 3: Production server with specific allowed origins
console.log('3. Creating QIDO server WITH SPECIFIC CORS origins:');
const qidoProd = new QidoServer(8045, {
  enableCors: true,
  corsAllowedOrigins: 'https://viewer.hospital.com,https://app.hospital.com',
  verbose: true
});
console.log('   ✓ Server configured for specific origins only\n');

console.log('=== Test the servers ===\n');
console.log('Test CORS from browser console or curl:\n');
console.log('# Without CORS (port 8043) - will fail from browser:');
console.log('curl -H "Origin: http://localhost:3000" \\');
console.log('     -H "Access-Control-Request-Method: GET" \\');
console.log('     -H "Access-Control-Request-Headers: X-Requested-With" \\');
console.log('     --head http://localhost:8043/studies\n');

console.log('# With CORS all origins (port 8044) - will succeed:');
console.log('curl -H "Origin: http://localhost:3000" \\');
console.log('     -H "Access-Control-Request-Method: GET" \\');
console.log('     -H "Access-Control-Request-Headers: X-Requested-With" \\');
console.log('     --head http://localhost:8044/studies\n');

console.log('# With specific CORS (port 8045) - only allowed origins:');
console.log('curl -H "Origin: https://viewer.hospital.com" \\');
console.log('     -H "Access-Control-Request-Method: GET" \\');
console.log('     -H "Access-Control-Request-Headers: X-Requested-With" \\');
console.log('     --head http://localhost:8045/studies\n');

console.log('# Test actual query with CORS:');
console.log('curl -v "http://localhost:8044/studies?PatientName=DOE&limit=10"\n');

console.log('Press Ctrl+C to stop servers...\n');

// Keep process alive
process.on('SIGINT', () => {
  console.log('\n\nStopping servers...');
  qidoDev.stop();
  qidoProd.stop();
  process.exit(0);
});
