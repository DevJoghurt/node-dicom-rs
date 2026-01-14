#!/usr/bin/env node
/**
 * Test onBeforeStore async callback functionality
 * 
 * This test:
 * 1. Starts a StoreSCP with onBeforeStore callback that anonymizes data
 * 2. Sends test DICOM files to the SCP
 * 3. Reads back the stored files and verifies the tags were modified
 * 4. Tests async database-like operations
 */

import { StoreScp, StoreScu, DicomFile } from '../index.js';
import { readFileSync, readdirSync, existsSync, mkdirSync, rmSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const TEST_PORT = 11119;
const TEST_AET = 'TEST_SCP';
const OUTPUT_DIR = join(__dirname, 'test-output-onbeforestore');
const TESTDATA_DIR = join(__dirname, 'testdata');

// Simulated async database for patient mapping
class PatientMappingDB {
  constructor() {
    this.mappings = new Map();
    this.counter = 1000;
  }

  // Simulate async database lookup
  async getMapping(originalId) {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 10));
    return this.mappings.get(originalId);
  }

  // Simulate async database insert
  async createMapping(originalId, originalName) {
    await new Promise(resolve => setTimeout(resolve, 15));
    
    const anonId = `ANON_${String(this.counter++).padStart(4, '0')}`;
    const mapping = {
      originalId,
      originalName,
      anonId,
      anonName: 'ANONYMOUS^PATIENT',
      anonBirthDate: '19700101',
      anonSex: 'O',
      timestamp: new Date().toISOString()
    };
    
    this.mappings.set(originalId, mapping);
    console.log(`  [DB] Created mapping: ${originalId} → ${anonId}`);
    return mapping;
  }
}

const db = new PatientMappingDB();

// Track what we receive
const receivedFiles = [];
const modifiedTags = new Map();

