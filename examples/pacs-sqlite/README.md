# PACS SQLite Example

Complete PACS (Picture Archiving and Communication System) implementation using:

- **StoreSCP** - Receive DICOM files via C-STORE
- **QIDO-RS** - Query DICOM metadata
- **WADO-RS** - Retrieve DICOM files
- **SQLite** - Local database for metadata storage
- **OHIF Viewer** - Web-based DICOM viewer
- **Nitro v3** - Modern server framework with auto-imports and hot reload

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         PACS Server                         │
│                       (Nitro v3/Vite)                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   StoreSCP   │  │   QIDO-RS    │  │   WADO-RS    │    │
│  │  Port: 11112 │  │  Port: 8042  │  │  Port: 8043  │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                 │                  │             │
│         │        ┌────────┴────────┐         │             │
│         └────────►  SQLite Database ◄─────────┘             │
│                  │  (pacs.db)      │                       │
│                  └─────────────────┘                       │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              OHIF Viewer (Web UI)                    │  │
│  │              http://localhost:3000                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Features

### 🏥 Complete DICOM Workflow

1. **Receive**: DICOM modalities send studies via C-STORE to StoreSCP
2. **Store**: Files saved to filesystem, metadata stored in SQLite
3. **Anonymize**: Automatic fake patient data generation on receive
4. **Query**: OHIF/clients search studies via QIDO-RS
5. **Retrieve**: OHIF/clients fetch DICOM files via WADO-RS
6. **View**: OHIF Viewer displays studies in web browser

### 📦 Technology Stack

- **Node.js** - Runtime environment
- **Vite** - Build tool with hot reload
- **Nitro v3** - Server framework with auto-imports
- **SQLite** - Embedded database (no external services needed)
- **Better-SQLite3** - Synchronous SQLite driver
- **OHIF Viewer v3** - Medical imaging viewer

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Download Test Data

```bash
npm run download-testdata
```

This downloads sample DICOM files to `./testdata`

### 3. Start PACS Server

```bash
npm run dev
```

This starts:
- **StoreSCP** on port 11112 (DICOM C-STORE)
- **QIDO-RS** on port 8042 (Query API)
- **WADO-RS** on port 8043 (Retrieval API)
- **OHIF Viewer** on http://localhost:3000
- **SQLite Database** at `./data/pacs.db`

### 4. Send DICOM Files

Send test DICOM files to the PACS:

```bash
# Send single file
storescu localhost 11112 -aec PACS_SQLITE ./testdata/study1/series1/image1.dcm

# Send entire directory
storescu localhost 11112 -aec PACS_SQLITE ./testdata/*
```

Or use the DicomFile API:

```bash
node scripts/send-test-files.mjs
```

### 5. Open OHIF Viewer

Navigate to http://localhost:3000 to view studies

## Project Structure

```
pacs-sqlite/
├── package.json              # Dependencies and scripts
├── vite.config.js           # Vite/Nitro configuration
├── README.md                # This file
│
├── server/                  # Nitro server directory
│   ├── plugins/            # Server plugins (auto-loaded)
│   │   ├── 01.database.js      # SQLite database initialization
│   │   ├── 02.storescp.js      # StoreSCP C-STORE receiver
│   │   ├── 03.qido.js          # QIDO-RS query service
│   │   └── 04.wado.js          # WADO-RS retrieval service
│   │
│   └── routes/             # API routes
│       └── api/
│           └── viewer/
│               └── studies.get.js  # OHIF study list endpoint
│
├── public/                 # Static files (OHIF Viewer)
│   └── ohif/              # OHIF Viewer build
│       ├── index.html
│       └── app-config.js  # OHIF configuration
│
├── data/                  # Runtime data (git-ignored)
│   ├── pacs.db           # SQLite database
│   └── dicom/            # DICOM file storage
│       └── {studyUID}/
│           └── {seriesUID}/
│               └── {instanceUID}.dcm
│
├── testdata/             # Downloaded test DICOM files
│
└── scripts/              # Utility scripts
    ├── downloadTestData.sh    # Download sample DICOM files
    └── send-test-files.mjs    # Send test files to StoreSCP
```

## Configuration

### Database Schema

The SQLite database automatically creates these tables:

**studies**
- study_instance_uid (PRIMARY KEY)
- patient_name
- patient_id
- patient_birth_date
- patient_sex
- study_date
- study_time
- study_description
- accession_number
- modalities_in_study
- number_of_series
- number_of_instances

**series**
- series_instance_uid (PRIMARY KEY)
- study_instance_uid (FOREIGN KEY)
- modality
- series_number
- series_description
- number_of_instances

**instances**
- sop_instance_uid (PRIMARY KEY)
- series_instance_uid (FOREIGN KEY)
- sop_class_uid
- instance_number
- file_path

### StoreSCP Configuration

**AE Title**: `PACS_SQLITE`  
**Port**: `11112`  
**Storage**: `./data/dicom/{studyUID}/{seriesUID}/{instanceUID}.dcm`

On file receive:
1. Generates fake patient data using consistent seeding
2. Updates DICOM tags with fake data
3. Saves file to organized directory structure
4. Stores metadata in SQLite database

### QIDO-RS Configuration

**Port**: `8042`  
**CORS**: Enabled for localhost  
**Endpoints**:
- `GET /studies` - Search for studies
- `GET /studies/{uid}/series` - Search for series
- `GET /studies/{uid}/instances` - Search for instances
- `GET /studies/{uid}/series/{uid}/instances` - Search for series instances

