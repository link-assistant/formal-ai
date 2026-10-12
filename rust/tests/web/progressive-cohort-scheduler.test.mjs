import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { captureProgressiveInputs } from '../../../scripts/select-progressive-work.mjs';
import { admitProgressivePass, executeProgressivePass, progressiveSchedulerBindings } from '../../../experiments/formal_ai_subagent/progressive-cohort-scheduler.mjs';
import { PLAN_PATH } from '../../../js/agentic/general_planner.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const binding = path => ({ path, sha256: hash(readFileSync(path)) });
const key = id => JSON.stringify(['docs/requirements/fixture.md', id]);
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'progressive-execution-'));
  const directory = join(root, 'data/meta/requirement-status-ledger');
  mkdirSync(directory, { recursive: true });
  const ledgerPath = join(directory, 'fixture.lino');
  writeFileSync(ledgerPath, 'requirement_status_ledger_shard\n  shard "docs/requirements/fixture.md"\n'
    + '  requirement\n    id "lower"\n    verdict "not-delivered"\n    automated_test ""\n'
    + '  requirement\n    id "higher"\n    verdict "partial"\n    automated_test "rust/tests/web/fixture.test.mjs"\n');
  const captured = captureProgressiveInputs(root);
  const cases = ['lower', 'higher'].map(id => {
    const workspace = join(root, id); mkdirSync(workspace);
    const oraclePath = join(root, id + '.oracle.mjs');
    writeFileSync(oraclePath, "import assert from 'node:assert/strict';\nimport { readFileSync } from 'node:fs';\nimport { join } from 'node:path';\nassert.equal(readFileSync(join(process.env.COHORT_WORKSPACE, 'result.txt'), 'utf8'), 'hello');\n");
    const task = 'create file result.txt containing hello';
    return { runId: id, requirementKey: key(id), task, taskSHA256: hash(task),
      taskKind: 'coding', category: 'test-implementation', expectedRelation: 'output-larger',
      workspace, allowedEffects: [join(workspace, 'result.txt'), join(workspace, PLAN_PATH)], oracle: binding(oraclePath), timeoutMilliseconds: 1000 };
  });
  const manifest = { schemaVersion: 1, cohortId: 'progressive-instrumentation-only', cases,
    bindings: [...progressiveSchedulerBindings(), ...captured.sources.map(source => binding(resolve(root, source.path))), ...cases.map(item => item.oracle)] };
  const journalPath = join(root, 'cohort.jsonl');
  const passPath = join(root, 'pass.json');
  return { root, ledgerPath, captured, manifest, journalPath, passPath,
    admit: () => admitProgressivePass(root, captured, manifest, journalPath, passPath),
    clean: () => rmSync(root, { recursive: true, force: true }) };
}

test('source-selected lowest task performs real exclusive creation and closed oracle acceptance', async () => {
  const f = fixture();
  try {
    const token = f.admit();
    const out = await executeProgressivePass(token, 'physical-first');
    assert.equal(out.level, 1);
    assert.equal(out.allAtLowestLevel, 1);
    assert.equal(out.accepted, true);
    assert.deepEqual(out.results.map(item => item.runId), ['lower']);
    assert.equal(out.results[0].result.transcript.find(item => item.tool === 'write').source_creation.success, true);
    assert.equal(readFileSync(join(f.root, 'lower/result.txt'), 'utf8'), 'hello');
    assert.equal(existsSync(join(f.root, 'higher/result.txt')), false);
    assert.equal(out.requirementsPromoted, false);
    assert.equal(out.fullAcceptance, false);
    assert.equal(out.results[0].result.semanticCompleteness, 'Unknown');
    assert.equal(out.results[0].result.usage.status, 'Unknown');
    assert.deepEqual(captureProgressiveInputs(f.root), f.captured);
    assert.deepEqual(readFileSync(f.journalPath, 'utf8').trim().split('\n').map(JSON.parse).map(row => row.kind), ['admitted', 'started', 'finished']);
    await assert.rejects(executeProgressivePass(token, 'physical-first'), /fresh/);
  } finally { f.clean(); }
});

test('missing lowest task remains Missing and never substitutes available higher work', async () => {
  const f = fixture();
  try {
    f.manifest.cases = f.manifest.cases.filter(item => item.runId === 'higher');
    const out = await executeProgressivePass(f.admit(), 'blocked-first');
    assert.equal(out.accepted, false);
    assert.deepEqual(out.results, []);
    assert.deepEqual(out.missing.map(item => item.requirementKey), [key('lower')]);
    assert.equal(out.missing[0].status, 'Missing');
    assert.equal(existsSync(join(f.root, 'higher/result.txt')), false);
    assert.equal(readFileSync(f.journalPath, 'utf8').trim().split('\n').length, 1);
    const finish = JSON.parse(readFileSync(f.passPath + '.attempt-' + hash('blocked-first') + '.finished.json'));
    assert.equal(finish.outcome.accepted, false);
    assert.equal(finish.outcome.missing[0].status, 'Missing');
  } finally { f.clean(); }
});

