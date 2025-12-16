#!/usr/bin/env node
import { DicomFile, getCommonTagSets, combineTags, createCustomTag, getAvailableTagNames } from '../../index.js';

console.log('\n=== Testing Consolidated Tag Functions ===\n');

// Test 1: getCommonTagSets
const tagSets = getCommonTagSets();
console.log('1. Tag Sets Available:', Object.keys(tagSets).join(', '));
console.log(`   - patientBasic: ${tagSets.patientBasic.length} tags`);
console.log(`   - default: ${tagSets.default.length} tags`);

// Test 2: combineTags
const combined = combineTags([
    tagSets.patientBasic,
    tagSets.studyBasic,
    tagSets.ct
]);
console.log(`\n2. Combined Tags: ${combined.length} tags (patient + study + CT)`);

// Test 3: getAvailableTagNames
const available = getAvailableTagNames();
console.log(`\n3. Available Tag Names: ${available.length} total tags`);
console.log(`   - Includes: ${available.slice(0, 5).join(', ')}...`);

// Test 4: createCustomTag
const customTag = createCustomTag('00091001', 'VendorPrivateTag');
console.log(`\n4. Custom Tag Created: ${customTag.tag} -> ${customTag.name}`);

// Test 5: Extract with combined tags
const file = new DicomFile();
await file.open('__test__/fixtures/test.dcm');

const data = file.extract(tagSets.patientBasic);
console.log(`\n5. Extracted Patient Data: ${Object.keys(data).length} fields`);
console.log(`   - Fields: ${Object.keys(data).join(', ')}`);

file.close();

console.log('\n=== All Tag Functions Working! ===\n');
