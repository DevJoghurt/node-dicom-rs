# PACS SQLite Example

Complete production-ready PACS (Picture Archiving and Communication System) implementation demonstrating all node-dicom-rs services:

- **StoreSCP** - Receive DICOM files via C-STORE (Port 11112)
- **QIDO-RS** - Query DICOM metadata (Port 8042)
- **WADO-RS** - Retrieve DICOM files (Port 8043)
- **SQLite** - Fast embedded database for metadata
- **Cornerstone3D** - Advanced DICOM viewer
- **Nuxt 4** - Modern full-stack framework with auto-imports

## Quick Start

```bash
# 0. Go to examples folder
cd ./examples/pacs-sqlite

# 1. Install dependencies
npm install

# 2. Start the server (all services)
npm run dev

# 3. In another terminal, send test files
cd ./examples/pacs-sqlite
./scripts/downloadTestData.sh  # If not already downloaded
node scripts/send-test-files.mjs ./testdata

# 4. Open browser
open http://localhost:3000

# 5. View DICOM studies in Cornerstone3D viewer
open http://localhost:3000/viewer
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Nuxt 4 Development Server                │
│              http://localhost:3000 (Web UI + API)           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Frontend (Nuxt App)                                        │
│  ├── Dashboard (/)           - Service status               │
│  └── Viewer (/viewer)        - Cornerstone3D DICOM viewer  │
│                                                             │
│  Backend (Nitro Server Plugins - Auto-loaded on startup)   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   StoreSCP   │  │   QIDO-RS    │  │   WADO-RS    │    │
│  │  Port: 11112 │  │  Port: 8042  │  │  Port: 8043  │    │
│  │  C-STORE RX  │  │  Query Meta  │  │  Retrieve    │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                 │                  │             │
│         │        ┌────────┴────────┐         │             │
│         └────────►  SQLite Database ◄─────────┘             │
│                  │ (.data/pacs.db) │                       │
│                  │ - studies       │                       │
│                  │ - series        │                       │
│                  │ - instances     │                       │
│                  └─────────────────┘                       │
│                         │                                  │
│                  ┌──────▼───────────────────┐              │
│                  │   Filesystem Storage     │              │
│                  │ .data/dicom/{study}/...  │              │
│                  └──────────────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

## Features

### 🏥 Complete DICOM Workflow

1. **Receive**: DICOM modalities send studies via C-STORE to StoreSCP
2. **Store**: Files saved to `.data/dicom/`, metadata stored in SQLite
3. **Anonymize**: Automatic fake patient data generation on receive
4. **Query**: Clients search studies via QIDO-RS API
5. **Retrieve**: Clients fetch DICOM files via WADO-RS API
6. **View**: Cornerstone3D viewer displays studies with advanced tools

### 📦 Technology Stack

- **Nuxt 4** - Full-stack Vue framework with file-based routing
- **Nitro** - Server engine with hot reload and auto-imports
- **SQLite** - Embedded database (no external services needed)
- **Cornerstone3D v4.14.5** - Medical imaging viewer with tools
- **node-dicom-rs** - High-performance Rust-powered DICOM operations

### ⚡ Developer Experience

- **Hot Reload** - Server restarts instantly on file changes
- **Auto-imports** - No need to import Nuxt/Vue composables
- **TypeScript** - Full type safety across frontend and backend
- **File-based Routing** - Pages in `app/pages/`, API in `server/api/`
- **Auto-loaded Plugins** - Server plugins load automatically in order

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Download Test Data (Optional)

```bash
cd ../../playground
./downloadTestData.sh
cd ../examples/pacs-sqlite
```

### 3. Start PACS Server

```bash
npm run dev
```

This starts:
- **Nuxt Dev Server** on http://localhost:3000
- **StoreSCP** on port 11112 (DICOM C-STORE)
- **QIDO-RS** on port 8042 (Query API)
- **WADO-RS** on port 8043 (Retrieval API)
- **SQLite Database** at `.data/pacs.db`

### 4. Send DICOM Files

```bash
# Send test files from playground
node scripts/send-test-files.mjs ../../playground/testdata

