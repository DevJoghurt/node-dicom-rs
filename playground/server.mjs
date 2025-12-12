#!/usr/bin/env node
/**
 * Test StoreScp with typed event callbacks
 * 
 * This test verifies that:
 * - Event callbacks receive native JS objects (no JSON.parse needed)
 * - TypeScript autocomplete works for event data fields
 * - Different tag extraction modes (Scoped/Flat/StudyLevel) populate correct fields
 */

import { StoreScp } from '../index.js';

// Create SCP with ByScope grouping (default)
// Note: Can use enum (GroupingStrategy.ByScope) or string ('ByScope') - both have autocomplete!
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
    groupingStrategy: 'ByScope',
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

// Test onFileStored with native object access
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
    
    // Check which tag format was populated based on grouping strategy
    if (data.tagsScoped) {
      console.log('  Tags (Scoped):');
      console.log('    Patient:', data.tagsScoped.patient);
      console.log('    Study:', data.tagsScoped.study);
      console.log('    Series:', data.tagsScoped.series);
      console.log('    Instance:', data.tagsScoped.instance);
    }
    
    if (data.tagsFlat) {
      console.log('  Tags (Flat):', data.tagsFlat);
    }
    
    if (data.tagsStudyLevel) {
      console.log('  Tags (Study Level):');
      console.log('    Study Level:', data.tagsStudyLevel.studyLevel);
      console.log('    Instance Level:', data.tagsStudyLevel.instanceLevel);
    }
  }
});

// Test onStudyCompleted with hierarchy data
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
    
    // Access tags at study level based on grouping strategy
    if (study.tagsScoped) {
      console.log('  Study Tags (Scoped):');
      if (study.tagsScoped.patient) console.log('    Patient:', study.tagsScoped.patient);
      if (study.tagsScoped.study) console.log('    Study:', study.tagsScoped.study);
    }
    if (study.tagsFlat) {
      console.log('  Study Tags (Flat):', study.tagsFlat);
    }
    if (study.tagsStudyLevel?.studyLevel) {
      console.log('  Study Tags (StudyLevel):', study.tagsStudyLevel.studyLevel);
      console.log('    Note: Instance-level tags are at the instance level, not duplicated here');
    }
    
    // Iterate series
    for (const series of study.series) {
      console.log(`  Series ${series.seriesInstanceUid}:`);
      console.log(`    Instance count: ${series.instances.length}`);
      
      // Access series-level tags based on grouping strategy
      // Note: StudyLevel grouping doesn't have tags at series level
      if (series.tagsScoped?.series) {
        console.log('    Series tags (Scoped):', series.tagsScoped.series);
      }
      if (series.tagsFlat) {
        console.log('    Series tags (Flat):', series.tagsFlat);
      }
      
      // Access instance-level tags
      for (const instance of series.instances) {
        console.log(`    Instance ${instance.sopInstanceUid}:`);
        console.log(`      File: ${instance.file}`);
        
        if (instance.tagsScoped?.instance) {
          console.log('      Instance tags (Scoped):', instance.tagsScoped.instance);
        }
        if (instance.tagsFlat) {
          console.log('      Instance tags (Flat):', instance.tagsFlat);
        }
        if (instance.tagsStudyLevel?.instanceLevel) {
          console.log('      Instance tags (StudyLevel):', instance.tagsStudyLevel.instanceLevel);
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
