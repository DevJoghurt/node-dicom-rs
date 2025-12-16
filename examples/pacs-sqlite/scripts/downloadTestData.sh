#!/bin/bash

# Download test DICOM files from public datasets
# This script downloads sample medical imaging data for testing

set -e

TESTDATA_DIR="./playground/testdata"

echo "Downloading test DICOM files..."
echo "================================="

# Create directory
#!/bin/bash
mkdir -p ./testdata

# Download from Medical Connections DICOM test files
# These are freely available test images
echo "Downloading sample CT series..."

# You can use any publicly available DICOM test files
# For example, from: https://www.rubomedical.com/dicom_files/
# Or from TCIA: https://www.cancerimagingarchive.net/

cd ./testdata
wget -O 1.3.6.1.4.1.9328.50.2.128367.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.128367"
wget -O 1.3.6.1.4.1.9328.50.2.126606.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.126606"
wget -O 1.3.6.1.4.1.9328.50.2.155237.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.155237"
wget -O 1.3.6.1.4.1.9328.50.2.160465.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160465"
wget -O 1.3.6.1.4.1.9328.50.2.125354.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.125354"
wget -O 1.3.6.1.4.1.9328.50.2.160111.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160111"
wget -O 1.3.6.1.4.1.9328.50.2.160730.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160730"
for f in *.zip; do unzip "$f" -d "${f%.zip}"; done && rm *.zip

echo ""
echo "Note: Please download DICOM test files from:"
echo "  - https://www.rubomedical.com/dicom_files/"
echo "  - https://www.cancerimagingarchive.net/"
echo "  - Medical Connections test files"
echo ""
echo "Place them in: $TESTDATA_DIR"
echo ""
echo "Expected structure:"
echo "  $TESTDATA_DIR/"
echo "    ├── study1/"
echo "    │   ├── series1/"
echo "    │   │   ├── image001.dcm"
echo "    │   │   └── image002.dcm"
echo "    │   └── series2/"
echo "    └── study2/"
echo ""
