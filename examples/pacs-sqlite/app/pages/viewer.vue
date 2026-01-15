<template>
  <div class="viewer-container">
    <!-- Header -->
    <header class="header">
      <h1>🔬 DICOM Viewer</h1>
      <div class="header-actions">
        <NuxtLink to="/" class="btn btn-secondary">← Back</NuxtLink>
        <button @click="loadStudies" class="btn btn-primary" :disabled="loading">
          {{ loading ? '⏳' : '🔄' }} Refresh
        </button>
      </div>
    </header>

    <!-- Main Container -->
    <div class="main-container">
      <!-- Sidebar -->
      <aside class="sidebar">
        <div class="sidebar-header">
          <h2>Studies</h2>
        </div>
        
        <div v-if="error" class="error-box">{{ error }}</div>
        
        <div v-if="!loading && studies.length === 0" class="empty-state">
          No studies available
        </div>
        
        <ul class="study-list" v-if="studies.length > 0">
          <li
            v-for="(study, index) in studies"
            :key="study['0020000D']?.Value?.[0] || index"
            @click="selectStudy(study, index)"
            :class="['study-item', { active: selectedStudyIndex === index }]"
          >
            <div class="study-info">
              <strong>{{ getPatientName(study) }}</strong>
              <div class="study-meta">
                🎂 {{ formatDate(study['00100030']?.Value?.[0]) }}
              </div>
              <div class="study-meta">
                📅 {{ formatDate(study['00080020']?.Value?.[0]) }}
              </div>
              <div class="study-meta">
                📝 {{ study['00081030']?.Value?.[0] || 'No description' }}
              </div>
              <div class="study-meta">
                🖼️ {{ study['00201208']?.Value?.[0] || '0' }} instances
              </div>
            </div>
          </li>
        </ul>
      </aside>

      <!-- Viewer Panel -->
      <main class="viewer-panel">
        <div v-if="!selectedStudy" class="no-study">
          <h2>No Study Selected</h2>
          <p>Select a study from the sidebar to view</p>
        </div>
        
        <div v-else class="viewer-content">
          <!-- Loading State -->
          <div v-if="viewerLoading" class="viewer-loading">
            <div class="spinner"></div>
            <p>{{ loadingMessage }}</p>
          </div>

          <!-- Cornerstone Viewport -->
          <div
            ref="viewportElement"
            class="viewport-container"
          ></div>

          <!-- Info Overlay -->
          <div v-if="currentImageInfo && !viewerLoading" class="info-overlay">
            <div class="info-header">
              <strong>📍 Image {{ currentImageIndex + 1 }} / {{ totalImages }}</strong>
            </div>
            <div><strong>👤 Patient:</strong> {{ currentImageInfo.patientName }}</div>
            <div><strong>📅 Study Date:</strong> {{ currentImageInfo.studyDate }}</div>
            <div><strong>📊 Modality:</strong> {{ currentImageInfo.modality }}</div>
            <div><strong>📋 Series #:</strong> {{ currentImageInfo.seriesNumber }}</div>
            <div><strong>🔢 Instance #:</strong> {{ currentImageInfo.instanceNumber }}</div>
            <div><strong>📏 Size:</strong> {{ currentImageInfo.columns }} × {{ currentImageInfo.rows }}</div>
            <div class="info-uid"><strong>🆔 SOP:</strong> <code>{{ currentImageInfo.sopInstanceUid }}</code></div>
          </div>

          <!-- Debug Panel -->
          <div v-if="debugInfo" class="debug-panel">
            <button @click="showDebugPanel = !showDebugPanel" class="debug-toggle">
              {{ showDebugPanel ? '▼' : '▶' }} Debug Info
            </button>
            
            <div v-if="showDebugPanel" class="debug-content">
              <div class="debug-section">
                <h3>Instance Analysis</h3>
                <div class="debug-row">
                  <span>Total Instances:</span>
                  <strong>{{ debugInfo.analysis.totalInstances }}</strong>
                </div>
                <div class="debug-row">
                  <span>Instance Range:</span>
                  <strong>{{ debugInfo.analysis.minInstanceNumber }} - {{ debugInfo.analysis.maxInstanceNumber }}</strong>
                </div>
                <div class="debug-row">
                  <span>Expected Count:</span>
                  <strong :class="{ 'text-danger': debugInfo.analysis.expectedCount !== debugInfo.analysis.totalInstances }">
                    {{ debugInfo.analysis.expectedCount }}
                  </strong>
                </div>
                
                <div v-if="debugInfo.analysis.invalidNumbers.length > 0" class="debug-alert alert-warning">
                  <strong>⚠️ Invalid Instance Numbers:</strong>
                  <span>{{ debugInfo.analysis.invalidNumbers.length }} instances missing Instance Number tag</span>
                </div>
                
                <div v-if="debugInfo.analysis.gaps.length > 0" class="debug-alert alert-warning">
                  <strong>⚠️ Gaps Detected:</strong>
                  <div v-for="(gap, idx) in debugInfo.analysis.gaps" :key="idx" style="margin-top: 5px; font-size: 11px;">
                    Instance #{{ gap.from }} → #{{ gap.to }}: Missing {{ gap.missing.length }} instances
                    <div style="color: #fca5a5;">{{ gap.missing.join(', ') }}</div>
                  </div>
                </div>
                
                <div v-if="debugInfo.analysis.duplicates.length > 0" class="debug-alert alert-danger">
                  <strong>🚨 Duplicate Instance Numbers:</strong>
                  <span>{{ debugInfo.analysis.duplicates.join(', ') }}</span>
                </div>
              </div>

              <div class="debug-section">
                <h3>Series Information</h3>
                <div class="debug-row">
                  <span>Study UID:</span>
                  <code class="debug-code">{{ debugInfo.studyUID.substring(0, 30) }}...</code>
                </div>
                <div class="debug-row">
                  <span>Series UID:</span>
                  <code class="debug-code">{{ debugInfo.seriesUID.substring(0, 30) }}...</code>
                </div>
                <div class="debug-row">
                  <span>Images Loaded:</span>
                  <strong>{{ debugInfo.imageCount }}</strong>
                </div>
                <div class="debug-row">
                  <span>Timestamp:</span>
                  <code class="debug-code">{{ new Date(debugInfo.timestamp).toLocaleTimeString() }}</code>
                </div>
              </div>

              <div class="debug-section">
                <h3>Current Slice</h3>
                <div class="debug-row">
                  <span>Image Index:</span>
                  <strong>{{ currentImageIndex }} / {{ totalImages - 1 }}</strong>
                </div>
                <div v-if="allInstanceMetadata[currentImageIndex]" class="debug-slice-info">
                  <div class="debug-row">
                    <span>Instance #:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00200013']?.Value?.[0] || 'N/A' }}</strong>
                  </div>
                  <div class="debug-row">
                    <span>Rows × Columns:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00280010']?.Value?.[0] || '?' }} × {{ allInstanceMetadata[currentImageIndex]['00280011']?.Value?.[0] || '?' }}</strong>
                  </div>
                  <div class="debug-row">
                    <span>Bits Allocated:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00280100']?.Value?.[0] || 'N/A' }}</strong>
                  </div>
                  <div class="debug-row">
                    <span>Bits Stored:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00280101']?.Value?.[0] || 'N/A' }}</strong>
                  </div>
                  <div class="debug-row">
                    <span>High Bit:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00280102']?.Value?.[0] || 'N/A' }}</strong>
                  </div>
                  <div class="debug-row">
                    <span>Pixel Representation:</span>
                    <strong>{{ allInstanceMetadata[currentImageIndex]['00280103']?.Value?.[0] || 'N/A' }}</strong>
                  </div>
                </div>
              </div>

              <div class="debug-section">
                <h3>Instance Numbers</h3>
                <div class="debug-instance-list">
                  <span v-for="(num, idx) in debugInfo.analysis.instanceNumbers" :key="idx" class="debug-instance">
                    {{ num }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
import { Enums } from '@cornerstonejs/core'

// State
const studies = ref([])
const selectedStudy = ref(null)
const selectedStudyIndex = ref(-1)
const loading = ref(false)
const viewerLoading = ref(false)
const loadingMessage = ref('')
const error = ref(null)
const viewportElement = ref(null)
const currentImageInfo = ref(null)
const debugInfo = ref(null)
const showDebugPanel = ref(true)
const currentImageIndex = ref(0)
const totalImages = ref(0)
const allInstanceMetadata = ref([])

// Store cleanup function for event listeners
let cleanupEventListener = null

// Composables
const { init: initCornerstone, initialized } = useCornerstone()
const { retrieveImageIds, fetchStudies, fetchSeries, fetchInstances } = useDicomWeb()
const { create2DToolGroup, destroyToolGroup } = useCornerstoneTools()
const { 
  resizeViewport,
  createRenderingEngine, 
  createStackViewport, 
  destroyRenderingEngine,
  setupViewportEventListener
} = useCornerstoneViewport()

// Constants
const RENDERING_ENGINE_ID = 'myRenderingEngine'
const VIEWPORT_ID = 'CT_STACK'
const TOOL_GROUP_ID = 'myToolGroup'

// Initialize Cornerstone on mount
onMounted(async () => {
  try {
    await initCornerstone()
    await loadStudies()
    
    // Handle window resize
    const handleResize = () => {
      if (viewportElement.value) {
        resizeViewport(VIEWPORT_ID)
      }
    }
    window.addEventListener('resize', handleResize)
    
    // Store cleanup function
    onBeforeUnmount(() => {
      window.removeEventListener('resize', handleResize)
    })
  } catch (err) {
    console.error('Initialization failed:', err)
    error.value = 'Failed to initialize viewer: ' + err.message
  }
})

// Cleanup on unmount
onBeforeUnmount(() => {
  if (cleanupEventListener) {
    cleanupEventListener()
  }
  destroyRenderingEngine()
  destroyToolGroup(TOOL_GROUP_ID)
})

// Load studies from QIDO-RS
async function loadStudies() {
  loading.value = true
  error.value = null
  
  try {
    studies.value = await fetchStudies()
    console.log(`Loaded ${studies.value.length} studies`)
  } catch (err) {
    console.error('Failed to load studies:', err)
    error.value = 'Failed to load studies: ' + err.message
  } finally {
    loading.value = false
  }
}

// Select and load a study
async function selectStudy(study, index) {
  selectedStudy.value = study
  selectedStudyIndex.value = index
  
  try {
    await loadStudyImages(study)
  } catch (err) {
    console.error('Failed to load study:', err)
    error.value = 'Failed to load study: ' + err.message
  }
}

// Load study images into viewport
async function loadStudyImages(study) {
  // Wait for initialization and DOM to be ready
  if (!initialized.value) {
    console.warn('Cornerstone not initialized, waiting...')
    await new Promise(resolve => setTimeout(resolve, 100))
  }
  
  // Wait for viewport element to be available in DOM
  await nextTick()
  
  if (!viewportElement.value) {
    console.warn('Viewport element not ready')
    return
  }
  
  viewerLoading.value = true
  loadingMessage.value = 'Loading study...'
  error.value = null
  
  try {
    const studyUID = study['0020000D']?.Value?.[0]
    if (!studyUID) {
      throw new Error('Study UID not found')
    }
    
    // Fetch series for the study
    loadingMessage.value = 'Fetching series...'
    const series = await fetchSeries(studyUID)
    
    if (!series || series.length === 0) {
      throw new Error('No series found for study')
    }
    
    // Use first series
    const firstSeries = series[0]
    const seriesUID = firstSeries['0020000E']?.Value?.[0]
    
    if (!seriesUID) {
      throw new Error('Series UID not found')
    }
    
    // Fetch instances
    loadingMessage.value = 'Fetching instances...'
    const instances = await fetchInstances(studyUID, seriesUID)
    
    console.log(`Loading ${instances.length} images`)
    
    // Store info for overlay
    const firstInstance = instances[0]
    currentImageInfo.value = {
      patientName: getPatientName(study),
      studyDate: formatDate(study['00080020']?.Value?.[0]),
      modality: firstInstance['00080060']?.Value?.[0] || 'N/A',
      seriesNumber: firstInstance['00200011']?.Value?.[0] || null,
      instanceNumber: firstInstance['00200013']?.Value?.[0] || null,
    }
    
    // Load image IDs
    loadingMessage.value = 'Loading DICOM images...'
    const result = await retrieveImageIds(studyUID, seriesUID)
    const { imageIds, analysis } = result
    
    // Store all instance metadata for live updates
    const allInstances = await fetchInstances(studyUID, seriesUID)
    allInstances.sort((a, b) => {
      const aNum = parseInt(a['00200013']?.Value?.[0] || '0')
      const bNum = parseInt(b['00200013']?.Value?.[0] || '0')
      return aNum - bNum
    })
    allInstanceMetadata.value = allInstances
    totalImages.value = imageIds.length
    
    // Store debug info
    debugInfo.value = {
      timestamp: new Date().toISOString(),
      studyUID,
      seriesUID,
      analysis,
      imageCount: imageIds.length,
    }
    console.log('[loadStudyImages] Debug info:', debugInfo.value)
    
    // Create tool group
    const toolGroup = create2DToolGroup(TOOL_GROUP_ID)
    
    // Create rendering engine and viewport
    const engine = createRenderingEngine(RENDERING_ENGINE_ID)
    await createStackViewport(
      engine,
      VIEWPORT_ID,
      viewportElement.value,
      imageIds,
      TOOL_GROUP_ID
    )
    
    // Set up viewport event listeners for live updates
    // Clean up previous listener if it exists
    if (cleanupEventListener) {
      cleanupEventListener()
    }
    
    cleanupEventListener = setupViewportEventListener(
      viewportElement.value,
      Enums.Events.STACK_NEW_IMAGE,
      (event) => {
        const imageIndex = event.detail.imageIdIndex
        if (imageIndex !== undefined) {
          updateCurrentImageInfo(imageIndex)
        }
      },
      { debug: true }
    )
    
    // Initial image info
    updateCurrentImageInfo(0)
    
    viewerLoading.value = false
    console.log('✓ Study loaded successfully')
    
  } catch (err) {
    console.error('Failed to load study:', err)
    error.value = 'Failed to load study: ' + err.message
    viewerLoading.value = false
  }
}

// Helper functions
function getPatientName(study) {
  const name = study['00100010']?.Value?.[0]
  if (!name) return 'Unknown'
  return name.Alphabetic || name
}

function formatDate(dateStr) {
  if (!dateStr || dateStr === 'N/A') return 'N/A'
  const year = dateStr.substring(0, 4)
  const month = dateStr.substring(4, 6)
  const day = dateStr.substring(6, 8)
  return `${year}-${month}-${day}`
}

// Update image info based on current slice
function updateCurrentImageInfo(imageIndex) {
  currentImageIndex.value = imageIndex
  
  if (allInstanceMetadata.value[imageIndex]) {
    const metadata = allInstanceMetadata.value[imageIndex]
    currentImageInfo.value = {
      patientName: getPatientName(selectedStudy.value),
      studyDate: formatDate(selectedStudy.value['00080020']?.Value?.[0]),
      modality: metadata['00080060']?.Value?.[0] || 'N/A',
      seriesNumber: metadata['00200011']?.Value?.[0] || 'N/A',
      instanceNumber: metadata['00200013']?.Value?.[0] || 'N/A',
      sopInstanceUid: metadata['00080018']?.Value?.[0]?.substring(0, 40) + '...' || 'N/A',
      rows: metadata['00280010']?.Value?.[0] || 'N/A',
      columns: metadata['00280011']?.Value?.[0] || 'N/A',
    }
  }
}
</script>

<style scoped>
.viewer-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #0f172a;
}

