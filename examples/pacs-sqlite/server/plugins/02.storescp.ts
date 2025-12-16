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
import { definePlugin } from "nitro";
import { useDatabase } from "nitro/database";

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

export default definePlugin(async (nitroApp) => {
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
  
  // Create StoreSCP instance with correct options format
  const storeScp = new StoreScp({
    port: STORESCP_PORT,
    callingAeTitle: STORESCP_AET,
    outDir: DICOM_STORAGE_PATH,
    verbose: true,
    extractTags: extractionTags,
    storeWithFileMeta: true,
    studyTimeout: 30
  });
  
  // Anonymize before storing
  storeScp.onBeforeStore((tags: Record<string, string>) => {
    try {
      // Extract original patient ID
      const originalPatientID: string = tags.PatientID || 'UNKNOWN';
      
      // Generate consistent fake data
      const fakeData: FakePatientData = generateFakePatientData(originalPatientID);
      
      console.log(`[StoreSCP] ✓ Anonymized: ${originalPatientID} → ${fakeData.patientID}`);
      
      // Return modified tags
      return {
        ...tags,
        PatientName: fakeData.patientName,
        PatientID: fakeData.patientID,
        PatientBirthDate: fakeData.patientBirthDate,
        PatientSex: fakeData.patientSex
      };
    } catch (error: any) {
      console.error('[StoreSCP] Anonymization error:', error.message);
      return tags; // Return original tags on error
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
      
      // Insert or update study
      await db.sql`
        INSERT OR REPLACE INTO studies (
          study_instance_uid, patient_name, patient_id, patient_birth_date, patient_sex,
          study_date, study_time, study_description, accession_number, modalities_in_study
        ) VALUES (
          ${tags.StudyInstanceUID},
          ${tags.PatientName},
          ${tags.PatientID},
          ${tags.PatientBirthDate},
          ${tags.PatientSex},
          ${tags.StudyDate},
          ${tags.StudyTime},
          ${tags.StudyDescription},
          ${tags.AccessionNumber},
          ${tags.Modality}
        )
      `;
      
      // Insert or update series
      await db.sql`
        INSERT OR REPLACE INTO series (
          series_instance_uid, study_instance_uid, modality,
          series_number, series_description
        ) VALUES (
          ${tags.SeriesInstanceUID},
          ${tags.StudyInstanceUID},
          ${tags.Modality},
          ${tags.SeriesNumber},
          ${tags.SeriesDescription}
        )
      `;
      
      // Insert instance
      await db.sql`
        INSERT OR REPLACE INTO instances (
          sop_instance_uid, series_instance_uid, study_instance_uid,
          sop_class_uid, instance_number, file_path,
          rows, columns, bits_allocated
        ) VALUES (
          ${tags.SOPInstanceUID},
          ${tags.SeriesInstanceUID},
          ${tags.StudyInstanceUID},
          ${tags.SOPClassUID},
          ${tags.InstanceNumber},
          ${event.data?.file || ''},
          ${tags.Rows},
          ${tags.Columns},
          ${tags.BitsAllocated}
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
  storeScp.listen();
  console.log(`[StoreSCP] ✓ Listening on port ${STORESCP_PORT} (AET: ${STORESCP_AET})`);
  console.log(`[StoreSCP] ✓ Storage path: ${DICOM_STORAGE_PATH}`);
  
  // Graceful shutdown
  nitroApp.hooks.hook('close', () => {
    console.log('[StoreSCP] Stopping C-STORE receiver...');
    storeScp.close();
  });
});
