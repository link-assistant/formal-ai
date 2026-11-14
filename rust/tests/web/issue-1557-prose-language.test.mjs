// Issue #1557: an English work item must not be answered in Spanish or
// Russian. Markers of a script several languages share match whole words
// only, and a work item's language is decided by the proportion of its prose
// (code, quotations, URLs and identifiers excluded), preferring the user's own
// turn, then the issue body, then its title.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { detect, detectProse, proseOf } from '../../../js/agentic/crate/language.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import {
  composeGeneralChangePlan,
  GeneralPlanMode,
  planResponseLanguage,
  plannedNotExecutedAnswer,
} from '../../../js/agentic/general_planner.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const fixture = JSON.parse(
  readFileSync(new URL('../fixtures/issue-1557-attempted-issues.json', import.meta.url), 'utf8'),
);
const issue = (number) => fixture.attempted.find((item) => item.number === number);
const workItemPlan = (goal, requestLanguage = null) => ({
  mode: GeneralPlanMode.RepositoryWorkItem,
  goal,
  target: 'https://github.com/link-assistant/formal-ai/issues/904',
  request_language: requestLanguage,
});

test('the fixture holds the 19 attempted issues and the synthetic cases the issue names', () => {
  assert.equal(fixture.attempted.length, 19);
  for (const number of [1235, 1330, 1396, 1446, 1548]) assert.ok(issue(number), `issue ${number}`);
  assert.ok(fixture.synthetic.some((item) => item.expected === 'ru'));
});

test('every attempted English issue body is detected as English prose', () => {
  for (const { number, title, body } of fixture.attempted) {
    const language = detectProse(body) ?? detectProse(title);
    assert.equal(language, 'en', `issue ${number}`);
  }
});

test('issues 1235, 1446 and 1548 no longer read as Spanish, even by whole-text detection', () => {
  for (const number of [1235, 1446, 1548]) {
    const { title, body } = issue(number);
    assert.equal(detectProse(body), 'en', `issue ${number} body`);
    assert.notEqual(detect(body), 'es', `issue ${number} detect(body)`);
    assert.equal(planResponseLanguage(workItemPlan(`${title}\n\n${body}`)), 'en', `issue ${number} plan`);
  }
});

test('issues 1330 and 1396 are English work items; a fully Russian issue is Russian', () => {
  for (const number of [1330, 1396]) {
    const { title, body } = issue(number);
    assert.equal(planResponseLanguage(workItemPlan(`${title}\n\n${body}`)), 'en', `issue ${number}`);
  }
  // Issue 1396's title alone is Russian; the body decides before the title.
  assert.equal(detectProse(issue(1396).title), 'ru');
  const russian = fixture.synthetic.find((item) => item.case === 'a fully Russian issue');
  assert.equal(planResponseLanguage(workItemPlan(`${russian.title}\n\n${russian.body}`)), 'ru');
});

test('each synthetic case is detected as its expected language', () => {
  for (const item of fixture.synthetic) {
    assert.equal(detectProse(item.body), item.expected, item.case);
    assert.equal(planResponseLanguage(workItemPlan(`${item.title}\n\n${item.body}`)), item.expected, item.case);
  }
});

test('no Latin-script marker matches as the prefix of a longer word', () => {
  for (const text of [
    'formalization described response',
    'The formalization is described in the response.',
    'Responsive formalizations get described',
  ]) {
    assert.equal(detect(text), 'en', text);
    assert.equal(detectProse(text), 'en', text);
  }
  // A whole marker still counts.
  assert.equal(detect('formaliza esto, por favor'), 'es');
  assert.equal(detect('responde en español'), 'es');
});

test('a single marker or a quotation does not outvote the rest of the prose', () => {
  assert.equal(detectProse('Please describe how the escribe command and the build pipeline should behave together.'), 'en');
  assert.equal(detectProse('Fix the greeting: the bot said «Привет, как дела?» and then stopped answering.'), 'en');
  assert.equal(proseOf('See `код` and "цитата" at https://пример.рф/путь'), 'See and at');
});

test('the existing es, ru, hi and zh detections are unchanged', () => {
  const cases = [
    ['¿Qué es la formalización?', 'es'],
    ['Привет, как дела?', 'ru'],
    ['नमस्ते, आप कैसे हैं?', 'hi'],
    ['你好，你是谁？', 'zh'],
    ['介绍一下 julián andrés quiñones?', 'zh'],
    ['Hello, how are you?', 'en'],
  ];
  for (const [text, expected] of cases) {
    assert.equal(detect(text), expected, text);
    assert.equal(detectProse(text), expected, text);
  }
});

