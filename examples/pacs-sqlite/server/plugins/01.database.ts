/**
 * Database Plugin - Initialize SQLite database
 * 
 * Creates PACS database schema for storing DICOM metadata
 * Uses Nitro's useDatabase utility
 */
import { defineNitroPlugin, useDatabase } from "#imports";

export default defineNitroPlugin(async (nitroApp) => {
  console.log('[Database] Initializing SQLite PACS database...');
  
  // Initialize database using Nitro's useDatabase
  const db = useDatabase();
  
  // Enable WAL mode for better concurrency
  await db.exec('PRAGMA journal_mode = WAL');
  
  // Create studies table
  await db.exec(`
    CREATE TABLE IF NOT EXISTS studies (
      study_instance_uid TEXT PRIMARY KEY,
      patient_name TEXT,
      patient_id TEXT,
      patient_birth_date TEXT,
      patient_sex TEXT,
      study_date TEXT,
      study_time TEXT,
      study_description TEXT,
      accession_number TEXT,
      modalities_in_study TEXT,
      number_of_series INTEGER DEFAULT 0,
      number_of_instances INTEGER DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `);
  
  // Create series table
  await db.exec(`
    CREATE TABLE IF NOT EXISTS series (
      series_instance_uid TEXT PRIMARY KEY,
      study_instance_uid TEXT NOT NULL,
      modality TEXT,
      series_number TEXT,
      series_description TEXT,
      number_of_instances INTEGER DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (study_instance_uid) REFERENCES studies(study_instance_uid)
    )
  `);
  
  // Create instances table
  await db.exec(`
    CREATE TABLE IF NOT EXISTS instances (
      sop_instance_uid TEXT PRIMARY KEY,
      series_instance_uid TEXT NOT NULL,
      study_instance_uid TEXT NOT NULL,
      sop_class_uid TEXT,
      instance_number TEXT,
      file_path TEXT NOT NULL,
      rows INTEGER,
      columns INTEGER,
      bits_allocated INTEGER,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (series_instance_uid) REFERENCES series(series_instance_uid),
      FOREIGN KEY (study_instance_uid) REFERENCES studies(study_instance_uid)
    )
  `);
  
  // Create patient mapping table for anonymization
  await db.exec(`
    CREATE TABLE IF NOT EXISTS patient_mapping (
      original_patient_id TEXT PRIMARY KEY,
      original_patient_name TEXT,
      anonymized_patient_id TEXT NOT NULL,
      anonymized_patient_name TEXT NOT NULL,
      anonymized_birth_date TEXT NOT NULL,
      anonymized_sex TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `);
  
  // Create indexes for fast queries
  await db.exec(`
    CREATE INDEX IF NOT EXISTS idx_studies_patient_id ON studies(patient_id);
    CREATE INDEX IF NOT EXISTS idx_studies_study_date ON studies(study_date);
    CREATE INDEX IF NOT EXISTS idx_series_study_uid ON series(study_instance_uid);
    CREATE INDEX IF NOT EXISTS idx_instances_series_uid ON instances(series_instance_uid);
    CREATE INDEX IF NOT EXISTS idx_instances_study_uid ON instances(study_instance_uid);
  `);
  
  console.log('[Database] ✓ Database initialized');
  console.log('[Database] ✓ Tables: studies, series, instances');
});
