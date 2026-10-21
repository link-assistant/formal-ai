import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost,childrenNamed} from '../../../js/agentic/host.mjs';
import {modePathsForClause} from '../../../js/agentic/file_read/ownership.mjs';
await installNodeHost(new WorkerHost());
const seedPath='data/seed/meanings-file-write.lino';

test('compiled read forms preserve exact source case quote and domain guards across requests',()=>{
 const cases=[
  ['Read src/first.txt.',['src/first.txt']],
  ['Show the first line of nested/Case.txt.',['nested/Case.txt']],
  ["Read 'src/Case.txt'.",['src/Case.txt']],
  ['value of APP.mode in config.json',['config.json']],
  ['Read lines 2 to 4 of src/value.txt.',['src/value.txt']],
  ['value of A/B in config.json',null],
  ['value of K in config.json',null],
  ['Read src/x.txt. Delete src/y.txt.',null],
  ['Reference material remains unchanged between requests.',null],
 ];
 for(let repeat=0;repeat<3;repeat++)for(const [request,expected]of cases)assert.deepEqual(modePathsForClause(request),expected,request);
});

test('grammar projection is reused without retaining prompts and changes with installed host',()=>{
 const original=host();let reads=0;
 try{
  installHost({...original,readText(path){if(path===seedPath)reads++;return original.readText(path);}});
  assert.equal(modePathsForClause(''),null);const first=reads;assert(first>0);
  for(let attempt=0;attempt<10;attempt++)assert.equal(modePathsForClause(''),null);
  assert.equal(reads,first);
  assert.deepEqual(modePathsForClause('Read current.txt.'),['current.txt']);
  const text=original.readText(seedPath);
  installHost({...original,parseLino(value){
   const model=structuredClone(original.parseLino(value));
   if(value===text)childrenNamed(model,'file-read-mode-contract')[0].name='disabled-read-mode-contract';
   return model;
  }});
  assert.equal(modePathsForClause('Read current.txt.'),null);
 }finally{installHost(original);}
});

test('invalid static capture domain refuses only that form after grammar compilation',()=>{
 const original=host(),text=original.readText(seedPath);
 try{
  installHost({...original,parseLino(value){
   const model=structuredClone(original.parseLino(value));
   if(value===text){
    const contract=childrenNamed(model,'file-read-mode-contract')[0];
    const form=childrenNamed(contract,'form').find(form=>form.value==='value');
    const domain=childrenNamed(form,'capture-domain')[0];
    const pattern=childrenNamed(domain,'pattern')[0];pattern.value='[';pattern.id='[';
   }
   return model;
  }});
  assert.equal(modePathsForClause('value of APP.mode in config.json'),null);
  assert.deepEqual(modePathsForClause('Read current.txt.'),['current.txt']);
 }finally{installHost(original);}
});


test('sentence spans preserve astral letters, lone surrogates and punctuation', async () => {
  const {sentences, proseSentences} = await import('../../../js/agentic/shell_command_policy.mjs');
  const source = 'A.𝒜 next. 😀 end; tail?';
  assert.deepEqual(sentences(source), [
    {text: 'A.𝒜 next', span: {start: 0, end: 10}},
    {text: '😀 end', span: {start: 10, end: 18}},
    {text: 'tail', span: {start: 18, end: 24}},
  ]);
  assert.deepEqual(proseSentences('one; two. three'), [
    {text: 'one; two', span: {start: 0, end: 9}},
    {text: 'three', span: {start: 9, end: 15}},
  ]);
  for (const text of ['', 'ASCII', '𝒜😀', '\ud800A\udc00', 'x\ud800\udc00y', '\udc00\ud800']) {
    for (let at = 0; at <= text.length; at++) {
      assert.equal(Array.from(text.slice(at, at + 2))[0], Array.from(text.slice(at))[0]);
      assert.equal(Array.from(text.slice(Math.max(0, at - 2), at)).pop(),
        Array.from(text.slice(0, at)).pop());
    }
  }
});
