import { StoreScp } from '../index.js'

const server = new StoreScp({
    port: 4445,
    outDir: './tmp/pacs',
    verbose: false
})

server.listen((event, msg)=>{
    console.log(event, msg.message)
})
