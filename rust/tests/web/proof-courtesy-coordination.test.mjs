// Replay unchanged native Euclid request cases and preserve explicit composition.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { formalizeIntentRecord } from '../../../js/agentic/crate/intent_formalization.mjs';
import { tokenize, testFunctions } from '../../../scripts/lib/rust-specification-cases.mjs';
let host;
before(async()=>{host=new WorkerHost();await installNodeHost(host);host.context.fetch=()=>{throw new Error('offline coordination probe');};});
const native=readFileSync(`${REPO_ROOT}/rust/tests/unit/proof_request.rs`,'utf8');
const original=testFunctions(tokenize(native)).find(item=>item.name==='prime_infinitude_proofs_are_supported_for_every_language');
const start=original.body.findIndex(token=>token.text==='cases');
const end=original.body.findIndex((token,index)=>index>start&&token.text===']');
const literals=original.body.slice(start,end).filter(token=>token.kind==='string').map(token=>token.text);
assert.equal(literals.length%4,0);
const cases=Array.from({length:literals.length/4},(_,index)=>literals.slice(index*4,index*4+4));

test('all unchanged native multilingual prime proof requests retain every original proof assertion',async()=>{
  assert.equal(cases.length,4);
  for(const [language,prompt,statement,planHeading] of cases){
    const response=await host.solve(prompt);
    assert.equal(response.intent,'proof_request',language);
    assert.ok(response.content.includes(statement),language);
    assert.ok(response.content.includes('relative-meta-logic')||response.content.includes('Peano')||response.content.includes('Пеано'),language);
    assert.ok(!response.content.includes(planHeading),language);
  }
});

test('an arbitrary greeting before a known directive remains one substantive request',async()=>{
  const response=await host.solve('Hey. Demonstrate that there are infinitely many primes');
  assert.equal(response.intent,'proof_request');
  assert.equal(response.solverEvents.find(event=>event.kind==='impulse').payload,'Hey. Demonstrate that there are infinitely many primes');
  assert.ok(response.content.includes('p₁'));
  assert.ok(response.content.includes('relative-meta-logic'));
  assert.ok(!response.content.includes('Proof plan'));
  for(const [language,prompt] of [['en','Prove that there are infinitely many prime numbers'],['ru','докажи что простых бесконечно']]){
    const binding=formalizeIntentRecord(prompt,language);
    assert.equal(binding.route,'proof_request');
    assert.notEqual(binding.kind,'question');
  }
  assert.equal(formalizeIntentRecord('Hello.','en').kind,'courtesy');
});

test('multiple explicit questions and greeting-prefixed procedural questions still compose',async()=>{
  const first=await host.solve('What is 2 + 2?');
  const second=await host.solve('Who are you?');
  const questions=await host.solve('What is 2 + 2? Who are you?');
  assert.equal(questions.intent,'compound_response');
  assert.equal(questions.content,`${first.content}\n\n${second.content}`);
  const procedure=await host.solve('Привет, как подключить mysql к node js');
  assert.equal(procedure.intent,'compound_response');
  assert.ok(procedure.content.startsWith('Здравствуйте! Чем могу помочь?\n\n'));
  assert.ok(procedure.content.includes('mysql к node js'));
  assert.ok(procedure.evidence.includes('sub_intent:procedural_how_to'));
  assert.equal(formalizeIntentRecord('как подключить mysql к node js','ru').kind,'question');
});

test('the raw courtesy boundary supports unspaced CJK without accepting dotted words',async()=>{
  assert.deepEqual(await host.run('splitLeadingGreetingCompoundPrompt(__prompt)',{__prompt:'你好。请证明任意命题'}),{greeting:'你好',remainder:'请证明任意命题'});
  assert.equal(await host.run('splitLeadingGreetingCompoundPrompt(__prompt)',{__prompt:'Hello.prove an arbitrary claim'}),null);
});
