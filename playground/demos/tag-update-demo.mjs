import { DicomFile } from '../../index.js';
import crypto from 'crypto';

console.log('🏷️  DICOM TAG UPDATE DEMO\n');

const inputFile = '__test__/fixtures/test.dcm';
const outputFile = 'test-received/updated-test.dcm';

// Demo 1: Basic tag update
console.log('1️⃣  Basic Tag Update');
console.log('=' .repeat(50));

const file1 = new DicomFile();
await file1.open(inputFile);

// Extract original values
const original = file1.extract(['PatientName', 'PatientID', 'StudyDescription']);
console.log('Original values:');
console.log('  PatientName:', original.PatientName || '(empty)');
console.log('  PatientID:', original.PatientID || '(empty)');
console.log('  StudyDescription:', original.StudyDescription || '(empty)');

// Update tags
const result1 = file1.updateTags({
    PatientName: 'DOE^JANE',
    PatientID: 'PAT12345',
    StudyDescription: 'Updated Study Description'
});
console.log('\n' + result1);

// Verify updates
const updated = file1.extract(['PatientName', 'PatientID', 'StudyDescription']);
console.log('\nUpdated values:');
console.log('  PatientName:', updated.PatientName);
console.log('  PatientID:', updated.PatientID);
console.log('  StudyDescription:', updated.StudyDescription);

// Save to file
await file1.saveAsDicom(outputFile);
console.log(`\n✅ Saved to: ${outputFile}`);
file1.close();

// Demo 2: Anonymization
console.log('\n\n2️⃣  Anonymization Example');
console.log('=' .repeat(50));

const file2 = new DicomFile();
await file2.open(inputFile);

// Extract PHI before anonymization
const beforeAnon = file2.extract([
    'PatientName', 
    'PatientID', 
    'PatientBirthDate',
    'PatientSex',
    'InstitutionName',
    'StudyDescription'
]);

console.log('Before anonymization:');
Object.entries(beforeAnon).forEach(([key, value]) => {
    console.log(`  ${key}: ${value || '(empty)'}`);
});

// Anonymize
file2.updateTags({
    PatientName: 'ANONYMOUS',
    PatientID: crypto.randomUUID(),
    PatientBirthDate: '',
    PatientSex: '',
    PatientAge: '',
    InstitutionName: 'ANONYMIZED',
    StudyDescription: 'ANONYMIZED STUDY'
});

// Extract after anonymization
const afterAnon = file2.extract([
    'PatientName', 
    'PatientID', 
    'PatientBirthDate',
    'PatientSex',
    'InstitutionName',
    'StudyDescription'
]);

console.log('\nAfter anonymization:');
Object.entries(afterAnon).forEach(([key, value]) => {
    console.log(`  ${key}: ${value || '(empty)'}`);
});

await file2.saveAsDicom('test-received/anonymized-test.dcm');
console.log('\n✅ Saved anonymized file to: test-received/anonymized-test.dcm');
file2.close();

// Demo 3: Using hex tag format
console.log('\n\n3️⃣  Hex Tag Format');
console.log('=' .repeat(50));

const file3 = new DicomFile();
await file3.open(inputFile);

console.log('Updating tags using hex format...');
file3.updateTags({
    '00100010': 'SMITH^JOHN',      // PatientName
    '00100020': 'HEX123456',        // PatientID
    '00080020': '20240101',         // StudyDate
    '00080030': '120000'            // StudyTime
});

const hexResult = file3.extract(['PatientName', 'PatientID', 'StudyDate', 'StudyTime']);
console.log('\nUpdated via hex format:');
Object.entries(hexResult).forEach(([key, value]) => {
    console.log(`  ${key}: ${value}`);
});

file3.close();

// Demo 4: Clear tag values
console.log('\n\n4️⃣  Clear Tag Values');
console.log('=' .repeat(50));

const file4 = new DicomFile();
await file4.open(outputFile); // Use the file we saved earlier

const beforeClear = file4.extract(['PatientName', 'PatientID']);
console.log('Before clearing:');
console.log('  PatientName:', beforeClear.PatientName);
console.log('  PatientID:', beforeClear.PatientID);

// Clear values by setting to empty string
file4.updateTags({
    PatientName: '',
    PatientID: ''
});

const afterClear = file4.extract(['PatientName', 'PatientID']);
console.log('\nAfter clearing:');
console.log('  PatientName:', afterClear.PatientName || '(empty)');
console.log('  PatientID:', afterClear.PatientID || '(empty)');

file4.close();

// Demo 5: Error handling
console.log('\n\n5️⃣  Error Handling');
console.log('=' .repeat(50));

const file5 = new DicomFile();
await file5.open(inputFile);

// Try to update file meta information (should fail)
try {
    file5.updateTags({
        TransferSyntaxUID: '1.2.840.10008.1.2.1'
    });
    console.log('❌ Should have thrown error for meta information tag');
} catch (err) {
    console.log('✅ Correctly prevented meta info update:', err.message);
}

// Try to update pixel data (should fail)
try {
    file5.updateTags({
        PixelData: 'invalid'
    });
    console.log('❌ Should have thrown error for pixel data tag');
} catch (err) {
    console.log('✅ Correctly prevented pixel data update:', err.message);
}

// Try unknown tag (should fail)
try {
    file5.updateTags({
        InvalidTagNameXYZ: 'value'
    });
    console.log('❌ Should have thrown error for unknown tag');
} catch (err) {
    console.log('✅ Correctly handled unknown tag:', err.message);
}

file5.close();

console.log('\n\n✨ All demos completed successfully!\n');
