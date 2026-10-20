import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import test from 'node:test';
import {workerModuleInventory, generatedFiles} from '../../../scripts/generate-worker-crate-modules.mjs';
import {REPO_ROOT} from '../../../js/server/lino.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';

const current = '// Keep the owned comments.\nself.FORMAL_AI_WORKER_MODULES = Object.freeze([\n]);\n';
test('new factories are registered before creation and obsolete generated modules disappear', () => {
  const generated = new Map([['js/worker/formal_ai_worker_crate_new.js', 'new factory']]);
  const result = workerModuleInventory(current, [
    'formal_ai_worker.js', 'notes.txt', 'z.js', 'a.js',
    'formal_ai_worker_crate_old.js', 'formal_ai_worker_crate_modules.js',
  ], generated);
  assert.ok(result.startsWith('// Keep the owned comments.\n'));
  assert.deepEqual(result.match(/"worker\/[^"\n]+"/gu), [
    '"worker/a.js"', '"worker/formal_ai_worker_crate_modules.js"',
    '"worker/formal_ai_worker_crate_new.js"', '"worker/z.js"',
  ]);
  assert.equal(workerModuleInventory(result, ['z.js', 'a.js', 'formal_ai_worker_crate_modules.js'], generated), result);
  assert.throws(() => workerModuleInventory('missing owner marker', [], generated), /Malformed worker module inventory/u);
});

test('the generated inventory exactly matches real regular modules and the worker loads canonical callbacks', async () => {
  const expected = generatedFiles().get('js/worker-modules.js');
  assert.equal(readFileSync(REPO_ROOT + '/js/worker-modules.js', 'utf8'), expected);
  const actual = (expected.match(/"worker\/[^"\n]+"/gu) ?? []).map(value => JSON.parse(value));
  const physical = readdirSync(REPO_ROOT + '/js/worker', {withFileTypes: true})
    .filter(entry => entry.isFile() && entry.name.endsWith('.js') && entry.name !== 'formal_ai_worker.js')
    .map(entry => 'worker/' + entry.name).sort();
  assert.deepEqual(actual, physical);
  const worker = new WorkerHost();
  const realm = await worker.boot();
  assert.equal(typeof realm.FORMAL_AI_CRATE_FACTORIES['callback_values.mjs'], 'function');
});
