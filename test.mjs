import { sum, dicomDump, StoreScp } from './index.js'
 
//dicomDump("./tmp/10DFA8F4.dcm");

const server = new StoreScp(4445)

server.listen((event, msg)=>{
    console.log(event, msg)
})