.header {
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid #334155;
  padding: 12px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 100;
}

.header h1 {
  font-size: 18px;
  font-weight: 600;
  color: #3b82f6;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  text-decoration: none;
  display: inline-block;
  border: none;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: #334155;
  color: #e2e8f0;
}

.btn-secondary:hover {
  background: #475569;
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 320px;
  background: rgba(15, 23, 42, 0.95);
  border-right: 1px solid #334155;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid #334155;
}

.sidebar-header h2 {
  font-size: 16px;
  margin: 0;
  color: #e2e8f0;
}

.error-box {
  background: #dc2626;
  color: white;
  padding: 12px;
  margin: 10px;
  border-radius: 6px;
  font-size: 14px;
}

.empty-state {
  padding: 20px;
  text-align: center;
  color: #64748b;
}

.study-list {
  list-style: none;
  padding: 10px;
  overflow-y: auto;
  flex: 1;
}

.study-item {
  background: #0f172a;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.study-item:hover {
  background: #334155;
  border-color: #3b82f6;
}

.study-item.active {
  border-color: #3b82f6;
  background: #1e3a8a;
}

.study-info strong {
  color: #3b82f6;
  display: block;
  margin-bottom: 5px;
  font-size: 14px;
}

.study-meta {
  color: #94a3b8;
  font-size: 12px;
  margin-bottom: 3px;
}