test('unsupported lowest request retains real failed driver attempt and never advances a level', async () => {
  const f = fixture();
  try {
    f.manifest.cases[0].task = 'Implement a new unknown protocol with unavailable capabilities.';
    f.manifest.cases[0].taskSHA256 = hash(f.manifest.cases[0].task);
    const out = await executeProgressivePass(f.admit(), 'unsupported-first');
    assert.equal(out.accepted, false);
    assert.equal(out.results.length, 1);
    assert.equal(out.results[0].result.accepted, false);
    assert.equal(out.results[0].status, 'Missing');
    assert.equal(out.missing.some(item => item.requirementKey === key('lower')), true);
    assert.equal(existsSync(join(f.root, 'higher/result.txt')), false);
    const rows = readFileSync(f.journalPath, 'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(rows.at(-1).kind, 'finished');
    assert.equal(rows.at(-1).result.accepted, false);
    assert.deepEqual(captureProgressiveInputs(f.root), f.captured);
  } finally { f.clean(); }
});

test('forged admission, altered pass and ledger drift refuse before task effects and retain failures', async () => {
  await assert.rejects(executeProgressivePass({}, 'forged'), /private/);
  for (const changed of ['pass', 'ledger']) {
    const f = fixture();
    try {
      const token = f.admit();
      if (changed === 'pass') writeFileSync(f.passPath, '{}');
      else writeFileSync(f.ledgerPath, readFileSync(f.ledgerPath, 'utf8').replace('not-delivered', 'implemented'));
      await assert.rejects(executeProgressivePass(token, 'drift-first'), /drift/);
      assert.equal(existsSync(join(f.root, 'lower/result.txt')), false);
      const finish = JSON.parse(readFileSync(f.passPath + '.attempt-' + hash('drift-first') + '.finished.json'));
      assert.equal(finish.outcome.accepted, false);
      assert.match(finish.outcome.failure.message, /drift/);
    } finally { f.clean(); }
  }
});

test('missing ledger binding, duplicate ownership and unknown requirement never admit work', () => {
  for (const changed of ['binding', 'duplicate', 'unknown']) {
    const f = fixture();
    try {
      if (changed === 'binding') f.manifest.bindings = f.manifest.bindings.filter(item => item.path !== f.ledgerPath);
      if (changed === 'duplicate') f.manifest.cases[1].requirementKey = f.manifest.cases[0].requirementKey;
      if (changed === 'unknown') f.manifest.cases[0].requirementKey = key('unknown');
      assert.throws(f.admit, /ledger|duplicate|unknown/);
      assert.equal(existsSync(f.passPath), false);
      assert.equal(existsSync(f.journalPath), false);
    } finally { f.clean(); }
  }
});


test('omitting the source-owned plan event effect remains a measured scope failure', async () => {
  const f = fixture();
  try {
    f.manifest.cases[0].allowedEffects = [join(f.root, 'lower/result.txt')];
    const out = await executeProgressivePass(f.admit(), 'undeclared-plan-first');
    assert.equal(out.accepted, false);
    assert.match(out.results[0].result.failure.message, /out-of-scope/);
    assert.equal(out.results[0].result.sourceEffects.some(effect => effect.path.endsWith(PLAN_PATH)), true);
    assert.equal(out.results[0].result.sourceEffects.some(effect => effect.path.endsWith('/result.txt')), true);
    assert.equal(existsSync(join(f.root, 'higher/result.txt')), false);
  } finally { f.clean(); }
});


test('public source binding metadata cannot mutate private loaded identities', () => {
  const first = progressiveSchedulerBindings();
  const original = first[0].sha256;
  first[0].sha256 = '0'.repeat(64);
  assert.equal(progressiveSchedulerBindings()[0].sha256, original);
  const f = fixture();
  try {
    f.manifest.bindings = f.manifest.bindings.filter(item => !item.path.endsWith('/scripts/select-progressive-work.mjs'));
    assert.throws(f.admit, /known runtime source/);
    assert.equal(existsSync(f.passPath), false);
  } finally { f.clean(); }
});


test('frozen scheduling and cohort metadata cannot be placed in a task workspace', () => {
  for (const changed of ['pass', 'journal', 'same']) {
    const f = fixture();
    try {
      const journal = changed === 'journal' ? join(f.root, 'lower/cohort.jsonl') : f.journalPath;
      const pass = changed === 'pass' ? join(f.root, 'lower/pass.json') : changed === 'same' ? journal : f.passPath;
      assert.throws(() => admitProgressivePass(f.root, f.captured, f.manifest, journal, pass), /outside|distinct/);
      assert.equal(existsSync(pass), false);
      assert.equal(existsSync(journal), false);
    } finally { f.clean(); }
  }
});
