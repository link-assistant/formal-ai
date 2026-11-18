// Supplied review harness: zero autonomous-authorship credit.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,readdirSync,mkdirSync} from 'node:fs';
import {isAbsolute,relative,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {deserialize} from 'node:v8';
import {gzipSync} from 'node:zlib';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';

const root=fileURLToPath(new URL('../../',import.meta.url));
const hash=data=>createHash('sha256').update(data).digest('hex');
const witness=resolve(root,'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8');
const reader=resolve(root,'scripts/lib/rust-specification-cases.mjs');
const beforeRaw=readFileSync(witness), before=deserialize(beforeRaw);
const readerSha=hash(readFileSync(reader));
assert.equal(readerSha,'861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca','the previously reviewed reader must remain unchanged');
const inventory=specificationCases(root);
const cases=structuredClone(inventory.cases);
for(const item of cases)for(const fixture of item.program?.fixtures??[]){
  if(isAbsolute(fixture.file)){
    const file=relative(root,fixture.file);
    assert(!file.startsWith('..'));
    fixture.file='repo:'+file;
  }
}
const sourceManifest=readdirSync(resolve(root,'rust/tests/unit/specification'),{recursive:true})
  .map(String).filter(file=>file.endsWith('.rs')).sort().map(file=>{
    const path='rust/tests/unit/specification/'+file;
    return {path,sha256:hash(readFileSync(resolve(root,path)))};
  });
const candidate={inventory:{...inventory,cases},sourceManifest};
assert.deepEqual(candidate.inventory,before.inventory,'every complete case and unsupported object must remain unchanged');
const differences=[];
assert.equal(candidate.sourceManifest.length,before.sourceManifest.length);
for(let i=0;i<sourceManifest.length;i++){
  assert.equal(sourceManifest[i].path,before.sourceManifest[i].path);
  if(sourceManifest[i].sha256!==before.sourceManifest[i].sha256)
    differences.push({index:i,path:sourceManifest[i].path,before:before.sourceManifest[i].sha256,after:sourceManifest[i].sha256});
}
assert.deepEqual(differences,[{
  index:sourceManifest.findIndex(row=>row.path==='rust/tests/unit/specification/formalization.rs'),
  path:'rust/tests/unit/specification/formalization.rs',
  before:'96038b52dc0f1ce195bf9f3034ca7a82a5707c58d3ffacbcb9d59a9f0e5dff65',
  after:'d7d4870303bb2c05843769a063942f3f75766e9215447088e37671bb688dd7b3',
}]);
const out=resolve(root,'experiments/issue-1580/evidence/checks/localized-unknown-opener/governed-refresh');
mkdirSync(out,{recursive:true});
writeFileSync(resolve(out,'previous-witness.v8.gz'),gzipSync(beforeRaw,{mtime:0}));
const argv=['--max-old-space-size=256','rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
  '--refresh-reviewed-baseline','--reviewed-reader-sha='+readerSha,'--previous-baseline-sha='+hash(beforeRaw)];
const log=execFileSync(process.execPath,argv,{cwd:root,maxBuffer:1024*1024});
writeFileSync(resolve(out,'refresh.log.gz'),gzipSync(log,{mtime:0}));
const afterRaw=readFileSync(witness);
assert.deepEqual(deserialize(afterRaw),candidate);
const report={readerSha256:readerSha,beforeSha256:hash(beforeRaw),afterSha256:hash(afterRaw),
  completeCaseObjectsUnchanged:true,unsupportedObjectsUnchanged:true,
  caseCount:cases.length,unsupportedCount:inventory.unsupported.length,
  differences,command:[process.execPath,...argv],nativeBehaviorVerified:false,autonomousAuthorshipCredit:0};
writeFileSync(resolve(out,'complete-delta.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
