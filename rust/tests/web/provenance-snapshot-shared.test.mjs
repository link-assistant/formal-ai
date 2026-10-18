import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {sourceSnapshot,languageExtensions,merkleRoot,merkleProof,verifyMerkleProof} from '../../../js/provenance/snapshot.mjs';
import {snapshotTreeShape,snapshotTreePair,snapshotByteOrder,snapshotBackend} from '../../../js/provenance/portable/snapshot.mjs';
import {blake3Hex} from '../../../js/crypto/blake3.mjs';
const fixture=JSON.parse(fs.readFileSync(new URL('../fixtures/provenance-snapshot.json',import.meta.url),'utf8'));
const extensions=languageExtensions(fs.readFileSync(new URL('../../../data/seed/page-formalization-rules.lino',import.meta.url),'utf8'));
test('shared native/JavaScript snapshot vectors are complete and preserve independently recorded roots',()=>{
 assert.equal(fixture.rows.length,5);
 assert.deepEqual(fixture.rows.map(row=>row.name),['empty','mixed','rust-only-change','javascript-only-change','unicode-order']);
 for(const row of fixture.rows){
  const files=row.files.map(file=>({path:file.path,content:Buffer.from(file.content)}));
  assert.deepEqual(sourceSnapshot(files,extensions),row.expected,row.name);
  assert.deepEqual(sourceSnapshot([...files].reverse(),extensions),row.expected,row.name);
 }
 assert.notEqual(fixture.rows[1].expected.total,fixture.rows[2].expected.total);
 assert.notEqual(fixture.rows[1].expected.total,fixture.rows[3].expected.total);
});
test('shared inclusion vectors reject extra, altered, truncated and foreign-leaf proofs',()=>{
 assert.equal(fixture.proofs.length,3);
 assert.deepEqual(fixture.proofs.map(row=>row.inputs.length),[1,3,5]);
 for(const row of fixture.proofs){
  const leaves=row.inputs.map(input=>blake3Hex(Buffer.from(input)));
  assert.deepEqual(leaves,row.leaves);assert.equal(merkleRoot(leaves),row.root);
  for(let index=0;index<leaves.length;index++){
   const proof=merkleProof(leaves,index);assert.deepEqual(proof,row.proofs[index]);
   assert(verifyMerkleProof(leaves[index],proof,row.root));
   assert(!verifyMerkleProof(blake3Hex(Buffer.from('foreign')),proof,row.root));
   assert(!verifyMerkleProof(leaves[index],{...proof,siblings:[...proof.siblings,'0'.repeat(64)]},row.root));
   if(proof.siblings.length){
    assert(!verifyMerkleProof(leaves[index],{...proof,siblings:proof.siblings.slice(1)},row.root));
    assert(!verifyMerkleProof(leaves[index],{...proof,siblings:['0'.repeat(64),...proof.siblings.slice(1)]},row.root));
   }
  }
 }
});
test('canonical indexed topology preserves odd duplication and bounded byte ordering',()=>{
 for(let count=0;count<65;count++){
  const shape=snapshotTreeShape(count);let level=Array.from({length:count},(_,index)=>index),next=count;
  while(level.length>1){
   const following=[];
   for(let at=0;at<level.length;at+=2){
    assert.deepEqual(snapshotTreePair(count,next),{$:'pair',left:level[at],right:level[at+1]??level[at]});following.push(next++);
   }
   level=following;
  }
  assert.equal(shape.total,next);assert.equal(shape.root,count===0?-1:next-1);
  assert.deepEqual(snapshotTreePair(count,next),{$:'not-pair'});
 }
 for(const left of ['a','é','中','😀'])for(const right of ['a','é','中','😀']){
  assert.equal(Math.sign(snapshotByteOrder([...Buffer.from(left)],[...Buffer.from(right)])),Math.sign(Buffer.compare(Buffer.from(left),Buffer.from(right))));
 }
 assert.equal(snapshotBackend('cpu'),'cpu');assert.equal(snapshotBackend('auto'),'cpu');
 assert.equal(snapshotBackend('gpu'),'gpu-unavailable');assert.equal(snapshotBackend('AUTO'),'invalid-backend');
});
