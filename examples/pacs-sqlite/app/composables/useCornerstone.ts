/**
 * Composable for initializing Cornerstone3D
 * Based on: https://medium.com/@k_raky/displaying-dicom-images-on-the-web-multiplanar-reconstruction-view-with-cornerstone3d-orthanc-and-react-f5a57fb1b363
 */

import { init as coreInit, volumeLoader, cornerstoneStreamingImageVolumeLoader } from '@cornerstonejs/core'
import { init as toolsInit } from '@cornerstonejs/tools'
import cornerstoneDICOMImageLoader from '@cornerstonejs/dicom-image-loader'
import 'dicom-parser'

export const useCornerstone = () => {
  const initialized = ref(false)
  
  const init = async () => {
    if (process.server || initialized.value) return
    
    await coreInit()
    await toolsInit()
    
    // Initialize DICOM Image Loader - wrap in try-catch to handle codec issues
    try {
      await cornerstoneDICOMImageLoader.init()
    } catch (error) {
      console.warn('DICOM Image Loader initialization warning:', error)
      // Continue anyway - uncompressed images should still work
    }
    
    await volumeLoader.registerVolumeLoader(
      'cornerstoneStreamingImageVolume',
      cornerstoneStreamingImageVolumeLoader
    )
    
    initialized.value = true
  }
  
  return {
    initialized: readonly(initialized),
    init,
  }
}
