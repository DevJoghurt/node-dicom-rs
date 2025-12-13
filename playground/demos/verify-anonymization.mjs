/**
 * Verify that anonymization was applied to stored files
 */

import { DicomFile } from '../../index.js';
import { readdirSync } from 'fs';
import { join } from 'path';

// Find a file in the anonymized directory
const baseDir = '/home/jhof/Dokumente/code/node-dicom-rs/playground/test-received-anon';
const studyDirs = readdirSync(baseDir);
const seriesDirs = readdirSync(join(baseDir, studyDirs[0]));
const files = readdirSync(join(baseDir, studyDirs[0], seriesDirs[0])).filter(f => f.endsWith('.dcm'));
const testFile = join(baseDir, studyDirs[0], seriesDirs[0], files[0]);

console.log('Reading anonymized file:', testFile);
console.log();

const file = new DicomFile();
await file.open(testFile);

const tags = file.extract([
    'PatientName',
    'PatientID',
    'PatientBirthDate',
    'PatientSex',
    'StudyDescription',
    'SOPInstanceUID'
]);

console.log('Tags in stored file:');
console.log('='.repeat(60));
console.log('PatientName:', tags.PatientName);
console.log('PatientID:', tags.PatientID);
console.log('PatientBirthDate:', tags.PatientBirthDate);
console.log('PatientSex:', tags.PatientSex);
console.log('StudyDescription:', tags.StudyDescription);
console.log('SOPInstanceUID:', tags.SOPInstanceUID);
console.log('='.repeat(60));

if (tags.PatientName === 'ANONYMOUS^PATIENT') {
    console.log('\n✅ SUCCESS! File was anonymized!');
} else {
    console.log('\n❌ File was NOT anonymized');
}

file.close();
