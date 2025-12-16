#!/usr/bin/env node

/**
 * Query QIDO-RS server
 * 
 * Usage:
 *   node scripts/query-studies.mjs
 */

async function queryStudies() {
  const url = 'http://localhost:8042/dicomweb/studies';
  
  console.log('Querying studies from QIDO-RS...');
  console.log(`URL: ${url}`);
  console.log('');
  
  try {
    const response = await fetch(url);
    const studies = await response.json();
    
    console.log(`Found ${studies.length} studies:\n`);
    
    for (const study of studies) {
      const patientName = study['00100010']?.Value?.[0]?.Alphabetic || 'Unknown';
      const patientID = study['00100020']?.Value?.[0] || 'Unknown';
      const studyDate = study['00080020']?.Value?.[0] || '';
      const studyDesc = study['00081030']?.Value?.[0] || '';
      const modalities = study['00080061']?.Value?.[0] || '';
      const numSeries = study['00201206']?.Value?.[0] || '0';
      const numInstances = study['00201208']?.Value?.[0] || '0';
      
      console.log(`Patient: ${patientName} (${patientID})`);
      console.log(`  Date: ${studyDate}`);
      console.log(`  Description: ${studyDesc}`);
      console.log(`  Modalities: ${modalities}`);
      console.log(`  Series: ${numSeries}, Instances: ${numInstances}`);
      console.log('');
    }
  } catch (error) {
    console.error('Error querying QIDO-RS:', error.message);
    console.error('Make sure the server is running (npm run dev)');
    process.exit(1);
  }
}

queryStudies();
