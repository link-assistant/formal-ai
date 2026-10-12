// Source-selected progressive execution; fixture acceptance never promotes requirements.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, relative, isAbsolute, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { captureProgressiveInputs, planProgressiveWork } from '../../scripts/select-progressive-work.mjs';
import { admitManifest, executeAdmittedCase, digest, validateManifest } from './cohort-runner.mjs';
import { drive } from '../js_dogfood/drive.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';

const admittedPasses = new WeakMap();
const requirementKey = record => JSON.stringify([record.shard, record.id]);
const schedulingBinding = Object.freeze({ path: fileURLToPath(import.meta.url),
  sha256: digest(readFileSync(fileURLToPath(import.meta.url))) });
export const progressiveSchedulerBinding = () => ({ ...schedulingBinding });
// Known loaded scheduling dependencies are pinned; the transitive runtime remains Unknown.
const loadedRuntimeBindings = (() => {
  const dependencies = [
    '../../scripts/select-progressive-work.mjs',
    '../../scripts/render-progressive-plan.mjs',
    '../../scripts/lib/requirement-ledger.mjs',
    '../../scripts/generate-requirement-status.mjs',
    '../../js/server/lino.mjs',
    './cohort-runner.mjs',
    './oracle-source-review.mjs',
    '../js_dogfood/drive.mjs',
    '../../js/agentic/planner.mjs',
  ];
  return [progressiveSchedulerBinding(), ...dependencies.map(specifier => {
    const path = fileURLToPath(import.meta.resolve(specifier));
    return { path, sha256: digest(readFileSync(path)) };
  })].map(binding => Object.freeze(binding));
})();
export function progressiveSchedulerBindings() {
  loadedRuntimeBindings.forEach(bound);
  return loadedRuntimeBindings.map(binding => ({ ...binding }));
}
const bound = binding => {
  assert(binding && typeof binding.path === 'string' && /^[a-f0-9]{64}$/u.test(binding.sha256));
  assert.equal(digest(readFileSync(binding.path)), binding.sha256, 'progressive execution source drift');
};

/** Freeze the whole lowest frontier, including unavailable Missing work, before execution. */
export function admitProgressivePass(root, captured, manifest, journalPath, passPath) {
  validateManifest(manifest);
  assert.equal(new Set(manifest.cases.map(item => item.requirementKey)).size, manifest.cases.length,
    'duplicate requirement work ownership');
  assert(manifest.cases.every(item => typeof item.requirementKey === 'string'), 'source requirement key required');
  const plan = planProgressiveWork(root, captured, manifest.cases.map(item => item.requirementKey));
  assert(plan.level !== null && plan.allAtLowestLevel > 0, 'no unfinished progressive work');
  const bindings = captured.sources.map(source => ({ path: resolve(root, source.path), sha256: source.sha256 }));
  for (const source of bindings) {
    assert(manifest.bindings.some(binding => resolve(binding.path) === source.path
      && binding.sha256 === source.sha256), 'complete ledger source must be an original manifest binding');
  }
  const runtimeSources = progressiveSchedulerBindings();
  for (const runtime of runtimeSources) {
    assert(manifest.bindings.some(binding => resolve(binding.path) === runtime.path
      && binding.sha256 === runtime.sha256), 'known runtime source must be an original manifest binding');
  }
  const source = runtimeSources[0];
  journalPath = resolve(journalPath);
  passPath = resolve(passPath);
  const inside = (workspace, path) => {
    const delta = relative(resolve(workspace), path);
    return delta === '' || (!isAbsolute(delta) && delta !== '..' && !delta.startsWith('..' + sep));
  };
  assert(passPath !== journalPath, 'distinct scheduling and cohort journals required');
  assert(manifest.cases.every(item => !inside(item.workspace, passPath) && !inside(item.workspace, journalPath)),
    'scheduling journals must be outside every writable workspace');
  const bytes = JSON.stringify({ schema: 'source-owned-progressive-pass/v1',
    captured, plan, originalManifest: manifest, source, runtimeSources, journalPath: resolve(journalPath),
    requirementsPromoted: false, fullAcceptance: false, runtimeClosure: 'Unknown' });
  writeFileSync(passPath, bytes, { flag: 'wx', mode: 0o600 });
  assert.equal(readFileSync(passPath, 'utf8'), bytes, 'progressive admission readback');
  const admission = admitManifest(manifest, journalPath);
  const token = Object.freeze({});
  admittedPasses.set(token, { root, captured: structuredClone(captured),
    manifest: structuredClone(manifest), plan: structuredClone(plan), source, runtimeSources,
    passPath, passSHA256: digest(bytes), admission, attempted: new Set() });
  return token;
}

