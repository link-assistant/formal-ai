import test from 'node:test';
import assert from 'node:assert/strict';
import {
mkdtempSync,mkdirSync,writeFileSync,readFileSync,symlinkSync,existsSync,rmSync}
 from 'node:fs';
import {
tmpdir}
 from 'node:os';
import {
join}
 from 'node:path';
import {
exclusiveCreate}
 from '../../../experiments/js_dogfood/exclusive-create.mjs';
function fixture(run){
const root=mkdtempSync(join(tmpdir(),'pr1188-create-physical-'));
try{
run(root);
}
finally{
rmSync(root,{
recursive:true,force:true}
);
}
}
for(const content of ['hello','\ufeffalpha\nβ','Error: ENOENT is authored data',''])test('exclusive actual bytes '+JSON.stringify(content),()=>fixture(root=>{
const receipt=exclusiveCreate(root,'note.txt',content);
assert.equal(receipt.source_creation?.success,true);
assert.equal(receipt.source_creation?.external_namespace_stability,'unknown');
assert.equal(readFileSync(join(root,'note.txt'),'utf8'),content);
}
));
for(const existing of ['old','hello','  '])test('existing leaf retains '+JSON.stringify(existing),()=>fixture(root=>{
writeFileSync(join(root,'note.txt'),existing);
const receipt=exclusiveCreate(root,'note.txt','hello');
assert.equal(receipt.is_error,true);
assert.equal(receipt.source_creation?.success,false);
assert.equal(receipt.source_creation?.error_code,'EEXIST');
assert.equal(readFileSync(join(root,'note.txt'),'utf8'),existing);
}
));
test('dangling leaf never creates outside',()=>fixture(root=>{
symlinkSync('../outside.txt',join(root,'note.txt'));
const receipt=exclusiveCreate(root,'note.txt','hello');
assert.equal(receipt.is_error,true);
assert.equal(receipt.source_creation?.error_code,'EEXIST');
assert.equal(existsSync(join(root,'../outside.txt')),false);
}
));
test('parent alias refuses without outside effects',()=>fixture(root=>{
mkdirSync(join(root,'outside'));
symlinkSync('outside',join(root,'alias'));
const receipt=exclusiveCreate(root,'alias/note.txt','hello');
assert.equal(receipt.is_error,true);
assert.equal(existsSync(join(root,'outside/note.txt')),false);
}
));
for(const path of ['../note.txt','./note.txt','a//note.txt'])test('noncanonical path refuses '+path,()=>fixture(root=>{
const receipt=exclusiveCreate(root,path,'hello');
assert.equal(receipt.is_error,true);
assert.equal(receipt.source_creation?.success,false);
}
));
