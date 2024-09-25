import { StoreScp } from '../index.js'

const server = new StoreScp({
    port: 4446,
    outDir: './tmp/pacs',
    verbose: false
})

server.listen((event, msg)=>{
    console.log(event, msg.message)
})

console.log('DICOM server listening on port 4446');

async function exitHandler(evtOrExitCodeOrError) {
    console.log('EXIT HANDLER', evtOrExitCodeOrError);
    try {
      // await async code here
      // Optionally: Handle evtOrExitCodeOrError here
    } catch (e) {
      console.error('EXIT HANDLER ERROR', e);
    }

    process.exit(isNaN(+evtOrExitCodeOrError) ? 1 : +evtOrExitCodeOrError);
}

[
    'beforeExit', 'uncaughtException', 'unhandledRejection',
    'SIGHUP', 'SIGINT', 'SIGQUIT', 'SIGILL', 'SIGTRAP',
    'SIGABRT','SIGBUS', 'SIGFPE', 'SIGUSR1', 'SIGSEGV',
    'SIGUSR2', 'SIGTERM',
].forEach(evt => process.on(evt, exitHandler));