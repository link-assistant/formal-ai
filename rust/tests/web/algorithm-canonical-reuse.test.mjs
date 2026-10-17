import test from 'node:test';
import assert from 'node:assert/strict';
import * as server from '../../../js/server/algorithm-discovery.mjs';
import * as canonical from '../../../js/agentic/crate/algorithm_discovery.mjs';
import {SequenceStore,SymbolTable} from '../../../js/agentic/crate/sequences.mjs';
function repeatedTraces(prefix='file') {
  return ['one','two','three'].map((id,index)=>({id,steps:[
    {operation:'read_file',arguments:[['path',prefix+index]]},
    {operation:'write_file',arguments:[['path',prefix+index]]},
  ]}));
}
test('server public validation and notation are exact canonical exports',()=>{
  assert.equal(server.candidateValidated,canonical.candidateValidated);
  assert.equal(server.candidateLinksNotation,canonical.candidateLinksNotation);
});
test('held-out validation retains truth values and exact thrown getter identities',()=>{
  assert.equal(server.candidateValidated({held_out:[]}),false);
  assert.equal(server.candidateValidated({held_out:[{passed:true}]}),true);
  assert.equal(server.candidateValidated({held_out:[{passed:false}]}),false);
  const sentinel=new Error('held-out getter');
  assert.throws(()=>server.candidateValidated({get held_out(){throw sentinel;}}),error=>error===sentinel);
});
test('discovery allocates fresh independent state across calls and output mutations',()=>{
  const input=repeatedTraces();
  const before=structuredClone(input);
  const first=server.discoverValidatedAlgorithms(input);
  assert(first.length>0);
  const expected=structuredClone(first);
  first[0].steps[0].operation='mutated output';
  first[0].support_trace_ids.push('mutated support');
  server.discoverValidatedAlgorithms(repeatedTraces('unrelated'));
  assert.deepEqual(server.discoverValidatedAlgorithms(input),expected);
  assert.deepEqual(input,before);
});
test('canonical public class prototype mutation cannot alter server private stores',()=>{
  const input=repeatedTraces(),expected=server.discoverValidatedAlgorithms(input);
  const create=SequenceStore.prototype.createPoint,get=SequenceStore.prototype.getOrCreate;
  const marker=SymbolTable.prototype.marker;
  const fail=()=>{throw new Error('foreign prototype mutation');};
  try {
    SequenceStore.prototype.createPoint=fail;
    SequenceStore.prototype.getOrCreate=fail;
    SymbolTable.prototype.marker=fail;
    assert.deepEqual(server.discoverValidatedAlgorithms(input),expected);
  } finally {
    SequenceStore.prototype.createPoint=create;
    SequenceStore.prototype.getOrCreate=get;
    SymbolTable.prototype.marker=marker;
  }
});
test('raw event adapter preserves original fallback errors and candidate exclusion',()=>{
  const events=[{kind:'read',conversation_id:'session',inputs:'{"answer":42}'},
    {kind:'write',conversation_id:'session',inputs:'unparsed input'},
    {kind:'algorithm_learning_candidate',conversation_id:'session',inputs:'ignored'}];
  const actual=server.tracesFromMemoryEvents(events);
  assert.equal(actual.length,1);
  assert.equal(actual[0].steps.length,2);
  assert.deepEqual(actual[0].steps[0].arguments,[['answer','42']]);
  assert.deepEqual(actual[0].steps[1].arguments,[]);
  assert.throws(()=>server.tracesFromMemoryEvents([{kind:'read',inputs:42}]),TypeError);
  const custom=server.tracesFromMemoryEvents([{kind:'custom',inputs:{split:()=>[42]}}]);
  assert.deepEqual(custom[0].steps[0].arguments,[]);
});
