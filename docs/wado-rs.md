# WADO-RS Server

WADO-RS (Web Access to DICOM Objects - RESTful Services) is a DICOMweb service for retrieving DICOM studies, series, instances, frames, and metadata over HTTP. This implementation follows the [DICOM Part 18](http://dicom.nema.org/medical/dicom/current/output/html/part18.html) specification.

## Overview

The WADO-RS server provides RESTful HTTP endpoints for retrieving DICOM objects and metadata. It supports:

- Multiple storage backends (Filesystem, S3)
- Content negotiation (DICOM, DICOM JSON, DICOM XML)
- Pixel data transcoding
- Frame-level retrieval
- Metadata-only retrieval
- CORS support
- Configurable feature flags

## Configuration

```typescript
const config = {
    storageType: 'Filesystem' | 'S3',
    
    // Storage backend configuration
    basePath?: string,              // Required for Filesystem
    s3Config?: S3Config,           // Required for S3
    
    // Feature toggles
    enableMetadata?: boolean,      // Enable metadata endpoints (default: true)
    enableFrames?: boolean,        // Enable frame retrieval (default: true)
    enableRendered?: boolean,      // Enable rendered endpoints (default: false)
    enableThumbnail?: boolean,     // Enable thumbnail generation (default: false)
    enableBulkdata?: boolean,      // Enable bulkdata retrieval (default: false)
    
    // Transcoding options
    defaultTranscoding?: 'None' | 'JpegBaseline' | 'Jpeg2000' | 'Png' | 'Uncompressed',
    
    // HTTP features
    enableCors?: boolean,          // Enable CORS headers (default: false)
    enableCompression?: boolean,   // Enable gzip compression (default: false)
    
    // Performance
    maxConnections?: number,       // Maximum concurrent connections
    
    // Rendering
    thumbnailOptions?: {
        quality?: number,          // JPEG quality 1-100 (default: 90)
        width?: number,            // Output width in pixels
        height?: number,           // Output height in pixels
        windowCenter?: number,     // Grayscale window center
        windowWidth?: number,      // Grayscale window width
    },
    
    // Logging
    verbose?: boolean,             // Enable verbose logging (default: false)
};
```

## Storage Backends

### Filesystem Storage

Files must be organized as: `{basePath}/{studyUID}/{seriesUID}/{instanceUID}.dcm`

```javascript
const config = {
    storageType: 'Filesystem',
    basePath: '/path/to/dicom/storage',
    verbose: true
};

const server = new WadoServer(8043, config);
server.start();
```

### S3 Storage

Uses S3-compatible object storage (AWS S3, MinIO, etc.):

```javascript
const config = {
    storageType: 'S3',
    s3Config: {
        endpoint: 'https://s3.amazonaws.com',
        region: 'us-east-1',
        bucket: 'my-dicom-bucket',
        accessKeyId: 'YOUR_ACCESS_KEY',
        secretAccessKey: 'YOUR_SECRET_KEY'
    },
    verbose: true
};

const server = new WadoServer(8043, config);
server.start();
```

## DICOM Part 18 Endpoints

### Retrieve Study

Retrieve all instances in a study:

```
GET /studies/{studyUID}
Accept: application/dicom
```

Returns multipart/related response with all DICOM instances.

**Status:** ✅ Implemented (Filesystem & S3)

### Retrieve Series

Retrieve all instances in a series:

```
GET /studies/{studyUID}/series/{seriesUID}
Accept: application/dicom
```

Returns multipart/related response with all DICOM instances in the series.

**Status:** ✅ Implemented (Filesystem & S3)

### Retrieve Instance

Retrieve a single DICOM instance:

```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}
Accept: application/dicom
```

**Status:** ✅ Implemented

**Example:**

```javascript
const response = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9',
    { headers: { 'Accept': 'application/dicom' } }
);

const dicomBuffer = await response.arrayBuffer();
fs.writeFileSync('retrieved.dcm', Buffer.from(dicomBuffer));
```

### Retrieve Instance Metadata

Retrieve metadata for a specific instance as DICOM JSON:

```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/metadata
```

Returns DICOM JSON array with one element (the instance metadata).

**Status:** ✅ Implemented

**Example:**

```javascript
const response = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/metadata'
);

const metadata = await response.json();
console.log(metadata[0]['0020000D']); // Study Instance UID
```

### Retrieve Study/Series Metadata

Retrieve metadata for all instances in a study or series:

```
GET /studies/{studyUID}/metadata
GET /studies/{studyUID}/series/{seriesUID}/metadata
```

Returns DICOM JSON array with metadata for all instances.

**Status:** Not yet implemented

### Retrieve Frames

Retrieve specific frames from a multi-frame instance:

```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/frames/{frameList}
Accept: application/octet-stream
```

Frame list can be:
- Single frame: `1`
- Multiple frames: `1,3,5`
- Range: `1-10`
- Mixed: `1,3-5,7`

Returns multipart/related response with raw pixel data for requested frames.

**Status:** ✅ Implemented (Filesystem & S3)

### Retrieve Rendered

Retrieve rendered pixel data:

```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/rendered
Accept: image/jpeg
```

Query parameters:
- `viewport` - Output dimensions (e.g., `512,512`)
- `quality` - JPEG quality 1-100
- `window` - Window center/width (e.g., `40,400`)

**Status:** ✅ Implemented

## Image Rendering and Processing

The WADO-RS server includes comprehensive image rendering capabilities with automatic VOI LUT (Value of Interest Lookup Table) support, windowing, and format conversion.

### Automatic VOI LUT Support

When retrieving rendered images without manual windowing parameters, the server automatically applies VOI LUT from the DICOM file:

```javascript
// Automatic VOI LUT from file metadata
const response = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/rendered'
);

const imageBlob = await response.blob();
// Image rendered with WindowCenter/WindowWidth from DICOM file
```

**Automatic Processing:**
- Reads `WindowCenter` and `WindowWidth` from DICOM file
- Applies `RescaleIntercept` and `RescaleSlope` for Hounsfield units (CT)
- Converts to 8-bit display range (0-255)
- Returns optimally windowed image

### Manual Windowing

Override automatic VOI LUT with custom window parameters:

```javascript
// Soft tissue window (CT)
const softTissue = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?window=40,400'
);

// Lung window (CT)
const lung = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?window=-600,1500'
);

// Bone window (CT)
const bone = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?window=300,1500'
);
```

**Common CT Windowing Presets:**
- **Soft Tissue**: `window=40,400` - Optimal for abdomen, pelvis
- **Lung**: `window=-600,1500` - Optimal for chest/lungs
- **Bone**: `window=300,1500` - Optimal for skeletal structures
- **Brain**: `window=40,80` - Optimal for head CT

### Viewport Transformation

Resize images while maintaining aspect ratio:

```javascript
// Resize to 512x512 viewport
const resized = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?viewport=512,512'
);

// Combined: resize + custom windowing
const combined = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?viewport=512,512&window=40,400'
);
```

**Viewport Features:**
- Maintains original aspect ratio
- High-quality Lanczos3 resampling
- Supports any output dimensions

### Image Quality Control

Control JPEG compression quality:

```javascript
// High quality (90)
const highQuality = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?quality=90'
);

// Medium quality (75, smaller file)
const mediumQuality = await fetch(
    'http://localhost:8043/studies/.../instances/.../rendered?quality=75'
);
```

**Quality Guidelines:**
- `90-100`: Diagnostic quality, larger files
- `75-89`: Good quality, balanced size
- `60-74`: Lower quality, smaller files

### Format Support

Request different image formats via Accept header:

```javascript
// JPEG (default)
const jpeg = await fetch(url, {
    headers: { 'Accept': 'image/jpeg' }
});

// PNG (lossless)
const png = await fetch(url, {
    headers: { 'Accept': 'image/png' }
});
```

### Complete Rendering Example

```javascript
import { WadoServer } from 'node-dicom-rs';

const server = new WadoServer(8043, {
    storageType: 'Filesystem',
    basePath: './dicom-storage',
    enableRendered: true,
    verbose: true
});

server.start();

// Example 1: Automatic VOI LUT
const auto = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/rendered'
);
await saveImage(auto, 'automatic-windowing.jpg');

// Example 2: CT soft tissue window with resizing
const ct = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/rendered?window=40,400&viewport=512,512&quality=90'
);
await saveImage(ct, 'ct-soft-tissue.jpg');

// Example 3: PNG format with automatic windowing
const png = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/rendered',
    { headers: { 'Accept': 'image/png' } }
);
await saveImage(png, 'lossless.png');

async function saveImage(response, filename) {
    const buffer = await response.arrayBuffer();
    fs.writeFileSync(filename, Buffer.from(buffer));
}
```

### Retrieve Thumbnail

Retrieve thumbnail image:

```
GET /studies/{studyUID}/series/{seriesUID}/instances/{instanceUID}/thumbnail
Accept: image/jpeg
```

**Status:** Not yet implemented

## Content Negotiation

The server supports content negotiation via the `Accept` header:

### DICOM Format (application/dicom)

Default format. Returns raw DICOM files.

```javascript
fetch(url, {
    headers: { 'Accept': 'application/dicom' }
});
```

### DICOM JSON (application/dicom+json)

Returns metadata as DICOM JSON:

```javascript
fetch(url, {
    headers: { 'Accept': 'application/dicom+json' }
});
```

**Example response:**

```json
{
  "00080005": {
    "vr": "CS",
    "Value": ["ISO_IR 100"]
  },
  "00080020": {
    "vr": "DA",
    "Value": ["20240101"]
  },
  "00100010": {
    "vr": "PN",
    "Value": [{"Alphabetic": "Doe^John"}]
  }
}
```

### DICOM XML (application/dicom+xml)

Returns metadata as DICOM XML.

**Status:** Not yet implemented

## Feature Flags

Control which endpoints are enabled:

```javascript
const config = {
    storageType: 'Filesystem',
    basePath: '/dicom/storage',
    
    // Enable/disable specific features
    enableMetadata: true,      // Metadata endpoints
    enableFrames: false,       // Frame retrieval
    enableRendered: false,     // Rendered images
    enableThumbnail: false,    // Thumbnail generation
    enableBulkdata: false,     // Bulkdata retrieval
};
```

Disabled endpoints return `404 Not Found`.

## Complete Example

```javascript
import { WadoServer } from 'node-dicom-rs';

// Configure server
const config = {
    storageType: 'Filesystem',
    basePath: './dicom-storage',
    enableMetadata: true,
    enableFrames: true,
    enableCors: true,
    verbose: true
};

// Start server
const server = new WadoServer(8043, config);
server.start();

// Retrieve DICOM instance
const response = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9',
    {
        headers: {
            'Accept': 'application/dicom'
        }
    }
);

if (response.ok) {
    const buffer = await response.arrayBuffer();
    console.log(`Retrieved DICOM file: ${buffer.byteLength} bytes`);
}

// Retrieve metadata as DICOM JSON
const metadataResponse = await fetch(
    'http://localhost:8043/studies/1.2.3/series/4.5.6/instances/7.8.9/metadata'
);

const metadata = await metadataResponse.json();
console.log('Metadata:', metadata);

// Stop server
server.stop();
```

## Error Handling

### 404 Not Found

Returned when:
- Resource does not exist
- Feature is disabled via configuration

```json
{
  "error": "Not found"
}
```

### 501 Not Implemented

Returned for endpoints not yet implemented:

```json
{
  "error": "Study retrieval not yet implemented",
  "studyUID": "1.2.3",
  "note": "This requires scanning storage for all series/instances in the study"
}
```

### 400 Bad Request

Returned for invalid requests or configuration errors.

## Performance Considerations

### Thread Pool

WADO-RS file I/O uses the Node.js libuv thread pool (default: 4 threads). For high-throughput scenarios, increase the thread pool:

```bash
UV_THREADPOOL_SIZE=16 node server.js
```

### Storage Backend

- **Filesystem:** Fast for local storage, simple deployment
- **S3:** Scales horizontally, suitable for cloud deployments

### Caching

Consider implementing caching at the application level for frequently accessed instances.

### Load Balancing

WADO-RS is stateless and can be load balanced across multiple instances.

## Integration with Nuxt/Nitro

The WADO-RS server runs on a separate port but can be integrated with Nuxt/Nitro:

```javascript
// server/api/dicom/wado.ts
import { WadoServer } from 'node-dicom-rs';

let wadoServer: WadoServer | null = null;

export default defineEventHandler(async (event) => {
    // Start WADO server on first request
    if (!wadoServer) {
        wadoServer = new WadoServer(8043, {
            storageType: 'Filesystem',
            basePath: './storage/dicom'
        });
        wadoServer.start();
    }
    
    // Proxy requests to WADO server or return info
    return {
        wadoUrl: 'http://localhost:8043',
        status: 'running'
    };
});
```

## Architecture

The WADO-RS server architecture:

```
┌─────────────────┐
│   Node.js App   │
│  (Nuxt/Nitro)   │
└────────┬────────┘
         │
         │ Start/Stop
         ▼
┌─────────────────┐
│  WADO-RS Server │
│   (Rust/Warp)   │
└────────┬────────┘
         │
         ├─────────────┐
         │             │
         ▼             ▼
┌─────────────┐  ┌──────────┐
│ Filesystem  │  │    S3    │
│   Storage   │  │ Storage  │
└─────────────┘  └──────────┘
```

**Key Points:**
- Warp HTTP server runs on Tokio thread pool (separate from Node.js event loop)
- File I/O uses libuv thread pool (shared with Node.js)
- DICOM parsing happens in Rust (high performance)
- Minimal overhead for serving files

## Roadmap

- [x] Basic instance retrieval
- [x] Instance metadata retrieval
- [x] Content negotiation
- [x] Filesystem storage backend
- [ ] Study/series retrieval with multipart responses
- [ ] Frame extraction and retrieval
- [ ] Pixel data transcoding
- [ ] Rendered/thumbnail endpoints
- [ ] S3 storage backend
- [ ] Compression support (gzip)
- [ ] Proper DICOM JSON serialization
- [ ] Bulkdata retrieval
- [ ] Query parameter support
- [ ] ETag/caching support

## References

- [DICOM Part 18 - Web Services](http://dicom.nema.org/medical/dicom/current/output/html/part18.html)
- [DICOMweb Standard](https://www.dicomstandard.org/using/dicomweb)
- [WADO-RS Specification](http://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_10.4)