/** The caller can choose an attempt identity, never a higher task or a driver callback. */
export async function executeProgressivePass(token, attemptId) {
  const state = admittedPasses.get(token);
  assert(state, 'private progressive admission required');
  assert(typeof attemptId === 'string' && attemptId.length > 0 && !state.attempted.has(attemptId),
    'fresh progressive attempt identity required');
  const attemptPath = state.passPath + '.attempt-' + digest(attemptId);
  writeFileSync(attemptPath + '.started.json', JSON.stringify({ attemptId, passSHA256: state.passSHA256 }), { flag: 'wx', mode: 0o600 });
  state.attempted.add(attemptId);
  let outcome;
  const results = [];
  try {
  state.runtimeSources.forEach(bound);
  assert.equal(digest(readFileSync(state.passPath)), state.passSHA256, 'progressive pass input drift');
  assert.deepEqual(captureProgressiveInputs(state.root), state.captured, 'progressive ledger drift');
  const plan = planProgressiveWork(state.root, state.captured, state.manifest.cases.map(item => item.requirementKey));
  assert.deepEqual(plan, state.plan, 'progressive selected frontier drift');
  for (const record of plan.selected) {
    assert.deepEqual(captureProgressiveInputs(state.root), state.captured, 'progressive ledger drift');
    const item = state.manifest.cases.find(item => item.requirementKey === requirementKey(record));
    assert(item, 'selected work lost its original task');
    const result = await executeAdmittedCase(state.admission, item.runId, attemptId,
      (task, workspace) => drive(planChatStep, workspace, task));
    results.push({ requirementKey: requirementKey(record), runId: item.runId,
      status: result.accepted ? 'Observed' : 'Missing', result });
    state.runtimeSources.forEach(bound);
    assert.equal(digest(readFileSync(state.passPath)), state.passSHA256, 'progressive pass input drift');
  }
  assert.deepEqual(captureProgressiveInputs(state.root), state.captured, 'progressive ledger drift');
  outcome = { level: plan.level, allAtLowestLevel: plan.allAtLowestLevel, results,
    missing: plan.blocked.map(record => ({ requirementKey: requirementKey(record), status: 'Missing',
      reason: 'no declared original task at the source-selected lowest level' })).concat(
      results.filter(item => !item.result.accepted).map(item => ({ requirementKey: item.requirementKey, status: 'Missing',
        reason: 'source-selected original task lacks closed independent oracle acceptance' }))),
    accepted: plan.blocked.length === 0 && results.length === plan.allAtLowestLevel
      && results.every(item => item.result.accepted),
    requirementsPromoted: false, fullAcceptance: false, runtimeClosure: 'Unknown' };
  return outcome;
  } catch (error) {
    outcome = { accepted: false, results, failure: { name: error?.name ?? 'Error', message: String(error?.message ?? error) }, requirementsPromoted: false, fullAcceptance: false };
    throw error;
  } finally {
    writeFileSync(attemptPath + '.finished.json', JSON.stringify({ attemptId, outcome }), { flag: 'wx', mode: 0o600 });
  }
}
