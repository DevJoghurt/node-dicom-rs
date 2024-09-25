import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4446',
    verbose: true
});

sender.addFile('./__test__/fixtures/test.dcm');


const result = sender.send();

console.log(result);