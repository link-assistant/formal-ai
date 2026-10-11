import test from 'node:test';
import assert from 'node:assert/strict';
import {createCandidateTransaction} from '../../../js/agentic/module_function/candidate-transaction.mjs';
function workspace() {
 let state={kind:'absent'}, leased=true;
 const receipts=new WeakMap();
 const io={ownsLease:()=>leased,inspect:()=>state,
  processDisposition:receipt=>{if(!receipts.has(receipt))throw Error('UnownedProcessReceipt');return receipts.get(receipt);},
  removeUnchangedCandidate:(_,identity)=>{if(state.kind!=='file'||state.identity!==identity)return false;state={kind:'absent'};return true;}};
 return {io,set:next=>{state=next;},revoke:()=>{leased=false;},
  receipt:disposition=>{const receipt=Object.freeze({});receipts.set(receipt,disposition);return receipt;}};
}
for(const disposition of ['succeeded','failed','incomplete'])test('owned '+disposition+' disposition conserves bytes',()=>{
 const scope=workspace(),transaction=createCandidateTransaction(scope.io,'candidate','digest');
 scope.set({kind:'file',identity:'digest'});transaction.recordWrite();
 const result=transaction.finish(scope.receipt(disposition));
 assert.equal(result.state,disposition==='succeeded'?'committed':'rolled-back');
 assert.equal(scope.io.inspect().kind,disposition==='succeeded'?'file':'absent');
});
test('copied process receipt cannot commit or remove candidate',()=>{
 const scope=workspace(),transaction=createCandidateTransaction(scope.io,'candidate','digest');
 scope.set({kind:'file',identity:'digest'});transaction.recordWrite();
 assert.throws(()=>transaction.finish({...scope.receipt('succeeded')}),/UnownedProcessReceipt/);
 assert.equal(scope.io.inspect().kind,'file');
 assert.equal(transaction.abort().processDisposition,'unverified');
});
test('abort preserves independently changed candidate',()=>{
 const scope=workspace(),transaction=createCandidateTransaction(scope.io,'candidate','digest');
 scope.set({kind:'file',identity:'digest'});transaction.recordWrite();scope.set({kind:'file',identity:'changed'});
 assert.deepEqual(transaction.abort(),{state:'refused-drift',removed:false});
 assert.equal(scope.io.inspect().identity,'changed');
});
test('expired lease prevents rollback',()=>{
 const scope=workspace(),transaction=createCandidateTransaction(scope.io,'candidate','digest');
 scope.set({kind:'file',identity:'digest'});transaction.recordWrite();scope.revoke();
 assert.throws(()=>transaction.abort(),/MissingSingleWriterLease/);assert.equal(scope.io.inspect().kind,'file');
});
test('existing destination cannot become candidate ownership',()=>{
 const scope=workspace();scope.set({kind:'file',identity:'digest'});
 assert.throws(()=>createCandidateTransaction(scope.io,'candidate','digest'),/UnownedExistingDestination/);
});
test('unobserved write and duplicate finalization refuse',()=>{
 const scope=workspace(),transaction=createCandidateTransaction(scope.io,'candidate','digest');
 assert.throws(()=>transaction.recordWrite(),/UnobservedCandidateWrite/);
 scope.set({kind:'file',identity:'digest'});transaction.recordWrite();transaction.abort();
 assert.throws(()=>transaction.abort(),/InvalidCandidatePhase/);
});
