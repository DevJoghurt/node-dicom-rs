/**
 * Composable for managing Cornerstone tools
 * Based on: https://medium.com/@k_raky/displaying-dicom-images-on-the-web-multiplanar-reconstruction-view-with-cornerstone3d-orthanc-and-react-f5a57fb1b363
 */

import {
  addTool,
  ZoomTool,
  PanTool,
  StackScrollTool,
  ToolGroupManager,
  WindowLevelTool,
} from '@cornerstonejs/tools'
import { MouseBindings } from '@cornerstonejs/tools/enums'

export const useCornerstoneTools = () => {
  const create2DToolGroup = (toolGroupId: string) => {
    // Check if tool group already exists
    let toolGroup2D = ToolGroupManager.getToolGroup(toolGroupId)
    
    if (toolGroup2D) {
      // Tool group already exists, return it
      return toolGroup2D
    }
    
    // Add tools to Cornerstone3D (only once)
    addTool(PanTool)
    addTool(ZoomTool)
    addTool(StackScrollTool)
    addTool(WindowLevelTool)

    // Create tool groups for 2D viewports
    toolGroup2D = ToolGroupManager.createToolGroup(toolGroupId)
    
    if (!toolGroup2D) {
      throw new Error('Failed to create tool group')
    }

    // Configure 2D tool group
    toolGroup2D.addTool(ZoomTool.toolName)
    toolGroup2D.addTool(PanTool.toolName)
    toolGroup2D.addTool(StackScrollTool.toolName)
    toolGroup2D.addTool(WindowLevelTool.toolName)

    toolGroup2D.setToolActive(ZoomTool.toolName, {
      bindings: [{ mouseButton: MouseBindings.Secondary }],
    })
    toolGroup2D.setToolActive(PanTool.toolName, {
      bindings: [{ mouseButton: MouseBindings.Auxiliary }],
    })
    toolGroup2D.setToolActive(WindowLevelTool.toolName, {
      bindings: [{ mouseButton: MouseBindings.Primary }],
    })
    toolGroup2D.setToolActive(StackScrollTool.toolName, {
      bindings: [{ mouseButton: MouseBindings.Wheel }],
    })

    return toolGroup2D
  }

  const destroyToolGroup = (toolGroupId: string) => {
    ToolGroupManager.destroyToolGroup(toolGroupId)
  }

  return {
    create2DToolGroup,
    destroyToolGroup,
  }
}
