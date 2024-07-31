import { StoreScp } from '../index.js'

const server = new StoreScp(4445, './tmp')

server.listen((event, msg)=>{
    console.log(event, msg)
})
