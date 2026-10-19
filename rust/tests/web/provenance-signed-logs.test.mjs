import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {openInstallationSigner,trustedInstallationKey} from '../../../js/provenance/installation-key.mjs';
import {verifyDriveEvidence} from '../../../js/provenance/verify-drive.mjs';
import {encodeTyped,decodeTyped,createLogger} from '../../../js/server/structured-log.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
before(async()=>installNodeHost(new WorkerHost()));
function temporary(t){
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'signed-provenance-'));
 t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
 return directory;
}
test('production drive retains redacted logical logs and seals the actual completed head with a stable installation key',async t=>{
 const directory=temporary(t),workspace=path.join(directory,'work'),keys=path.join(directory,'keys');
 fs.mkdirSync(workspace);
 const result=await drive(planChatStep,workspace,'Create file heldout.mjs containing export const measure = 37;',
  {steps:12,provenance:{directory:path.join(directory,'evidence'),signing:{directory:keys},environment:{}}});
 assert.equal(result.stop,'final');
 assert.equal(fs.readFileSync(path.join(workspace,'heldout.mjs'),'utf8'),'export const measure = 37;');
 const first=openInstallationSigner(keys),second=openInstallationSigner(keys);
 assert.equal(first.publicKey,second.publicKey);
 assert.equal(first.publicKey,trustedInstallationKey(keys));
 assert.equal(fs.statSync(path.join(keys,'installation-ed25519.pk8')).mode&0o777,0o600);
 const report=verifyDriveEvidence(result.provenance.directory,workspace,result.provenance.anchor,{trustedPublicKey:first.publicKey});
 assert.equal(report.installation.signature,'verified');
 assert.equal(report.installation.trust,'caller-pinned-installation-key');
 assert(report.structuredLogs>0);
 assert.equal(report.authorship,'not-established');
 assert.equal(report.replay,'not-performed');
 const other=openInstallationSigner(path.join(directory,'other'));
 assert.throws(()=>verifyDriveEvidence(result.provenance.directory,workspace,result.provenance.anchor,{trustedPublicKey:other.publicKey}),/installation_seal_binding/);
 assert.equal(verifyDriveEvidence(result.provenance.directory,workspace).installation.signature,'not-verified');
 const chain=fs.readFileSync(path.join(result.provenance.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
 const log=chain.find(row=>row.kind==='structured_log').payload.log;
 const logFile=path.join(result.provenance.directory,log.file),original=fs.readFileSync(logFile,'utf8');
 fs.writeFileSync(logFile,original.replace('provenance.genesis','provenance.changed'));
 assert.throws(()=>verifyDriveEvidence(result.provenance.directory,workspace,result.provenance.anchor,{trustedPublicKey:first.publicKey}),/structured_log_hash/);
 fs.writeFileSync(logFile,original);
 const sealFile=path.join(result.provenance.directory,'seal.lino'),seal=decodeTyped(fs.readFileSync(sealFile,'utf8').trimEnd());
 seal.signature=Buffer.alloc(64).toString('base64');fs.writeFileSync(sealFile,encodeTyped(seal)+'\n');
 assert.throws(()=>verifyDriveEvidence(result.provenance.directory,workspace,result.provenance.anchor,{trustedPublicKey:first.publicKey}),/ed25519_seal_invalid/);
});
test('installation custody refuses public-key substitution, insecure private modes, locks and symlink directories',t=>{
 const directory=temporary(t),keys=path.join(directory,'keys');
 openInstallationSigner(keys);
 const privateFile=path.join(keys,'installation-ed25519.pk8');
 fs.chmodSync(privateFile,0o644);
 assert.throws(()=>openInstallationSigner(keys),/installation_key_file_custody/);
 fs.chmodSync(privateFile,0o600);
 fs.mkdirSync(path.join(keys,'installation.lock'));
 assert.throws(()=>openInstallationSigner(keys),/installation_key_busy/);
 fs.rmdirSync(path.join(keys,'installation.lock'));
 const other=path.join(directory,'other');openInstallationSigner(other);
 fs.copyFileSync(path.join(other,'installation-ed25519.spki'),path.join(keys,'installation-ed25519.spki'));
 assert.throws(()=>openInstallationSigner(keys),/installation_key_pair_mismatch/);
 fs.symlinkSync(other,path.join(directory,'linked'));
 assert.throws(()=>openInstallationSigner(path.join(directory,'linked')),/installation_key_directory_custody/);
});
test('missing trust stores stay read-only and parent aliases cannot place signing keys in the task workspace',async t=>{
 const directory=temporary(t),missing=path.join(directory,'missing'),workspace=path.join(directory,'work');
 assert.throws(()=>trustedInstallationKey(missing));
 assert.equal(fs.existsSync(missing),false);
 fs.mkdirSync(workspace);
 const alias=path.join(directory,'alias');fs.symlinkSync(workspace,alias);
 await assert.rejects(drive(planChatStep,workspace,'unknown',{steps:0,provenance:{directory:path.join(directory,'evidence'),
  signing:{directory:path.join(alias,'keys')}}}),/installation_key_inside_workspace/);
 assert.deepEqual(fs.readdirSync(workspace),[]);
});
test('logical record observer sees exact post-redaction bytes and fails closed before physical writes',()=>{
 const observed=[],physical=[],secret='synthetic\nprivate-secret';
 const logger=createLogger({env:{GH_TOKEN:secret},recordObserver:line=>observed.push(line),
  stderrWrite:bytes=>(physical.push(Buffer.from(bytes)),bytes.length)});
 assert.equal(logger.log('debug','source',{message:secret}).emitted,true);
 assert.equal(observed.length,1);
 assert.equal(observed[0]+'\n',Buffer.concat(physical).toString());
 assert(!observed[0].includes(secret));
 const refused=createLogger({env:{},recordObserver:()=>{throw Error('private sink');},stderrWrite:()=>assert.fail()});
 assert.equal(refused.log('info','event').reason,'record_observer_failed');
});
