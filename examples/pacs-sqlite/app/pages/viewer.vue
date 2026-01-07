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
            <div><strong>Patient:</strong> {{ currentImageInfo.patientName }}</div>
            <div><strong>Study Date:</strong> {{ currentImageInfo.studyDate }}</div>
            <div><strong>Modality:</strong> {{ currentImageInfo.modality }}</div>
            <div v-if="currentImageInfo.seriesNumber"><strong>Series:</strong> {{ currentImageInfo.seriesNumber }}</div>
            <div v-if="currentImageInfo.instanceNumber"><strong>Instance:</strong> {{ currentImageInfo.instanceNumber }}</div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
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

// Composables
const { init: initCornerstone, initialized } = useCornerstone()
const { retrieveImageIds } = useDicomWeb()
const { create2DToolGroup, destroyToolGroup } = useCornerstoneTools()
const { resizeViewport } = useCornerstoneViewport()
const { 
  createRenderingEngine, 
  createStackViewport, 
  destroyRenderingEngine 
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
  destroyRenderingEngine()
  destroyToolGroup(TOOL_GROUP_ID)
})

// Load studies from QIDO-RS
async function loadStudies() {
  loading.value = true
  error.value = null
  
  try {
    const { fetchStudies } = useDicomWeb()
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
    const { fetchSeries, fetchInstances } = useDicomWeb()
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
    const imageIds = await retrieveImageIds(studyUID, seriesUID)
    
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
</style>
