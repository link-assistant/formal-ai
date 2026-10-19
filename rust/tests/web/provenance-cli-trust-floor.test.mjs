import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {provenancePrimitives} from '../../../js/provenance/session.mjs';
import {encodeTyped,decodeTyped} from '../../../js/logging/typed-codec.mjs';
const cli=new URL('../../../js/provenance/cli.mjs',import.meta.url).pathname;
function run(args){return spawnSync(process.execPath,[...process.execArgv,cli,...args],{encoding:'utf8',timeout:30000,env:{...process.env,NODE_TEST_CONTEXT:undefined}});}
test('self-anchored internally consistent evidence never exits successfully as trusted verification',t=>{
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'provenance-trust-cli-'));t.after(()=>fs.rmSync(temp,{recursive:true,force:true}));
 const evidence=path.join(temp,'evidence'),workspace=path.join(temp,'work');fs.mkdirSync(evidence);fs.mkdirSync(workspace);
 const primitives=provenancePrimitives({encodeTyped,decodeTyped});
 const session=primitives.createSession({sessionId:'held-out',contextId:'scope',initialMemory:Buffer.alloc(0),runtimeId:process.version,sourceId:'caller-supplied-id'});
 session.append('terminal',{reason:'complete'});
 fs.writeFileSync(path.join(evidence,'chain.lino'),session.lines().join('\n')+'\n');fs.writeFileSync(path.join(evidence,'anchor.lino'),encodeTyped(session.anchor())+'\n');
 const inspected=run(['inspect-drive',evidence,workspace]);assert.equal(inspected.status,0,inspected.stderr);
 const inspection=decodeTyped(inspected.stdout.trimEnd());assert.equal(inspection.integrity,true);assert.equal(inspection.verification,'inspection-only');assert.equal(inspection.trustedVerification,false);
 const verified=run(['verify-drive',evidence,workspace]);assert.equal(verified.status,2,verified.stderr);
 const unknown=decodeTyped(verified.stdout.trimEnd());assert.equal(unknown.verification,'Unknown');assert.equal(unknown.trustedVerification,false);assert.equal(unknown.fullReplay.verified,false);
 assert(unknown.verificationGaps.includes('IndependentExpectedAnchorRequired'));assert.equal(unknown.anchorTrust,'self-supplied');
});
test('repository verification without actual pinned proof is Unknown for ordinary replay/share flags',()=>{
 const result=run(['verify-provenance','/caller/repository','HEAD~3..HEAD','--replay','--min-share','0.1']);
 assert.equal(result.status,2,result.stderr);const report=decodeTyped(result.stdout.trimEnd());
 assert.equal(report.verification,'Unknown');assert.equal(report.fullReplay.verified,false);assert.equal(report.trustedVerification,false);
 assert(report.verificationGaps.includes('ActualRepositoryChangeEvidenceUnavailable'));
});