test('the user turn outranks the body, and the body outranks the title', () => {
  const { title, body } = issue(1548);
  assert.equal(planResponseLanguage(workItemPlan(`${title}\n\n${body}`, 'ru')), 'ru');
  assert.equal(planResponseLanguage(workItemPlan(`Не помог...\n\n${body}`)), 'en');
  assert.equal(planResponseLanguage(workItemPlan('Не помог...\n\nhttps://example.com/only-a-link')), 'ru');
  const url = 'https://github.com/link-assistant/formal-ai/issues/904';
  const russianTurn = composeGeneralChangePlan(`Пожалуйста, реши задачу ${url}`);
  assert.equal(russianTurn?.request_language, 'ru');
  assert.equal(composeGeneralChangePlan(`${url} please fix it`)?.request_language ?? null, null);
});

test('the planned answer for an English body naming formalization stays English', () => {
  const answer = plannedNotExecutedAnswer({
    ...workItemPlan('The formalization described here returns the wrong response.'),
    id: 'issue-1557',
    content: '',
    steps: [],
    verification_command: '',
    terminal_state: 'planned_not_executed',
  });
  assert.match(answer, /^Planned, not executed: /u);
});

test('a fetched English issue body is answered in English end to end', async () => {
  const url = 'https://github.com/link-assistant/formal-ai/issues/904';
  const request = `${url}\n\nProceed.`;
  const body = issue(1548).body;
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-issue-1557-'));
  const messages = [{ role: 'user', content: request }];
  try {
    let final = null;
    for (let turn = 0; turn < 12 && final === null; turn++) {
      const step = await planChatStep(messages, ['fetch_url', 'read', 'write']);
      assert.ok(step);
      if (step.kind === 'final') {
        final = step.answer;
        break;
      }
      for (const call of step.calls) {
        const id = `replay-${messages.length}`;
        messages.push({
          role: 'assistant',
          content: '',
          tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }],
        });
        const receipt = call.tool === 'fetch_url' ? { content: body, is_error: false } : executeResult(directory, call);
        messages.push({ role: 'tool', tool_call_id: id, name: call.tool, ...receipt });
      }
    }
    assert.ok(final, 'the replay reaches a final answer');
    // Before the fix this answer opened with "Planificado, no ejecutado".
    assert.match(final, /^Planned, not executed: /u);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

// The browser worker's no-WASM fallback reads the same seed registry; its
// fallback-script markers must match whole words too.
function workerDetector() {
  const source = readFileSync(new URL('../../../js/worker/formal_ai_worker_seed_responses_and_language.js', import.meta.url), 'utf8');
  const start = source.indexOf('function detectLanguageFromRules');
  const end = source.indexOf('\n\n// Issue #324', start);
  assert.ok(start >= 0 && end > start, 'the worker fallback detector is present');
  const seed = readFileSync(new URL('../../../data/seed/language-detection.lino', import.meta.url), 'utf8');
  const rules = [];
  for (const line of seed.split('\n').map((text) => text.trim())) {
    if (line.startsWith('rule ')) rules.push({ markers: [] });
    const rule = rules.at(-1);
    if (!rule) continue;
    const [key, ...rest] = line.split(' ');
    const value = rest.join(' ');
    if (key === 'language' || key === 'script') rule[key] = value;
    if (key === 'start' || key === 'end') rule[key] = Number(value);
    if (key === 'markers') rule.markers = [...value.matchAll(/"([^"]*)"/gu)].map((match) => match[1]);
    if (line === 'fallback yes') rule.fallback = true;
    if (line === 'alphabetic-only yes') rule.alphabeticOnly = true;
  }
  return new Function('LANGUAGE_RULES', `${source.slice(start, end)}\nreturn detectLanguageFromRules;`)(rules);
}

test('the worker fallback detector matches fallback-script markers as whole words', () => {
  const detectInWorker = workerDetector();
  for (const text of ['formalization described response', issue(1548).body]) {
    assert.equal(detectInWorker(text), 'en', text.slice(0, 40));
  }
  for (const [text, expected] of [
    ['¿Qué es la formalización?', 'es'],
    ['formaliza esto, por favor', 'es'],
    ['Привет, как дела?', 'ru'],
    ['你好吗', 'zh'],
    ['hello there', 'en'],
  ]) {
    assert.equal(detectInWorker(text), expected, text);
  }
});
