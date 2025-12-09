#!/bin/bash
mkdir -p ./testdata
cd ./testdata
wget -O 1.3.6.1.4.1.9328.50.2.128367.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.128367"
wget -O 1.3.6.1.4.1.9328.50.2.126606.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.126606"
wget -O 1.3.6.1.4.1.9328.50.2.155237.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.155237"
wget -O 1.3.6.1.4.1.9328.50.2.160465.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160465"
wget -O 1.3.6.1.4.1.9328.50.2.125354.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.125354"
wget -O 1.3.6.1.4.1.9328.50.2.160111.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160111"
wget -O 1.3.6.1.4.1.9328.50.2.160730.zip "https://services.cancerimagingarchive.net/nbia-api/services/v1/getImage?SeriesInstanceUID=1.3.6.1.4.1.9328.50.2.160730"
for f in *.zip; do unzip "$f" -d "${f%.zip}"; done && rm *.zip
