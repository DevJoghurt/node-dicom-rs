import { defineHandler } from "nitro/h3";

export default defineHandler(() => {
  return new Response(
`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>DICOM Viewer - PACS SQLite</title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
      height: 100vh;
      overflow: hidden;
      background: #1e293b;
      color: #e2e8f0;
    }
    
    .header {
      background: #0f172a;
      border-bottom: 1px solid #334155;
      padding: 12px 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    
    .header h1 {
      font-size: 18px;
      font-weight: 600;
      color: #3b82f6;
    }
    
    .header-actions {
      display: flex;
      gap: 10px;
    }
    
    .btn {
      padding: 8px 16px;
      border-radius: 6px;
      border: none;
      cursor: pointer;
      font-size: 14px;
      transition: all 0.2s;
      text-decoration: none;
      display: inline-block;
    }
    
    .btn-primary {
      background: #3b82f6;
      color: white;
    }
    
    .btn-primary:hover {
      background: #2563eb;
    }
    
    .btn-secondary {
      background: #475569;
      color: white;
    }
    
    .btn-secondary:hover {
      background: #334155;
    }
    
    .container {
      height: calc(100vh - 61px);
      display: flex;
    }
    
    .sidebar {
      width: 300px;
      background: #0f172a;
      border-right: 1px solid #334155;
      overflow-y: auto;
      padding: 20px;
    }
    
    .viewer-panel {
      flex: 1;
      background: #000;
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
    }
    
    .study-list {
      list-style: none;
    }
    
    .study-item {
      background: #1e293b;
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
    
    .study-info {
      font-size: 14px;
    }
    
    .study-info strong {
      color: #3b82f6;
      display: block;
      margin-bottom: 5px;
    }
    
    .study-info div {
      color: #94a3b8;
      font-size: 12px;
      margin-bottom: 3px;
    }
    
    .loading {
      text-align: center;
      padding: 20px;
      color: #64748b;
    }
    
    .viewer-frame {
      width: 100%;
      height: 100%;
      border: none;
    }
    
    .no-study {
      text-align: center;
      color: #64748b;
    }
    
    .no-study h2 {
      font-size: 24px;
      margin-bottom: 10px;
    }
    
    .no-study p {
      font-size: 14px;
    }
  </style>
</head>
<body>
  <div class="header">
    <h1>🔬 DICOM Viewer</h1>
    <div class="header-actions">
      <a href="/" class="btn btn-secondary">← Back to Dashboard</a>
      <button class="btn btn-primary" onclick="loadStudies()">🔄 Refresh Studies</button>
    </div>
  </div>
  
  <div class="container">
    <div class="sidebar">
      <h2 style="margin-bottom: 15px; font-size: 16px;">Studies</h2>
      <div id="studies-container">
        <div class="loading">Loading studies...</div>
      </div>
    </div>
    
    <div class="viewer-panel" id="viewer-panel">
      <div class="no-study">
        <h2>No Study Selected</h2>
        <p>Select a study from the sidebar to view</p>
      </div>
    </div>
  </div>
  
  <script>
    let studies = [];
    let selectedStudy = null;
    
    async function loadStudies() {
      try {
        const container = document.getElementById('studies-container');
        container.innerHTML = '<div class="loading">Loading studies...</div>';
        
        const response = await fetch('http://localhost:8042/studies');
        studies = await response.json();
        
        if (studies.length === 0) {
          container.innerHTML = '<div class="loading">No studies available</div>';
          return;
        }
        
        const list = document.createElement('ul');
        list.className = 'study-list';
        
        studies.forEach((study, index) => {
          const item = document.createElement('li');
          item.className = 'study-item';
          item.onclick = () => viewStudy(study, index);
          
          const patientName = study['00100010']?.Value?.[0]?.Alphabetic || 'Unknown';
          const studyDate = study['00080020']?.Value?.[0] || 'N/A';
          const studyDesc = study['00081030']?.Value?.[0] || 'No Description';
          const studyUID = study['0020000D']?.Value?.[0] || '';
          const instances = study['00201208']?.Value?.[0] || '0';
          
          item.innerHTML = \`
            <div class="study-info">
              <strong>\${patientName}</strong>
              <div>📅 \${formatDate(studyDate)}</div>
              <div>📝 \${studyDesc}</div>
              <div>🖼️ \${instances} instances</div>
            </div>
          \`;
          
          list.appendChild(item);
        });
        
        container.innerHTML = '';
        container.appendChild(list);
      } catch (error) {
        console.error('Failed to load studies:', error);
        document.getElementById('studies-container').innerHTML = 
          '<div class="loading">Failed to load studies</div>';
      }
    }
    
    function formatDate(dateStr) {
      if (!dateStr || dateStr === 'N/A') return 'N/A';
      const year = dateStr.substring(0, 4);
      const month = dateStr.substring(4, 6);
      const day = dateStr.substring(6, 8);
      return \`\${year}-\${month}-\${day}\`;
    }
    
    function viewStudy(study, index) {
      selectedStudy = study;
      
      // Update active state
      document.querySelectorAll('.study-item').forEach((item, i) => {
        item.classList.toggle('active', i === index);
      });
      
      const studyUID = study['0020000D']?.Value?.[0];
      if (!studyUID) {
        alert('Study UID not found');
        return;
      }
      
      // Use OHIF Viewer demo with our DICOMweb servers
      const ohifUrl = \`https://viewer.ohif.org/viewer?StudyInstanceUIDs=\${studyUID}\` +
        \`&url=http://localhost:8042\` +
        \`&wadoUriRoot=http://localhost:8043\`;
      
      const viewerPanel = document.getElementById('viewer-panel');
      viewerPanel.innerHTML = \`<iframe class="viewer-frame" src="\${ohifUrl}" allow="fullscreen"></iframe>\`;
    }
    
    // Load studies on page load
    loadStudies();
  </script>
</body>
</html>`,
    { headers: { "content-type": "text/html; charset=utf-8" } });
});
