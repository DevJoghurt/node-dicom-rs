import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4242',
    verbose: true
});

sender.addFile('./__test__/fixtures/study/00000001.dcm');


const result = sender.send();

console.log(result);