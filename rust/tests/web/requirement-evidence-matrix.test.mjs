import assert from 'node:assert/strict';
import {
  test}
from 'node:test';
import fs from 'node:fs';
import {
  join}
from 'node:path';
import {
  tmpdir}
from 'node:os';
import {
  requirementEvidenceMatrix,
  checkRequirementEvidenceMatrix,
  requirementCitations}
from '../../../scripts/lib/requirement-evidence-matrix.mjs';
function fixture() {
  const root=fs.mkdtempSync(join(tmpdir(),
  'requirement-matrix-'));
  for(const directory of ['docs/requirements/assembled',
  'rust/tests/web',
  'js',
  'data/meta/requirement-status-ledger'])fs.mkdirSync(join(root,
  directory),
  {
    recursive:true}
  );
  fs.writeFileSync(join(root,
  'docs/requirements/assembled/standing-doctrine.md'),
  '| R42-1 | A complete registered task | Implemented: rust/tests/web/answer.test.mjs |\n| R42-2 | A partial task | Partial: js/answer.mjs and rust/tests/web/answer.test.mjs |\n');
  fs.writeFileSync(join(root,
  'docs/requirements/issue-0042-held-out.md'),
  '| R42-1 | A complete registered task | Implemented: js/answer.mjs and rust/tests/web/answer.test.mjs |\n| R42-2 | A partial task | Partial: delivered helper js/answer.mjs; pending behavior rust/tests/web/answer.test.mjs |\n');
  fs.writeFileSync(join(root,
  'rust/tests/web/answer.test.mjs'),
  "throw new Error('this test must never run for an inventory');\n");
  fs.writeFileSync(join(root,
  'js/answer.mjs'),
  'export const answer = 42;\n');
  return {
    root,
    cleanup:()=>fs.rmSync(root,
    {
      recursive:true,
      force:true}
    )}
  ;
}
test('canonical claimed delivery stays distinct from Partial and absent declared work',
()=>{
  const f=fixture();
  try{
    const matrix=requirementEvidenceMatrix(f.root,
    {
      declaredIds:['R42-1',
      'R42-2',
      'R91-7']}
    );
    assert.deepEqual(matrix.counts,
    {
      DeliveredClaim:1,
      Partial:1,
      Undrafted:1}
    );
    assert.equal(matrix.records.find(row=>row.id==='R42-2').documentedVerdict,
    'partial');
    assert.ok(matrix.records.every(row=>row.freshAcceptance==='Unknown'));
    assert.equal(matrix.records[0].automatedTest.status,
    'Bound');
    assert.equal(matrix.suppliedProgramAutonomyCredit,
    0);
    assert.deepEqual(checkRequirementEvidenceMatrix(matrix,
    f.root,
    {
      declaredIds:['R42-1',
      'R42-2',
      'R91-7']}
    ),
    matrix);
  }
  finally{
    f.cleanup();
  }
}
);
test('source, test, definition and declared cohort drift invalidate frozen matrix',
()=>{
  for(const file of ['js/answer.mjs',
  'rust/tests/web/answer.test.mjs',
  'docs/requirements/issue-0042-held-out.md']){
    const f=fixture();
    try{
      const original=requirementEvidenceMatrix(f.root);
      fs.appendFileSync(join(f.root,
      file),
      '\n// drift\n');
      assert.throws(()=>checkRequirementEvidenceMatrix(original,
      f.root),
      /stale or counterfeit/u);
    }
    finally{
      f.cleanup();
    }
  }
  const f=fixture();
  try{
    const original=requirementEvidenceMatrix(f.root);
    assert.throws(()=>checkRequirementEvidenceMatrix(original,
    f.root,
    {
      declaredIds:['R91-7']}
    ),
    /stale or counterfeit/u);
    assert.throws(()=>requirementEvidenceMatrix(f.root,
    {
      declaredIds:['R42-1',
      'R42-1']}
    ),
    /duplicate declared/u);
    const scoped=requirementEvidenceMatrix(f.root,
    {
      declaredIds:['R42-1']}
    );
    assert.throws(()=>checkRequirementEvidenceMatrix(scoped,
    f.root),
    /stale or counterfeit/u);
  }
  finally{
    f.cleanup();
  }
}
);
test('ready candidate sources have zero semantic count or completion credit',
()=>{
  const f=fixture();
  try{
    const candidate=join(f.root,
    'candidate.json');
    fs.writeFileSync(candidate,
    JSON.stringify({
      source:[{
        path:'js/new.mjs',
        afterSha256:'a'.repeat(64)}
      ]}
    ));
    const original=requirementEvidenceMatrix(f.root,
    {
      candidateManifests:[candidate]}
    );
    assert.equal(original.candidateManifests[0].sources[0].status,
    'ReadyOnlyAbsent');
    assert.equal(original.candidateManifests[0].sources[0].semanticCountCredit,
    0);
    fs.writeFileSync(join(f.root,
    'js/new.mjs'),
    'unqualified source');
    assert.throws(()=>checkRequirementEvidenceMatrix(original,
    f.root,
    {
      candidateManifests:[candidate]}
    ),
    /stale or counterfeit/u);
    assert.equal(requirementEvidenceMatrix(f.root,
    {
      candidateManifests:[candidate]}
    ).candidateManifests[0].sources[0].status,
    'ReadyOnlyDifferent');
  }
  finally{
    f.cleanup();
  }
}
);
test('quoted brace source families expand without external URL or traversal authority',
()=>{
  assert.deepEqual(requirementCitations('`scripts/held-out.{mjs,rs}` and https://example.com/js/foreign.mjs'),
  ['scripts/held-out.mjs',
  'scripts/held-out.rs']);
  const f=fixture();
  try{
    fs.appendFileSync(join(f.root,
    'docs/requirements/issue-0042-held-out.md'),
    '| R42-3 | unsafe cited path | Partial: js/../outside.mjs |\n');
    fs.appendFileSync(join(f.root,
    'docs/requirements/assembled/standing-doctrine.md'),
    '| R42-3 | unsafe cited path | Partial: js/../outside.mjs |\n');
    const record=requirementEvidenceMatrix(f.root).records.find(row=>row.id==='R42-3');
    assert.equal(record.citedPaths[0].status,
    'Unsafe');
    assert.equal(record.freshAcceptance,
    'Unknown');
  }
  finally{
    f.cleanup();
  }
}
);
test('stable but inflated generated ledger cannot pass the matrix check',
()=>{
  const f=fixture();
  try{
    fs.writeFileSync(join(f.root,
    'data/meta/requirement-status-ledger/held-out.lino'),
    'requirement_status_ledger_shard\n  shard "docs/requirements/issue-0042-held-out.md"\n  requirement\n    id "R42-2"\n    verdict "implemented"\n    automated_test "rust/tests/web/answer.test.mjs"\n');
    const original=requirementEvidenceMatrix(f.root);
    assert.deepEqual(original.ledgerMismatches,
    ['R42-2']);
    assert.equal(original.records.find(record=>record.id==='R42-2').classification,
    'Partial');
    assert.throws(()=>checkRequirementEvidenceMatrix(original,
    f.root),
    /disagrees with owning source/u);
  }
  finally{
    f.cleanup();
  }
}
);
test('concurrent physical definition changes refuse an internally mixed matrix',
()=>{
  const f=fixture();
  const originalRead=fs.readFileSync;
  let changed=false;
  try{
    fs.readFileSync=function(file,
    ...args){
      const bytes=originalRead.call(this,
      file,
      ...args);
      if(!changed&&String(file).endsWith('/js/answer.mjs')){
        changed=true;
        fs.appendFileSync(join(f.root,
        'docs/requirements/issue-0042-held-out.md'),
        '\n');
      }
      return bytes;
    }
    ;
    assert.throws(()=>requirementEvidenceMatrix(f.root),
    /source changed during requirement matrix capture/u);
    assert.equal(changed,
    true);
  }
  finally{
    fs.readFileSync=originalRead;
    f.cleanup();
  }
}
);
