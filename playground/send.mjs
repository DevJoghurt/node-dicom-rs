import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4445'
});

sender.addFile('./tmp/8B1FA77C.dcm');

sender.send();