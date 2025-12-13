#!/usr/bin/env node
/**
 * Test StoreScp with typed event callbacks
 * 
 * This test verifies that:
 * - Event callbacks receive native JS objects (no JSON.parse needed)
 * - TypeScript autocomplete works for event data fields
 * - Flat tags for OnFileStored, hierarchical with flat tags for OnStudyCompleted
 */

import { StoreScp } from '../index.js';

// Create SCP with simplified tag extraction
const scp = new StoreScp({
    port: 4446,
    storageBackend: 'S3',
    s3Config: {
        accessKey: 'user',
        secretKey: 'password',
        bucket: 'dicom-resend',
        endpoint: 'http://localhost:7070'
    },
    verbose: false,
    extractTags: ['PatientName', 'PatientID', 'StudyInstanceUID', 'SeriesInstanceUID', 'Modality', 'InstanceNumber', 'SOPInstanceUID', 'ImageType', 'Rows', 'Columns'],
    studyTimeout: 40
});

// Test onServerStarted
scp.onServerStarted((err, event) => {
  if (err) {
    console.error('❌ Server error:', err);
    return;
  }
  console.log('✓ Server started:', event.message);
});

// Test onConnection
scp.onConnection((err, event) => {
  if (err) {
    console.error('❌ Connection error:', err);
    return;
  }
  
  console.log('✓ New connection:', event.message);
});

// Test onFileStored with flat tags
scp.onFileStored((err, event) => {
  if (err) {
    console.error('❌ File storage error:', err);
    return;
  }
  
  console.log('✓ File stored:', event.message);
  
  if (event.data) {
    const { data } = event;
    
    // Direct property access - no JSON.parse needed!
    console.log('  File:', data.file);
    console.log('  SOP Instance UID:', data.sopInstanceUid);
    console.log('  Study Instance UID:', data.studyInstanceUid);
    console.log('  Series Instance UID:', data.seriesInstanceUid);
    
    // Tags are always flat for simple, direct access
    if (data.tags) {
      console.log('  Tags (Flat):');
      console.log('    Patient Name:', data.tags.PatientName);
      console.log('    Patient ID:', data.tags.PatientID);
      console.log('    Modality:', data.tags.Modality);
      console.log('    Instance Number:', data.tags.InstanceNumber);
      // All tags are at the root level
    }
  }
});

// Test onStudyCompleted with hierarchical structure
scp.onStudyCompleted((err, event) => {
  if (err) {
    console.error('❌ Study completion error:', err);
    return;
  }
  
  console.log('✓ Study completed:', event.message);
  
  if (event.data?.study) {
    const { study } = event.data;
    
    console.log('  Study UID:', study.studyInstanceUid);
    console.log('  Series count:', study.series.length);
    
    // Access tags at study level (Patient + Study tags)
    if (study.tags) {
      console.log('  Study Tags:');
      console.log('    Patient Name:', study.tags.PatientName);
      console.log('    Patient ID:', study.tags.PatientID);
      console.log('    Study Description:', study.tags.StudyDescription);
    }
    
    // Iterate series
    for (const series of study.series) {
      console.log(`  Series ${series.seriesInstanceUid}:`);
      console.log(`    Instance count: ${series.instances.length}`);
      
      // Access series-level tags
      if (series.tags) {
        console.log('    Series tags:');
        console.log('      Modality:', series.tags.Modality);
        console.log('      Series Description:', series.tags.SeriesDescription);
      }
      
      // Access instance-level tags
      for (const instance of series.instances) {
        console.log(`    Instance ${instance.sopInstanceUid}:`);
        console.log(`      File: ${instance.file}`);
        
        if (instance.tags) {
          console.log('      Instance tags:');
          console.log('        Instance Number:', instance.tags.InstanceNumber);
        }
      }
    }
  }
});

// Test onError
scp.onError((err, event) => {
  if (err) {
    console.error('❌ Error handler error:', err);
    return;
  }
  
  console.log('⚠️  Error event:', event.message);
  if (event.data?.error) {
    console.log('  Error details:', event.data.error);
  }
});

// Start server
console.log('Starting SCP server on port 11113...');
console.log('Send DICOM files with:');
console.log('  node playground/send.mjs');
console.log('');

await scp.listen();

// Graceful shutdown
async function exitHandler(evtOrExitCodeOrError) {
    console.log('EXIT HANDLER', evtOrExitCodeOrError);
    try {
      if(platform() !== 'win32') {
        await scp.close();
      }
    } catch (e) {
      console.error('EXIT HANDLER ERROR', e);
    }
    console.log('EXIT HANDLER DONE');
    process.exit(isNaN(+evtOrExitCodeOrError) ? 1 : +evtOrExitCodeOrError);
}

[
    'beforeExit', 'uncaughtException', 'unhandledRejection',
    'SIGHUP', 'SIGINT', 'SIGQUIT', 'SIGILL', 'SIGTRAP',
    'SIGABRT','SIGBUS', 'SIGFPE', 'SIGUSR1', 'SIGSEGV',
    'SIGUSR2', 'SIGTERM',
].forEach(evt => process.on(evt, exitHandler));
