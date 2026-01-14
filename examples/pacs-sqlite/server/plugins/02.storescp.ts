/**
 * StoreSCP Plugin - DICOM C-STORE receiver
 * 
 * Receives DICOM files, anonymizes with fake data, stores to filesystem,
 * and saves metadata to SQLite database
 */

import { StoreScp, getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';
import { join } from 'path';
import { mkdirSync } from 'fs';
import crypto from 'crypto';
import { defineNitroPlugin, useDatabase } from "#imports";

const STORESCP_PORT: number = 11112;
const STORESCP_AET: string = 'PACS_SQLITE';
const DICOM_STORAGE_PATH: string = join(process.cwd(), '.data', 'dicom');

// Fake data generators (same as initialize-testdata.mjs)
const FIRST_NAMES = [
  'James', 'Mary', 'John', 'Patricia', 'Robert', 'Jennifer', 'Michael', 'Linda',
  'William', 'Barbara', 'David', 'Elizabeth', 'Richard', 'Susan', 'Joseph', 'Jessica'
];

const LAST_NAMES = [
  'Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis',
  'Rodriguez', 'Martinez', 'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson', 'Thomas'
];

interface FakePatientData {
  patientName: string;
  patientID: string;
  patientBirthDate: string;
  patientSex: string;
}

function generateFakePatientData(patientID: string | undefined): FakePatientData {
  if (!patientID) {
    patientID = 'UNKNOWN_' + Math.random().toString(36).substring(7);
  }
  
  // Use patient ID as seed for consistent data
  const hash: Buffer = crypto.createHash('sha256').update(patientID).digest();
  const seed: number = hash.readUInt32BE(0);
  
  function seededRandom(index: number): number {
    const x: number = Math.sin(seed + index) * 10000;
    return x - Math.floor(x);
  }
  
  const firstName: string = FIRST_NAMES[Math.floor(seededRandom(1) * FIRST_NAMES.length)];
  const lastName: string = LAST_NAMES[Math.floor(seededRandom(2) * LAST_NAMES.length)];
  
  const year: number = 1940 + Math.floor(seededRandom(4) * 70);
  const month: number = 1 + Math.floor(seededRandom(5) * 12);
  const day: number = 1 + Math.floor(seededRandom(6) * 28);
  const birthDate: string = `${year}${month.toString().padStart(2, '0')}${day.toString().padStart(2, '0')}`;
  
  const sex: string = seededRandom(7) > 0.5 ? 'M' : 'F';
  
  return {
    patientName: `${lastName}^${firstName}`,
    patientID: patientID,
    patientBirthDate: birthDate,
    patientSex: sex
  };
}

export default defineNitroPlugin(async (nitroApp) => {
  console.log('[StoreSCP] Starting DICOM C-STORE receiver...');
  
  // Create storage directory
  mkdirSync(DICOM_STORAGE_PATH, { recursive: true });
  
  // Extract comprehensive metadata tags
  const tagSets = getCommonTagSets();
  const extractionTags = combineTags([
    tagSets.patientBasic,
    tagSets.studyBasic,
    tagSets.seriesBasic,
    tagSets.instanceBasic,
    tagSets.imagePixelInfo
  ]);
  
  // DEBUG: Log extraction tags
  console.log('[StoreSCP] Configured with', extractionTags.length, 'tags:', extractionTags.slice(0, 10), '...');
  
  // Create StoreSCP instance with correct options format
  const storeScp = new StoreScp({
    port: STORESCP_PORT,
    callingAeTitle: STORESCP_AET,
    outDir: DICOM_STORAGE_PATH,
    verbose: false,
    extractTags: extractionTags,
    storeWithFileMeta: true,
    studyTimeout: 30
  });
  
  // Anonymize before storing (async callback with database mapping)
  storeScp.onBeforeStore(async (error: Error | null, tagsJson: string) => {
    // Check for errors first (error-first callback pattern)
    if (error) {
      console.error('[StoreSCP] Callback error:', error);
      throw error;
    }
    
    try {
      // Parse JSON string to object
      const tags = JSON.parse(tagsJson);
      console.log('[StoreSCP] Processing tags:', Object.keys(tags).length, 'tags received');
      
      // Extract original patient data
      const originalPatientID: string = tags.PatientID || 'UNKNOWN';
      const originalPatientName: string = tags.PatientName || '';
      
      // Look up existing mapping in database
      const db = useDatabase();
      
      // Generate fake data first (deterministic based on patient ID)
      const fakeData = generateFakePatientData(originalPatientID);
      
      // Use INSERT OR IGNORE to handle race conditions when multiple files
      // with same patient arrive simultaneously
      await db.sql`
        INSERT OR IGNORE INTO patient_mapping (
          original_patient_id, original_patient_name,
          anonymized_patient_id, anonymized_patient_name,
          anonymized_birth_date, anonymized_sex
        ) VALUES (
          ${originalPatientID}, ${originalPatientName},
          ${fakeData.patientID}, ${fakeData.patientName},
          ${fakeData.patientBirthDate}, ${fakeData.patientSex}
        )
      `;
      
      // Fetch the mapping (either just inserted or existing)
      const result = await db.sql`
        SELECT * FROM patient_mapping WHERE original_patient_id = ${originalPatientID}
      `;
      
      // Database wrapper returns { rows: [...], success: true }
      const mappingRows = result.rows;
      
      if (!mappingRows || mappingRows.length === 0) {
        console.error('[StoreSCP] Failed to fetch mapping after insert for:', originalPatientID);
        throw new Error('Mapping not found after insert');
      }
      
      const mapping = mappingRows[0];
      
      // Use the mapping from database (ensures consistency even if INSERT was ignored)
      const finalFakeData: FakePatientData = {
        patientName: mapping.anonymized_patient_name as string,
        patientID: mapping.anonymized_patient_id as string,
        patientBirthDate: mapping.anonymized_birth_date as string,
        patientSex: mapping.anonymized_sex as string
      };
      
      console.log(`[StoreSCP] ✓ Mapped: ${originalPatientID} → ${finalFakeData.patientID}`);
      
      // Return ONLY the modified tags (not all tags)
      // This prevents corruption of pixel data and other technical tags
      const modified = {
        PatientName: finalFakeData.patientName,
        PatientID: finalFakeData.patientID,
        PatientBirthDate: finalFakeData.patientBirthDate,
        PatientSex: finalFakeData.patientSex
      };
      
      return JSON.stringify(modified);
    } catch (error: any) {
      console.error('[StoreSCP] Anonymization error:', error.message);
      console.error('[StoreSCP] Stack:', error.stack);
      // Return original tags to allow storage
      return tagsJson;
    }
  });
  
  // Store metadata in database after file is saved
  storeScp.onFileStored(async (err: Error | null, event: any) => {
    if (err) {
      console.error('[StoreSCP] Error in onFileStored:', err);
      return;
    }
    
    try {
      const db = useDatabase();
      
      // Extract metadata from event data
      const tags: Record<string, any> = event.data?.tags || {};
      
      // Log received tags for debugging
      console.log('[StoreSCP] onFileStored received tags:', Object.keys(tags));
      console.log('[StoreSCP] Sample tags:', {
        StudyInstanceUID: tags.StudyInstanceUID,
        SeriesInstanceUID: tags.SeriesInstanceUID,
        SOPInstanceUID: tags.SOPInstanceUID,
        PatientName: tags.PatientName
      });
      
      // Validate required tags
      if (!tags.StudyInstanceUID || !tags.SeriesInstanceUID || !tags.SOPInstanceUID) {
        console.error('[StoreSCP] Missing required UIDs, skipping database insert');
        console.error('[StoreSCP] Available keys:', Object.keys(tags));
        return;
      }
      
      // Insert or update study
      await db.sql`
        INSERT OR REPLACE INTO studies (
          study_instance_uid, patient_name, patient_id, patient_birth_date, patient_sex,
          study_date, study_time, study_description, accession_number, modalities_in_study
        ) VALUES (
          ${tags.StudyInstanceUID || null},
          ${tags.PatientName || null},
          ${tags.PatientID || null},
          ${tags.PatientBirthDate || null},
          ${tags.PatientSex || null},
          ${tags.StudyDate || null},
          ${tags.StudyTime || null},
          ${tags.StudyDescription || null},
          ${tags.AccessionNumber || null},
          ${tags.Modality || null}
        )
      `;
      
      // Insert or update series
      await db.sql`
        INSERT OR REPLACE INTO series (
          series_instance_uid, study_instance_uid, modality,
          series_number, series_description
        ) VALUES (
          ${tags.SeriesInstanceUID || null},
          ${tags.StudyInstanceUID || null},
          ${tags.Modality || null},
          ${tags.SeriesNumber || null},
          ${tags.SeriesDescription || null}
        )
      `;
      
      // Insert instance
      await db.sql`
        INSERT OR REPLACE INTO instances (
          sop_instance_uid, series_instance_uid, study_instance_uid,
          sop_class_uid, instance_number, file_path,
          rows, columns, bits_allocated
        ) VALUES (
          ${tags.SOPInstanceUID || null},
          ${tags.SeriesInstanceUID || null},
          ${tags.StudyInstanceUID || null},
          ${tags.SOPClassUID || null},
          ${tags.InstanceNumber || null},
          ${event.data?.file || null},
          ${tags.Rows || null},
          ${tags.Columns || null},
          ${tags.BitsAllocated || null}
        )
      `;
      
      // Update counts
      await db.sql`
        UPDATE studies SET
          number_of_series = (SELECT COUNT(DISTINCT series_instance_uid) FROM series WHERE study_instance_uid = ${tags.StudyInstanceUID}),
          number_of_instances = (SELECT COUNT(*) FROM instances WHERE study_instance_uid = ${tags.StudyInstanceUID}),
          updated_at = CURRENT_TIMESTAMP
        WHERE study_instance_uid = ${tags.StudyInstanceUID}
      `;
      
      await db.sql`
        UPDATE series SET
          number_of_instances = (SELECT COUNT(*) FROM instances WHERE series_instance_uid = ${tags.SeriesInstanceUID})
        WHERE series_instance_uid = ${tags.SeriesInstanceUID}
      `;
      
      console.log(`[StoreSCP] ✓ Stored: ${tags.PatientName} - ${tags.StudyDescription}`);
    } catch (error: any) {
      console.error('[StoreSCP] Database error:', error.message);
    }
  });
  
  // Start server
  storeScp.start();
  console.log(`[StoreSCP] ✓ Listening on port ${STORESCP_PORT} (AET: ${STORESCP_AET})`);
  console.log(`[StoreSCP] ✓ Storage path: ${DICOM_STORAGE_PATH}`);
  
  // Graceful shutdown
  nitroApp.hooks.hook('close', () => {
    console.log('[StoreSCP] Stopping C-STORE receiver...');
    storeScp.stop();
  });
});
