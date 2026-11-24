import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, existsSync} from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import {fileURLToPath} from 'node:url';
import {durationLookup, partitionProblems, planShards} from '../../scripts/lib/ci-speed-shards.mjs';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';

const languages = readdirSync(new URL('../../data/parity/issue-1584-cnl/', import.meta.url))
  .filter(name => name.endsWith('.json')).map(name => name.slice(0, -5)).sort();
const owners = ['cnl-solver-dialogue', 'cnl-solver-dialogue-b', 'cnl-solver-dialogue-c']
  .map(name => 'rust/tests/web/' + name + '.test.mjs');


test('registered real dialogue cases are conserved across bounded maintained owners', async () => {
  const helper = new URL('../../rust/tests/web/helpers/cnl-solver-dialogue.mjs', import.meta.url);
  const original = new URL('../../rust/tests/web/cnl-solver-dialogue.test.mjs', import.meta.url);
  const selected = existsSync(helper) ? helper : original;
  const raw = readFileSync(selected, 'utf8');
  const registered = [];
  const source = raw.replace(/^import [^\n]*\n/gmu, '').replace(/^export /gmu, '')
    .replaceAll('import.meta.dirname', JSON.stringify(path.dirname(fileURLToPath(selected))));
  const context = {
    test: name => registered.push(name), assert, path, readFileSync,
    fixtureLanguages: () => languages,
    parityPath: language => 'data/parity/issue-1584-cnl/' + language + '.json',
    createWorkerContext: () => ({}), evaluate: async () => {}, plain: value => value,
  };
  const groups = [];
  for (const shard of existsSync(helper) ? [0, 1, 2] : [0]) {
    registered.length = 0;
    const partition = existsSync(helper)
      ? await import('../../rust/tests/web/helpers/cnl-dialogue-partition.mjs') : {};
    await vm.runInNewContext('(async () => {\n' + source
      + (existsSync(helper) ? '\nregisterCnlDialogueCases(' + shard + ');' : '') + '\n})()',
    {...context, ...partition}, {timeout: 1000});
    groups.push(registered.filter(name => name.includes('worker answers every controlled dialogue'))
      .map(name => name.split(':')[0]));
  }
  assert.deepEqual(groups.flat().sort(), languages);
  assert(groups.every(group => group.length <= 2), 'one original owner registers all five expensive language cases');
});

test('all fixture languages belong to exactly one bounded dialogue owner', async () => {
  const {dialogueLanguagesForShard} = await import('../../rust/tests/web/helpers/cnl-dialogue-partition.mjs');
  for (const input of [languages, [...languages, 'future-a', 'future-b', 'future-c']]) {
    const groups = [0, 1, 2].map(shard => dialogueLanguagesForShard(input, shard));
    assert.deepEqual(groups.flat().sort(), [...input].sort());
    assert.equal(new Set(groups.flat()).size, input.length);
    assert(groups.every(group => group.length <= Math.ceil(input.length / 3)));
  }
  assert.deepEqual([0, 1, 2].map(shard => dialogueLanguagesForShard(languages, shard)),
    [['en', 'ru'], ['es', 'zh'], ['hi']]);
});

test('invalid owner indices cannot silently discard every fixture', async () => {
  const {dialogueLanguagesForShard} = await import('../../rust/tests/web/helpers/cnl-dialogue-partition.mjs');
  for (const shard of [-1, 3, 0.5, NaN, '0', undefined])
    assert.throws(() => dialogueLanguagesForShard(languages, shard), /dialogue shard/u);
});

test('maintained entries select every owner once and the deployed planner separates them', () => {
  for (const [shard, file] of owners.entries()) {
    const source = readFileSync(new URL('../../' + file, import.meta.url), 'utf8');
    const selected = [];
    vm.runInNewContext(source.replace(/^import [^\n]*\n/gmu, ''), {
      test: () => {}, assert,
      registerCnlDialogueCases: index => selected.push(index),
    }, {timeout: 1000});
    assert.deepEqual(selected, [shard], file + ' registers exactly its own language group');
  }
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url))
    .filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const weights = parseTestDurations(recorded);
  const secondsOf = durationLookup(weights, Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1]));
  const {shards} = planShards(files, 9, secondsOf);
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  assert.equal(new Set(owners.map(file => shards.findIndex(shard => shard.includes(file)))).size, 3);
  for (const file of owners) assert(weights.get(file) >= 284, file + ' retains the original aggregate upper bound');
});
