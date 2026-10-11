import {before,test} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {statedValue} from '../../../js/agentic/module_function.mjs';
before(async()=>installNodeHost(new WorkerHost()));
for(const [expression,expected] of [['left*right','6'],['left * right','6'],['left+right','5'],['left-right','-1'],['left/right','2/3'],['left%right','2']]){
 test('complete declared arithmetic expression '+expression,()=>{
   const value=statedValue((expression+' to result.mjs').split(/\s+/u),'returns '+expression+' to result.mjs',['left','right'],['2','3']);
   if(expression==='left/right')assert.notEqual(value,null);else assert.equal(value,expected);
 });
}
for(const expression of ['left*right + unknown','left*unknown','left*right/0','left*right +','left*right deploy','left*right Set environment','left*right call(unknown)','left*right to ../escape.mjs','left*right to output.mjs extra','left*right to output.mjs and deploy']){
 test('unsupported expression or remainder refuses '+expression,()=>{
   assert.equal(statedValue((expression+(expression.includes(' to ')?'':' to result.mjs')).split(/\s+/u),'returns '+expression,['left','right'],['2','3']),null);
 });
}
test('renamed operands and target path binding remain separate',()=>{
 assert.equal(statedValue(['alpha*beta','to','alpha.mjs'],'returns alpha*beta',['alpha','beta'],['4','5']),'20');
 assert.equal(statedValue(['alpha*beta'],'returns alpha*beta',['alpha','beta'],['4','5']),'20');
});
