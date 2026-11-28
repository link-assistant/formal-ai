import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {renderBootstrap, readBootstrapSeeds, OUTPUT} from '../../../scripts/generate-worker-bootstrap.mjs';
import {createWorkerContext, loadWorkerMirror, evaluate, plain} from './support/browser-runtime.mjs';
const loader = readFileSync('js/seed_loader.js','utf8');
const seed = readBootstrapSeeds(new URL('../../../',import.meta.url).pathname);
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

test('bootstrap projection preserves every field while merging response shards', () => {
  const context = vm.createContext({}); context.self = context;
  const first = 'multilingual_responses\n  response first\n    intent arbitrary\n    language en\n    text "one"\n    variant "two"\n    ack_variant "three"\n    follow_up_variant "four"\n';
  const second = 'multilingual_responses\n  response second\n    intent arbitrary\n    language ru\n    text "five"\n';
  vm.runInContext(renderBootstrap(loader,[first,second]),context);
  assert.deepEqual(JSON.parse(JSON.stringify(context.FORMAL_AI_BOOTSTRAP_RESPONSES.arbitrary)), {
    en: {text:'one',variants:['two'],acknowledgements:['three'],followUps:['four']},
    ru: {text:'five',variants:['five'],acknowledgements:[],followUps:[]},
  });
});
test('offline entry retains moved administration responses in all original languages', () => {
  const context = createWorkerContext({fetch:()=>Promise.reject(new Error('offline'))});
  for (const language of ['en','ru','hi','zh']) {
    assert.equal(evaluate(context,'answerFor("lexeme_import_refused_batch", "'+language+'")'),
      evaluate(context,'self.FORMAL_AI_BOOTSTRAP_RESPONSES.lexeme_import_refused_batch.'+language+'.text'));
    assert(evaluate(context,'self.FORMAL_AI_BOOTSTRAP_RESPONSES.budget_search_solution.'+language+'.text').includes('{expression}'));
  }
});