# Or send specific directory
node scripts/send-test-files.mjs /path/to/dicom/files
```

### 5. View Studies

- **Dashboard**: http://localhost:3000 - Service status
- **Viewer**: http://localhost:3000/viewer - Cornerstone3D DICOM viewer

## Project Structure

```
pacs-sqlite/
├── package.json              # Dependencies and scripts
├── nuxt.config.ts           # Nuxt 4 configuration
├── tsconfig.json            # TypeScript configuration
├── README.md                # This file
│
├── app/                     # Nuxt application (frontend)
│   ├── pages/
│   │   ├── index.vue            # Dashboard page
│   │   └── viewer.vue           # Cornerstone3D viewer
│   └── composables/
│       ├── useCornerstone.ts    # Cornerstone initialization
│       ├── useCornerstoneTools.ts   # Tool setup
│       ├── useCornerstoneViewport.ts # Viewport management
│       └── useDicomWeb.ts       # DICOM data fetching
│
├── server/                  # Nitro server (backend)
│   ├── plugins/            # Auto-loaded on startup
│   │   ├── 01.database.ts      # SQLite initialization
│   │   ├── 02.storescp.ts      # StoreSCP C-STORE receiver
│   │   ├── 03.qido.ts          # QIDO-RS query service
│   │   └── 04.wado.ts          # WADO-RS retrieval service
│   └── api/                # API routes
│       └── studies.get.ts      # Get studies endpoint
│
├── scripts/                # Utility scripts
│   ├── send-test-files.mjs    # Send DICOM files
│   └── query-studies.mjs      # Query QIDO-RS
│
├── .data/                  # Runtime data (git-ignored)
│   ├── pacs.db            # SQLite database
│   └── dicom/             # DICOM file storage
│       └── {studyUID}/
│           └── {seriesUID}/
│               └── {instanceUID}.dcm
│
└── testdata/              # Test DICOM files (optional)
```

## Configuration

### Database Schema

The SQLite database automatically creates these tables on first run:

**studies** - Study-level metadata
```sql
CREATE TABLE studies (
  study_instance_uid TEXT PRIMARY KEY,
  patient_name TEXT,
  patient_id TEXT,
  patient_birth_date TEXT,
  patient_sex TEXT,
  study_date TEXT,
  study_time TEXT,
  study_description TEXT,
  accession_number TEXT,
  modalities_in_study TEXT,
  number_of_series INTEGER,
  number_of_instances INTEGER
);
CREATE INDEX idx_patient_id ON studies(patient_id);
CREATE INDEX idx_study_date ON studies(study_date);
```

**series** - Series-level metadata
```sql
CREATE TABLE series (
  series_instance_uid TEXT PRIMARY KEY,
  study_instance_uid TEXT,
  modality TEXT,
  series_number INTEGER,
  series_description TEXT,
  number_of_instances INTEGER,
  FOREIGN KEY(study_instance_uid) REFERENCES studies(study_instance_uid)
);
CREATE INDEX idx_series_study ON series(study_instance_uid);
```

**instances** - Instance-level metadata
```sql
CREATE TABLE instances (
  sop_instance_uid TEXT PRIMARY KEY,
  series_instance_uid TEXT,
  sop_class_uid TEXT,
  instance_number INTEGER,
  file_path TEXT,
  FOREIGN KEY(series_instance_uid) REFERENCES series(series_instance_uid)
);
CREATE INDEX idx_instance_series ON instances(series_instance_uid);
```

### StoreSCP Configuration

Located in `server/plugins/02.storescp.ts`:

```typescript
const storeScp = new StoreScp({
  port: 11112,
  callingAeTitle: 'PACS_SQLITE',
  outDir: DICOM_STORAGE_PATH,  // .data/dicom
  verbose: true,
  extractTags: [
    'PatientName', 'PatientID', 'PatientBirthDate', 'PatientSex',
    'StudyInstanceUID', 'StudyDate', 'StudyTime', 'StudyDescription',
    'SeriesInstanceUID', 'Modality', 'SeriesNumber', 'SeriesDescription',
    'SOPInstanceUID', 'SOPClassUID', 'InstanceNumber'
  ],
  studyTimeout: 30
});
```

**File Storage Structure:**
```
.data/dicom/
  └── {studyUID}/
      └── {seriesUID}/
          └── {instanceUID}.dcm
