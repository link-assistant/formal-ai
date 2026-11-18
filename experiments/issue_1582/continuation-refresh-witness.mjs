// Supplied review of complete objects before the maintained witness refresh.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {deserialize} from 'node:v8';
import {isAbsolute,relative} from 'node:path';
import {spawnSync} from 'node:child_process';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';
const witness='experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const beforeBytes=fs.readFileSync(witness),before=deserialize(beforeBytes);
const inventory=specificationCases(process.cwd());
for(const item of inventory.cases)for(const fixture of item.program?.fixtures??[])if(isAbsolute(fixture.file))fixture.file='repo:'+relative(process.cwd(),fixture.file);
const expected=structuredClone(before.inventory), changes=[];
const path='rust/tests/unit/mod.rs', newHash=digest(fs.readFileSync(path));
function updateReviewedProvenance(value,trail=[]){
 if(!value||typeof value!=='object')return;
 if(value.file==='repo:'+path&&typeof value.sha256==='string'){
   changes.push({location:[...trail,'sha256'],file:path,previousSha256:value.sha256,sha256:newHash});
   value.sha256=newHash;
 }
 for(const [key,child]of Object.entries(value))updateReviewedProvenance(child,[...trail,key]);
}
updateReviewedProvenance(expected);
assert.equal(changes.length,4,'exactly four existing programs observe the ancestor module source');
assert.deepEqual(inventory,expected,'compare every complete object; only reviewed ancestor source hash changes');
const readerSha=digest(fs.readFileSync('scripts/lib/rust-specification-cases.mjs'));
const refreshed=spawnSync(process.execPath,['rust/tests/web/rust-specification-unicode-identifiers.test.mjs','--refresh-reviewed-baseline','--reviewed-reader-sha='+readerSha,'--previous-baseline-sha='+digest(beforeBytes)],{stdio:'inherit'});
assert.equal(refreshed.status,0);
const afterBytes=fs.readFileSync(witness),after=deserialize(afterBytes);
assert.deepEqual(after.inventory,expected);
assert.deepEqual(after.sourceManifest,before.sourceManifest);
const receipt={readerSha256:readerSha,previousWitnessSha256:digest(beforeBytes),refreshedWitnessSha256:digest(afterBytes),completeObjectsCompared:true,cases:after.inventory.cases.length,governedInventoryTotal:after.inventory.total,reviewedChange:'Add mod issue_1582_permissions; only four existing programs observing the ancestor unit/mod.rs source change their provenance hash. All case identities, assertions, initial values and unsupported records are compared unchanged.',provenanceChanges:changes};
fs.writeFileSync('docs/case-studies/issue-1582/raw-data/local/continuation-witness-refresh.json',JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify(receipt));
