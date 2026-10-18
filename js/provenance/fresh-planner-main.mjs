// Fixed fresh-process entrypoint; it accepts requests, never executable source.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const configuration=JSON.parse(fs.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG,'utf8'));
const executable=configuration.runtimeExecutable;
if(!executable||executable.path!==fs.realpathSync(process.execPath)||executable.version!==process.version
 ||executable.platform!==process.platform||executable.architecture!==process.arch)throw Error('FreshRuntimeExecutableBinding');
const {bootFreshPlanner}=await import(pathToFileURL(path.join(configuration.repository,'js/provenance/fresh-planner-engine.mjs')).href);
const {worker,planChatStep,drive,withDriveCapture,activeDriveCapture,collectRuntimeObservation,blake3Hex,openMemoryCustody,packageVersion,host}=await bootFreshPlanner(configuration);
const memoryFactory=configuration.memoryFile?({sessionId:sessionIdentifier,workspace})=>openMemoryCustody({
 file:configuration.memoryFile,workspace,sessionId:sessionIdentifier,taskURL:configuration.taskURL}):undefined;
const initial={...collectRuntimeObservation(configuration.transportDirectory),runtimeExecutable:executable};
const sourceIdentity=blake3Hex(Buffer.from(JSON.stringify(initial.sources.map(source=>({url:source.url,blake3:source.blake3})))));
const result=configuration.replayInput?await (await import(pathToFileURL(path.join(configuration.repository,
 'js/provenance/fresh-recompute.mjs')).href)).recomputeFreshRun(configuration,{worker,planChatStep,openMemoryCustody,host,packageVersion}):await withDriveCapture({...configuration.provenance,...(memoryFactory?{memoryFactory}:{})},configuration.workspace,configuration.prompt,sourceIdentity,async()=>{
 await worker.run('0');
 activeDriveCapture().runtimeContext({schema:'formal-ai-fresh-runtime-context-v1',
  productVersion:packageVersion(),runtime:process.version,observedAt:new Date().toISOString(),
  workingDirectory:process.cwd(),environment:{...process.env},prerequisiteGrant:host().prerequisiteGrant?.()??null,
  bootstrapArtifacts:configuration.bootstrapArtifacts,engineProfile:'javascript-product-wasm-disabled',
  request:{prompt:configuration.prompt,steps:configuration.steps}});
 activeDriveCapture().runtimeObservation(initial);
 const observed=await drive(planChatStep,configuration.workspace,configuration.prompt,{steps:configuration.steps});
 activeDriveCapture().runtimeObservation({...collectRuntimeObservation(configuration.transportDirectory),runtimeExecutable:executable});
 return observed;
});
fs.writeFileSync(configuration.resultFile,JSON.stringify({result,sourceIdentity,engineProfile:'javascript-product-wasm-disabled',
 environment:process.env,workingDirectory:process.cwd(),runtime:process.version,
 fullReplay:{verified:false,reason:'PinnedFreshReplayNotYetPerformed'}}),{flag:'wx',mode:0o600});
