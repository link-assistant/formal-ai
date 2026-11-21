import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {gunzipSync} from 'node:zlib';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {readTraceability,traceabilityPaths,TRACEABILITY,TRACEABILITY_CONTINUATION} from '../../scripts/lib/requirements-traceability.mjs';
import {traceRows} from '../../scripts/generate-requirement-status.mjs';
import {requirementEvidenceMatrix,checkRequirementEvidenceMatrix} from '../../scripts/lib/requirement-evidence-matrix.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const original=gunzipSync(readFileSync(new URL('./fixtures/traceability-partition/original.md.gz',import.meta.url))).toString();
const rows=text=>text.split('\n').filter(line=>line.startsWith('| R'));
test('traceability pages obey the existing source warning threshold',()=>{
  for(const file of traceabilityPaths(root))assert.ok(readFileSync(join(root,file),'utf8').trimEnd().split('\n').length<1400,file);
  assert.equal(traceabilityPaths(root).length,2);
});
test('every traceability row and generated status field survives the partition',()=>{
  const assembled=readTraceability(root);
  assert.deepEqual(rows(assembled),rows(original));
  assert.deepEqual(traceRows(assembled,root),traceRows(original,root));
  assert.ok(assembled.includes(original.slice(0,original.indexOf('| ID |'))));
});
test('legacy fixtures work and an indexed missing continuation fails explicitly',t=>{
  const directory=mkdtempSync(join(tmpdir(),'traceability-index-'));t.after(()=>rmSync(directory,{recursive:true,force:true}));
  mkdirSync(join(directory,'docs/requirements/assembled'),{recursive:true});
  writeFileSync(join(directory,'docs/requirements/assembled/standing-doctrine.md'),'# Empty requirement fixture\n');
  writeFileSync(join(directory,TRACEABILITY),'| R1 | shard | delivered | test | unknown |\n');
  assert.equal(traceabilityPaths(directory).length,1);
  assert.match(readTraceability(directory),/R1/u);
  writeFileSync(join(directory,TRACEABILITY),'[Continued rows](requirements-traceability-continuation.md)\n');
  assert.throws(()=>readTraceability(directory),/cannot read .*requirements-traceability-continuation[.]md/u);
  writeFileSync(join(directory,TRACEABILITY_CONTINUATION),'| R2 | shard | pending | test | unknown |\n');
  assert.match(readTraceability(directory),/R2/u);
});
test('evidence inventory binds continuation bytes and rejects later drift',t=>{
  const matrix=requirementEvidenceMatrix(root);
  const bytes=readFileSync(join(root,TRACEABILITY_CONTINUATION));
  assert.deepEqual(matrix.bindings.find(binding=>binding.path===TRACEABILITY_CONTINUATION),{
    path:TRACEABILITY_CONTINUATION,status:'Bound',sha256:createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length,
  });
  const directory=mkdtempSync(join(tmpdir(),'traceability-evidence-'));t.after(()=>rmSync(directory,{recursive:true,force:true}));
  mkdirSync(join(directory,'docs/requirements/assembled'),{recursive:true});
  writeFileSync(join(directory,'docs/requirements/assembled/standing-doctrine.md'),'# Empty requirement fixture\n');
  writeFileSync(join(directory,TRACEABILITY),'[Continued rows](requirements-traceability-continuation.md)\n');
  writeFileSync(join(directory,TRACEABILITY_CONTINUATION),'original continuation\n');
  const captured=requirementEvidenceMatrix(directory);
  writeFileSync(join(directory,TRACEABILITY_CONTINUATION),'changed continuation\n');
  assert.throws(()=>checkRequirementEvidenceMatrix(captured,directory),/stale or counterfeit/u);
});
