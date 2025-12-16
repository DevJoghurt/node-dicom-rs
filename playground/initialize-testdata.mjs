#!/usr/bin/env node

/**
 * Initialize DICOM Test Data
 * 
 * 1. Anonymizes DICOM files with consistent fake personal data
 * 2. Reorganizes files into WADO-RS/StoreSCP compatible structure:
 *    testdata/store/{studyUID}/{seriesUID}/{instanceUID}.dcm
 * 
 * Fake data generation uses patient ID as seed for consistency.
 */

import { DicomFile } from '../index.js';
import { promises as fs } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import crypto from 'crypto';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SOURCE_DIR = path.join(__dirname, 'testdata');
const TARGET_DIR = path.join(__dirname, 'testdata', 'store');

// Fake data generators
const FIRST_NAMES = [
    'James', 'Mary', 'John', 'Patricia', 'Robert', 'Jennifer', 'Michael', 'Linda',
    'William', 'Barbara', 'David', 'Elizabeth', 'Richard', 'Susan', 'Joseph', 'Jessica',
    'Thomas', 'Sarah', 'Charles', 'Karen', 'Christopher', 'Nancy', 'Daniel', 'Lisa'
];

const LAST_NAMES = [
    'Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis',
    'Rodriguez', 'Martinez', 'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson', 'Thomas',
    'Taylor', 'Moore', 'Jackson', 'Martin', 'Lee', 'Perez', 'Thompson', 'White'
];

const CITIES = [
    'New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia',
    'San Antonio', 'San Diego', 'Dallas', 'San Jose', 'Austin', 'Jacksonville',
    'Fort Worth', 'Columbus', 'Charlotte', 'Indianapolis', 'Seattle', 'Denver'
];

// Generate consistent fake data based on patient ID
function generateFakePatientData(patientID) {
    if (!patientID) {
        patientID = 'UNKNOWN_' + Math.random().toString(36).substring(7);
    }
    
    // Use patient ID as seed for consistent data
    const hash = crypto.createHash('sha256').update(patientID).digest();
    const seed = hash.readUInt32BE(0);
    
    // Simple seeded random
    function seededRandom(index) {
        const x = Math.sin(seed + index) * 10000;
        return x - Math.floor(x);
    }
    
    const firstName = FIRST_NAMES[Math.floor(seededRandom(1) * FIRST_NAMES.length)];
    const lastName = LAST_NAMES[Math.floor(seededRandom(2) * LAST_NAMES.length)];
    const city = CITIES[Math.floor(seededRandom(3) * CITIES.length)];
    
    // Generate birth date (between 1940 and 2010)
    const year = 1940 + Math.floor(seededRandom(4) * 70);
    const month = 1 + Math.floor(seededRandom(5) * 12);
    const day = 1 + Math.floor(seededRandom(6) * 28);
    const birthDate = `${year}${month.toString().padStart(2, '0')}${day.toString().padStart(2, '0')}`;
    
    // Generate sex
    const sex = seededRandom(7) > 0.5 ? 'M' : 'F';
    
    // Generate address
    const streetNumber = 100 + Math.floor(seededRandom(8) * 9900);
    const streetNames = ['Main St', 'Oak Ave', 'Maple Dr', 'Cedar Ln', 'Pine Rd', 'Elm St'];
    const streetName = streetNames[Math.floor(seededRandom(9) * streetNames.length)];
    const zipCode = 10000 + Math.floor(seededRandom(10) * 89999);
    
    return {
        patientName: `${lastName}^${firstName}`,
        patientID: patientID,
        patientBirthDate: birthDate,
        patientSex: sex,
        patientAddress: `${streetNumber} ${streetName}, ${city}, ${zipCode}`
    };
}

// Find all DICOM files
async function findDicomFiles(dir, files = []) {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        
        if (entry.isDirectory() && entry.name !== 'store') {
            await findDicomFiles(fullPath, files);
        } else if (entry.isFile() && entry.name.endsWith('.dcm')) {
            files.push(fullPath);
        }
    }
    
    return files;
}

console.log('='.repeat(80));
console.log('DICOM Test Data Initialization');
console.log('='.repeat(80));
console.log();
console.log(`Source: ${SOURCE_DIR}`);
console.log(`Target: ${TARGET_DIR}`);
console.log();

// Create target directory
try {
    await fs.mkdir(TARGET_DIR, { recursive: true });
    console.log(`✓ Created target directory: ${TARGET_DIR}`);
} catch (error) {
    console.error(`✗ Failed to create target directory: ${error.message}`);
    process.exit(1);
}

console.log('\n' + '-'.repeat(80));
console.log('Step 1: Scanning for DICOM files...');
console.log('-'.repeat(80));