```

**Anonymization**: Uses `onBeforeStore` callback to generate fake patient data before saving.

### QIDO-RS Configuration

Located in `server/plugins/03.qido.ts`:

```typescript
const qidoServer = new QidoServer(8042);
qidoServer.start();
```

**CORS**: Enabled for `http://localhost:3000`

**Endpoints:**
- `GET /studies` - Search all studies
- `GET /studies?PatientID={id}` - Search by patient
- `GET /studies/{uid}/series` - Get series in study
- `GET /series` - Search all series
- `GET /instances` - Search all instances

### WADO-RS Configuration

Located in `server/plugins/04.wado.ts`:

```typescript
const wadoServer = new WadoServer(8043, {
  storageType: 'filesystem',
  basePath: DICOM_STORAGE_PATH  // .data/dicom
});
wadoServer.start();
```

**CORS**: Enabled for `http://localhost:3000`

**Endpoints:**
- `GET /studies/{studyUID}` - Retrieve all study instances
- `GET /studies/{studyUID}/series/{seriesUID}` - Retrieve series
- `GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}` - Retrieve instance
- `GET /studies/{studyUID}/metadata` - Get study metadata JSON

### Cornerstone3D Viewer

Located in `app/pages/viewer.vue` with composables:

**Features:**
- Stack viewport for CT/MR slices
- Mouse wheel scrolling through frames
- Window/Level adjustment
- Pan and Zoom tools
- Proper frame ordering by Instance Number
- Automatic aspect ratio maintenance

**Tools Available:**
- WindowLevel - Adjust brightness/contrast
- Pan - Move image
- Zoom - Scale image
- StackScroll - Scroll through frames with mouse wheel

## Fake Patient Data Generation

To protect privacy, incoming DICOM files are automatically anonymized with consistent, realistic fake data.

**Implementation** in `server/plugins/02.storescp.ts`:

```typescript
storeScp.onBeforeStore((tags) => {
  const patientId = tags.PatientID || 'UNKNOWN';
  const fakeData = generateFakePatientData(patientId);
  
  return {
    ...tags,
    PatientName: fakeData.patientName,
    PatientID: fakeData.patientId,
    PatientBirthDate: fakeData.patientBirthDate,
    PatientSex: fakeData.patientSex
  };
});
```

**Characteristics:**
- **Consistent**: Same patient ID always generates same fake data
- **Realistic**: Names, dates, and demographics look authentic
- **Seeded**: Uses SHA-256 hash of patient ID as random seed
- **Diverse**: Multiple name combinations and demographics

**Example Output:**
```javascript
{
  patientName: 'Smith^John',
  patientId: 'PAT12345',
  patientBirthDate: '19750315',
  patientSex: 'M'
}
```

## Development

### Hot Reload

Nuxt provides instant hot reload for all changes:

```bash
npm run dev
```

- **Frontend changes** (app/) - Instant browser update
- **Server plugins** (server/plugins/) - Auto-restart on save
- **API routes** (server/api/) - Auto-reload on save

### Database Inspection

Query the SQLite database directly:

```bash
# Using better-sqlite3 REPL
node
> const Database = require('better-sqlite3');
> const db = new Database('.data/pacs.db');
> db.prepare('SELECT * FROM studies').all();

# Or use sqlite3 CLI
sqlite3 .data/pacs.db
sqlite> SELECT * FROM studies;
sqlite> SELECT COUNT(*) FROM instances;
```

### Testing Services