async function runTest() {
  console.log('='.repeat(70));
  console.log('Testing onBeforeStore Async Callback');
  console.log('='.repeat(70));
  console.log();

  // Clean up previous test output
  if (existsSync(OUTPUT_DIR)) {
    console.log('Cleaning up previous test output...');
    rmSync(OUTPUT_DIR, { recursive: true, force: true });
  }
  mkdirSync(OUTPUT_DIR, { recursive: true });

  // Step 1: Create SCP with async onBeforeStore callback
  console.log('Step 1: Creating StoreSCP with async onBeforeStore callback...');
  const scp = new StoreScp({
    port: TEST_PORT,
    callingAeTitle: TEST_AET,
    outDir: OUTPUT_DIR,
    storeWithFileMeta: true, // Important: store complete DICOM files
    extractTags: [
      'PatientName',
      'PatientID',
      'PatientBirthDate',
      'PatientSex',
      'StudyInstanceUID',
      'SeriesInstanceUID',
      'SOPInstanceUID',
      'StudyDescription',
      'Modality'
    ],
    verbose: true
  });

  // Register async callback (receives/returns JSON strings)
  scp.onBeforeStore(async (error, tagsJson) => {
    if (error) {
        console.error('  [Callback] Error received:', error);
        throw error;
    }
    try {
      const tags = JSON.parse(tagsJson);
      console.log(`\n  [Callback] Received object with`, Object.keys(tags).length, 'tags');
      
      const originalId = tags.PatientID || 'UNKNOWN';
      const originalName = tags.PatientName || 'UNKNOWN';

      // Async database lookup
      let mapping = await db.getMapping(originalId);
      
      if (!mapping) {
        // Create new mapping with async database insert
        mapping = await db.createMapping(originalId, originalName);
      } else {
        console.log(`  [Callback] Using existing mapping: ${originalId} → ${mapping.anonId}`);
      }

      // Track the modified tags for verification
      modifiedTags.set(tags.SOPInstanceUID, {
        original: { ...tags },
        modified: {
          PatientID: mapping.anonId,
          PatientName: mapping.anonName,
          PatientBirthDate: mapping.anonBirthDate,
          PatientSex: mapping.anonSex
        }
      });

      // Return modified tags as JSON string
      const modified = {
        ...tags,
        PatientName: mapping.anonName,
        PatientID: mapping.anonId,
        PatientBirthDate: mapping.anonBirthDate,
        PatientSex: mapping.anonSex
      };

      console.log(`  [Callback] ✓ Anonymized: ${originalName} (${originalId}) → ${modified.PatientName} (${modified.PatientID})`);
      return JSON.stringify(modified);
    } catch (error) {
      console.error(`  [Callback] Error:`, error.message);
      throw error;
    }
  });

  // Track received files
  scp.onFileStored((err, event) => {
    if (err) {
      console.error('  [SCP] Error storing file:', err);
      return;
    }
    
    const data = event.data;
    if (data) {
      receivedFiles.push({
        file: data.file,
        sopInstanceUid: data.sopInstanceUid,
        tags: data.tags
      });
      console.log(`  [SCP] File stored: ${data.sopInstanceUid}`);
      console.log(`        Patient: ${data.tags?.PatientName} (${data.tags?.PatientID})`);
    }
  });

  // Start SCP
  scp.start();
  console.log(`✓ SCP started on port ${TEST_PORT}\n`);

  // Wait for server to be ready
  await new Promise(resolve => setTimeout(resolve, 500));

  // Step 2: Find test files and send them
  console.log('Step 2: Sending test DICOM files...');
  
  const testFiles = [];
  const studyDirs = readdirSync(TESTDATA_DIR, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .slice(0, 2); // Use first 2 studies

  for (const studyDir of studyDirs) {
    const studyPath = join(TESTDATA_DIR, studyDir.name);
    const files = readdirSync(studyPath)
      .filter(f => f.endsWith('.dcm'))
      .slice(0, 3) // Take 3 files per study
      .map(f => join(studyPath, f));
    testFiles.push(...files);
  }

  if (testFiles.length === 0) {
    console.error('ERROR: No test DICOM files found!');
    console.error(`Please ensure there are .dcm files in: ${TESTDATA_DIR}`);
    scp.stop();
    process.exit(1);
  }

  console.log(`Found ${testFiles.length} test files to send\n`);

  // Send files
  const scu = new StoreScu({
    callingAeTitle: 'TEST_SCU',
    calledAeTitle: TEST_AET,
    addr: `localhost:${TEST_PORT}`,
    verbose: false
  });

  // Add all test files
  for (const filePath of testFiles) {
    scu.addFile(filePath);
  }

  console.log(`Sending ${testFiles.length} files...\n`);
  
  try {
    await scu.send({
      onTransferStarted: (err, event) => {
        if (!err && event) {
          console.log(`  ✓ Sent: ${event.filename}`);
        }
      },
      onTransferCompleted: (err, event) => {
        if (err) {
          console.error(`  ✗ Transfer failed: ${err}`);
        }
      }
    });
  } catch (error) {
    console.error(`Transfer error: ${error.message}`);
  }

  console.log();

  // Wait for all files to be processed
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Wait for all files to be processed
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Step 3: Verify the modifications by reading back the stored files
  console.log('Step 3: Verifying tag modifications in stored files...');
  console.log();

  let passed = 0;
  let failed = 0;

  for (const received of receivedFiles) {
    const expectedMods = modifiedTags.get(received.sopInstanceUid);
    if (!expectedMods) {
      console.log(`  [SKIP] No modification record for ${received.sopInstanceUid}`);
      continue;
    }

    console.log(`  Verifying: ${received.file}`);
    
    try {
      // Read the stored DICOM file
      const storedFile = new DicomFile(received.file);
      const storedTags = storedFile.toObject();

      // Check if modifications were applied
      const checks = [
        {
          tag: 'PatientName',
          expected: expectedMods.modified.PatientName,
          actual: storedTags.PatientName
        },
        {
          tag: 'PatientID',
          expected: expectedMods.modified.PatientID,
          actual: storedTags.PatientID
        },
        {
          tag: 'PatientBirthDate',
          expected: expectedMods.modified.PatientBirthDate,
          actual: storedTags.PatientBirthDate
        },
        {
          tag: 'PatientSex',
          expected: expectedMods.modified.PatientSex,
          actual: storedTags.PatientSex
        }
      ];

      let fileOk = true;
      for (const check of checks) {
        if (check.actual === check.expected) {
          console.log(`    ✓ ${check.tag}: "${check.expected}"`);
        } else {
          console.log(`    ✗ ${check.tag}: expected "${check.expected}", got "${check.actual}"`);
          fileOk = false;
        }
      }

      if (fileOk) {
        console.log(`  ✓ File verification PASSED`);
        passed++;
      } else {
        console.log(`  ✗ File verification FAILED`);
        failed++;
      }
    } catch (error) {
      console.log(`  ✗ Error reading file: ${error.message}`);
      failed++;
    }
    console.log();
  }

  // Stop SCP
  scp.stop();
  console.log('SCP stopped\n');

  // Step 4: Summary
  console.log('='.repeat(70));
  console.log('Test Summary');
  console.log('='.repeat(70));
  console.log(`Files sent: ${testFiles.length}`);
  console.log(`Files received: ${receivedFiles.length}`);
  console.log(`Verifications passed: ${passed}`);
  console.log(`Verifications failed: ${failed}`);
  console.log(`Database mappings created: ${db.mappings.size}`);
  console.log();

  // Display database mappings
  console.log('Patient Mappings:');
  for (const [originalId, mapping] of db.mappings.entries()) {
    console.log(`  ${originalId} → ${mapping.anonId} (created: ${mapping.timestamp})`);
  }
  console.log();

  if (failed === 0 && passed > 0) {
    console.log('✓ ALL TESTS PASSED!');
    console.log('  - Async callback executed successfully');
    console.log('  - Tags were modified before storage');
    console.log('  - Stored files contain anonymized data');
    console.log('  - Async database operations worked correctly');
    process.exit(0);
  } else {
    console.log('✗ TESTS FAILED!');
    if (passed === 0) {
      console.log('  - No files were verified successfully');
      console.log('  - Check if onBeforeStore callback is working');
    } else {
      console.log(`  - ${failed} file(s) had incorrect modifications`);
    }
    process.exit(1);
  }
}

// Run the test
runTest().catch(error => {
  console.error('Test failed with error:', error);
  process.exit(1);
});
