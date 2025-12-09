import { StoreScu } from '../index.js'

const scu = new StoreScu({
    addr: 'localhost:4446',
    verbose: false,
    concurrency: 4,
    s3Config: {
        accessKey: 'user',
        secretKey: 'password',
        bucket: 'dicom',
        endpoint: 'http://localhost:7070'
    },
})

// Add event listeners for transfer monitoring
scu.addEventListener('OnTransferStarted', (error, event) => {
    const data = JSON.parse(event.data)
    console.log('\n🚀 Transfer Started:', {
        address: data.address,
        callingAeTitle: data.callingAeTitle,
        totalFiles: data.totalFiles
    })
    console.log('─'.repeat(60))
})

scu.addEventListener('OnFileSending', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`📤 Sending: ${data.file}`)
    console.log(`   SOP Instance UID: ${data.sopInstanceUid}`)
    console.log(`   Transfer Syntax: ${data.transferSyntax}`)
})

scu.addEventListener('OnFileSent', (error, event) => {
    const data = JSON.parse(event.data)
    console.log(`✅ Sent: ${data.file}`)
    console.log(`   Duration: ${data.durationSeconds.toFixed(3)}s (${data.durationMs}ms)`)
    console.log(`   Status: ${data.status}`)
    console.log('─'.repeat(60))
})

scu.addEventListener('OnFileError', (error, event) => {
    const data = JSON.parse(event.data)
    console.error(`❌ Error: ${data.file}`)
    console.error(`   ${data.error}`)
    console.error(`   Status Code: ${data.statusCode}`)
    console.error('─'.repeat(60))
})

scu.addEventListener('OnTransferCompleted', (error, event) => {
    const data = JSON.parse(event.data)
    console.log('\n✨ Transfer Completed')
    console.log(`   Total Files: ${data.totalFiles}`)
    console.log(`   Status: ${data.status}`)
    console.log('═'.repeat(60))
})

scu.addEventListener('OnError', (error, event) => {
    console.error('\n🔥 General Error:', event.message)
    if (event.data) {
        console.error('   Details:', event.data)
    }
})

// Add files from S3
scu.addFolder('./1.3.6.1.4.1.9328.50.2.126368')

console.log('Starting DICOM file transfer...')
console.log('═'.repeat(60))

// Send files
scu.send().then((result) => {
    console.log('\n🎉 All operations completed successfully!')
}).catch((err) => {
    console.error('\n💥 Transfer failed:', err)
    process.exit(1)
})
