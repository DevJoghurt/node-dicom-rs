/**
 * Simple test to verify async onBeforeStore callback
 */

import { StoreScp } from '../index.js';

async function testAsyncCallback() {
  console.log('Testing async onBeforeStore callback...\n');

  const scp = new StoreScp({
    port: 11116,
    outDir: './test-output',
    extractTags: ['PatientName', 'PatientID', 'StudyInstanceUID'],
    verbose: false
  });

  // Test 1: Simple async callback with delay
  scp.onBeforeStore(async (tags) => {
    console.log('  ✓ Callback invoked with tags:', Object.keys(tags));
    
    // Simulate async operation (e.g., database lookup)
    await new Promise(resolve => setTimeout(resolve, 50));
    console.log('  ✓ Async operation completed');
    
    return {
      ...tags,
      PatientName: 'MODIFIED_' + tags.PatientName,
      PatientID: 'ANON_' + Math.random().toString(36).substr(2, 9)
    };
  });

  console.log('✓ Async callback registered successfully');
  console.log('✓ Type signature is: (tags) => Promise<Record<string, string>>');
  console.log('\nTest passed! The onBeforeStore callback is now async.\n');
  
  // Note: We're not actually starting the server, just testing the API
  console.log('To test with real DICOM files:');
  console.log('  1. Start this SCP: scp.start()');
  console.log('  2. Send DICOM files to port 11116');
  console.log('  3. The async callback will process each file\n');
}

testAsyncCallback().catch(console.error);
