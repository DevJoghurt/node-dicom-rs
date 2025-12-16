/**
 * QIDO-RS Plugin - Query service for DICOM metadata
 * 
 * Provides RESTful endpoints for searching studies, series, and instances
 * Data served from SQLite database
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
} from '@nuxthealth/node-dicom';
import { definePlugin } from "nitro";
import { useDatabase } from "nitro/database";

const QIDO_PORT: number = 8042;

export default definePlugin(async (nitroApp) => {
  console.log('[QIDO-RS] Starting query service...');
  
  const qido = new QidoServer(QIDO_PORT, {
    enableCors: true,
    corsAllowedOrigins: 'http://localhost:3000',
    verbose: true
  });
  
  // Get database connection once
  const db = useDatabase();
  
  // Search for Studies
  qido.onSearchForStudies(async (err, query) => {
    if (err) {
      console.error('Error:', err);
      return JSON.stringify([]);
    }
    try {
      
      const limit = query.limit || 25;
      const offset = query.offset || 0;
      
      let studies: any;
      
      // Build query based on filters - Nitro's db.sql doesn't support dynamic WHERE
      if (query.studyInstanceUid) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE study_instance_uid = ${query.studyInstanceUid}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.patientId && query.studyDate) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE patient_id LIKE ${'%' + query.patientId + '%'}
          AND study_date = ${query.studyDate}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.patientId) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE patient_id LIKE ${'%' + query.patientId + '%'}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.patientName) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE patient_name LIKE ${'%' + query.patientName + '%'}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.studyDate) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE study_date = ${query.studyDate}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.accessionNumber) {
        studies = await db.sql`
          SELECT * FROM studies
          WHERE accession_number = ${query.accessionNumber}
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else {
        studies = await db.sql`
          SELECT * FROM studies
          ORDER BY study_date DESC, study_time DESC
          LIMIT ${limit} OFFSET ${offset}
        `;
      }
      
      // Nitro's database returns {rows: [...], success: true} structure
      const studiesArray = Array.isArray(studies)
        ? studies
        : (studies && 'rows' in studies && Array.isArray(studies.rows))
        ? studies.rows
        : (studies ? [studies] : []);
      
      console.log(`[QIDO-RS] Raw query result:`, studiesArray.length, 'rows');
      if (studiesArray.length > 0) {
        console.log(`[QIDO-RS] First row sample:`, JSON.stringify(studiesArray[0]).substring(0, 200));
      }
      
      // Filter out any null/undefined results and ensure study_instance_uid exists
      const validStudies = studiesArray.filter((study: any) => 
        study && study.study_instance_uid
      );
      
      if (validStudies.length === 0) {
        console.log(`[QIDO-RS] ✓ No studies found`);
        return createQidoEmptyResponse();
      }
      
      // Build QIDO response
      const results: QidoStudyResult[] = validStudies.map((study: any) => {
        const result = new QidoStudyResult();
        
        // Ensure all values are strings or empty strings
        const getString = (val: any): string => {
          if (val === null || val === undefined) return '';
          if (typeof val === 'object') return '';
          return String(val);
        };
        
        result.patientName(getString(study.patient_name));
        result.patientId(getString(study.patient_id));
        result.patientBirthDate(getString(study.patient_birth_date));
        result.patientSex(getString(study.patient_sex));
        result.studyInstanceUid(getString(study.study_instance_uid));
        result.studyDate(getString(study.study_date));
        result.studyTime(getString(study.study_time));
        result.studyDescription(getString(study.study_description));
        result.accessionNumber(getString(study.accession_number));
        result.modalitiesInStudy(getString(study.modalities_in_study));
        result.numberOfStudyRelatedSeries(getString(study.number_of_series || 0));
        result.numberOfStudyRelatedInstances(getString(study.number_of_instances || 0));
        return result;
      });
      
      console.log(`[QIDO-RS] ✓ Found ${results.length} studies`);
      const response = createQidoStudiesResponse(results);
      console.log(`[QIDO-RS] Response type: ${typeof response}, length: ${response?.length || 0}`);
      return response;
    } catch (error) {
      console.error('[QIDO-RS] Database error:', error);
      return createQidoEmptyResponse();
    }
  });
  
  // Search for Series
  qido.onSearchForSeries(async (err, query) => {
    if (err) {
      console.error('Error:', err);
      return JSON.stringify([]);
    }
    try {
      const db = useDatabase();
      
      const limit = query.limit || 100;
      const offset = query.offset || 0;
      
      let seriesData: any;
      
      if (query.modality) {
        seriesData = await db.sql`
          SELECT * FROM series
          WHERE study_instance_uid = ${query.studyInstanceUid}
          AND modality = ${query.modality}
          ORDER BY series_number
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else if (query.seriesNumber) {
        seriesData = await db.sql`
          SELECT * FROM series
          WHERE study_instance_uid = ${query.studyInstanceUid}
          AND series_number = ${query.seriesNumber}
          ORDER BY series_number
          LIMIT ${limit} OFFSET ${offset}
        `;
      } else {
        seriesData = await db.sql`
          SELECT * FROM series
          WHERE study_instance_uid = ${query.studyInstanceUid}
          ORDER BY series_number
          LIMIT ${limit} OFFSET ${offset}
        `;
      }
      
      const seriesArray = Array.isArray(seriesData)
        ? seriesData
        : (seriesData && 'rows' in seriesData && Array.isArray(seriesData.rows))
        ? seriesData.rows
        : (seriesData ? [seriesData] : []);
      
      const results: QidoSeriesResult[] = seriesArray.map((s: any) => {
        const result = new QidoSeriesResult();
        
        const getString = (val: any): string => {
          if (val === null || val === undefined) return '';
          if (typeof val === 'object') return '';
          return String(val);
        };
        
        result.seriesInstanceUid(getString(s.series_instance_uid));
        result.modality(getString(s.modality));
        result.seriesNumber(getString(s.series_number));
        result.seriesDescription(getString(s.series_description));
        result.numberOfSeriesRelatedInstances(getString(s.number_of_instances || 0));
        return result;
      });
      
      console.log(`[QIDO-RS] ✓ Found ${results.length} series`);
      return createQidoSeriesResponse(results);
    } catch (error) {
      console.error('[QIDO-RS] Database error:', error);
      return createQidoEmptyResponse();
    }
  });
  
  // Search for Study Instances
  qido.onSearchForStudyInstances(async (err, query) => {
    if (err) {
      console.error('Error:', err);
      return JSON.stringify([]);
    }
    try {
      const db = useDatabase();
      
      const limit = query.limit || 1000;
      const offset = query.offset || 0;
      
      const instancesData = await db.sql`
        SELECT * FROM instances
        WHERE study_instance_uid = ${query.studyInstanceUid}
        ORDER BY instance_number
        LIMIT ${limit} OFFSET ${offset}
      `;
      
      const instancesArray = Array.isArray(instancesData)
        ? instancesData
        : (instancesData && 'rows' in instancesData && Array.isArray(instancesData.rows))
        ? instancesData.rows
        : (instancesData ? [instancesData] : []);
      
      const results: QidoInstanceResult[] = instancesArray.map((i: any) => {
        const result = new QidoInstanceResult();
        
        const getString = (val: any): string => {
          if (val === null || val === undefined) return '';
          if (typeof val === 'object') return '';
          return String(val);
        };
        
        result.sopInstanceUid(getString(i.sop_instance_uid));
        result.sopClassUid(getString(i.sop_class_uid));
        result.instanceNumber(getString(i.instance_number));
        if (i.rows) result.rows(getString(i.rows));
        if (i.columns) result.columns(getString(i.columns));
        if (i.bits_allocated) result.bitsAllocated(getString(i.bits_allocated));
        return result;
      });
      
      console.log(`[QIDO-RS] ✓ Found ${results.length} instances`);
      return createQidoInstancesResponse(results);
    } catch (error) {
      console.error('[QIDO-RS] Database error:', error);
      return createQidoEmptyResponse();
    }
  });
  
  // Search for Series Instances
  qido.onSearchForSeriesInstances(async (err, query) => {
    if (err) {
      console.error('Error:', err);
      return JSON.stringify([]);
    }
    try {
      const db = useDatabase();
      
      const limit = query.limit || 1000;
      const offset = query.offset || 0;
      
      const instancesData = await db.sql`
        SELECT * FROM instances
        WHERE study_instance_uid = ${query.studyInstanceUid}
        AND series_instance_uid = ${query.seriesInstanceUid}
        ORDER BY instance_number
        LIMIT ${limit} OFFSET ${offset}
      `;
      
      const instancesArray = Array.isArray(instancesData)
        ? instancesData
        : (instancesData && 'rows' in instancesData && Array.isArray(instancesData.rows))
        ? instancesData.rows
        : (instancesData ? [instancesData] : []);
      
      const results = instancesArray.map((i: any) => {
        const result = new QidoInstanceResult();
        
        const getString = (val: any): string => {
          if (val === null || val === undefined) return '';
          if (typeof val === 'object') return '';
          return String(val);
        };
        
        result.sopInstanceUid(getString(i.sop_instance_uid));
        result.sopClassUid(getString(i.sop_class_uid));
        result.instanceNumber(getString(i.instance_number));
        if (i.rows) result.rows(getString(i.rows));
        if (i.columns) result.columns(getString(i.columns));
        if (i.bits_allocated) result.bitsAllocated(getString(i.bits_allocated));
        return result;
      });
      
      console.log(`[QIDO-RS] ✓ Found ${results.length} instances`);
      return createQidoInstancesResponse(results);
    } catch (error) {
      console.error('[QIDO-RS] Database error:', error);
      return createQidoEmptyResponse();
    }
  });
  
  // Start server
  qido.start();
  console.log(`[QIDO-RS] ✓ Listening on port ${QIDO_PORT}`);
  console.log(`[QIDO-RS] ✓ CORS enabled for: http://localhost:3000`);
  
  // Graceful shutdown
  nitroApp.hooks.hook('close', () => {
    console.log('[QIDO-RS] Stopping query service...');
    qido.stop();
  });
});