const dicomFiles = await findDicomFiles(SOURCE_DIR);
console.log(`Found ${dicomFiles.length} DICOM files`);

let anonymized = 0;
let processed = 0;
let skipped = 0;
let errors = 0;

const patientDataCache = new Map();

console.log('\n' + '-'.repeat(80));
console.log('Step 2: Anonymizing and reorganizing files...');
console.log('-'.repeat(80));

for (const filePath of dicomFiles) {
    try {
        // Get relative path from playground directory
        const relativePath = path.relative(__dirname, filePath);
        
        // Load DICOM file with filesystem backend (playground as root)
        const dicomFile = new DicomFile({
            backend: 'Filesystem',
            rootDir: __dirname
        });
        
        // Open the file (async)
        await dicomFile.open(relativePath);
        
        // Extract UIDs using the extract method
        const tags = dicomFile.extract([
            'StudyInstanceUID',    // 0020,000D
            'SeriesInstanceUID',   // 0020,000E
            'SOPInstanceUID',      // 0008,0018
            'PatientID'            // 0010,0020
        ]);
        
        const studyUID = tags.StudyInstanceUID;
        const seriesUID = tags.SeriesInstanceUID;
        const instanceUID = tags.SOPInstanceUID;
        const patientID = tags.PatientID;
        
        if (!studyUID || !seriesUID || !instanceUID) {
            console.log(`⚠ Skipped (missing UIDs): ${path.basename(filePath)}`);
            skipped++;
            continue;
        }
        
        // Generate or retrieve fake patient data
        let fakeData;
        if (patientDataCache.has(patientID)) {
            fakeData = patientDataCache.get(patientID);
        } else {
            fakeData = generateFakePatientData(patientID);
            patientDataCache.set(patientID, fakeData);
        }
        
        // Update DICOM file with fake data (anonymize original)
        try {
            dicomFile.updateTags({
                'PatientName': fakeData.patientName,
                'PatientID': fakeData.patientID,
                'PatientBirthDate': fakeData.patientBirthDate,
                'PatientSex': fakeData.patientSex,
                'PatientAddress': fakeData.patientAddress
            });
            
            // Save back to original location (anonymize in place)
            await dicomFile.saveAsDicom(relativePath);
            anonymized++;
        } catch (e) {
            // Some tags might not exist, continue anyway
        }
        
        // Create target directory structure
        const targetStudyDir = path.join(TARGET_DIR, studyUID);
        const targetSeriesDir = path.join(targetStudyDir, seriesUID);
        const targetFile = path.join(targetSeriesDir, `${instanceUID}.dcm`);
        
        // Check if file already exists in target
        try {
            await fs.access(targetFile);
            skipped++;
            continue; // File already exists, skip
        } catch {
            // File doesn't exist, proceed
        }
        
        // Create directory structure
        await fs.mkdir(targetSeriesDir, { recursive: true });
        
        // Copy file to organized structure
        await fs.copyFile(filePath, targetFile);
        
        processed++;
        
        if ((anonymized + processed) % 50 === 0) {
            console.log(`Progress: ${anonymized} anonymized, ${processed} organized...`);
        }
        
    } catch (error) {
        console.error(`✗ Error processing ${path.basename(filePath)}: ${error.message}`);
        errors++;
    }
}

console.log('\n' + '='.repeat(80));
console.log('Summary');
console.log('='.repeat(80));
console.log(`Total files found: ${dicomFiles.length}`);
console.log(`✓ Anonymized with fake data: ${anonymized}`);
console.log(`✓ Organized into structure: ${processed}`);
console.log(`⚠ Skipped: ${skipped}`);
console.log(`✗ Errors: ${errors}`);
console.log(`\nUnique patients: ${patientDataCache.size}`);

if (patientDataCache.size > 0) {
    console.log('\nGenerated fake patient data:');
    let count = 0;
    for (const [id, data] of patientDataCache) {
        if (count < 5) {
            console.log(`  ${id}: ${data.patientName} (${data.patientSex}, ${data.patientBirthDate})`);
            count++;
        }
    }
    if (patientDataCache.size > 5) {
        console.log(`  ... and ${patientDataCache.size - 5} more`);
    }
}

console.log();

if (processed > 0) {
    console.log('Storage structure created:');
    console.log(`  ${TARGET_DIR}/`);
    console.log(`    └── {studyUID}/`);
    console.log(`        └── {seriesUID}/`);
    console.log(`            └── {instanceUID}.dcm`);
    console.log();
    console.log('✓ Test data is now anonymized and ready for WADO-RS testing');
    console.log(`  Use basePath: './playground/testdata/store'`);
}
