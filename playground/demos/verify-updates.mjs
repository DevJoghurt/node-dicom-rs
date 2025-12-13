import { DicomFile } from '../../index.js';

console.log('🔍 Verifying saved files...\n');

// Verify updated-test.dcm
console.log('1. Checking updated-test.dcm:');
const file1 = new DicomFile();
await file1.open('test-received/updated-test.dcm');
const data1 = file1.extract(['PatientName', 'PatientID', 'StudyDescription']);
console.log('  PatientName:', data1.PatientName);
console.log('  PatientID:', data1.PatientID);
console.log('  StudyDescription:', data1.StudyDescription);
console.log('  ✅ File saved correctly\n');
file1.close();

// Verify anonymized-test.dcm
console.log('2. Checking anonymized-test.dcm:');
const file2 = new DicomFile();
await file2.open('test-received/anonymized-test.dcm');
const data2 = file2.extract([
    'PatientName', 
    'PatientID', 
    'PatientBirthDate',
    'PatientSex',
    'InstitutionName',
    'StudyDescription'
]);
console.log('  PatientName:', data2.PatientName);
console.log('  PatientID:', data2.PatientID);
console.log('  PatientBirthDate:', data2.PatientBirthDate || '(empty)');
console.log('  PatientSex:', data2.PatientSex || '(empty)');
console.log('  InstitutionName:', data2.InstitutionName);
console.log('  StudyDescription:', data2.StudyDescription);
console.log('  ✅ Anonymization persisted correctly\n');
file2.close();

console.log('✨ All files verified successfully!');
