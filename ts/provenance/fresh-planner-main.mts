// Fixed fresh-process entrypoint; it accepts requests, never executable source.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const configuration=JSON.parse(fs.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG,'utf8'));
const executable=configuration.runtimeExecutable;
if(!executable||executable.path!==fs.realpathSync(process.execPath)||executable.version!==process.version
 ||executable.platform!==process.platform||executable.architecture!==process.arch)throw Error('FreshRuntimeExecutableBinding');
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
// This explicit JS-engine profile cannot execute the Formal AI WASM product.
const worker=new WorkerHost({WebAssembly:{
 compile(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiate(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiateStreaming(){throw Error('ProductWasmDisabledByRuntimeProfile');}
}},{provenance:true});
await installNodeHost(worker);
setLearnedImporter(statements=>worker.run('metaImportLearned(__learned)',{__learned:statements}));
const memoryFactory=configuration.memoryFile?({sessionId:sessionIdentifier,workspace})=>openMemoryCustody({
 file:configuration.memoryFile,workspace,sessionId:sessionIdentifier,taskURL:configuration.taskURL}):undefined;
const initial={...collectRuntimeObservation(configuration.transportDirectory),runtimeExecutable:executable};
const sourceIdentity=blake3Hex(Buffer.from(JSON.stringify(initial.sources.map(source=>({url:source.url,blake3:source.blake3})))));
const result=await withDriveCapture({...configuration.provenance,...(memoryFactory?{memoryFactory}:{})},configuration.workspace,configuration.prompt,sourceIdentity,async()=>{
 await worker.run('0');
 activeDriveCapture().runtimeObservation(initial);
 const observed=await drive(planChatStep,configuration.workspace,configuration.prompt,{steps:configuration.steps});
 activeDriveCapture().runtimeObservation({...collectRuntimeObservation(configuration.transportDirectory),runtimeExecutable:executable});
 return observed;
});
fs.writeFileSync(configuration.resultFile,JSON.stringify({result,engineProfile:'javascript-product-wasm-disabled',
 environment:process.env,workingDirectory:process.cwd(),runtime:process.version,
 fullReplay:{verified:false,reason:'PinnedFreshReplayNotYetPerformed'}}),{flag:'wx',mode:0o600});