**Test QIDO-RS:**
```bash
# Search all studies
curl http://localhost:8042/studies | jq

# Search by patient
curl "http://localhost:8042/studies?PatientID=PAT12345" | jq

# Get series in study
curl http://localhost:8042/studies/{studyUID}/series | jq
```

**Test WADO-RS:**
```bash
# Retrieve DICOM instance
curl http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID} \
  -o retrieved.dcm

# Get metadata
curl http://localhost:8043/studies/{studyUID}/metadata | jq
```

**Test StoreSCP:**
```bash
# Use send-test-files script
node scripts/send-test-files.mjs ../../playground/testdata

# Or use dcmtk storescu (if installed)
storescu localhost 11112 -aec PACS_SQLITE /path/to/file.dcm
```

### Available Scripts

```bash
# Development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Send test DICOM files
node scripts/send-test-files.mjs <directory>

# Query studies
node scripts/query-studies.mjs
```

## Production Deployment

### Build for Production

```bash
# Build Nuxt application
npm run build

# Preview production build
npm run preview
```

Output: `.output/` directory with optimized server and client bundles.

### Environment Variables

Create `.env` file:

```bash
# Database path
DATABASE_PATH=./.data/pacs.db

# DICOM storage path
DICOM_STORAGE_PATH=./.data/dicom

# Service ports
STORESCP_PORT=11112
QIDO_PORT=8042
WADO_PORT=8043

# CORS origins (production)
CORS_ORIGINS=https://viewer.hospital.com,https://pacs.hospital.com
```

### Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM node:20-alpine

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --production

# Copy application
COPY . .

# Build
RUN npm run build

# Expose ports
EXPOSE 3000 11112 8042 8043

# Create data directories
RUN mkdir -p .data/dicom

# Start server
CMD ["node", ".output/server/index.mjs"]
```

Build and run:

```bash
# Build image
docker build -t pacs-sqlite .

# Run container
docker run -p 3000:3000 -p 11112:11112 -p 8042:8042 -p 8043:8043 \
  -v $(pwd)/.data:/app/.data \
  pacs-sqlite
```

### Systemd Service

Create `/etc/systemd/system/pacs-sqlite.service`:

```ini
[Unit]
Description=PACS SQLite Server
After=network.target

[Service]
Type=simple
User=pacs
WorkingDirectory=/opt/pacs-sqlite
Environment=NODE_ENV=production
ExecStart=/usr/bin/node .output/server/index.mjs
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable pacs-sqlite
sudo systemctl start pacs-sqlite
sudo systemctl status pacs-sqlite
```

## Troubleshooting

### Port Already in Use

Change ports in server plugins if needed:

```typescript
// server/plugins/02.storescp.ts
const STORESCP_PORT = 11113;  // Change from 11112

// server/plugins/03.qido.ts
const QIDO_PORT = 8044;  // Change from 8042

// server/plugins/04.wado.ts
const WADO_PORT = 8045;  // Change from 8043
```

### Database Locked

SQLite database locked by another process:

```bash
# Stop all node processes
pkill -f node

# Remove lock file if exists
rm .data/pacs.db-wal .data/pacs.db-shm

# Restart
npm run dev
```

### DICOM Files Not Received

**Check StoreSCP is running:**
```bash
# Look for "StoreSCP Starting" in console
# Server should show: ✓ Listening on port 11112
```

**Test with dcmtk echoscu:**
```bash
# Install dcmtk
sudo apt-get install dcmtk  # Ubuntu/Debian
brew install dcmtk          # macOS

# Test echo
echoscu localhost 11112 -aec PACS_SQLITE
```

**Check AE Title matches:**
```bash
# Sender must use correct AE Title
storescu localhost 11112 -aec PACS_SQLITE file.dcm
```

### Viewer Not Loading Studies

**Check QIDO-RS responds:**
```bash
curl http://localhost:8042/studies | jq
# Should return JSON array of studies
```

**Check CORS headers:**
```bash
curl -H "Origin: http://localhost:3000" -v http://localhost:8042/studies
# Should include: Access-Control-Allow-Origin: http://localhost:3000
```

**Check browser console:**
- Open DevTools (F12)
- Look for CORS errors or network failures
- Check Network tab for failed requests

### Viewer Not Displaying Images

**Check WADO-RS responds:**
```bash
curl "http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}" \
  -o test.dcm
