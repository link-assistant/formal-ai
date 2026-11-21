import assert from 'node:assert/strict';
import test from 'node:test';
import {splitMeaningRecords} from '../../../scripts/lib/split-meaning-records.mjs';

const source='# Preserve authored grounding and text.\nmeanings\n  first\n    role action\n    example "{value}\\nfirst"\n\n  second\n    role concept\n    grounding "data/seed/arbitrary.lino"\n';
test('complete meaning records retain exact bytes, order and the root header',()=>{
  const result=splitMeaningRecords(source,7);
  assert.deepEqual(result.recordIds,['first','second']);assert.equal(result.shards.length,2);
  assert.equal(result.shards.map(shard=>shard.slice(result.header.length)).join(''),source.slice(result.header.length));
  assert(result.shards.every(shard=>shard.startsWith(result.header)&&shard.split('\n').length-1<=7));
});
test('unknown roots, partial records and duplicate identities refuse a split',()=>{
  for(const candidate of [source.replace('\nmeanings\n','\nresponses\n'),source+'\nother-root\n',source.replace('    role action','  unknown field'),source+'  first\n    role duplicate\n',source+'  empty\n',source.slice(0,-1)]){
    assert.throws(()=>splitMeaningRecords(candidate,7));
  }
  assert.throws(()=>splitMeaningRecords(source,1));
});
test('an indivisible meaning fails rather than raising its supplied budget',()=>{
  assert.throws(()=>splitMeaningRecords(source,5),/complete meaning exceeds/u);
});
