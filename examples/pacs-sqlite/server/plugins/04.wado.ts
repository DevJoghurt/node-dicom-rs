/**
 * WADO-RS Plugin - Retrieval service for DICOM files
 * 
 * Provides RESTful endpoints for retrieving DICOM instances and metadata
 * Files served from filesystem
 */

import { WadoServer, WadoStorageType } from '@nuxthealth/node-dicom';
import { join } from 'path';
import { defineNitroPlugin, useDatabase } from "#imports";

const WADO_PORT: number = 8043;
const DICOM_STORAGE_PATH: string = join(process.cwd(), '.data', 'dicom');

export default defineNitroPlugin(async (nitroApp) => {
  console.log('[WADO-RS] Starting retrieval service...');
  
  const wado = new WadoServer(WADO_PORT, {
    storageType: WadoStorageType.Filesystem,
    basePath: DICOM_STORAGE_PATH,
    enableCors: true,
    corsAllowedOrigins: 'http://localhost:3000',
    enableMetadata: true,
    enableRendered: true,
    verbose: true
  });
  
  // Start server
  wado.start();
  console.log(`[WADO-RS] ✓ Listening on port ${WADO_PORT}`);
  console.log(`[WADO-RS] ✓ Storage path: ${DICOM_STORAGE_PATH}`);
  console.log(`[WADO-RS] ✓ CORS enabled for: http://localhost:3000`);
  
  // Graceful shutdown
  nitroApp.hooks.hook('close', () => {
    console.log('[WADO-RS] Stopping retrieval service...');
    wado.stop();
  });
});
