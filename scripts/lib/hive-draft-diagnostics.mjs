// A source-qualified adapter for the maintained shared runner, not a second orchestrator.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
export const HIVE_DRAFT_PIN='2083d6fd86ba0a7ee54e36110c8aebcfb5f2774d';
export const HIVE_DRAFT_RUNNER_SHA256='b1b14aa853542644c5f9a852dbc28f20c75881d47506e0e4ca7bf3fbc62898dd';
export function adaptDraftDiagnostics(source) {
 assert.equal(createHash('sha256').update(source).digest('hex'),HIVE_DRAFT_RUNNER_SHA256,'unreviewed shared draft runner');
 const before='setEnv: { ...buildGitIdentityEnv(), ...buildRunContextEnv(env) },';
 assert.equal(source.split(before).length,2,'shared diagnostic use site changed');
 return source.replace(before,'setEnv: { ...buildGitIdentityEnv(), ...buildRunContextEnv(env),'
  +' FORMAL_AI_LOG_DIR: containerLogDir + "/formal-ai-diagnostics",'
  +' FORMAL_AI_DIALOG_LOG_DIR: containerLogDir + "/formal-ai-dialog",'
  +' FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR: containerLogDir + "/formal-ai-native",'
  +' FORMAL_AI_LOG_LEVEL: "trace", FORMAL_AI_TRACE_REQUESTS: "true" },');
}
export function prepareDraftDiagnostics({hive,logDirectory}) {
 const file=path.join(hive,'scripts/formal-ai-draft.mjs'),source=fs.readFileSync(file,'utf8');
 const adapted=adaptDraftDiagnostics(source);
 fs.mkdirSync(logDirectory,{recursive:true});
 const report={schema:'hive-draft-diagnostic-adapter/v1',upstreamCommit:HIVE_DRAFT_PIN,
  path:'scripts/formal-ai-draft.mjs',originalSha256:HIVE_DRAFT_RUNNER_SHA256,
  adaptedSha256:createHash('sha256').update(adapted).digest('hex'),
  containerStorage:'/home/box/logs',sharedSolveCommandChanged:false,taskChanged:false,
  authorshipCredit:0,fullPayloads:'dialog and native files; diagnostic metadata can contain previews'};
 fs.writeFileSync(path.join(logDirectory,'draft-diagnostics-adapter.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});
 fs.writeFileSync(file,adapted);
 return report;
}
if(process.argv[1]&&fs.realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
 assert.equal(process.argv.length,4);
 console.log(JSON.stringify(prepareDraftDiagnostics({hive:process.argv[2],logDirectory:process.argv[3]})));
}
