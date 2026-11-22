import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { createWorkerContext, evaluate } from './support/browser-runtime.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const directory = new URL('../../../data/benchmarks/llm-task-parity/', import.meta.url);
const read = (file) => readFileSync(new URL(file, directory), 'utf8').trimEnd();
const jsonl = (file) => read(file).split('\n').map(JSON.parse);
const prompts = jsonl('prompts.jsonl');
const fixture = (id, language) => prompts.find((probe) => probe.id === id)[language];
const expected = {
  labels: {
    en: ['commute', '«The office is too far from my home»'],
    ru: ['дорога', '«Офис слишком далеко от дома»'],
  },
  sentiment: {
    en: ['neutral', '«Thank you, but I have already accepted another offer.»'],
    ru: ['нейтральная', '«Спасибо, но я уже приняла другое предложение.»'],
  },
  records: {
    en: ['Anna', '«1. Anna - forklift licence: yes - city: Riverton»'],
    ru: ['Анна', '«1. Анна - права на погрузчик: да - город: Речной»'],
  },
  job: {
    en: ['City: Kazan', '«City: Kazan»', '«Requirements: two years of sales experience and a driving licence»'],
    ru: ['Город: Казань', '«Город: Казань»', '«Требования: опыт продаж от двух лет и водительские права»'],
  },
  language: {
    en: ['Russian', 'Dictionary matches: около, лет, в', '«Около трёх лет, в основном оборудование.»'],
    ru: ['Английский', 'Словарные совпадения: about, three', '«About three years, mostly equipment.»'],
  },
  agreement: {
    en: [null, 'She goes to work every day.', 'https://en.wiktionary.org/wiki/go#Verb'],
    ru: [null, 'Подскажите, пожалуйста, сколько лет вы работаете в продажах?', 'https://ru.wiktionary.org/wiki/работать', 'https://ru.wiktionary.org/wiki/пожалуйста'],
  },
};
const cannedSearch = /No captured provider response|no source[- ]grounded (?:statement|claim)|no captured source|ни одного подтвержд[её]нного источником|не получено ни одного|web_search/i;
const otherLanguages = [
  ['Identify the language: Hola, buenos días. ¿Cómo estás?', ['Spanish', 'Dictionary matches: hola']],
  ['Identify the language: नमस्ते, आप कैसे हैं?', ['Hindi', 'Dictionary matches: नमस्ते']],
  ['Identify the language: 你好，谢谢。', ['Chinese', 'Dictionary matches: 你好, 谢谢']],
];
const localClassification = [
  ['Classify this message as Salary, Commute, Schedule or Other: Wages are low.', ['Salary', '«Wages are low.»']],
  ['Классифицируй замечание как зарплата, дорога, график или другое: Офис слишком далеко.', ['дорога', '«Офис слишком далеко.»']],
  ['Определи тональность (положительная, отрицательная или нейтральная): Мне понравилось.', ['положительная', '«Мне понравилось.»']],
  ['Определи тональность (положительная, отрицательная или нейтральная): Мне не понравилось.', ['отрицательная', '«Мне не понравилось.»']],
];
const additionalExamples = [
  ['Choose one label [wear and tear, Waiting, OTHER]: We are still Waiting.', ['Waiting', '«We are still Waiting.»']],
  ['Choose one label [wear and tear, Waiting, OTHER]: There is wear and tear on the door.', ['wear and tear', '«There is wear and tear on the door.»']],
  ['Choose one label [salary, commute, OTHER]: The wages are low and the office is too far.', ['OTHER']],
  ['Choose one label [salary, commute]: Blue.', ['unknown']],
  ['Records:\nName: Irena\nCity: Bern\n\nName: Jo\nCity: Graz\nQuestion: What city is Jo in?', ['Graz', '«Name: Jo\nCity: Graz»']],
  ['Records:\nCity: Graz\nQuestion: What is the city?', ['Graz', '«City: Graz»']],
  ['Table:\n| ID | Forklift licence |\n| --- | --- |\n| Irena | no |\n| Jo | yes |\nWhich person has no forklift licence?', ['Irena', '«| Irena | no |»', '«| ID | Forklift licence |»']],
  ['Records:\n1. Jo - city: Graz\n2. Jo - city: Bern\nQuestion: What city is Jo in?', ['I could not determine the requested answer from the supplied text.']],
  ['Extract city and schedule:\nCity: Graz\nCity: Bern\nSchedule: Monday', ['city: No unique value in the supplied text.', '«Schedule: Monday»']],
  ['Identify the language: 984 715 —', ['unknown']],
  ['Исправь ошибки: Скажите пожалуйста что вы читаите сейчас?', [null, 'Скажите, пожалуйста, что вы читаете сейчас?', 'https://ru.wiktionary.org/wiki/читать']],
  ...otherLanguages,
  ...localClassification,
];

