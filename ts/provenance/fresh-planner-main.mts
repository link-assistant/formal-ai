// Fixed fresh-process entrypoint; it accepts requests, never executable source.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const configuration=JSON.parse(fs.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG,'utf8'));
const load=relative=>import(pathToFileURL(path.join(configuration.repository,relative)).href);
const {WorkerHost}=await load('js/server/worker-host.mjs');
const {installNodeHost}=await load('js/agentic/node-host.mjs');
const {planChatStep}=await load('js/agentic/planner.mjs');
const {drive}=await load('experiments/js_dogfood/drive.mjs');
const {withDriveCapture,activeDriveCapture}=await load('js/provenance/drive-capture.mjs');
const {collectRuntimeObservation}=await load('js/provenance/fresh-runtime.mjs');
const {blake3Hex}=await load('js/crypto/blake3.mjs');
// This explicit JS-engine profile cannot execute the Formal AI WASM product.
const worker=new WorkerHost({WebAssembly:{
 compile(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiate(){throw Error('ProductWasmDisabledByRuntimeProfile');},
 instantiateStreaming(){throw Error('ProductWasmDisabledByRuntimeProfile');}
}},{provenance:true});
await installNodeHost(worker);
const initial=collectRuntimeObservation(configuration.transportDirectory);
const sourceIdentity=blake3Hex(Buffer.from(JSON.stringify(initial.sources.map(source=>({url:source.url,blake3:source.blake3})))));
const result=await withDriveCapture(configuration.provenance,configuration.workspace,configuration.prompt,sourceIdentity,async()=>{
 activeDriveCapture().runtimeObservation(initial);
 const observed=await drive(planChatStep,configuration.workspace,configuration.prompt,{steps:configuration.steps});
 activeDriveCapture().runtimeObservation(collectRuntimeObservation(configuration.transportDirectory));
 return observed;
});
fs.writeFileSync(configuration.resultFile,JSON.stringify({result,engineProfile:'javascript-product-wasm-disabled',
 environment:process.env,workingDirectory:process.cwd(),runtime:process.version,
 fullReplay:{verified:false,reason:'PinnedFreshReplayNotYetPerformed'}}),{flag:'wx',mode:0o600});