# Should download DICOM file
```

**Check file paths in database:**
```bash
sqlite3 .data/pacs.db "SELECT file_path FROM instances LIMIT 5;"
# Paths should exist and be readable
```

**Check Cornerstone console errors:**
- Open browser DevTools
- Look for image loading errors
- Check if WADO URLs are correct

### Ctrl+C Not Working

The refactored StoreSCP now uses proper shutdown signals. If server doesn't stop:

```bash
# Force kill (last resort)
pkill -9 -f "nuxt dev"

# Or find and kill process
ps aux | grep "nuxt dev"
kill -9 <PID>
```

### Module Not Found Errors

Reinstall dependencies:

```bash
rm -rf node_modules package-lock.json
npm install
```

## Performance

### Database Optimization

The SQLite schema includes indexes for fast queries:

```sql
-- Patient lookups
CREATE INDEX idx_patient_id ON studies(patient_id);

-- Date range queries
CREATE INDEX idx_study_date ON studies(study_date);

-- Hierarchical lookups
CREATE INDEX idx_series_study ON series(study_instance_uid);
CREATE INDEX idx_instance_series ON instances(series_instance_uid);
```

**WAL Mode** enabled for better concurrency:
```javascript
db.pragma('journal_mode = WAL');
```

### File Storage Organization

DICOM files stored in hierarchical structure for fast UID-based retrieval:

```
.data/dicom/
  └── {studyUID}/           # Study directory
      └── {seriesUID}/      # Series directory
          └── {instanceUID}.dcm  # Instance file
