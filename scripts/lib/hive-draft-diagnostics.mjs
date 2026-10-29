// A source-qualified adapter for the maintained shared runner, not a second orchestrator.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {lex} from '../self-translation/lexer.mjs';
const environmentPattern=lex('setEnv: { ...buildGitIdentityEnv(), ...buildRunContextEnv(env)', 'JavaScript');
export function adaptDraftDiagnostics(source) {
 assert.equal(typeof source,'string');
 assert(Buffer.byteLength(source)<=262144,'shared runner source budget');
 // Lexical offsets preserve every comment and whitespace character. Ignore a
 // shebang without moving offsets; a comment/string cannot impersonate code.
 const input=source.replace(/^#![^\n]*/u,line=>' '.repeat(line.length));
 const tokens=lex(input,'JavaScript').filter(token=>token.type!=='comment');
 const calls=[];
 for(let i=0;i<tokens.length-2;i++)if(tokens[i].type==='ident'&&tokens[i].text==='buildDockerArgv'
  &&tokens[i+1].text==='('&&tokens[i+2].text==='{')calls.push(i+2);
 assert.equal(calls.length,1,'one shared Docker invocation required');
 let depth=0;const matches=[];
 for(let i=calls[0];i<tokens.length;i++){
  if(depth===1&&environmentPattern.every((token,n)=>token.text===tokens[i+n]?.text&&token.type===tokens[i+n]?.type)){
   let end=i+environmentPattern.length;
   const trailing=tokens[end]?.text===',';
   if(trailing)end++;
   if(tokens[end]?.text==='}')matches.push({offset:tokens[end].start,trailing});
  }
  if(tokens[i].text==='{')depth++;
  if(tokens[i].text==='}'&&--depth===0)break;
 }
 assert.equal(matches.length,1,'shared diagnostic use site changed');
 const {offset,trailing}=matches[0];
 const insertion=(trailing?' ':', ')
  +' FORMAL_AI_LOG_DIR: containerLogDir + "/formal-ai-diagnostics",'
  +' FORMAL_AI_DIALOG_LOG_DIR: containerLogDir + "/formal-ai-dialog",'
  +' FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR: containerLogDir + "/formal-ai-native",'
  +' FORMAL_AI_LOG_LEVEL: "trace", FORMAL_AI_TRACE_REQUESTS: "true" ';
 return source.slice(0,offset)+insertion+source.slice(offset);
}
const gitHead=hive=>execFileSync('git',['-C',hive,'rev-parse','HEAD'],{encoding:'utf8',timeout:15000,maxBuffer:4096}).trim();
export function prepareDraftDiagnostics({hive,logDirectory,upstreamCommit=process.env.HIVE_MIND_SOURCE_SHA,readHead=gitHead}) {
 assert.match(upstreamCommit??'',/^[a-f0-9]{40}$/u,'resolved Hive source identity required');
 assert.equal(readHead(hive),upstreamCommit,'Hive checkout differs from the resolved attempt source');
 const file=path.join(hive,'scripts/formal-ai-draft.mjs'),source=fs.readFileSync(file,'utf8');
 const adapted=adaptDraftDiagnostics(source);
 fs.mkdirSync(logDirectory,{recursive:true});
 const report={schema:'hive-draft-diagnostic-adapter/v1',upstreamCommit,
  path:'scripts/formal-ai-draft.mjs',originalSha256:createHash('sha256').update(source).digest('hex'),
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