### WADO-RS Configuration

**Port**: `8043`  
**Storage**: Filesystem (`./data/dicom`)  
**CORS**: Enabled for localhost  
**Endpoints**:
- `GET /studies/{uid}/series/{uid}/instances/{uid}` - Retrieve instance
- `GET /studies/{uid}/series/{uid}/instances/{uid}/metadata` - Retrieve metadata

## Fake Patient Data Generation

To protect privacy, incoming DICOM files are automatically anonymized with consistent fake data:

```javascript
// Generated fake data (consistent per patient ID)
{
  patientName: 'Smith^John',
  patientId: 'PAT12345',
  patientBirthDate: '19750315',
  patientSex: 'M',
  patientAddress: '1234 Main St, New York, 10001'
}
```

The fake data is:
- **Consistent**: Same patient ID always gets same fake data
- **Realistic**: Names, dates, addresses look authentic
- **Seeded**: Uses SHA-256 hash of patient ID as seed
- **Diverse**: Multiple name/location combinations

## Development

### Hot Reload

Vite provides instant hot reload for server changes:

```bash
npm run dev
```

Edit any file in `server/` and changes apply immediately.

### Database Inspection

View the SQLite database:

```bash
# Install sqlite3 CLI
npm install -g sqlite3

# Open database
sqlite3 data/pacs.db

# Query studies
SELECT * FROM studies;

# Query series
SELECT * FROM series;

# Query instances
SELECT * FROM instances;
```

### Testing QIDO-RS

```bash
# Search all studies
curl http://localhost:8042/studies | jq

# Search by patient
curl "http://localhost:8042/studies?PatientID=PAT12345" | jq

# Get series in study
curl http://localhost:8042/studies/{studyUID}/series | jq
```

### Testing WADO-RS

```bash
# Retrieve DICOM instance
curl http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID} \
  -o retrieved.dcm

# Get metadata
curl http://localhost:8043/studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/metadata \
  | jq
```

## OHIF Viewer Integration

### Configuration

OHIF is configured via `public/ohif/app-config.js`:

```javascript
window.config = {
  dataSources: [{
    namespace: '@ohif/extension-default.dataSourcesModule.dicomweb',
    sourceName: 'dicomweb',
    configuration: {
      friendlyName: 'SQLite PACS',
      name: 'PACS_SQLITE',
      wadoUriRoot: 'http://localhost:8043',
      qidoRoot: 'http://localhost:8042',
      wadoRoot: 'http://localhost:8043'
    }
  }]
};
```

### Viewing Studies

1. Open http://localhost:3000
2. Click "Study List"
3. Studies from SQLite database appear
4. Click study to open in viewer
5. Use OHIF tools to manipulate images

## Production Deployment

### Build for Production

```bash
npm run build
```

This creates:
- Optimized server bundle in `.output/`
- Ready for deployment to Node.js hosting

### Environment Variables

```bash
# Database path
DATABASE_PATH=./data/pacs.db

# DICOM storage path
DICOM_STORAGE_PATH=./data/dicom

# Ports
STORESCP_PORT=11112
QIDO_PORT=8042
WADO_PORT=8043

# CORS origins (production)
CORS_ORIGINS=https://viewer.hospital.com
```

### Docker Deployment

```dockerfile
FROM node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --production

COPY . .
RUN npm run build

EXPOSE 3000 11112 8042 8043

CMD ["node", ".output/server/index.mjs"]
```

## Troubleshooting

### Port Already in Use

If ports are already occupied:

```javascript
// server/plugins/02.storescp.js
const STORESCP_PORT = 11113; // Change port

// server/plugins/03.qido.js
const QIDO_PORT = 8044; // Change port

// server/plugins/04.wado.js
const WADO_PORT = 8045; // Change port
```

### Database Locked

SQLite database is locked by another process:

```bash
# Stop all node processes
pkill -f node

# Restart
npm run dev
```

### DICOM Files Not Received

Check StoreSCP is running:

```bash
# Test with echo
echoscu localhost 11112 -aec PACS_SQLITE
```

Check AE Title matches:

```bash
# Send with correct AE Title
storescu localhost 11112 -aec PACS_SQLITE file.dcm
```

### OHIF Not Loading Studies

Check QIDO-RS responds:

```bash
curl http://localhost:8042/studies | jq
```

Check CORS headers:

```bash
curl -H "Origin: http://localhost:3000" \
     -v http://localhost:8042/studies
```

## Performance

### Database Indexes

The SQLite schema includes indexes on:
- `studies.patient_id`
- `studies.study_date`
- `series.study_instance_uid`
- `instances.series_instance_uid`

### File Storage

DICOM files stored in organized structure:
```
data/dicom/
  └── {studyUID}/
      └── {seriesUID}/
          └── {instanceUID}.dcm
```

This allows fast retrieval by UID without database lookup.

### Concurrency

- StoreSCP handles multiple simultaneous C-STORE operations
- QIDO-RS handles concurrent queries
- WADO-RS handles concurrent retrievals
- SQLite uses WAL mode for better concurrency

## License

MIT

## See Also

- [Node DICOM RS Documentation](../../README.md)
- [StoreSCP Documentation](../../docs/storescp.md)
- [QIDO-RS Documentation](../../docs/qido-rs.md)
- [WADO-RS Documentation](../../docs/wado-rs.md)
- [OHIF Viewer](https://ohif.org)
- [Nitro Documentation](https://nitro.unjs.io)
