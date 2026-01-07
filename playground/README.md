# Playground - Demo Examples

Simple, focused demos for each node-dicom-rs service.

## Setup

Download test DICOM files first:

```bash
./downloadTestData.sh
```

This downloads sample DICOM CT scan data to `./testdata/`.

## Demos

### 1. DicomFile - Read and manipulate DICOM files

```bash
node dicomfile-demo.mjs
```

Demonstrates:
- Opening DICOM files
- Extracting tags
- Getting pixel data info
- Processing pixels for display
- Updating tags (anonymization)
- Saving modified files

### 2. StoreScp - Receive DICOM files

```bash
node storescp-demo.mjs
```

Demonstrates:
- Starting C-STORE SCP server
- Receiving DICOM files
- Extracting metadata
- Study completion detection

Keep this running while sending files with StoreScu.

### 3. StoreScu - Send DICOM files

```bash
node storescu-demo.mjs
```

Demonstrates:
- Sending DICOM files via C-STORE
- Progress tracking
- Error handling
- Transfer completion

Requires StoreScp demo to be running first.

### 4. DICOMweb - Query and retrieve servers

```bash
node dicomweb-demo.mjs
```

Demonstrates:
- QIDO-RS query server (port 8042)
- WADO-RS retrieval server (port 8043)
- RESTful DICOM access

Test with curl:
```bash
# Query all studies
curl http://localhost:8042/studies

# Retrieve a study
curl http://localhost:8043/studies/1.3.6.1.4.1.9328.50.2.125354
```

## File Structure

```
playground/
├── README.md                  # This file
├── downloadTestData.sh        # Download sample DICOM data
├── dicomfile-demo.mjs        # DicomFile demo
├── storescp-demo.mjs         # StoreScp demo  
├── storescu-demo.mjs         # StoreScu demo
├── dicomweb-demo.mjs         # QIDO-RS + WADO-RS demo
├── testdata/                 # Downloaded test DICOM files
└── test-received/            # Files received by StoreScp
```

## Tips

- Run `downloadTestData.sh` first to get sample data
- StoreScu requires StoreScp to be running
- DICOMweb servers read from `testdata/` directory
- All demos use minimal configuration for clarity
- Check each demo file's header comments for prerequisites
