/**
 * Composable for fetching DICOM data from WADO-RS server
 * Based on: https://medium.com/@k_raky/displaying-dicom-images-on-the-web-multiplanar-reconstruction-view-with-cornerstone3d-orthanc-and-react-f5a57fb1b363
 */

import { calibratedPixelSpacingMetadataProvider, getPixelSpacingInformation } from '@cornerstonejs/core/utilities'
import cornerstoneDICOMImageLoader from '@cornerstonejs/dicom-image-loader'
import dcmjs from 'dcmjs'

const QIDO_PORT = 8042
const QIDO_BASE_URL = `http://localhost:${QIDO_PORT}`
const WADO_PORT = 8043
const WADO_BASE_URL = `http://localhost:${WADO_PORT}`

export const useDicomWeb = () => {
  // Fetch study metadata from QIDO-RS
  const fetchStudies = async () => {
    try {
      const response = await fetch(`${QIDO_BASE_URL}/studies`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch (error) {
      console.error('Failed to load studies:', error)
      throw error
    }
  }

  // Fetch series for a specific study from QIDO_RS
  const fetchSeries = async (studyUID: string) => {
    try {
      const response = await fetch(`${QIDO_BASE_URL}/studies/${studyUID}/series`)
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
        `${QIDO_BASE_URL}/studies/${studyUID}/series/${seriesUID}/instances`
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

  // Analyze instance numbering for gaps and issues
  const analyzeInstanceSequence = (instances: any[]) => {
    const analysis = {
      totalInstances: instances.length,
      instanceNumbers: [] as number[],
      gaps: [] as { from: number; to: number; missing: number[] }[],
      duplicates: [] as number[],
      invalidNumbers: [] as number[],
      minInstanceNumber: 0,
      maxInstanceNumber: 0,
      expectedCount: 0,
    }

    if (instances.length === 0) return analysis

    // Extract instance numbers
    instances.forEach((inst: any, idx: number) => {
      const num = parseInt(inst['00200013']?.Value?.[0] || '-1')
      if (num === -1) {
        analysis.invalidNumbers.push(idx)
      } else {
        analysis.instanceNumbers.push(num)
      }
    })

    if (analysis.instanceNumbers.length === 0) return analysis

    analysis.minInstanceNumber = Math.min(...analysis.instanceNumbers)
    analysis.maxInstanceNumber = Math.max(...analysis.instanceNumbers)
    analysis.expectedCount = analysis.maxInstanceNumber - analysis.minInstanceNumber + 1

    // Find duplicates
    const counts = new Map<number, number>()
    analysis.instanceNumbers.forEach((num) => {
      counts.set(num, (counts.get(num) || 0) + 1)
    })
    counts.forEach((count, num) => {
      if (count > 1) analysis.duplicates.push(num)
    })

    // Find gaps
    const sorted = [...new Set(analysis.instanceNumbers)].sort((a, b) => a - b)
    for (let i = 0; i < sorted.length - 1; i++) {
      const current = sorted[i]
      const next = sorted[i + 1]
      if (next - current > 1) {
        const missing = []
        for (let m = current + 1; m < next; m++) {
          missing.push(m)
        }
        analysis.gaps.push({
          from: current,
          to: next,
          missing,
        })
      }
    }

    return analysis
  }

  // Convert instances to Cornerstone image IDs (following blog post pattern)
  const retrieveImageIds = async (studyUID: string, seriesUID: string) => {
    // Ask the DICOM server for all instances in the selected study and series
    const instances = await fetchInstances(studyUID, seriesUID)
    console.log('[retrieveImageIds] Fetched instances:', instances.length)

    // Analyze instance sequence BEFORE sorting
    const sequenceAnalysis = analyzeInstanceSequence(instances)
    console.log('[retrieveImageIds] Instance sequence analysis:', sequenceAnalysis)

    // Sort instances by Instance Number (0020,0013) to ensure correct order
    instances.sort((a: any, b: any) => {
      const aInstanceNumber = parseInt(a['00200013']?.Value?.[0] || '0')
      const bInstanceNumber = parseInt(b['00200013']?.Value?.[0] || '0')
      return aInstanceNumber - bInstanceNumber
    })

    // Convert each instance into a Cornerstone compatible image id
    let imageIds = instances.map((instanceMetaData: any, idx: number) => {
      // Read the SOP Instance UID from the metadata
      const SOPInstanceUIDToUse = instanceMetaData['00080018']?.Value?.[0]
      const instanceNumber = parseInt(instanceMetaData['00200013']?.Value?.[0] || '0')

      if (!SOPInstanceUIDToUse) {
        console.warn(`[retrieveImageIds] Missing SOP Instance UID at index ${idx}`)
      }

      // Construct the full image id used by Cornerstone to locate and load the frame
      // Using wadors scheme pointing to our WADO-RS server on port 8043
      const imageId = `wadors:${WADO_BASE_URL}/studies/${studyUID.trim()}/series/${seriesUID.trim()}/instances/${SOPInstanceUIDToUse?.trim()}/frames/1`

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

      console.log(`[retrieveImageIds] Image ${idx + 1}/${instances.length} - Instance#: ${instanceNumber}, SOP: ${SOPInstanceUIDToUse?.substring(0, 20)}...`)

      return imageId
    })

    // Return the list of image ids with analysis
    return {
      imageIds,
      analysis: sequenceAnalysis,
      studyUID,
      seriesUID,
    }
  }

  return {
    fetchStudies,
    fetchSeries,
    fetchInstances,
    retrieveImageIds,
    analyzeInstanceSequence,
  }
}
