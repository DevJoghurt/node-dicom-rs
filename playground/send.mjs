import { StoreScu } from '../index.js'


const sender = new StoreScu({
    addr: '127.0.0.1:4446',
    verbose: false
});

sender.addFolder('./__test__/fixtures/study');


const result = await sender.send();

console.log(result);