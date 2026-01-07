#!/usr/bin/env node
/**
 * StoreScp Demo - Receive DICOM files via C-STORE
 * 
 * Run: node playground/storescp-demo.mjs
 * 
 * In another terminal, send files with:
 * node playground/storescu-demo.mjs
 */

import { StoreScp } from '../index.js';

console.log('🏥 StoreSCP Demo - DICOM C-STORE Receiver\n');

const receiver = new StoreScp({
  port: 4446,
  callingAeTitle: 'DEMO-SCP',
  outDir: './playground/test-received',
  verbose: false,
  extractTags: [
    'PatientName',
    'PatientID',
    'StudyDate',
    'Modality',
    'SeriesDescription'
  ],
  studyTimeout: 10
});

// Server started
receiver.onServerStarted((err, event) => {
  if (err) {
    console.error('❌ Server error:', err);
    return;
  }
  console.log('✅', event.message);
  console.log('   Ready to receive DICOM files...\n');
});

// File received
receiver.onFileStored((err, event) => {
  if (err) {
    console.error('❌ Storage error:', err);
    return;
  }
  
  const data = event.data;
  if (!data) return;
  
  console.log('📁 File received:', data.file);
  console.log('   Patient:', data.tags?.PatientName);
  console.log('   Modality:', data.tags?.Modality);
  console.log('   Series:', data.tags?.SeriesDescription);
});

// Study completed
receiver.onStudyCompleted((err, event) => {
  if (err) {
    console.error('❌ Study completion error:', err);
    return;
  }
  
  const study = event.data?.study;
  if (!study) return;
  
  const totalInstances = study.series.reduce((sum, s) => sum + s.instances.length, 0);
  
  console.log('\n🎉 Study Complete!');
  console.log('   Study UID:', study.studyInstanceUid);
  console.log('   Patient:', study.tags?.PatientName);
  console.log('   Series:', study.series.length);
  console.log('   Total Instances:', totalInstances);
  console.log('');
});

// Start server
receiver.start();

// Handle Ctrl+C
process.on('SIGINT', () => {
  console.log('\n\n🛑 Stopping server...');
  receiver.stop();
  process.exit(0);
});
