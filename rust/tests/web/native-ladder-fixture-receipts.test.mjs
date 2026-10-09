// Execute the shell mock declared by the native fixture, without compiling Rust.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
const repository=fileURLToPath(new URL('../../../',import.meta.url));
const fixture=readFileSync(join(repository,'rust/tests/unit/ci-cd/desktop_release_resolve.rs'),'utf8');
function functionSource(name){
 const start=fixture.indexOf('fn '+name+'(');assert.ok(start>=0);
 const next=fixture.indexOf('\nfn ',start+1),test=fixture.indexOf('\n#[test]',start+1);
 return fixture.slice(start,Math.min(...[next,test].filter(index=>index>=0)));
}
function quoted(source){return [...source.matchAll(/"([^"\n]+)"/g)].map(match=>match[1]);}
function names(version){
 const assets=quoted(functionSource('expected_asset_names')).filter(value=>/\.(dmg|zip|exe|AppImage|deb|tar\.gz|yml|vsix|txt)$/.test(value));
 const cli=quoted(functionSource('cli_archive_names')).filter(value=>value.startsWith('formal-ai-cli-'));
 const evidence=quoted(functionSource('native_evidence_names'));
 const targets=evidence.filter(value=>/^[a-z0-9_]+-[a-z0-9_-]+$/.test(value));
 const records=evidence.filter(value=>value.endsWith('.json')).flatMap(value=>value.includes('{target}')?targets.map(target=>value.replace('{target}',target)):[value]);
 return [...assets,...cli,...records].map(value=>value.replaceAll('{version}',version));
}
function replay(context,assets){
 const root=mkdtempSync(join(tmpdir(),'formal-ai-native-fixture-'));
 context.after(()=>rmSync(root,{recursive:true,force:true}));
 const bin=join(root,'bin');mkdirSync(bin);
 const match=fixture.match(/let mock = r#"([\s\S]*?)"#;/);assert.ok(match,'actual native shell fixture');
 const executable=join(bin,'gh');writeFileSync(executable,match[1]);chmodSync(executable,0o755);
 const output=join(root,'output');writeFileSync(output,'');
 const result=spawnSync('/bin/bash',[join(repository,'scripts/desktop-release-resolve.sh')],{encoding:'utf8',timeout:30000,
  env:{...process.env,PATH:bin+':'+process.env.PATH,GITHUB_OUTPUT:output,REPO:'link-assistant/formal-ai',GH_TOKEN:'fixture-token',EVENT:'workflow_run',
   WORKFLOW_RUN_HEAD_SHA:'a'.repeat(40),MOCK_PARENT_SHA:'a'.repeat(40),MOCK_TAGS_JQ_OUTPUT:'',MOCK_LATEST_TAG:'v0.201.0',MOCK_RELEASE_EXISTS:'1',
   MOCK_ASSET_NAMES:assets.join('\n'),MOCK_EVIDENCE_CREATOR:join(repository,'rust/tests/fixtures/native-release-evidence/observations.mjs')}});
 return {...result,outputs:Object.fromEntries(readFileSync(output,'utf8').trim().split('\n').filter(Boolean).map(line=>line.split('=')))};
}
test('the native pipefail membership fixture executes its exact mock and source-bound assets',(context)=>{
 const declared=names('0.201.0');assert.equal(new Set(declared).size,declared.length);
 const padding=Array.from({length:3000},(_,index)=>'formal-ai-desktop-padding-'+String(index).padStart(4,'0'));
 const result=replay(context,[...declared,...padding]);
 assert.equal(result.status,0,result.stdout+'\n'+result.stderr);assert.equal(result.outputs.tag,'v0.201.0');assert.equal(result.outputs.should_build,'false',result.stdout+'\n'+result.stderr);
});
test('the same native mock really requests healing when a declared CLI archive is absent',(context)=>{
 const missing='formal-ai-cli-x86_64-pc-windows-msvc.zip';
 const result=replay(context,names('0.201.0').filter(name=>name!==missing));
 assert.equal(result.status,0,result.stderr);assert.equal(result.outputs.should_build,'true');assert.ok(result.stdout.includes(missing));
});
test('the packaging source pin observes executable YAML rather than historical comments',()=>{
 const workflow=readFileSync(join(repository,'.github/workflows/desktop-release.yml'),'utf8');
 const block=workflow.match(/^  build:[\s\S]*?(?=^  [a-z][a-z-]*:|$(?![\s\S]))/m)?.[0];assert.ok(block);
 const executable=block.split('\n').filter(line=>!line.trimStart().startsWith('#')).join('\n');
 assert.equal(executable.includes('cargo build'),false);
 assert.ok(executable.includes('$FORMAL_AI_NATIVE_PROTOCOL_DIR/native-release-artifact.mjs" verify'));
 assert.equal(executable.includes('uses: actions/cache@'),false);
});
