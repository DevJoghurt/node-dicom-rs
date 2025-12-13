/**
 * StoreScp Tag Validation Demo
 * 
 * This demo demonstrates using the onBeforeStore callback to validate
 * and enrich DICOM tags before storage. This can be used to:
 * - Ensure required tags are present
 * - Add institution-specific metadata
 * - Standardize tag values
 * - Reject files that don't meet requirements
 */

import { StoreScp } from '../../index.js';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const SCP_PORT = 11116;
const OUTPUT_DIR = join(__dirname, '../test-received-validated');

console.log('='.repeat(80));
console.log('DICOM C-STORE SCP - Tag Validation & Enrichment Demo');
console.log('='.repeat(80));
console.log();

// Create SCP with tag extraction
const scp = new StoreScp({
  port: SCP_PORT,
  outDir: OUTPUT_DIR,
  verbose: true,
  extractTags: [
    'PatientName',
    'PatientID',
    'StudyDescription',
    'Modality',
    'InstitutionName'
  ]
});

/**
 * Validation and enrichment callback
 */
scp.onBeforeStore((err, tags) => {
  if (err) {
    console.error('Error in callback:', err);
    return tags;
  }

  console.log('\n📋 Processing DICOM file...');
  console.log('Original tags:', tags);

  // Enrich with institution-specific metadata
  const enrichedTags = {
    ...tags,
    // Add institution name if missing
    InstitutionName: tags.InstitutionName || 'Default Institution',
    
    // Standardize patient name format (convert to uppercase)
    PatientName: tags.PatientName ? tags.PatientName.toUpperCase() : '',
    
    // Prefix study description with modality for easier searching
    StudyDescription: tags.Modality && tags.StudyDescription ?
      `[${tags.Modality}] ${tags.StudyDescription}` :
      tags.StudyDescription
  };

  console.log('✅ Enriched tags:', enrichedTags);
  console.log();

  return enrichedTags;
});

// Event listeners
scp.onServerStarted((data) => {
  console.log('✅ Server started on port', SCP_PORT);
  console.log('📁 Output directory:', OUTPUT_DIR);
  console.log('🔍 Validation & Enrichment: ENABLED');
  console.log();
  console.log('Waiting for DICOM files...');
  console.log();
});

scp.onFileStored((data) => {
  const details = data.data;
  console.log('💾 File stored:', details.file);
  
  if (details.tags) {
    console.log('   Final tags:', details.tags);
  }
  console.log();
});

scp.onError((data) => {
  console.error('❌ Error:', data.message);
});

// Handle shutdown
process.on('SIGINT', async () => {
  console.log('\n\nShutting down server...');
  await scp.close();
  console.log('✅ Server stopped');
  process.exit(0);
});

// Start listening
console.log('Starting DICOM C-STORE SCP server...\n');
await scp.listen();
