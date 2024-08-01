import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4445',
    verbose: true
});

sender.addFile('./tmp/8B1FA77C.dcm');
sender.addFile('./tmp/6AD34A72.dcm');


const result = sender.send();

console.log(result)