```

This allows direct file access without database lookup:
```javascript
const filePath = `${basePath}/${studyUID}/${seriesUID}/${instanceUID}.dcm`;
```

### Concurrency

All services handle concurrent operations:

- **StoreSCP**: Multiple simultaneous C-STORE associations
- **QIDO-RS**: Concurrent query requests
- **WADO-RS**: Concurrent instance retrievals
- **SQLite WAL**: Multiple readers, single writer

### Benchmarks

Typical performance on modern hardware:

- **StoreSCP Throughput**: 50-100 instances/second
- **QIDO-RS Query**: < 50ms for typical queries
- **WADO-RS Retrieval**: Limited by network/disk I/O
- **Database Insert**: ~1000 records/second with WAL mode

### Scaling Considerations

For large-scale deployments:

1. **Use PostgreSQL** instead of SQLite for > 10TB
2. **S3 storage** for DICOM files instead of filesystem
3. **Load balancer** for multiple QIDO/WADO instances
4. **Separate StoreSCP** instances for high ingestion rates

## API Reference

### QIDO-RS Endpoints

**Search Studies:**
```
GET /studies
GET /studies?PatientID={id}
GET /studies?PatientName={name}
GET /studies?StudyDate={date}
GET /studies?Modality={modality}
```

**Search Series:**
```
GET /studies/{studyUID}/series
GET /series
GET /series?Modality={modality}
```

**Search Instances:**
```
GET /studies/{studyUID}/series/{seriesUID}/instances
GET /instances
```

**Response Format:** DICOM JSON Model (array of objects)

### WADO-RS Endpoints

**Retrieve Study:**
```
GET /studies/{studyUID}
Accept: multipart/related; type=application/dicom
```

**Retrieve Series:**
```
GET /studies/{studyUID}/series/{seriesUID}
Accept: multipart/related; type=application/dicom
```

**Retrieve Instance:**
```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}
Accept: application/dicom
```

**Retrieve Metadata:**
```
GET /studies/{studyUID}/metadata
GET /studies/{studyUID}/series/{seriesUID}/metadata
Accept: application/dicom+json
```

**Response Format:** 
- DICOM files: `multipart/related` or `application/dicom`
- Metadata: `application/dicom+json`

### StoreSCP

**Protocol:** DICOM C-STORE (TCP)  
**Port:** 11112  
**AE Title:** PACS_SQLITE  
**Accepted SOP Classes:** All Storage SOP Classes  
**Accepted Transfer Syntaxes:** All (Implicit/Explicit VR, compressed)

**Events:**
- `onServerStarted` - Server listening
- `onConnection` - New association
- `onFileStored` - File received and stored
- `onStudyCompleted` - All files in study received
- `onError` - Error occurred

## Technology Details

### Nuxt 4 Features Used

- **File-based Routing** - Pages in `app/pages/` automatically routed
- **Auto-imports** - No need to import Vue/Nuxt composables
- **TypeScript** - Full type safety
- **Server Engine** - Nitro powers the backend
- **Hot Module Replacement** - Instant updates during development

### Nitro Server Features

- **Plugin Auto-loading** - Files in `server/plugins/` load in order
- **API Routes** - Files in `server/api/` become REST endpoints
- **Event Hooks** - `nitroApp.hooks.hook('close', ...)` for cleanup
- **Hot Reload** - Server restarts on file changes

### Cornerstone3D Integration

**Stack Viewport** for CT/MR slice viewing:
```typescript
const viewport = renderingEngine.getViewport(viewportId);
await viewport.setStack(imageIds, 0);
viewport.resetCamera();  // Maintain aspect ratio
```

**Tool Group** with synchronized tools:
```typescript
const toolGroup = ToolGroupManager.createToolGroup(toolGroupId);
toolGroup.addViewport(viewportId, renderingEngineId);
toolGroup.setToolActive(WindowLevelTool.toolName);
```

**DICOM Image Loader** with proper codec handling:
```typescript
import * as dicomImageLoader from '@cornerstonejs/dicom-image-loader';
import dicomParser from 'dicom-parser';
dicomImageLoader.external.cornerstone = cornerstoneCore;
dicomImageLoader.external.dicomParser = dicomParser;
```

### SQLite Better-SQLite3

**Synchronous API** for predictable transactions:
```typescript
const db = new Database('.data/pacs.db');
const stmt = db.prepare('INSERT INTO studies VALUES (?, ?, ?)');
stmt.run(studyUID, patientName, patientID);
```

**WAL Mode** for better concurrency:
```typescript
db.pragma('journal_mode = WAL');
```

**Transactions** for atomic operations:
```typescript
const insert = db.transaction((studies) => {
  for (const study of studies) {
    insertStmt.run(study);
  }
});
insert(studyArray);
```

## Learn More

### Documentation

- **[node-dicom-rs](../../README.md)** - Main library documentation
- **[StoreSCP Guide](../../docs/storescp.md)** - C-STORE receiver details
- **[Playground](../../playground/README.md)** - Simple demo examples

### External Resources

- **[Nuxt 4 Docs](https://nuxt.com)** - Full-stack Vue framework
- **[Nitro Docs](https://nitro.unjs.io)** - Server engine
- **[Cornerstone3D](https://www.cornerstonejs.org)** - Medical imaging
- **[Better-SQLite3](https://github.com/WiseLibs/better-sqlite3)** - SQLite driver
- **[DICOM Standard](https://www.dicomstandard.org)** - Official spec

## License

MIT - See [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions welcome! This example demonstrates integration patterns - feel free to adapt for your use case.

**Common Enhancements:**
- Add user authentication
- Implement DICOM modality worklist (MWL)
- Add study anonymization UI
- Integrate with HL7/FHIR systems
- Add automated QC checks
- Implement DICOM query/retrieve (C-FIND/C-MOVE)
- Add admin dashboard with statistics

## Credits

Built with:
- **[dicom-rs](https://github.com/Enet4/dicom-rs)** by Eduardo Pinho
- **[napi-rs](https://napi.rs)** - Rust ↔ Node.js bindings
- **[Nuxt](https://nuxt.com)** by Nuxt Team
- **[Cornerstone3D](https://www.cornerstonejs.org)** by OHIF Team
