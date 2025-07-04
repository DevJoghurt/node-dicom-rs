import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4446',
    verbose: true
});

sender.addFolder('./__test__/fixtures/study');
//sender.addFile('./__test__/fixtures/test.dcm');

const result = await sender.send();

console.log(result);