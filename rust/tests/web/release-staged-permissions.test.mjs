import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
const read=()=>YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/release-staged.yml',import.meta.url),'utf8'));
function validate(doc){
  assert.deepEqual(doc.permissions,{contents:'read'});
  assert.equal(Object.keys(doc.jobs).length,16);
  for(const [id,job] of Object.entries(doc.jobs)){
    const stage=id.replace(/^(auto|manual)_/,'');
    assert.match(id,/^(auto|manual)_/);
    const required=stage==='prepare-source'||stage==='create-release'?{contents:'write'}:stage==='publish-verify-images'?{contents:'read',packages:'write'}:undefined;
    assert.deepEqual(job.permissions,required,id);
    assert.equal(job.concurrency,undefined,'caller holds writer lease');
    const commands=job.steps.map(x=>x.run??'').join('\n');
    if(stage==='prepare-source')assert.match(commands,/version-and-commit|version_and_commit/);
    if(stage==='publish-verify-images')assert.match(commands,/release-image-factory/);
    if(stage==='create-release')assert.match(commands,/bash scripts\/invoke-create-github-release\.sh/);
    if(stage==='publish-crate')assert.ok(job.steps.some(x=>x.env?.CARGO_TOKEN),'crate credentials remain independent');
  }
}
test('dormant release grants only operation-bound GitHub token rights',()=>validate(read()));
for(const mode of ['auto','manual'])for(const stage of ['prepare-source','compile-release','verify-package','publish-crate','registry-availability','published-crate-smoke','publish-verify-images','create-release'])test(`${mode} ${stage} rejects missing or excess authority`,()=>{const doc=read();doc.jobs[mode+'_'+stage].permissions={contents:'write',packages:'write',actions:'write'};assert.throws(()=>validate(doc));});
test('top-level write authority refuses',()=>{const doc=read();doc.permissions.contents='write';assert.throws(()=>validate(doc));});

for(const mode of ['auto','manual'])for(const stage of ['prepare-source','publish-verify-images','create-release'])test(`${mode} ${stage} missing necessary authority refuses`,()=>{const doc=read();delete doc.jobs[mode+'_'+stage].permissions;assert.throws(()=>validate(doc));});
