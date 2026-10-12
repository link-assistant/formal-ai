// Real Git/source and byte checks over a shell stand-in; no native compilation is claimed.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdirSync,writeFileSync,chmodSync,readFileSync} from 'node:fs';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {sealSource} from '../../../../scripts/native-release-source.mjs';
import {expectedIdentity,validateExecutableReceipt,verifyExecutableReceipt,NATIVE_TARGETS,RECEIPT_FILE} from '../../../../scripts/native-release-artifact.mjs';
const scripts=fileURLToPath(new URL('../../../../scripts/',import.meta.url));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
export function cliArchiveFixture(directory,target) {
 const git=(...arguments_)=>execFileSync('git',arguments_,{cwd:directory,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
 mkdirSync(join(directory,'rust/src'),{recursive:true});
 writeFileSync(join(directory,'rust/Cargo.toml'),'[package]\nname="formal-ai"\nversion="9.9.9"\n');
 writeFileSync(join(directory,'rust/Cargo.lock'),'version = 4\n');
 writeFileSync(join(directory,'rust/src/main.rs'),'fn main() {}\n');
 writeFileSync(join(directory,'LICENSE'),'license text\n');writeFileSync(join(directory,'README.md'),'# formal-ai\n');
 git('init','--initial-branch=main');git('config','user.name','CLI archive fixture');git('config','user.email','fixture@example.invalid');git('config','core.autocrlf','false');
 git('add','rust','LICENSE','README.md');git('commit','-m','declare actual fixture source');git('tag','v9.9.9');
 const sourceDirectory=join(directory,'native-source');mkdirSync(sourceDirectory);
 const compiler='rustc 1.91.0\ncommit-hash: '+'c'.repeat(40)+'\nhost: '+NATIVE_TARGETS[target].host+'\nrelease: 1.91.0\n';
 const metadata={workspace_members:['fixture'],packages:[{id:'fixture',name:'formal-ai',version:'9.9.9',features:{default:[]},targets:[{name:'formal-ai',kind:['bin']}]}]};
 const selected=sealSource(directory,sourceDirectory,{head:git('rev-parse','HEAD'),tag:'v9.9.9',run:'1181',metadata,compiler});
 const identity=expectedIdentity(selected.record,target,selected.sha256,'1181');
 const binaryDirectory=join(directory,'native-bin');mkdirSync(binaryDirectory);
 const name=identity.platform==='win32'?'formal-ai.exe':'formal-ai';
 const binary=join(binaryDirectory,name);
 writeFileSync(binary,'#!/bin/sh\ncase "$1" in\n --version) echo "formal-ai 9.9.9" ;;\n *) echo \'{"choices":[{"message":{"content":"4"}}]}\' ;;\nesac\n');chmodSync(binary,0o755);
 const bytes=readFileSync(binary);
 // Compiler/producer fields are typed canned observations, independently marked as fixtures.
 const receipt=validateExecutableReceipt({version:1,identity,producer:{job:'cli_fixture',attempt:'1',compiler,environment:{RUSTFLAGS:'-Dwarnings'}},executable:{name,bytes:bytes.length,sha256:hash(bytes)}},identity);
 writeFileSync(join(binaryDirectory,RECEIPT_FILE),JSON.stringify(receipt,null,2)+'\n');
 const verified=verifyExecutableReceipt(binaryDirectory,identity);assert.equal(verified.binary,binary);
 return {binary,receipt,source:selected.record,environment:{...process.env,GITHUB_RUN_ID:'1181',NATIVE_SELECTION_SHA256:selected.sha256,FORMAL_AI_NATIVE_PROTOCOL_DIR:scripts,FORMAL_AI_VERIFIED_NATIVE_BINARY:verified.binary}};
}
