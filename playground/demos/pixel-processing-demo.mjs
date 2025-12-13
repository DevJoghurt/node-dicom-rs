import { DicomFile } from '../../index.js';

/**
 * Demo: Advanced Pixel Data Processing
 * 
 * This demo showcases the new getProcessedPixelData() method which combines:
 * - Decompression
 * - Frame extraction
 * - Windowing (VOI LUT)
 * - 8-bit conversion
 * 
 * All in-memory, no file I/O!
 */

async function demonstratePixelProcessing() {
    const file = new DicomFile();
    
    try {
        // Open a DICOM file
        const testFile = './__test__/fixtures/test.dcm';
        await file.open(testFile);
        
        console.log('='.repeat(80));
        console.log('ADVANCED PIXEL DATA PROCESSING DEMO');
        console.log('='.repeat(80));
        
        // Get image information
        const info = file.getPixelDataInfo();
        console.log('\n📊 Image Information:');
        console.log(`   Dimensions: ${info.width} x ${info.height}`);
        console.log(`   Frames: ${info.frames}`);
        console.log(`   Bits: ${info.bitsAllocated}/${info.bitsStored}`);
        console.log(`   Compressed: ${info.isCompressed}`);
        console.log(`   Transfer Syntax: ${info.transferSyntaxUID}`);
        
        if (info.windowCenter && info.windowWidth) {
            console.log(`   Window: C=${info.windowCenter} W=${info.windowWidth}`);
        }
        
        if (info.rescaleSlope && info.rescaleIntercept) {
            console.log(`   Rescale: Slope=${info.rescaleSlope} Intercept=${info.rescaleIntercept}`);
        }
        
        console.log('\n' + '='.repeat(80));
        console.log('METHOD COMPARISON');
        console.log('='.repeat(80));
        
        // Method 1: Raw pixel data
        console.log('\n1️⃣  getPixelData() - Raw extraction (fastest)');
        const rawBuffer = file.getPixelData();
        console.log(`   ✓ Size: ${rawBuffer.length} bytes`);
        console.log(`   ✓ No decompression, no processing`);
        console.log(`   ✓ Use case: Custom processing pipelines`);
        
        // Method 2: Decoded pixel data
        if (info.isCompressed) {
            console.log('\n2️⃣  getDecodedPixelData() - Decompression only');
            const decodedBuffer = file.getDecodedPixelData();
            console.log(`   ✓ Compressed: ${info.dataSize} bytes`);
            console.log(`   ✓ Decompressed: ${decodedBuffer.length} bytes`);
            console.log(`   ✓ Compression ratio: ${(info.dataSize / decodedBuffer.length).toFixed(2)}x`);
            console.log(`   ✓ Use case: Access uncompressed pixel values`);
        } else {
            console.log('\n2️⃣  getDecodedPixelData() - Not needed (already uncompressed)');
        }
        
        // Method 3: Processed pixel data (NEW!)
        console.log('\n3️⃣  getProcessedPixelData() - Advanced processing (NEW!)');
        console.log('   ✨ Combines: decode + frame extract + window + 8-bit convert');
        
        // Example 3a: Apply windowing from file
        if (info.windowCenter && info.windowWidth) {
            console.log('\n   📺 Example 3a: Apply windowing from file');
            const windowedBuffer = file.getProcessedPixelData({
                applyVoiLut: true,
                convertTo8bit: true
            });
            console.log(`      ✓ Input: ${info.bitsAllocated}-bit`);
            console.log(`      ✓ Output: 8-bit (${windowedBuffer.length} bytes)`);
            console.log(`      ✓ Window: C=${info.windowCenter} W=${info.windowWidth}`);
            console.log(`      ✓ Ready for: Canvas, PNG, JPEG encoding`);
            
            // Calculate some stats on 8-bit data
            let min = 255, max = 0, sum = 0;
            for (let i = 0; i < windowedBuffer.length; i++) {
                const val = windowedBuffer[i];
                if (val < min) min = val;
                if (val > max) max = val;
                sum += val;
            }
            const mean = sum / windowedBuffer.length;
            console.log(`      ✓ 8-bit range: ${min}-${max} (mean: ${mean.toFixed(1)})`);
        }
        
        // Example 3b: Custom windowing
        console.log('\n   🎨 Example 3b: Custom windowing presets');
        
        if (info.rescaleSlope !== undefined) {
            const presets = [
                { name: 'Soft Tissue', center: 40, width: 400, emoji: '🫁' },
                { name: 'Lung', center: -600, width: 1500, emoji: '🫁' },
                { name: 'Bone', center: 300, width: 1500, emoji: '🦴' },
                { name: 'Brain', center: 40, width: 80, emoji: '🧠' }
            ];
            
            console.log('      Common CT windowing presets:');
            for (const preset of presets) {
                try {
                    const windowedBuffer = file.getProcessedPixelData({
                        windowCenter: preset.center,
                        windowWidth: preset.width,
                        convertTo8bit: true
                    });
                    
                    // Calculate contrast
                    let min = 255, max = 0;
                    for (let i = 0; i < Math.min(1000, windowedBuffer.length); i++) {
                        const val = windowedBuffer[i];
                        if (val < min) min = val;
                        if (val > max) max = val;
                    }
                    const contrast = max - min;
                    
                    console.log(`      ${preset.emoji} ${preset.name.padEnd(15)} C=${preset.center.toString().padStart(5)} W=${preset.width.toString().padStart(5)} → Contrast: ${contrast}`);
                } catch (error) {
                    console.log(`      ⚠️  ${preset.name}: ${error.message}`);
                }
            }
        } else {
            console.log('      ⚠️  No rescale parameters - windowing not applicable');
        }
        
        // Example 3c: Frame extraction (if multi-frame)
        if (info.frames > 1) {
            console.log('\n   🎞️  Example 3c: Frame extraction (multi-frame)');
            const middleFrame = Math.floor(info.frames / 2);
            const frameBuffer = file.getProcessedPixelData({
                frameNumber: middleFrame,
                convertTo8bit: true
            });
            console.log(`      ✓ Extracted frame ${middleFrame} of ${info.frames}`);
            console.log(`      ✓ Frame size: ${frameBuffer.length} bytes`);
            console.log(`      ✓ Use case: Cine loops, 3D volumes`);
        }
        
        // Example 3d: Complete pipeline
        console.log('\n   ⚙️  Example 3d: Complete processing pipeline');
        try {
            const processed = file.getProcessedPixelData({
                frameNumber: 0,
                windowCenter: 40,
                windowWidth: 400,
                convertTo8bit: true
            });
            console.log(`      ✓ Frame extraction: frame 0`);
            console.log(`      ✓ Windowing: C=40 W=400 (soft tissue)`);
            console.log(`      ✓ 8-bit conversion: ${processed.length} bytes`);
            console.log(`      ✓ Pipeline: decode → extract → window → convert`);
        } catch (error) {
            console.log(`      ⚠️  ${error.message}`);
        }
        
        console.log('\n' + '='.repeat(80));
        console.log('PERFORMANCE COMPARISON');
        console.log('='.repeat(80));
        
        // Benchmark different methods
        const iterations = 100;
        
        console.log(`\n⏱️  Running ${iterations} iterations of each method...\n`);
        
        // Benchmark 1: Raw
        let start = Date.now();
        for (let i = 0; i < iterations; i++) {
            file.getPixelData();
        }
        const rawTime = Date.now() - start;
        console.log(`   getPixelData():              ${rawTime}ms (${(rawTime/iterations).toFixed(2)}ms/call)`);
        
        // Benchmark 2: Decoded
        if (info.isCompressed) {
            start = Date.now();
            for (let i = 0; i < iterations; i++) {
                file.getDecodedPixelData();
            }
            const decodedTime = Date.now() - start;
            console.log(`   getDecodedPixelData():       ${decodedTime}ms (${(decodedTime/iterations).toFixed(2)}ms/call)`);
        }
        
        // Benchmark 3: Processed (simple)
        start = Date.now();
        for (let i = 0; i < iterations; i++) {
            file.getProcessedPixelData({ convertTo8bit: true });
        }
        const processedSimpleTime = Date.now() - start;
        console.log(`   getProcessedPixelData():     ${processedSimpleTime}ms (${(processedSimpleTime/iterations).toFixed(2)}ms/call)`);
        
        // Benchmark 4: Processed (with windowing)
        if (info.windowCenter && info.windowWidth) {
            start = Date.now();
            for (let i = 0; i < iterations; i++) {
                file.getProcessedPixelData({ 
                    applyVoiLut: true,
                    convertTo8bit: true 
                });
            }
            const processedWindowTime = Date.now() - start;
            console.log(`   getProcessedPixelData(+win): ${processedWindowTime}ms (${(processedWindowTime/iterations).toFixed(2)}ms/call)`);
        }
        
        console.log('\n' + '='.repeat(80));
        console.log('USE CASES');
        console.log('='.repeat(80));
        
        console.log('\n📱 Web DICOM Viewer:');
        console.log('   → Use getProcessedPixelData({ applyVoiLut: true, convertTo8bit: true })');
        console.log('   → Render directly to HTML Canvas');
        console.log('   → Fast window/level adjustments');
        
        console.log('\n🎬 Video/Animation Export:');
        console.log('   → Loop through frames with getProcessedPixelData({ frameNumber: i })');
        console.log('   → Encode each frame to PNG/JPEG');
        console.log('   → Create video with ffmpeg');
        
        console.log('\n🖼️  Image Format Conversion:');
        console.log('   → getProcessedPixelData({ convertTo8bit: true })');
        console.log('   → Save as PNG/JPEG with standard libraries');
        console.log('   → No need for complex DICOM-specific tools');
        
        console.log('\n🔬 Image Analysis:');
        console.log('   → getDecodedPixelData() for full precision');
        console.log('   → Perform calculations on 16-bit values');
        console.log('   → Apply custom algorithms');
        
        console.log('\n⚡ Real-time Processing:');
        console.log('   → Pre-generate multiple windowed versions');
        console.log('   → Fast switching between presets');
        console.log('   → No file I/O overhead');
        
        console.log('\n' + '='.repeat(80));
        console.log('✨ DEMO COMPLETE!');
        console.log('='.repeat(80));
        
    } catch (error) {
        console.error('\n❌ Error:', error.message);
    } finally {
        file.close();
    }
}

// Run the demo
demonstratePixelProcessing().catch(console.error);
