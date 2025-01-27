import { StoreScp } from '../index.js'
import { platform } from 'node:os'

const server = new StoreScp({
    port: 4446,
    outDir: './tmp/pacs',
    verbose: false
})

server.listen()

server.addEventListener('OnServerStarted',(error, event) => {
  console.log('OnServerStarted', event)
})

server.addEventListener('OnFileStored',(error, event) => {
  console.log('OnFileStored', JSON.parse(event.data))
})


console.log('DICOM server listening on port 4446');

async function exitHandler(evtOrExitCodeOrError) {
    console.log('EXIT HANDLER', evtOrExitCodeOrError);
    try {
      if(platform() !== 'win32') {
        await server.close();
      }
    } catch (e) {
      console.error('EXIT HANDLER ERROR', e);
    }
    console.log('EXIT HANDLER DONE');
    process.exit(isNaN(+evtOrExitCodeOrError) ? 1 : +evtOrExitCodeOrError);
}

[
    'beforeExit', 'uncaughtException', 'unhandledRejection',
    'SIGHUP', 'SIGINT', 'SIGQUIT', 'SIGILL', 'SIGTRAP',
    'SIGABRT','SIGBUS', 'SIGFPE', 'SIGUSR1', 'SIGSEGV',
    'SIGUSR2', 'SIGTERM',
].forEach(evt => process.on(evt, exitHandler));