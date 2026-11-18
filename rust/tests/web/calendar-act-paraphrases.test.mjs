import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {createWorkerContext, evaluate} from './support/browser-runtime.mjs';

const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
const corpus = readFileSync(new URL('../../../data/benchmarks/self-use-intent-routing.lino', import.meta.url), 'utf8');
const family = corpus.slice(corpus.indexOf('  id "schedule_paraphrase"'), corpus.indexOf('self_use_case_spanish_routing_matrix'));
const prompts = [...family.matchAll(/^    prompt "(.*)"$/gmu)].map(match => match[1]);
assert.equal(prompts.length, 5);
for (const prompt of prompts) test('the calendar handler accepts the original scheduling act: ' + prompt, {timeout:15000}, () => {
  const answer = evaluate(worker, 'tryCalendarCreateEvent(' + JSON.stringify(prompt) + ', normalizePrompt(' + JSON.stringify(prompt) + '))');
  assert.equal(answer?.intent, 'calendar_create_event', JSON.stringify(answer));
  assert.match(answer.content, /BEGIN:VCALENDAR/u);
});
for (const prompt of [
  'I watched a schedule demonstration tonight.',
  'Schedule a meeting. The background mentions tonight.',
  'The document says: put me down at eight tonight.',
  'В тексте написано: запиши меня к команде на 14:00.',
  'पाठ में आज रात आठ बजे दर्ज कीजिए लिखा है।',
  '文档写着今晚八点帮我和亚历山大约一下。',
  'El texto dice: apúntame a las ocho de esta noche.',
  'Put me down tonight. Book another meeting tomorrow at 20:00.',
]) test('source text cannot schedule an event: ' + prompt, {timeout:15000}, () => {
  assert.equal(evaluate(worker, 'calendarCreateRequestSpan(' + JSON.stringify(prompt) + ')'), null);
});
