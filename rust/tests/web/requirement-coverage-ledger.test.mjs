import assert from 'node:assert/strict';
import {test} from 'node:test';
import {parseCoverageLedger} from '../../../experiments/formal_ai_subagent/requirement-coverage.mjs';

test('coverage consumes shared verdict/test defaults and explicit minority/empty overrides',()=>{
 const ledger=parseCoverageLedger('requirement_status_ledger_shard\n'
  +'  shard "docs/requirements/owned.md"\n  issue "42"\n  verdict "not-delivered"\n'
  +'  automated_test "rust/tests/shared.test.mjs"\n  manual "not yet confirmed"\n'
  +'  requirement\n    id "R42-1"\n'
  +'  requirement\n    id "R42-2"\n    verdict "partial"\n    automated_test "rust/tests/minority.test.mjs"\n    delivered "owned detail"\n'
  +'  requirement\n    id "R42-3"\n    automated_test ""\n'
  +'  requirement\n    id "R42-4"\n');
 assert.deepEqual([...ledger.keys()],['R42-1','R42-2','R42-3','R42-4']);
 assert.deepEqual(ledger.get('R42-1'),{id:'R42-1',shard:'docs/requirements/owned.md',issue:'42',verdict:'not-delivered',delivered:'',automatedTest:'rust/tests/shared.test.mjs',manual:'not yet confirmed'});
 assert.deepEqual(ledger.get('R42-2'),{...ledger.get('R42-1'),id:'R42-2',verdict:'partial',automatedTest:'rust/tests/minority.test.mjs',delivered:'owned detail'});
 assert.equal(ledger.get('R42-3').automatedTest,'');assert.equal(ledger.get('R42-3').verdict,'not-delivered');
 assert.deepEqual(ledger.get('R42-4'),{...ledger.get('R42-1'),id:'R42-4'});
});
test('coverage retains explicit legacy record fields and refuses malformed owned scopes',()=>{
 const explicit='requirement_status_ledger_shard\n  requirement\n    id "R1"\n    verdict "implemented"\n    automated_test "rust/tests/owned.test.mjs"\n';
 const row=parseCoverageLedger(explicit).get('R1');assert.equal(row.verdict,'implemented');assert.equal(row.automatedTest,'rust/tests/owned.test.mjs');
 assert.throws(()=>parseCoverageLedger('requirement_status_ledger_shard\n  verdict "partial"\n  requirement\n    automated_test "tests/foreign"\n'),/identity is missing/);
 assert.throws(()=>parseCoverageLedger(explicit+'    verdict "partial"\n'),/duplicate ledger field/);
});
