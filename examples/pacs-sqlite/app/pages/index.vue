<template>
  <div class="dashboard">
    <header class="header">
      <h1>🏥 PACS SQLite Example</h1>
    </header>
    
    <main class="content">
      <p class="intro">
        Complete PACS implementation using Nuxt 4, SQLite, and Cornerstone3D Viewer.
      </p>
      
      <div class="status-grid">
        <div class="status-card">
          <h3><span class="status-indicator status-ok"></span>StoreSCP</h3>
          <p>Port: <strong>11112</strong></p>
          <p>AE Title: <strong>PACS_SQLITE</strong></p>
          <p>Status: Running</p>
        </div>
        
        <div class="status-card">
          <h3><span class="status-indicator status-ok"></span>QIDO-RS</h3>
          <p>Port: <strong>8042</strong></p>
          <p><a href="http://localhost:8042/studies" target="_blank">Query Studies</a></p>
          <p>Status: Running</p>
        </div>
        
        <div class="status-card">
          <h3><span class="status-indicator status-ok"></span>WADO-RS</h3>
          <p>Port: <strong>8043</strong></p>
          <p>Endpoint: Retrieval Service</p>
          <p>Status: Running</p>
        </div>
        
        <div class="status-card">
          <h3><span class="status-indicator status-ok"></span>Database</h3>
          <p>Type: <strong>SQLite</strong></p>
          <p>Location: <code>.data/pacs.db</code></p>
          <p>{{ dbStats }}</p>
        </div>
      </div>
      
      <div class="info-box">
        <strong>📖 Quick Start:</strong>
        <ol>
          <li>Send DICOM files: <code>node scripts/send-test-files.mjs ./testdata</code></li>
          <li>Query studies: <code>node scripts/query-studies.mjs</code></li>
          <li>Inspect database: <code>node scripts/inspect-db.mjs</code></li>
        </ol>
      </div>
      
      <h2>Actions</h2>
      
      <div class="actions">
        <a href="http://localhost:8042/studies" class="button" target="_blank">
          📋 Query Studies (QIDO-RS)
        </a>
        
        <NuxtLink to="/viewer" class="button button-primary">
          🔬 Open DICOM Viewer
        </NuxtLink>
        
        <a href="https://github.com/DevJoghurt/node-dicom-rs" class="button button-secondary" target="_blank">
          📚 Documentation
        </a>
      </div>
      
      <div class="features">
        <h2>Features</h2>
        <ul>
          <li>✅ StoreSCP server for receiving DICOM files</li>
          <li>✅ QIDO-RS for querying studies, series, and instances</li>
          <li>✅ WADO-RS/WADO-URI for retrieving DICOM files</li>
          <li>✅ SQLite database for metadata storage</li>
          <li>✅ Cornerstone3D DICOM viewer with advanced tools</li>
          <li>✅ Async/await callback support</li>
        </ul>
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';

const dbStats = ref('Loading stats...');

onMounted(() => {
  // Placeholder for future API call
  dbStats.value = 'Ready';
});
</script>

<style scoped>
.dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
}

.header {
  background: rgba(15, 23, 42, 0.8);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid #334155;
  padding: 20px 40px;
  position: sticky;
  top: 0;
  z-index: 100;
}

.header h1 {
  color: #3b82f6;
  font-size: 28px;
  font-weight: 700;
}

.content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 20px;
}

.intro {
  font-size: 18px;
  color: #cbd5e1;
  margin-bottom: 40px;
  line-height: 1.6;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 40px;
}

.status-card {
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid #334155;
  border-radius: 12px;
  padding: 24px;
  transition: all 0.3s;
}

.status-card:hover {
  border-color: #3b82f6;
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.1);
}

.status-card h3 {
  margin-bottom: 12px;
  color: #e2e8f0;
  font-size: 18px;
  display: flex;
  align-items: center;
}

.status-indicator {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-right: 10px;
}

.status-ok {
  background: #10b981;
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.5);
}

.status-card p {
  color: #94a3b8;
  margin-bottom: 8px;
  font-size: 14px;
}

.status-card a {
  color: #3b82f6;
  text-decoration: none;
}

.status-card a:hover {
  text-decoration: underline;
}

.status-card code {
  background: #0f172a;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: #60a5fa;
}

.info-box {
  background: rgba(37, 99, 235, 0.1);
  border-left: 4px solid #3b82f6;
  padding: 20px;
  margin-bottom: 40px;
  border-radius: 8px;
}

.info-box strong {
  color: #3b82f6;
  font-size: 16px;
}

.info-box ol {
  margin-top: 12px;
  margin-left: 20px;
  color: #cbd5e1;
}

.info-box li {
  margin-bottom: 8px;
  line-height: 1.6;
}

.info-box code {
  background: rgba(15, 23, 42, 0.8);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 13px;
  color: #60a5fa;
  font-family: 'Monaco', 'Menlo', monospace;
}

h2 {
  color: #e2e8f0;
  font-size: 24px;
  margin-bottom: 20px;
  margin-top: 40px;
}

.actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 40px;
}

.button {
  display: inline-block;
  padding: 12px 24px;
  border-radius: 8px;
  text-decoration: none;
  font-weight: 500;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.button {
  background: #1e293b;
  color: #e2e8f0;
  border-color: #334155;
}

.button:hover {
  background: #334155;
  border-color: #3b82f6;
}

.button-primary {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.button-primary:hover {
  background: #2563eb;
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.3);
}

.button-secondary {
  background: #475569;
  color: white;
  border-color: #475569;
}

.button-secondary:hover {
  background: #334155;
}

.features {
  margin-top: 40px;
}

.features ul {
  list-style: none;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 12px;
}

.features li {
  color: #cbd5e1;
  padding: 12px;
  background: rgba(30, 41, 59, 0.4);
  border-radius: 8px;
  border: 1px solid #334155;
  font-size: 14px;
}
</style>
