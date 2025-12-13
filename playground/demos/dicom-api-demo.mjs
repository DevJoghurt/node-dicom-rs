import { DicomFile } from '../../index.js';

console.log('=== DicomFile API Demo ===\n');

(async () => {
    const file = new DicomFile();
    
    try {
        // Open a DICOM file
        console.log('1. Opening DICOM file...');
        await file.open('./__test__/fixtures/test.dcm');
        console.log('   ✓ File opened successfully\n');
        
        // Extract metadata
        console.log('2. Extracting metadata...');
        const data = file.extract(['PatientName', 'Modality', 'Rows', 'Columns']);
        console.log('   Patient:', data.PatientName);
        console.log('   Modality:', data.Modality);
        console.log('   Image size:', data.Rows, 'x', data.Columns);
        console.log('   ✓ Metadata extracted\n');
        
        // Get DICOM as JSON (new method!)
        console.log('3. Getting DICOM as JSON...');
        const json = file.toJson(false); // compact format
        console.log('   JSON length:', json.length, 'bytes');
        console.log('   First 80 chars:', json.substring(0, 80) + '...');
        
        // Parse and access a tag
        const jsonObj = JSON.parse(json);
        const patientNameTag = jsonObj['00100010'];
        console.log('   Patient Name from JSON:', patientNameTag?.Value?.[0]?.Alphabetic);
        console.log('   ✓ JSON conversion successful\n');
        
        // Get pixel data info
        console.log('4. Getting pixel data info...');
        const pixelInfo = file.getPixelDataInfo();
        console.log('   Dimensions:', pixelInfo.width, 'x', pixelInfo.height);
        console.log('   Frames:', pixelInfo.frames);
        console.log('   Bits Allocated:', pixelInfo.bitsAllocated);
        console.log('   Bits Stored:', pixelInfo.bitsStored);
        console.log('   Samples per Pixel:', pixelInfo.samplesPerPixel);
        console.log('   Photometric:', pixelInfo.photometricInterpretation);
        console.log('   Compressed:', pixelInfo.isCompressed);
        console.log('   Transfer Syntax:', pixelInfo.transferSyntaxUID);
        console.log('   Data size:', pixelInfo.dataSize, 'bytes');
        console.log('   ✓ Pixel info retrieved\n');
        
        // Get raw pixel data (new method!)
        console.log('5. Getting raw pixel data as Buffer...');
        const pixelBuffer = file.getPixelData();
        console.log('   Buffer length:', pixelBuffer.length, 'bytes');
        console.log('   Buffer type:', pixelBuffer.constructor.name);
        console.log('   First 16 bytes:', Array.from(pixelBuffer.slice(0, 16)));
        console.log('   ✓ Pixel data retrieved as Buffer\n');
        
        // Demonstrate the difference between methods
        console.log('6. Comparing return vs save methods...');
        console.log('   toJson() returns:', typeof file.toJson(), '- No file I/O!');
        console.log('   getPixelData() returns:', file.getPixelData().constructor.name, '- No file I/O!');
        console.log('   saveAsJson() saves to:', 'file or S3 - Async operation');
        console.log('   saveRawPixelData() saves to:', 'file - Async operation');
        console.log('   ✓ Methods compared\n');
        
        // Calculate some statistics
        console.log('7. Calculating pixel data statistics...');
        const pixels = new Uint8Array(pixelBuffer);
        let min = 255, max = 0, sum = 0;
        for (let i = 0; i < pixels.length; i++) {
            const val = pixels[i];
            if (val < min) min = val;
            if (val > max) max = val;
            sum += val;
        }
        const mean = sum / pixels.length;
        console.log('   Min pixel value:', min);
        console.log('   Max pixel value:', max);
        console.log('   Mean pixel value:', mean.toFixed(2));
        console.log('   ✓ Statistics calculated\n');
        
        console.log('=== Demo Complete ===');
        console.log('\nNew methods demonstrated:');
        console.log('  • toJson() - Get DICOM as JSON string without file I/O');
        console.log('  • getPixelData() - Get raw pixel data as Buffer');
        console.log('  • getDecodedPixelData() - Get decoded pixel data (transcode feature)');
        console.log('\nExisting methods:');
        console.log('  • extract() - Extract specific tags (returns object directly)');
        console.log('  • getPixelDataInfo() - Get pixel metadata');
        console.log('  • saveAsJson() - Save as JSON file');
        console.log('  • saveRawPixelData() - Save pixel data to file');
        
    } catch (error) {
        console.error('Error:', error.message);
    } finally {
        file.close();
        console.log('\n✓ File closed');
    }
})();
