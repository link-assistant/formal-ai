// A version producer may refresh only the governed Cargo-version source witness.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {resolve,relative,isAbsolute} from 'node:path';
import {fileURLToPath} from 'node:url';
import {deserialize,serialize} from 'node:v8';
export const WITNESS = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const digest = value => createHash('sha256').update(value).digest('hex');
export function validateVersionWitness(beforeBytes,afterBytes,previousCargo,currentCargo) {
  const version = /^(version\s*=\s*")[^"]+(".*)$/mu;
  assert.ok(version.test(previousCargo)&&version.test(currentCargo),'actual Cargo package version required');
  assert.notEqual(previousCargo,currentCargo,'Cargo version must change');
  assert.equal(previousCargo.replace(version,'$1<VERSION>$2'),currentCargo.replace(version,'$1<VERSION>$2'),'only package version may change');
  const before = deserialize(beforeBytes),after = deserialize(afterBytes),expected = structuredClone(before);
  const previousHash = digest(previousCargo),currentHash = digest(currentCargo);
  let changed = 0;
  for(const item of expected.inventory.cases) for(const fixture of item.program?.fixtures??[]) {
    if(fixture.file !== 'repo:rust/Cargo.toml')continue;
    assert.equal(fixture.sha256,previousHash,'governed Cargo witness must match actual preceding manifest');
    fixture.sha256=currentHash;changed+=1;
  }
  assert.ok(changed>0,'governed inventory has no Cargo source witness');
  assert.deepEqual(after,expected,'version refresh cannot change cases, scope, assertions or other source witnesses');
  return changed;
}
export async function refreshVersionWitness(root,previousCargo,write=false) {
  const {specificationCases}=await import('./lib/rust-specification-cases.mjs');
  const file=resolve(root,WITNESS),before=readFileSync(file),currentCargo=readFileSync(resolve(root,'rust/Cargo.toml'),'utf8');
  const inventory=specificationCases(root),cases=structuredClone(inventory.cases);
  for(const item of cases)for(const fixture of item.program?.fixtures??[])if(isAbsolute(fixture.file)) {
    const name=relative(root,fixture.file);assert.ok(!name.startsWith('..'),'fixture escaped source root');fixture.file='repo:'+name;
  }
  const sourceManifest=readdirSync(resolve(root,'rust/tests/unit/specification'),{recursive:true}).map(String).filter(name=>name.endsWith('.rs')).sort().map(name=> {
    const path='rust/tests/unit/specification/'+name;return {path,sha256:digest(readFileSync(resolve(root,path)))};
  });
  const after=serialize({inventory:{...inventory,cases},sourceManifest});
  const changed=validateVersionWitness(before,after,previousCargo,currentCargo);
  assert.equal(digest(readFileSync(file)),digest(before),'governed witness changed during generation');
  if(write)writeFileSync(file,after);
  return {changed,beforeSha256:digest(before),afterSha256:digest(after)};
}
if(process.argv[1]===fileURLToPath(import.meta.url)) {
  assert.ok(process.argv.includes('--previous-cargo-stdin'),'actual previous manifest input required');
  const result=await refreshVersionWitness(resolve(fileURLToPath(new URL('../',import.meta.url))),readFileSync(0,'utf8'),process.argv.includes('--write'));
  console.log(JSON.stringify(result));
}