.viewer-panel {
  flex: 1;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.no-study {
  text-align: center;
  color: #64748b;
}

.no-study h2 {
  font-size: 24px;
  margin-bottom: 10px;
}

.viewer-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}

.viewer-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: #e2e8f0;
  z-index: 1000;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #334155;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 10px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.viewport-container {
  flex: 1;
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.info-overlay {
  position: absolute;
  bottom: 20px;
  left: 20px;
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(10px);
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 12px;
  color: #e2e8f0;
  border: 1px solid #334155;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.info-overlay div {
  margin-bottom: 4px;
}

.info-overlay div:last-child {
  margin-bottom: 0;
}

.info-overlay strong {
  color: #3b82f6;
  margin-right: 4px;
}

.info-header {
  background: linear-gradient(135deg, #1e3a8a 0%, #1e40af 100%);
  margin: -12px -16px 8px -16px;
  padding: 8px 16px;
  border-radius: 8px 8px 0 0;
  border-bottom: 1px solid #334155;
  font-weight: 600;
}

.info-header strong {
  color: #3b82f6;
}

.info-uid {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #334155;
  font-size: 10px;
}

.info-uid code {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 4px;
  border-radius: 2px;
  color: #22c55e;
  font-family: 'Monaco', monospace;
}

.debug-panel {
  position: absolute;
  right: 20px;
  bottom: 20px;
  background: rgba(15, 23, 42, 0.98);
  backdrop-filter: blur(10px);
  border: 1px solid #334155;
  border-radius: 8px;
  max-width: 380px;
  max-height: 60vh;
  overflow: hidden;
  z-index: 1000;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
}

.debug-toggle {
  display: block;
  width: 100%;
  padding: 12px 16px;
  background: linear-gradient(135deg, #1e3a8a 0%, #1e40af 100%);
  color: #e2e8f0;
  border: none;
  border-radius: 8px 8px 0 0;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s;
}

.debug-toggle:hover {
  background: linear-gradient(135deg, #1e40af 0%, #1e3a8a 100%);
}

.debug-content {
  padding: 12px;
  overflow-y: auto;
  max-height: calc(60vh - 44px);
  font-size: 11px;
}

.debug-section {
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid #334155;
}

.debug-section:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

.debug-section h3 {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #3b82f6;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.debug-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  color: #cbd5e1;
}

.debug-row span {
  color: #94a3b8;
  flex: 0 0 auto;
}

.debug-row strong {
  color: #f1f5f9;
  text-align: right;
  flex: 1;
  margin-left: 8px;
}

.debug-row .text-danger {
  color: #ef4444;
}

.debug-code {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 6px;
  border-radius: 3px;
  color: #22c55e;
  font-family: 'Monaco', 'Courier New', monospace;
  word-break: break-all;
  text-align: right;
  flex: 1;
  margin-left: 8px;
  display: block;
}

.debug-alert {
  margin-top: 8px;
  padding: 8px;
  border-radius: 4px;
  border-left: 3px solid;
  font-size: 10px;
}

.alert-warning {
  background: rgba(217, 119, 6, 0.1);
  border-left-color: #f59e0b;
  color: #fbbf24;
}

.alert-danger {
  background: rgba(220, 38, 38, 0.1);
  border-left-color: #ef4444;
  color: #fca5a5;
}

.debug-alert strong {
  display: block;
  margin-bottom: 4px;
}

.debug-instance-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}

.debug-instance {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid #3b82f6;
  border-radius: 3px;
  padding: 2px 6px;
  color: #3b82f6;
  font-weight: 500;
  font-family: 'Monaco', monospace;
}

.debug-slice-info {
  background: rgba(59, 130, 246, 0.05);
  padding: 6px;
  border-radius: 4px;
  border-left: 2px solid #3b82f6;
}
</style>
