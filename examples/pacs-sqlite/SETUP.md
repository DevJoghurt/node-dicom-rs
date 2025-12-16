# PACS SQLite Example - Complete Implementation

Complete PACS system using Vite, Nitro v3, SQLite, and node-dicom-rs.

## Features

- 📥 **StoreSCP**: Receive DICOM files via C-STORE (port 11112)
- 🔍 **QIDO-RS**: Query studies/series/instances (port 8042)
- 📤 **WADO-RS**: Retrieve DICOM files (port 8043)
- 💾 **SQLite**: Fast metadata storage
- 🔐 **Anonymization**: Auto-generate fake patient data
- 🌐 **Web UI**: Simple status dashboard (port 3000)

## Quick Start

```bash
# 1. Install dependencies
npm install

# 2. Start the server (runs all services)
npm run dev

# 3. In another terminal, send test files
node scripts/send-test-files.mjs ../playground/testdata

# 4. Query studies
node scripts/query-studies.mjs

# 5. Open browser
open http://localhost:3000
```

## Project Structure

```
examples/pacs-sqlite/
├── server/
│   └── plugins/          # Auto-loaded Nitro plugins
│       ├── 01.database.js    # SQLite initialization
│       ├── 02.storescp.js    # DICOM C-STORE receiver
│       ├── 03.qido.js        # QIDO-RS query service
│       └── 04.wado.js        # WADO-RS retrieval service
├── scripts/
│   ├── downloadTestData.sh   # Download test DICOM files
│   ├── send-test-files.mjs   # Send files to StoreSCP
│   ├── query-studies.mjs     # Query QIDO-RS
│   └── inspect-db.mjs        # Inspect SQLite database
├── public/
│   ├── index.html            # Status dashboard
│   └── app-config.js         # OHIF Viewer config
├── data/                 # Created automatically
│   ├── pacs.db              # SQLite database
│   └── dicom/               # DICOM file storage
├── package.json
├── vite.config.js
└── README.md
```

## Services

### StoreSCP (Port 11112)

Receives DICOM files and:
- Anonymizes with fake patient data (consistent seeding)
- Saves files to `data/dicom/{studyUID}/{seriesUID}/{instanceUID}.dcm`
- Stores metadata in SQLite

**Configuration:**
- Port: 11112
- AE Title: PACS_SQLITE
- Storage: Filesystem

**Send files:**
```bash
node scripts/send-test-files.mjs <directory>
```

### QIDO-RS (Port 8042)

Query service with four endpoints:

1. **Search Studies**
   ```
   GET http://localhost:8042/dicomweb/studies
   GET http://localhost:8042/dicomweb/studies?PatientID=12345
   GET http://localhost:8042/dicomweb/studies?StudyDate=20240101
   ```

2. **Search Series**
   ```
   GET http://localhost:8042/dicomweb/studies/{studyUID}/series
   ```

3. **Search Study Instances**
   ```
   GET http://localhost:8042/dicomweb/studies/{studyUID}/instances
   ```

4. **Search Series Instances**
   ```
   GET http://localhost:8042/dicomweb/studies/{studyUID}/series/{seriesUID}/instances
   ```

### WADO-RS (Port 8043)

Retrieval service:

**Retrieve Instance:**
```
GET http://localhost:8043/dicomweb/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}
```

**Retrieve Metadata:**
```
GET http://localhost:8043/dicomweb/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/metadata
```

**Retrieve Rendered:**
```
GET http://localhost:8043/dicomweb/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/rendered
```

## Database Schema

SQLite database at `data/pacs.db`:

```sql
-- Studies table
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
  number_of_instances INTEGER,
  created_at DATETIME,
  updated_at DATETIME
);

-- Series table
CREATE TABLE series (
  series_instance_uid TEXT PRIMARY KEY,
  study_instance_uid TEXT,
  modality TEXT,
  series_number TEXT,
  series_description TEXT,
  number_of_instances INTEGER,
  created_at DATETIME
);

-- Instances table
CREATE TABLE instances (
  sop_instance_uid TEXT PRIMARY KEY,
  series_instance_uid TEXT,
  study_instance_uid TEXT,
  sop_class_uid TEXT,
  instance_number TEXT,
  file_path TEXT,
  rows INTEGER,
  columns INTEGER,
  bits_allocated INTEGER,
  created_at DATETIME
);
```

**Inspect database:**
```bash
node scripts/inspect-db.mjs
```

## Fake Patient Data Generation

The StoreSCP plugin generates consistent fake data using SHA-256 seeding:

```javascript
// Original: Patient ID "12345"
// Generated consistently:
{
  patientName: "Smith^John",
  patientID: "12345",
  patientBirthDate: "19550315",
  patientSex: "M"
}
```

This ensures:
- Same patient ID always gets same fake data
- Different patient IDs get different fake data
- Data is consistent across multiple sends

## Scripts

### Send Test Files
```bash
# Send all DICOM files from a directory
node scripts/send-test-files.mjs ./path/to/dicom/files

# Example
node scripts/send-test-files.mjs ../playground/testdata
```

### Query Studies
```bash
# Query all studies from QIDO-RS
node scripts/query-studies.mjs
```

### Inspect Database
```bash
# View database statistics and recent studies
node scripts/inspect-db.mjs
```

## Development

### Hot Reload

Vite provides hot reload for all changes:

```bash
npm run dev
```

Changes to server plugins are automatically reloaded.

