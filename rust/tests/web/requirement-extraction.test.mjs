// R1188-U20: requirement extraction lists the obligations an issue states,
// in every seeded language, from seed vocabulary alone.

import { test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { extractRequirements, requirementUnits } from '../../../js/agentic/crate/requirement_extraction.mjs';
import { measure, readFloors } from '../../../scripts/measure-requirement-extraction.mjs';

await installNodeHost(new WorkerHost());

const ISSUE = [
  '## Context',
  'The cache was added last year. It works well.',
  '',
  'The page must load in under a second. Add a progress bar to the upload form.',
  '',
  '```js',
  'must(notBeRead);',
  '```',
  '> We should ignore quoted text.',
  '',
  '## Acceptance criteria',
  '- The upload resumes after a reload',
  '- [ ] Errors are shown in the user\'s language',
  '',
  '## Notes',
  '- The page MUST load in under a second!',
  '- Something nice to know',
].join('\n');

test('obligations, directives, checklists and requirement sections are listed once, in order', () => {
  assert.deepEqual(extractRequirements(ISSUE), [
    'The page must load in under a second.',
    'Add a progress bar to the upload form.',
    'The upload resumes after a reload',
    'Errors are shown in the user\'s language',
  ]);
});

test('code, quotes and headings are never units', () => {
  const texts = requirementUnits(ISSUE).map((unit) => unit.text);
  assert.equal(texts.some((text) => text.includes('notBeRead')), false);
  assert.equal(texts.some((text) => text.includes('quoted')), false);
  assert.equal(texts.some((text) => text.includes('Acceptance')), false);
});

test('every seeded language states requirements', () => {
  assert.deepEqual(extractRequirements('Страница должна загружаться быстро. Это просто заметка.'), [
    'Страница должна загружаться быстро.',
  ]);
  assert.deepEqual(extractRequirements('页面必须在一秒内加载。这是背景。'), ['页面必须在一秒内加载。']);
  assert.deepEqual(extractRequirements('La página debe cargar rápido. Es una nota.'), ['La página debe cargar rápido.']);
  assert.deepEqual(extractRequirements('पेज को जल्दी लोड होना चाहिए। यह एक नोट है।'), ['पेज को जल्दी लोड होना चाहिए।']);
});

test('a definition of done states a requirement in every seeded language', () => {
  assert.deepEqual(extractRequirements('Fixed means: every node carries a record. The loop is old.'), [
    'Fixed means: every node carries a record.',
  ]);
  assert.deepEqual(extractRequirements('Исправлено значит: каждый узел хранит запись. Цикл старый.'), [
    'Исправлено значит: каждый узел хранит запись.',
  ]);
  assert.deepEqual(extractRequirements('完成标准：每个节点都有记录。循环很旧。'), ['完成标准：每个节点都有记录。']);
  assert.deepEqual(extractRequirements('Arreglado significa: cada nodo guarda un registro. El bucle es viejo.'), [
    'Arreglado significa: cada nodo guarda un registro.',
  ]);
  assert.deepEqual(extractRequirements('पूरा तब माना जाएगा जब हर नोड रिकॉर्ड रखे। लूप पुराना है।'), [
    'पूरा तब माना जाएगा जब हर नोड रिकॉर्ड रखे।',
  ]);
});

test('the issue benchmark keeps its floors', () => {
  const score = measure();
  const floors = readFloors();
  assert.ok(score.recall >= floors.recall, `recall ${score.recall} < ${floors.recall}`);
  assert.ok(score.precision >= floors.precision, `precision ${score.precision} < ${floors.precision}`);
});
