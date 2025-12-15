#!/usr/bin/env node

/**
 * Complete DICOMweb Server Demo
 * 
 * Demonstrates the high-level QIDO-RS API with all query types:
 * - Search for Studies
 * - Search for Series
 * - Search for Study Instances
 * - Search for Series Instances
 * 
 * Uses typed builders - no DICOM tags needed!
 */

import { 
  QidoServer,
  QidoStudyResult,
  QidoSeriesResult,
  QidoInstanceResult,
  createQidoStudiesResponse,
  createQidoSeriesResponse,
  createQidoInstancesResponse,
  createQidoEmptyResponse
} from '../../index.js';

// Configuration
const QIDO_PORT = 8080;

// ============================================================================
// Mock Database
// ============================================================================

const mockDatabase = {
  studies: [
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      studyDate: '20240101',
      studyTime: '120000',
      accessionNumber: 'ACC001',
      studyDescription: 'CT Chest with Contrast',
      studyId: 'STU001',
      patientName: 'Doe^John',
      patientId: '12345',
      patientBirthDate: '19800115',
      patientSex: 'M',
      referringPhysicianName: 'Smith^Jane',
      modalitiesInStudy: 'CT',
      numberOfStudyRelatedSeries: '2',
      numberOfStudyRelatedInstances: '50'
    },
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.2',
      studyDate: '20240102',
      studyTime: '140000',
      accessionNumber: 'ACC002',
      studyDescription: 'MRI Brain',
      studyId: 'STU002',
      patientName: 'Smith^Mary',
      patientId: '67890',
      patientBirthDate: '19900320',
      patientSex: 'F',
      referringPhysicianName: 'Jones^Robert',
      modalitiesInStudy: 'MR',
      numberOfStudyRelatedSeries: '3',
      numberOfStudyRelatedInstances: '120'
    }
  ],
  
  series: [
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.1',
      modality: 'CT',
      seriesNumber: '1',
      seriesDescription: 'Chest Axial',
      seriesDate: '20240101',
      seriesTime: '120500',
      performingPhysicianName: 'Brown^Alice',
      numberOfSeriesRelatedInstances: '25',
      bodyPartExamined: 'CHEST',
      protocolName: 'Chest Routine'
    },
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.2',
      modality: 'CT',
      seriesNumber: '2',
      seriesDescription: 'Chest Coronal',
      seriesDate: '20240101',
      seriesTime: '121000',
      performingPhysicianName: 'Brown^Alice',
      numberOfSeriesRelatedInstances: '25',
      bodyPartExamined: 'CHEST',
      protocolName: 'Chest Routine'
    },
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.2',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.2.1',
      modality: 'MR',
      seriesNumber: '1',
      seriesDescription: 'T1 Axial',
      seriesDate: '20240102',
      seriesTime: '140500',
      performingPhysicianName: 'Davis^Carol',
      numberOfSeriesRelatedInstances: '40',
      bodyPartExamined: 'BRAIN',
      protocolName: 'Brain Standard'
    }
  ],
  
  instances: [
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.1',
      sopInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.1.1',
      sopClassUid: '1.2.840.10008.5.1.4.1.1.2', // CT Image Storage
      instanceNumber: '1',
      rows: '512',
      columns: '512',
      bitsAllocated: '16',
      numberOfFrames: '1'
    },
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.1',
      sopInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.1.2',
      sopClassUid: '1.2.840.10008.5.1.4.1.1.2',
      instanceNumber: '2',
      rows: '512',
      columns: '512',
      bitsAllocated: '16',
      numberOfFrames: '1'
    },
    {
      studyInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1',
      seriesInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.2',
      sopInstanceUid: '1.2.840.113619.2.55.3.604688119.868.1234567890.1.2.1',
      sopClassUid: '1.2.840.10008.5.1.4.1.1.2',
      instanceNumber: '1',
      rows: '512',
      columns: '512',
      bitsAllocated: '16',
      numberOfFrames: '1'
    }
  ]
};

// ============================================================================
// Helper Functions - Using High-Level Builders
// ============================================================================

/**
 * Create Study result using typed builder
 */
function createStudyResult(study) {
  const result = new QidoStudyResult();
  result.patientName(study.patientName);
  result.patientId(study.patientId);
  result.patientBirthDate(study.patientBirthDate);
  result.patientSex(study.patientSex);
  result.studyInstanceUid(study.studyInstanceUid);
  result.studyDate(study.studyDate);
  result.studyTime(study.studyTime);
  result.accessionNumber(study.accessionNumber);
  result.studyDescription(study.studyDescription);
  result.studyId(study.studyId);
  result.referringPhysicianName(study.referringPhysicianName);
  result.modalitiesInStudy(study.modalitiesInStudy);
  result.numberOfStudyRelatedSeries(study.numberOfStudyRelatedSeries);
  result.numberOfStudyRelatedInstances(study.numberOfStudyRelatedInstances);
  return result;
}

