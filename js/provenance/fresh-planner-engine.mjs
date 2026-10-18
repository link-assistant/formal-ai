// The one actual fixed-profile planner bootstrap, shared by capture and fresh replay.
import path from 'node:path';
import {pathToFileURL}from 'node:url';
export async function bootFreshPlanner(configuration){
const load=relative=>import(pathToFileURL(path.join(configuration.repository,relative)).href);
const {WorkerHost}=await load('js/server/worker-host.mjs');
const {installNodeHost}=await load('js/agentic/node-host.mjs');
const {planChatStep}=await load('js/agentic/planner.mjs');
const {drive}=await load('experiments/js_dogfood/drive.mjs');
const {withDriveCapture,activeDriveCapture}=await load('js/provenance/drive-capture.mjs');
const {collectRuntimeObservation}=await load('js/provenance/fresh-runtime.mjs');
const {blake3Hex}=await load('js/crypto/blake3.mjs');
const {openMemoryCustody}=await load('js/server/provenance-memory.mjs');
const {setLearnedImporter}=await load('js/server/meta-learned.mjs');
const {packageVersion}=await load('js/server/seed.mjs');
const {host}=await load('js/agentic/host.mjs');
// This explicit JS-engine profile cannot execute the Formal AI WASM product.
const worker=new WorkerHost({WebAssembly:{
 compile(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiate(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiateStreaming(){throw Error('ProductWasmDisabledByRuntimeProfile');}
}},{provenance:true});
await installNodeHost(worker);
setLearnedImporter(statements=>worker.run('metaImportLearned(__learned)',{__learned:statements}));
return{worker,planChatStep,drive,withDriveCapture,activeDriveCapture,collectRuntimeObservation,blake3Hex,openMemoryCustody,packageVersion,host};
}
