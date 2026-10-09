import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {renderBootstrap, SOURCE, OUTPUT} from '../../../scripts/generate-worker-bootstrap.mjs';
import {createWorkerContext, loadWorkerMirror, evaluate, plain} from './support/browser-runtime.mjs';
const loader = readFileSync('js/seed_loader.js','utf8');
const seed = readFileSync(SOURCE,'utf8');
test('shipped bootstrap responses equal their actual canonical seed projection', () => {
  assert.equal(readFileSync(OUTPUT,'utf8'),renderBootstrap(loader,seed));
});
test('bootstrap generator follows changed seed data without source phrase branches', () => {
  const context = vm.createContext({}); context.self=context;
  const fixture='multilingual_responses\n  response arbitrary\n    intent arbitrary\n    language xy\n    text "held out response"\n    variant "another response"\n';
  vm.runInContext(renderBootstrap(loader,fixture),context);
  assert.equal(context.FORMAL_AI_BOOTSTRAP_RESPONSES.arbitrary.xy.text,'held out response');
  assert.deepEqual(Array.from(context.FORMAL_AI_BOOTSTRAP_RESPONSES.arbitrary.xy.variants),['another response']);
});
test('offline worker entry and mirror use the same seeded language responses', () => {
  const entry=createWorkerContext({fetch:()=>Promise.reject(new Error('offline'))});
  const mirror=loadWorkerMirror();
  for(const language of ['en','ru','hi','zh']) {
    assert.deepEqual(plain(evaluate(entry,'MULTILINGUAL_ANSWERS.greeting.'+language)),plain(evaluate(mirror,'MULTILINGUAL_ANSWERS.greeting.'+language)));
    assert.equal(evaluate(entry,'answerFor("greeting", "'+language+'")'),evaluate(entry,'self.FORMAL_AI_BOOTSTRAP_RESPONSES.greeting.'+language+'.text'));
  }
});