/**
 * Create Series result using typed builder
 */
function createSeriesResult(series) {
  const result = new QidoSeriesResult();
  result.seriesInstanceUid(series.seriesInstanceUid);
  result.modality(series.modality);
  result.seriesNumber(series.seriesNumber);
  result.seriesDescription(series.seriesDescription);
  result.seriesDate(series.seriesDate);
  result.seriesTime(series.seriesTime);
  result.performingPhysicianName(series.performingPhysicianName);
  result.numberOfSeriesRelatedInstances(series.numberOfSeriesRelatedInstances);
  result.bodyPartExamined(series.bodyPartExamined);
  result.protocolName(series.protocolName);
  return result;
}

/**
 * Create Instance result using typed builder
 */
function createInstanceResult(instance) {
  const result = new QidoInstanceResult();
  result.sopInstanceUid(instance.sopInstanceUid);
  result.sopClassUid(instance.sopClassUid);
  result.instanceNumber(instance.instanceNumber);
  result.rows(instance.rows);
  result.columns(instance.columns);
  result.bitsAllocated(instance.bitsAllocated);
  result.numberOfFrames(instance.numberOfFrames);
  return result;
}

// ============================================================================
// QIDO-RS Server Setup
// ============================================================================

async function main() {
  console.log('Starting DICOMweb QIDO-RS server...\n');

  const qidoServer = new QidoServer(QIDO_PORT);

  // --------------------------------------------------------------------------
  // Handler 1: Search for Studies (GET /studies)
  // --------------------------------------------------------------------------
  qidoServer.onSearchForStudies((err, query) => {
    if (err) {
      console.error('✗ Error in Search for Studies:', err);
      return createQidoEmptyResponse();
    }
    
    console.log('📥 Search for Studies');
    console.log('  Query:', JSON.stringify(query, null, 2));
    
    // Filter studies based on query parameters
    let results = mockDatabase.studies;
    
    if (query.patientId) {
      results = results.filter(s => s.patientId === query.patientId);
    }
    
    if (query.studyDate) {
      results = results.filter(s => s.studyDate === query.studyDate);
    }
    
    if (query.accessionNumber) {
      results = results.filter(s => s.accessionNumber === query.accessionNumber);
    }
    
    if (query.studyInstanceUid) {
      results = results.filter(s => s.studyInstanceUid === query.studyInstanceUid);
    }
    
    // Apply pagination
    if (query.offset) {
      results = results.slice(query.offset);
    }
    
    if (query.limit) {
      results = results.slice(0, query.limit);
    }
    
    console.log(`  ✓ Returning ${results.length} study(ies)\n`);
    
    // Use high-level builders - no DICOM tags!
    const studyResults = results.map(createStudyResult);
    return createQidoStudiesResponse(studyResults);
  });

  // --------------------------------------------------------------------------
  // Handler 2: Search for Series (GET /studies/{uid}/series)
  // --------------------------------------------------------------------------
  qidoServer.onSearchForSeries((err, query) => {
    if (err) {
      console.error('✗ Error in Search for Series:', err);
      return createQidoEmptyResponse();
    }
    
    console.log('📥 Search for Series');
    console.log('  StudyInstanceUID:', query.studyInstanceUid);
    console.log('  Filters:', { modality: query.modality });
    
    // Filter series by study UID
    let results = mockDatabase.series.filter(
      s => s.studyInstanceUid === query.studyInstanceUid
    );
    
    // Apply additional filters
    if (query.modality) {
      results = results.filter(s => s.modality === query.modality);
    }
    
    if (query.seriesInstanceUid) {
      results = results.filter(s => s.seriesInstanceUid === query.seriesInstanceUid);
    }
    
    // Apply pagination
    if (query.offset) {
      results = results.slice(query.offset);
    }
    
    if (query.limit) {
      results = results.slice(0, query.limit);
    }
    
    console.log(`  ✓ Returning ${results.length} series\n`);
    
    const seriesResults = results.map(createSeriesResult);
    return createQidoSeriesResponse(seriesResults);
  });

  // --------------------------------------------------------------------------
  // Handler 3: Search for Study Instances (GET /studies/{uid}/instances)
  // --------------------------------------------------------------------------
  qidoServer.onSearchForStudyInstances((err, query) => {
    if (err) {
      console.error('✗ Error in Search for Study Instances:', err);
      return createQidoEmptyResponse();
    }
    
    console.log('📥 Search for Instances in Study');
    console.log('  StudyInstanceUID:', query.studyInstanceUid);
    
    // Filter instances by study UID
    let results = mockDatabase.instances.filter(
      i => i.studyInstanceUid === query.studyInstanceUid
    );
    
    // Apply additional filters
    if (query.sopInstanceUid) {
      results = results.filter(i => i.sopInstanceUid === query.sopInstanceUid);
    }
    
    if (query.instanceNumber) {
      results = results.filter(i => i.instanceNumber === query.instanceNumber);
    }
    
    // Apply pagination
    if (query.offset) {
      results = results.slice(query.offset);
    }
    
    if (query.limit) {
      results = results.slice(0, query.limit);
    }
    
    console.log(`  ✓ Returning ${results.length} instance(s)\n`);
    
    const instanceResults = results.map(createInstanceResult);
    return createQidoInstancesResponse(instanceResults);
  });

  // --------------------------------------------------------------------------
  // Handler 4: Search for Series Instances (GET /studies/{uid}/series/{uid}/instances)
  // --------------------------------------------------------------------------
  qidoServer.onSearchForSeriesInstances((err, query) => {
    if (err) {
      console.error('✗ Error in Search for Series Instances:', err);
      return createQidoEmptyResponse();
    }
    
    console.log('📥 Search for Instances in Series');
    console.log('  StudyInstanceUID:', query.studyInstanceUid);
    console.log('  SeriesInstanceUID:', query.seriesInstanceUid);
    
    // Filter instances by study and series UID
    let results = mockDatabase.instances.filter(
      i => i.studyInstanceUid === query.studyInstanceUid &&
           i.seriesInstanceUid === query.seriesInstanceUid
    );
    
    // Apply additional filters
    if (query.sopInstanceUid) {
      results = results.filter(i => i.sopInstanceUid === query.sopInstanceUid);
    }
    
    // Apply pagination
    if (query.offset) {
      results = results.slice(query.offset);
    }
    
    if (query.limit) {
      results = results.slice(0, query.limit);
    }
    
    console.log(`  ✓ Returning ${results.length} instance(s)\n`);
    
    const instanceResults = results.map(createInstanceResult);
    return createQidoInstancesResponse(instanceResults);
  });

  // --------------------------------------------------------------------------
  // Start Server
  // --------------------------------------------------------------------------
  
  try {
    qidoServer.start();
    
    console.log('═'.repeat(70));
    console.log('QIDO-RS Server Running - High-Level Typed API');
    console.log('═'.repeat(70));
    console.log(`\n✓ Server listening on http://0.0.0.0:${QIDO_PORT}\n`);
    
    console.log('Available Endpoints:');
    console.log('  GET /studies');
    console.log('  GET /studies/{uid}/series');
    console.log('  GET /studies/{uid}/instances');
    console.log('  GET /studies/{uid}/series/{uid}/instances\n');
    
    console.log('Test Commands:');
    console.log('  # Search all studies:');
    console.log(`  curl http://localhost:${QIDO_PORT}/studies | jq .\n`);
    
    console.log('  # Filter by PatientID:');
    console.log(`  curl "http://localhost:${QIDO_PORT}/studies?PatientID=12345" | jq .\n`);
    
    console.log('  # Get series in study:');
    console.log(`  curl "http://localhost:${QIDO_PORT}/studies/1.2.840.113619.2.55.3.604688119.868.1234567890.1/series" | jq .\n`);
    
    console.log('  # Get instances in series:');
    console.log(`  curl "http://localhost:${QIDO_PORT}/studies/1.2.840.113619.2.55.3.604688119.868.1234567890.1/series/1.2.840.113619.2.55.3.604688119.868.1234567890.1.1/instances" | jq .\n`);
    
    console.log('Press Ctrl+C to stop\n');
    
  } catch (error) {
    console.error('✗ Failed to start QIDO server:', error.message);
    process.exit(1);
  }

  // --------------------------------------------------------------------------
  // Graceful Shutdown
  // --------------------------------------------------------------------------
  
  const cleanup = () => {
    console.log('\n\nShutting down server...');
    
    try {
      qidoServer.stop();
      console.log('✓ QIDO server stopped');
    } catch (error) {
      console.error('✗ Error stopping QIDO server:', error.message);
    }
    
    console.log('\nServer stopped successfully');
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);

  // Keep process alive
  await new Promise(() => {});
}

// Run the example
main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
