import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { meaningIndex } from '../../../js/agentic/crate/text_formalization.mjs';
import { isSourceLinksTask } from '../../../js/agentic/source_links.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

test('projection scope reuses universal quantifier identity across statements', () => {
  const english = meaningIndex('en');
  for (const word of ['all', 'every']) {
    assert.equal(english.get(word), 'quantifier_all', word);
  }
  assert.equal(english.get('each'), 'word_problem_unit_price');
  assert.equal(meaningIndex('ru').get('все'), 'quantifier_all');
});

test('projection format preserves the canonical link identity in plural forms', () => {
  const english = meaningIndex('en');
  assert.equal(english.get('link'), 'skill_procedure_object_link');
  assert.equal(english.get('links'), english.get('link'));
  assert.equal(meaningIndex('ru').get('ссылки'), 'skill_procedure_object_link');
});

test('reused meanings keep positive source projection ownership', () => {
  for (const request of [
    'Translate all source to links and back.',
    'Translate the entire source to links and back.',
    'Пересобери исходный код.',
    'पुनः संकलित करो स्रोत।',
    '重新编译源码。',
    'Recompila el código fuente.',
  ]) {
    assert.equal(isSourceLinksTask(request), true, request);
  }
});

test('a cue in data, a path, negation or another sentence grants no recipe', () => {
  for (const request of [
    'Create note.txt containing «Translate all source to links and back».',
    'Read tools/source-links/manifest.txt.',
    'Translate all source. Links and back are discussed separately.',
    'Never recompile the source.',
    'No crees source links.',
  ]) {
    assert.equal(isSourceLinksTask(request), false, request);
  }
});