function assertAnswer(content, [firstLine, ...evidence], label) {
  assert.equal(typeof content, 'string', label);
  if (firstLine) assert.equal(content.split('\n')[0], firstLine, label);
  for (const span of evidence) assert.ok(content.includes(span), `${label}: ${content}`);
  assert.doesNotMatch(content, cannedSearch, label);
}

test('release measurement covers every registered task in Russian and English', () => {
  const registry = readFileSync(new URL('../../../data/meta/llm-task-classes.lino', import.meta.url), 'utf8');
  const classes = [...registry.matchAll(/^  class (\S+)$/gm)].map((match) => match[1]);
  assert.deepEqual(new Set(prompts.map((probe) => probe.class)), new Set(classes));
  const measured = jsonl('v0.356.0.jsonl');
  const expectedResponses = prompts.flatMap(({ class: task, id, en, ru }) => {
    assert.ok(en && ru, id);
    return ['en', 'ru'].map((language) => [task, id, language]);
  });
  assert.deepEqual(measured.map((item) => [item.class, item.id, item.language]), expectedResponses);
  assert.ok(measured.every((item) => item.version === 'v0.356.0' && item.status === 200 && item.elapsedMs >= 0));
  const verdictPairs = read('v0.356.0-verdicts.tsv').split('\n').slice(1)
    .map((line) => line.split('\t').slice(0, 2));
  assert.deepEqual(verdictPairs, prompts.map((probe) => [probe.class, probe.id]));
});

test('worker answers from prompt data and records literal spans with no tools', { timeout: 45000, skip: Boolean(process.env.FORMAL_AI_TEST_NATIVE_CLI) }, async () => {
  const worker = createWorkerContext();
  await evaluate(worker, 'loadSeed()');
  const solve = (prompt) => worker.solve(prompt, [], {}, {}, [], {});
  for (const [id, languages] of Object.entries(expected)) {
    for (const [language, evidence] of Object.entries(languages)) {
      const result = await solve(fixture(id, language));
      assertAnswer(result.content, evidence, `${id}/${language}`);
      assert.equal(result.toolCalls.length, 0);
      if (id !== 'agreement') {
        assert.ok(result.evidence.includes('prompt_data:network:none'));
        assert.equal(result.metaExecution.shape, 'leaf', `${id}/${language}`);
      }
      if (id === 'records') assert.ok(result.evidence.some((item) => item.startsWith('prompt_data:link:')));
    }
  }
  for (const [prompt, evidence] of additionalExamples) {
    const result = await solve(prompt);
    assertAnswer(result.content, evidence, prompt);
    assert.equal(result.toolCalls.length, 0, prompt);
  }
});

test('OpenAI completions answer the issue prompts in default and agent modes', { timeout: 190000 }, async (t) => {
  const binary = process.env.FORMAL_AI_TEST_NATIVE_CLI;
  for (const agentMode of [false, true]) {
    await t.test(`${binary ? 'native' : 'JavaScript'} ${agentMode ? 'agent' : 'default'} mode`, async (subtest) => {
      const run = await startCli(subtest, {
        binary,
        args: agentMode ? ['--agent-mode'] : [],
        env: { FORMAL_AI_LIVE_FETCH: '0' },
        lifetime: 90000,
      });
      await run.ready();
      async function complete(prompt, label) {
        const response = await fetch(run.url + '/v1/chat/completions', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ model: 'formal-ai', messages: [{ role: 'user', content: prompt }] }),
          signal: AbortSignal.timeout(10000),
        });
        const body = await response.json();
        assert.equal(response.status, 200, JSON.stringify(body));
        const choice = body.choices[0];
        assert.equal(choice.finish_reason, 'stop', label);
        assert.deepEqual(choice.message.tool_calls ?? [], [], label);
        return choice.message.content;
      }
      for (const [id, languages] of Object.entries(expected)) {
        for (const [language, evidence] of Object.entries(languages)) {
          const label = `${id}/${language}`;
          await subtest.test(label, async () => {
            assertAnswer(await complete(fixture(id, language), label), evidence, label);
          });
        }
      }
      for (const [prompt, evidence] of additionalExamples) {
        await subtest.test(prompt.replaceAll('\n', ' '), async () => {
          assertAnswer(await complete(prompt, prompt), evidence, prompt);
        });
      }
      for (const id of ['decision', 'sales', 'redaction', 'mobile', 'yaml']) {
        for (const language of ['en', 'ru']) {
          const label = `${id}/${language}`;
          await subtest.test(label, async () => {
            assert.doesNotMatch(await complete(fixture(id, language), label), cannedSearch, label);
          });
        }
      }
    });
  }
});
