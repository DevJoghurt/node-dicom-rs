#!/usr/bin/env node
/**
 * StoreScu Demo - Send DICOM files via C-STORE
 * 
 * Run: node playground/storescu-demo.mjs
 * Prerequisites: 
 * 1. Run ./playground/downloadTestData.sh first
 * 2. Start receiver: node playground/storescp-demo.mjs
 */

import { StoreScu } from '../index.js';

console.log('📤 StoreSCU Demo - DICOM C-STORE Sender\n');

const sender = new StoreScu({
  callingAeTitle: 'DEMO-SCU',
  calledAeTitle: 'DEMO-SCP',
  addr: '127.0.0.1:4446',
  verbose: false
});

// Add files from testdata
sender.addFolder('./playground/testdata/1.3.6.1.4.1.9328.50.2.125354');

console.log('🚀 Starting transfer...\n');

// Send with progress tracking
await sender.send({
  onTransferStarted: (err, event) => {
    if (err) {
      console.error('❌ Transfer error:', err);
      return;
    }
    console.log('✅', event.message);
    console.log('   Total files:', event.data?.totalFiles);
    console.log('');
  },
  
  onFileSent: (err, event) => {
    if (err) {
      console.error('❌ File error:', err);
      return;
    }
    const data = event.data;
    if (!data) return;
    
    console.log('✓ Sent:', data.file);
    console.log('  SOP Instance:', data.sopInstanceUid);
    console.log('  Duration:', data.durationSeconds.toFixed(2), 'seconds');
  },
  
  onFileError: (err, event) => {
    const data = event.data;
    console.error('✗ Failed:', data?.file);
    console.error('  Error:', data?.error);
  },
  
  onTransferCompleted: (err, event) => {
    if (err) {
      console.error('❌ Completion error:', err);
      return;
    }
    const data = event.data;
    if (!data) return;
    
    console.log('\n🎉 Transfer Complete!');
    console.log('   Successful:', data.successful, '/', data.totalFiles);
    console.log('   Failed:', data.failed);
    console.log('   Duration:', data.durationSeconds.toFixed(2), 'seconds');
  }
});

console.log('\n✅ Demo complete!');
