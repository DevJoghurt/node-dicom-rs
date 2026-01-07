// ESM wrapper for CommonJS native addon
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);

// Import the native addon using CommonJS require
const nativeAddon = require('./index.js');

// Re-export everything
export const {
  DicomFile,
  QidoInstanceResult,
  QidoSeriesResult,
  QidoStudyResult,
  QidoServer,
  StoreScp,
  StoreScu,
  WadoServer,
  WadoStorageType,
  StorageBackendType,
  createQidoEmptyResponse,
  createQidoInstancesResponse,
  createQidoSeriesResponse,
  createQidoStudiesResponse,
  createCustomTag,
  getAvailableTagNames,
  getCommonSopClasses,
  getCommonTagSets,
  getCommonTransferSyntaxes,
  combineTags
} = nativeAddon;

export default nativeAddon;
