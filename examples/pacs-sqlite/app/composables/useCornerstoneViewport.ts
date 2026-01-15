/**
 * Composable for managing Cornerstone viewports
 * Based on: https://medium.com/@k_raky/displaying-dicom-images-on-the-web-multiplanar-reconstruction-view-with-cornerstone3d-orthanc-and-react-f5a57fb1b363
 */

import { Enums, RenderingEngine, type Types } from '@cornerstonejs/core'
import { ToolGroupManager } from '@cornerstonejs/tools'

export const useCornerstoneViewport = () => {
  let renderingEngine: RenderingEngine | null = null
  
  const createRenderingEngine = (renderingEngineId: string) => {
    if (renderingEngine) {
      renderingEngine.destroy()
    }
    
    renderingEngine = new RenderingEngine(renderingEngineId)
    return renderingEngine
  }
  
  const createStackViewport = async (
    engine: RenderingEngine,
    viewportId: string,
    element: HTMLDivElement,
    imageIds: string[],
    toolGroupId?: string
  ) => {
    const viewportInput: Types.PublicViewportInput = {
      viewportId,
      type: Enums.ViewportType.STACK,
      element,
      defaultOptions: {
        background: [0, 0, 0] as Types.Point3
      }
    }
    
    // Enable the element with initial imageIds
    engine.enableElement(viewportInput)
    
    // Get the viewport
    const viewport = engine.getViewport(viewportId) as Types.IStackViewport
    
    // Set the image stack on the viewport
    await viewport.setStack(imageIds, 0)
    
    // Reset camera to fit image to canvas while maintaining aspect ratio
    viewport.resetCamera()
    
    // Add viewport to tool group
    if (toolGroupId) {
      const toolGroup = ToolGroupManager.getToolGroup(toolGroupId)
      if (toolGroup) {
        toolGroup.addViewport(viewportId, engine.id)
      }
    }
    
    // Render the viewport
    viewport.render()
    
    return viewport
  }
  
  const resizeViewport = (viewportId: string) => {
    if (renderingEngine) {
      const viewport = renderingEngine.getViewport(viewportId) as Types.IStackViewport
      if (viewport) {
        viewport.resetCamera()
        viewport.render()
      }
    }
  }
  
  const destroyRenderingEngine = () => {
    if (renderingEngine) {
      renderingEngine.destroy()
      renderingEngine = null
    }
  }
  
  const setupViewportEventListener = <T = any>(
    element: HTMLDivElement,
    eventType: Enums.Events,
    callback: (event: CustomEvent<T>) => void,
    options?: { debug?: boolean }
  ) => {
    const debug = options?.debug ?? false
    
    if (debug) {
      console.log(`[setupViewportEventListener] Setting up listener for event: ${eventType}`)
    }
    
    // Create typed event listener
    const listener = (evt: Event) => {
      const customEvent = evt as CustomEvent<T>
      
      if (debug) {
        console.log(`[${eventType}] Event fired:`, customEvent.detail)
      }
      
      callback(customEvent)
    }
    
    element.addEventListener(eventType, listener)
    
    if (debug) {
      console.log(`[setupViewportEventListener] Listener setup complete for ${eventType}`)
    }
    
    // Return cleanup function
    return () => {
      element.removeEventListener(eventType, listener)
      if (debug) {
        console.log(`[setupViewportEventListener] Listener removed for ${eventType}`)
      }
    }
  }
  
  return {
    createRenderingEngine,
    createStackViewport,
    resizeViewport,
    destroyRenderingEngine,
    setupViewportEventListener,
  }
}