### Database Inspection

Use SQLite CLI:
```bash
sqlite3 data/pacs.db

# Example queries
SELECT COUNT(*) FROM studies;
SELECT patient_name, study_date, study_description FROM studies;
```

### Testing QIDO-RS

```bash
# Search all studies
curl http://localhost:8042/dicomweb/studies

# Search by patient ID
curl "http://localhost:8042/dicomweb/studies?PatientID=12345"

# Get series for a study
curl "http://localhost:8042/dicomweb/studies/1.2.3.4.5/series"
```

### Testing WADO-RS

```bash
# Retrieve DICOM file
curl -o test.dcm "http://localhost:8043/dicomweb/studies/{study}/series/{series}/instances/{instance}"

# Get metadata
curl "http://localhost:8043/dicomweb/studies/{study}/series/{series}/instances/{instance}/metadata"
```

## OHIF Viewer Integration

Configure OHIF Viewer to use local endpoints:

1. **Install OHIF Viewer** (example using Docker):
   ```bash
   docker run -d -p 3001:80 \
     -v $(pwd)/public/app-config.js:/usr/share/nginx/html/app-config.js \
     ohif/viewer:latest
   ```

2. **Configuration** (`public/app-config.js`):
   ```javascript
   window.config = {
     dataSources: [{
       name: 'local-pacs',
       wadoUriRoot: 'http://localhost:8043/dicomweb',
       qidoRoot: 'http://localhost:8042/dicomweb',
       wadoRoot: 'http://localhost:8043/dicomweb'
     }]
   };
   ```

3. **Open viewer**:
   ```
   http://localhost:3001
   ```

## Production Deployment

### Docker Example

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

# Start
CMD ["npm", "run", "preview"]
```

### Environment Variables

```bash
# Ports
STORESCP_PORT=11112
QIDO_PORT=8042
WADO_PORT=8043
WEB_PORT=3000

# Storage
DICOM_STORAGE_PATH=/data/dicom
DATABASE_PATH=/data/pacs.db

# Security
ENABLE_CORS=true
ALLOWED_ORIGINS=http://localhost:3000
```

### Docker Compose

```yaml
version: '3.8'

services:
  pacs:
    build: .
    ports:
      - "3000:3000"
      - "11112:11112"
      - "8042:8042"
      - "8043:8043"
    volumes:
      - ./data:/app/data
    environment:
      - NODE_ENV=production
```

## Troubleshooting

### Port Already in Use

If ports are already in use, modify the ports in the plugin files:

- StoreSCP: `server/plugins/02.storescp.js` (line 13)
- QIDO-RS: `server/plugins/03.qido.js` (line 11)
- WADO-RS: `server/plugins/04.wado.js` (line 9)

### Database Locked

If you get "database is locked" errors:

```bash
# Close all connections and restart
pkill -f "vite"
rm -f data/pacs.db-shm data/pacs.db-wal
npm run dev
```

### DICOM Files Not Received

Check StoreSCP logs:
```bash
# Should see:
[StoreSCP] ✓ Listening on port 11112 (AET: PACS_SQLITE)
[StoreSCP] ✓ Anonymized: PATIENT001 → 12345
[StoreSCP] ✓ Stored: Smith^John - CT Chest
```

Test connectivity:
```bash
# Use echoscu to test connection
echoscu -v localhost 11112 -aec PACS_SQLITE
```

### QIDO-RS Returns Empty

Check database:
```bash
node scripts/inspect-db.mjs
```

Verify files were stored:
```bash
ls -R data/dicom/
```

### CORS Errors

CORS is enabled for `http://localhost:3000` by default.

To allow other origins, modify:
- `server/plugins/03.qido.js` (line 17)
- `server/plugins/04.wado.js` (line 17)

## Performance

### Database Indexes

The database includes indexes on commonly queried fields:
- `patient_id`
- `study_date`
- `study_instance_uid`
- `series_instance_uid`

### File Storage

Files are organized by hierarchy:
```
data/dicom/
  {studyUID}/
    {seriesUID}/
      {instanceUID}.dcm
```

### Concurrency

- SQLite uses WAL mode for better concurrency
- Each service runs on its own port
- Nitro handles multiple concurrent requests

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Vite Dev Server                      │
│                    (Hot Reload)                          │
│                      Port 3000                           │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼────┐        ┌─────▼─────┐      ┌─────▼─────┐
   │ StoreSCP│        │  QIDO-RS  │      │  WADO-RS  │
   │  11112  │        │   8042    │      │   8043    │
   └────┬────┘        └─────┬─────┘      └─────┬─────┘
        │                   │                   │
        │              ┌────▼────────────────┐  │
        │              │   SQLite Database   │  │
        │              │   (Metadata)        │  │
        │              └─────────────────────┘  │
        │                                       │
        └────────►  Filesystem  ◄──────────────┘
                   (DICOM Files)
```

## License

Same as parent project (node-dicom-rs).

## Support

For issues or questions:
- Check the [main documentation](../../docs/)
- Review logs in the console
- Use the scripts for debugging

## Next Steps

1. ✅ Install dependencies: `npm install`
2. ✅ Start server: `npm run dev`
3. ✅ Send test files: `node scripts/send-test-files.mjs`
4. ✅ Query studies: `node scripts/query-studies.mjs`
5. 🔜 Integrate OHIF Viewer
6. 🔜 Add authentication
7. 🔜 Deploy to production
