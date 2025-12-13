/**
 * StoreScp Anonymization Demo
 * 
 * This demo demonstrates how to use the onBeforeStore callback to anonymize
 * DICOM files before they are saved. This is useful for:
 * - De-identifying patient data before storage
 * - Applying institution-specific tag modifications
 * - Validating and enriching metadata
 * 
 * The callback receives the extracted tags and can modify them before the
 * file is written to disk.
 */

import { StoreScp } from '../../index.js';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration
const SCP_PORT = 11115;
const OUTPUT_DIR = join(__dirname, '../test-received-anon');

console.log('='.repeat(80));
console.log('DICOM C-STORE SCP - Anonymization Demo');
console.log('='.repeat(80));
console.log();

// Create SCP with tag extraction
const scp = new StoreScp({
  port: SCP_PORT,
  outDir: OUTPUT_DIR,
  verbose: true,
  storeWithFileMeta: true, // Store complete DICOM files so we can read them back
  // Extract patient and study tags that we want to anonymize
  extractTags: [
    'PatientName',
    'PatientID', 
    'PatientBirthDate',
    'PatientSex',
    'PatientAge',
    'StudyDescription',
    'StudyInstanceUID',
    'SeriesInstanceUID',
    'SOPInstanceUID'
  ]
});

// Counter for generating anonymous IDs
let anonymousCounter = 1000;
const patientMapping = new Map(); // Map original patient IDs to anonymous IDs

/**
 * Anonymization callback
 * 
 * This function is called for EVERY received DICOM file BEFORE it is saved.
 * It receives the extracted tags and returns modified tags.
 */
console.log('🔧 Registering anonymization callback...');
scp.onBeforeStore((err, tags) => {
  console.log('\n🎯 CALLBACK INVOKED!'); // Debug log
  
  if (err) {
    console.error('Error in callback:', err);
    return tags;
  }

  console.log('\n📋 Received DICOM file - Anonymizing...');
  console.log('Original tags:', {
    PatientName: tags.PatientName,
    PatientID: tags.PatientID,
    PatientBirthDate: tags.PatientBirthDate,
    StudyDescription: tags.StudyDescription
  });

  // Get or create anonymous patient ID
  let anonymousID;
  if (patientMapping.has(tags.PatientID)) {
    anonymousID = patientMapping.get(tags.PatientID);
  } else {
    anonymousID = `ANON_${String(anonymousCounter++).padStart(4, '0')}`;
    patientMapping.set(tags.PatientID, anonymousID);
  }

  // Create anonymized tags
  const anonymizedTags = {
    ...tags,
    // Remove patient identifying information
    PatientName: 'ANONYMOUS^PATIENT',
    PatientID: anonymousID,
    PatientBirthDate: '', // Clear birth date
    PatientSex: '', // Clear sex (or could map to 'O' for Other)
    PatientAge: '', // Clear age
    
    // Optionally modify study description
    StudyDescription: tags.StudyDescription ? 
      `ANONYMIZED - ${tags.StudyDescription}` : 
      'ANONYMIZED STUDY',
    
    // Keep UIDs unchanged for proper DICOM structure
    StudyInstanceUID: tags.StudyInstanceUID,
    SeriesInstanceUID: tags.SeriesInstanceUID,
    SOPInstanceUID: tags.SOPInstanceUID
  };

  console.log('✅ Anonymized tags:', {
    PatientName: anonymizedTags.PatientName,
    PatientID: anonymizedTags.PatientID,
    PatientBirthDate: anonymizedTags.PatientBirthDate,
    StudyDescription: anonymizedTags.StudyDescription
  });
  console.log();

  // Return the modified tags - these will be written to the file
  return anonymizedTags;
});

console.log('✅ Callback registered successfully');
console.log();

// Set up event listeners
scp.onServerStarted((data) => {
  console.log('✅ Server started on port', SCP_PORT);
  console.log('📁 Output directory:', OUTPUT_DIR);
  console.log('🔐 Anonymization: ENABLED');
  console.log();
  console.log('Waiting for DICOM files...');
  console.log('Send files using:');
  console.log(`  node playground/send.mjs`);
  console.log();
});

scp.onFileStored((data) => {
  if (!data || !data.data) {
    console.log('⚠️  File stored (no details available)');
    return;
  }
  
  const details = data.data;
  console.log('💾 File stored:', details.file);
  console.log('   SOP Instance UID:', details.sopInstanceUid);
  
  if (details.tags) {
    console.log('   Final tags in file:', {
      PatientName: details.tags.PatientName,
      PatientID: details.tags.PatientID,
      StudyDescription: details.tags.StudyDescription
    });
  }
  console.log();
});

scp.onStudyCompleted((data) => {
  const studyData = data.data.study;
  console.log('\n📚 Study completed!');
  console.log('   Study Instance UID:', studyData.studyInstanceUid);
  console.log('   Total series:', studyData.series.length);
  
  let totalInstances = 0;
  for (const series of studyData.series) {
    totalInstances += series.instances.length;
  }
  console.log('   Total instances:', totalInstances);
  
  if (studyData.tags) {
    console.log('   Study tags:', {
      PatientName: studyData.tags.PatientName,
      PatientID: studyData.tags.PatientID,
      StudyDescription: studyData.tags.StudyDescription
    });
  }
  console.log();
});

scp.onError((data) => {
  console.error('❌ Error:', data.message);
  if (data.data && data.data.error) {
    console.error('   Details:', data.data.error);
  }
});

// Handle shutdown
process.on('SIGINT', async () => {
  console.log('\n\nShutting down server...');
  
  // Print mapping summary
  console.log('\n📊 Anonymization Summary:');
  console.log('   Total patients anonymized:', patientMapping.size);
  console.log('   Mapping:');
  for (const [original, anonymous] of patientMapping.entries()) {
    console.log(`     ${original} → ${anonymous}`);
  }
  
  await scp.close();
  console.log('✅ Server stopped');
  process.exit(0);
});

// Start listening
console.log('Starting DICOM C-STORE SCP server...\n');
await scp.listen();
