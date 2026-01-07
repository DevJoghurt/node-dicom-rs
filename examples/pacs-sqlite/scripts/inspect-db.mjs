#!/usr/bin/env node

/**
 * Inspect SQLite database
 * 
 * Usage:
 *   node scripts/inspect-db.mjs
 */

import Database from 'better-sqlite3';
import { join } from 'path';

const dbPath = join(process.cwd(), 'data', 'pacs.db');

console.log('PACS Database Inspector');
console.log('======================');
console.log(`Database: ${dbPath}\n`);

try {
  const db = new Database(dbPath, { readonly: true });
  
  // Count studies
  const studyCount = db.prepare('SELECT COUNT(*) as count FROM studies').get();
  console.log(`Studies: ${studyCount.count}`);
  
  // Count series
  const seriesCount = db.prepare('SELECT COUNT(*) as count FROM series').get();
  console.log(`Series: ${seriesCount.count}`);
  
  // Count instances
  const instanceCount = db.prepare('SELECT COUNT(*) as count FROM instances').get();
  console.log(`Instances: ${instanceCount.count}`);
  
  console.log('');
  
  // List studies
  const studies = db.prepare(`
    SELECT patient_name, patient_id, study_date, study_description,
           number_of_series, number_of_instances
    FROM studies
    ORDER BY study_date DESC
    LIMIT 10
  `).all();
  
  if (studies.length > 0) {
    console.log('Recent Studies:');
    console.log('---------------');
    
    for (const study of studies) {
      console.log(`${study.patient_name} (${study.patient_id})`);
      console.log(`  Date: ${study.study_date}`);
      console.log(`  Description: ${study.study_description || 'N/A'}`);
      console.log(`  Series: ${study.number_of_series}, Instances: ${study.number_of_instances}`);
      console.log('');
    }
  }
  
  db.close();
} catch (error) {
  console.error('Error:', error.message);
  console.error('Make sure data/pacs.db exists (start the server first)');
  process.exit(1);
}
