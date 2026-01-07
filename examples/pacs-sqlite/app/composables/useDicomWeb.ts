/**
 * Composable for fetching DICOM data from WADO-RS server
 * Based on: https://medium.com/@k_raky/displaying-dicom-images-on-the-web-multiplanar-reconstruction-view-with-cornerstone3d-orthanc-and-react-f5a57fb1b363
 */

import { calibratedPixelSpacingMetadataProvider, getPixelSpacingInformation } from '@cornerstonejs/core/utilities'
import cornerstoneDICOMImageLoader from '@cornerstonejs/dicom-image-loader'
import dcmjs from 'dcmjs'

export const useDicomWeb = () => {
  // Fetch study metadata from QIDO-RS
  const fetchStudies = async () => {
    try {
      const response = await fetch('http://localhost:8042/studies')
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch (error) {
      console.error('Failed to load studies:', error)
      throw error
    }
  }

  // Fetch series for a specific study
  const fetchSeries = async (studyUID: string) => {
    try {
      const response = await fetch(`http://localhost:8042/studies/${studyUID}/series`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch (error) {
      console.error('Failed to load series:', error)
      throw error
    }
  }

  // Fetch instances for a specific series
  const fetchInstances = async (studyUID: string, seriesUID: string) => {
    try {
      const response = await fetch(
        `http://localhost:8042/studies/${studyUID}/series/${seriesUID}/instances`
      )
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch (error) {
      console.error('Failed to load instances:', error)
      throw error
    }
  }

  // Clean dataset by removing invalid tags
  const removeInvalidTags = (srcMetadata: any) => {
    const dstMetadata = Object.create(null)
    const tagIds = Object.keys(srcMetadata)
    let tagValue
    
    tagIds.forEach((tagId) => {
      tagValue = srcMetadata[tagId]
      if (tagValue !== undefined && tagValue !== null) {
        dstMetadata[tagId] = tagValue
      }
    })
    
    // Ensure Modality tag exists (required by Cornerstone)
    if (!dstMetadata['00080060']) {
      dstMetadata['00080060'] = { vr: 'CS', Value: ['OT'] } // OT = Other
    }
    
    return dstMetadata
  }

  // Convert instances to Cornerstone image IDs (following blog post pattern)
  const retrieveImageIds = async (studyUID: string, seriesUID: string) => {
    // Ask the DICOM server for all instances in the selected study and series
    const instances = await fetchInstances(studyUID, seriesUID)

    // Sort instances by Instance Number (0020,0013) to ensure correct order
    instances.sort((a: any, b: any) => {
      const aInstanceNumber = parseInt(a['00200013']?.Value?.[0] || '0')
      const bInstanceNumber = parseInt(b['00200013']?.Value?.[0] || '0')
      return aInstanceNumber - bInstanceNumber
    })

    // Convert each instance into a Cornerstone compatible image id
    let imageIds = instances.map((instanceMetaData: any) => {
      // Read the SOP Instance UID from the metadata
      const SOPInstanceUIDToUse = instanceMetaData['00080018'].Value[0]

      // Construct the full image id used by Cornerstone to locate and load the frame
      // Using wadors scheme pointing to our WADO-RS server on port 8043
      const imageId =
        'wadors:' +
        'http://localhost:8043' + // Our WADO-RS server
        '/studies/' +
        studyUID.trim() +
        '/series/' +
        seriesUID.trim() +
        '/instances/' +
        SOPInstanceUIDToUse.trim() +
        '/frames/1'

      // Clean the dataset by removing invalid tags BEFORE registering
      const cleanedMetadata = removeInvalidTags(instanceMetaData)

      // Register the cleaned metadata so Cornerstone can access it
      cornerstoneDICOMImageLoader.wadors.metaDataManager.add(
        imageId,
        cleanedMetadata
      )

      // Turn the dataset into a natural form and attempt to read pixel spacing
      const metadata = dcmjs.data.DicomMetaDictionary.naturalizeDataset(cleanedMetadata)
      const pixelSpacingInformation = getPixelSpacingInformation(metadata)
      const pixelSpacing = pixelSpacingInformation?.PixelSpacing

      // If pixel spacing exists, store it so Cornerstone can display calibrated measurements
      if (pixelSpacing) {
        calibratedPixelSpacingMetadataProvider.add(imageId, {
          rowPixelSpacing: parseFloat(pixelSpacing[0]),
          columnPixelSpacing: parseFloat(pixelSpacing[1]),
          type: pixelSpacingInformation.type,
        })
      }

      return imageId
    })

    // Return the list of image ids ready for use in the viewer
    return imageIds
  }

  return {
    fetchStudies,
    fetchSeries,
    fetchInstances,
    retrieveImageIds,
  }
}
