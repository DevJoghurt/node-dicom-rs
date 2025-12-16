#!/usr/bin/env node

/**
 * Send test DICOM files to StoreSCP
 * 
 * Usage:
 *   node scripts/send-test-files.mjs <directory>
 *   node scripts/send-test-files.mjs ./playground/testdata
 */

import { StoreScu, DicomFile } from '@nuxthealth/node-dicom';
import { readdir, stat } from 'fs/promises';
import { join } from 'path';

const STORESCP_HOST = 'localhost';
const STORESCP_PORT = 11112;
const STORESCP_AET = 'PACS_SQLITE';
const STORESCU_AET = 'SEND_TEST';

async function findDicomFiles(dir) {
  const files = [];
  
  async function walk(currentPath) {
    const entries = await readdir(currentPath);
    
    for (const entry of entries) {
      const fullPath = join(currentPath, entry);
      const stats = await stat(fullPath);
      
      if (stats.isDirectory()) {
        await walk(fullPath);
      } else if (entry.endsWith('.dcm')) {
        files.push(fullPath);
      }
    }
  }
  
  await walk(dir);
  return files;
}

async function sendFiles(files) {
  console.log(`Found ${files.length} DICOM files`);
  console.log(`Sending to ${STORESCP_HOST}:${STORESCP_PORT} (AET: ${STORESCP_AET})`);
  console.log('');
  
  const storescu = new StoreScu({
    addr: `${STORESCP_HOST}:${STORESCP_PORT}`,
    callingAeTitle: STORESCU_AET,
    calledAeTitle: STORESCP_AET,
    verbose: true
  });
  
  // Add all files to the queue
  for (const filePath of files) {
    storescu.addFile(filePath);
  }
  
  let sent = 0;
  let failed = 0;
  
  // Send all files with event callbacks
  await storescu.send({
    onFileSending: (err, event) => {
      if (!err && event.data) {
        const dicomFile = new DicomFile(event.data.file);
        const tags = dicomFile.extract(['PatientID', 'StudyDescription', 'Modality']);
        console.log(`Sending: ${tags.PatientID} - ${tags.StudyDescription} (${tags.Modality})`);
      }
    },
    onFileSent: (err, event) => {
      if (!err) {
        sent++;
        console.log(`  ✓ Success`);
      }
    },
    onFileError: (err, event) => {
      failed++;
      console.error(`  ✗ Failed: ${err?.message || 'Unknown error'}`);
    },
    onTransferCompleted: (err, event) => {
      console.log('');
      console.log('=====================================');
      console.log(`Total: ${files.length} files`);
      console.log(`Sent: ${sent}`);
      console.log(`Failed: ${failed}`);
      console.log('=====================================');
    }
  });
}

// Main
const args = process.argv.slice(2);

if (args.length === 0) {
  console.error('Usage: node scripts/send-test-files.mjs <directory>');
  console.error('Example: node scripts/send-test-files.mjs ./playground/testdata');
  process.exit(1);
}

const directory = args[0];

try {
  const files = await findDicomFiles(directory);
  
  if (files.length === 0) {
    console.error(`No DICOM files found in: ${directory}`);
    process.exit(1);
  }
  
  await sendFiles(files);
} catch (error) {
  console.error('Error:', error.message);
  process.exit(1);
}
