import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, openSync, closeSync, fsyncSync, mkdirSync, rmdirSync, readdirSync, lstatSync, realpathSync } from 'node:fs';
import { resolve, dirname, relative, isAbsolute, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

export const digest = value => createHash('sha256').update(value).digest('hex');
const runnerIdentity = () => digest(readFileSync(fileURLToPath(import.meta.url)));
const bound = value => {
  if (!value || typeof value.path !== 'string' || !/^[a-f0-9]{64}$/u.test(value.sha256)) throw new Error('invalid source binding');
  const bytes = readFileSync(value.path);
  if (digest(bytes) !== value.sha256) throw new Error('source binding drift: ' + value.path);
};
const within = (directory, path) => {
  const delta = relative(resolve(directory), resolve(path));
  return delta === '' || (!isAbsolute(delta) && delta !== '..' && !delta.startsWith('..' + sep));
};
const verifyBindings = manifest => manifest.bindings.forEach(bound);
export function validateManifest(manifest) {
  if (manifest?.schemaVersion !== 1 || typeof manifest.cohortId !== 'string' || !manifest.cohortId
      || !Array.isArray(manifest.cases) || manifest.cases.length === 0 || manifest.cases.length > 200
      || !Array.isArray(manifest.bindings) || !manifest.bindings.length) throw new Error('invalid manifest');
  const ids = new Set();
  for (const item of manifest.cases) {
    if (!item || typeof item.runId !== 'string' || !item.runId || ids.has(item.runId)) throw new Error('duplicate or missing run identity');
    ids.add(item.runId);
    if (!['coding', 'self-coding'].includes(item.taskKind) || typeof item.task !== 'string' || !item.task
        || digest(item.task) !== item.taskSHA256) throw new Error('invalid original task');
    const unrestricted = ['repair', 'refactoring', 'deletion', 'translation'].includes(item.category);
    if (!unrestricted && !['feature-implementation', 'test-implementation'].includes(item.category)) throw new Error('invalid category');
    if (item.expectedRelation !== (unrestricted ? 'unrestricted' : 'output-larger')) throw new Error('wrong size relation');
    if (!Number.isInteger(item.timeoutMilliseconds) || item.timeoutMilliseconds < 1 || item.timeoutMilliseconds > 60000) throw new Error('invalid timeout');
    if (typeof item.workspace !== 'string' || !Array.isArray(item.allowedEffects) || !item.allowedEffects.length
        || item.allowedEffects.some(path => typeof path !== 'string' || !within(item.workspace, path))) throw new Error('invalid effect scope');
    if (!item.oracle || typeof item.oracle.path !== 'string' || within(item.workspace, item.oracle.path)) throw new Error('oracle must be outside writable workspace');
    bound(item.oracle);
    if (!manifest.bindings.some(binding => binding.path === item.oracle.path && binding.sha256 === item.oracle.sha256)) throw new Error('oracle must be a manifest binding');
  }
  verifyBindings(manifest);
  return manifest;
}
const syncedWrite = (path, bytes, flags) => {
  const fd = openSync(path, flags, 0o600);
  try { writeFileSync(fd, bytes); fsyncSync(fd); } finally { closeSync(fd); }
};
const syncDirectory = path => {
  const fd = openSync(dirname(path), 'r');
  try { fsyncSync(fd); } finally { closeSync(fd); }
};
const readJournal = path => {
  const lines = readFileSync(path, 'utf8').trimEnd().split('\n');
  const records = lines.map(line => JSON.parse(line));
  records.forEach((record, index) => {
    if (record.sequence !== index || record.previousDigest !== (index ? digest(lines[index - 1]) : null)) throw new Error('journal sequence or chain mismatch');
  });
  return records;
};
export function admitManifest(manifest, journalPath) {
  validateManifest(manifest);
  const manifestBytes = JSON.stringify(manifest);
  if (Buffer.byteLength(manifestBytes) > 1048576) throw new Error('manifest exceeds bound');
  const record = { kind: 'admitted', sequence: 0, previousDigest: null,
    cohortId: manifest.cohortId, manifestSHA256: digest(manifestBytes), runnerSHA256: runnerIdentity(), manifest };
  const bytes = JSON.stringify(record) + '\n';
  syncedWrite(journalPath, bytes, 'wx');
  syncDirectory(journalPath);
  if (readFileSync(journalPath, 'utf8') !== bytes) throw new Error('admission readback mismatch');
  return { journalPath, manifestSHA256: record.manifestSHA256, runnerSHA256: record.runnerSHA256, admissionSHA256: digest(bytes) };
}
function appendJournal(admission, kind, runId, attemptId, payload) {
  const lock = admission.journalPath + '.lock';
  mkdirSync(lock);
  try {
    const records = readJournal(admission.journalPath);
    const first = records[0];
    if (first.kind !== 'admitted' || first.manifestSHA256 !== admission.manifestSHA256
        || digest(JSON.stringify(first.manifest)) !== admission.manifestSHA256
        || first.runnerSHA256 !== runnerIdentity() || admission.runnerSHA256 !== first.runnerSHA256
        || digest(JSON.stringify(first) + '\n') !== admission.admissionSHA256) throw new Error('admission identity mismatch');
    if (kind === 'started') validateManifest(first.manifest);
    const item = first.manifest.cases.find(item => item.runId === runId);
    if (!item || typeof attemptId !== 'string' || !attemptId) throw new Error('undeclared run or attempt');
    const prior = records.filter(record => record.runId === runId && record.attemptId === attemptId);
    if (kind === 'started' && prior.length) throw new Error('duplicate attempt');
    if (kind === 'finished' && (prior.length !== 1 || prior[0].kind !== 'started')) throw new Error('finish requires exactly one start');
    const last = records.at(-1);
    const record = { kind, sequence: records.length, previousDigest: digest(JSON.stringify(last)),
      manifestSHA256: admission.manifestSHA256, runId, attemptId, taskSHA256: item.taskSHA256, ...payload };
    syncedWrite(admission.journalPath, JSON.stringify(record) + '\n', 'a');
    const observed = readJournal(admission.journalPath).at(-1);
    if (JSON.stringify(observed) !== JSON.stringify(record)) throw new Error('journal readback mismatch');
    return { item, record };
  } finally { rmdirSync(lock); }
}
function snapshot(directory) {
  const result = {};
  function visit(path) {
    const entry = lstatSync(path);
    if (entry.isSymbolicLink()) throw new Error('symlink in workspace');
    if (entry.isDirectory()) for (const name of readdirSync(path).sort()) visit(resolve(path, name));
    else if (entry.isFile()) {
      const bytes = readFileSync(path);
      result[path] = { sha256: digest(bytes), bytes: bytes.length, content: bytes.toString('utf8') };
      if (!Buffer.from(result[path].content, 'utf8').equals(bytes)) throw new Error('non-UTF8 workspace source');
    }
    else throw new Error('unsupported workspace entry');
  }
  visit(realpathSync(directory));
  return result;
}
export async function executeAdmittedCase(admission, runId, attemptId, actualDriver) {
  if (typeof actualDriver !== 'function') throw new Error('actual driver callback required');
  const { item } = appendJournal(admission, 'started', runId, attemptId, {});
  let result = { accepted: false, usage: { status: 'Unknown', reason: 'host supplied no usage receipt' } };
  try {
    const before = snapshot(item.workspace);
    let execution;
    let executionFailure;
    let executionFailed = false;
    try { execution = await actualDriver(item.task, item.workspace); }
    catch (error) { executionFailed = true; executionFailure = error; }
    result.transcript = execution?.transcript ?? null;
    result.answer = execution?.answer ?? null;
    result.stop = execution?.stop ?? null;
    if (execution?.usageReceipt !== undefined) result.usage = {
      status: 'CapturedUnnormalized', raw: execution.usageReceipt,
      rawSHA256: digest(JSON.stringify(execution.usageReceipt)),
      reason: 'actual driver receipt requires provider identity and metric normalization',
    };
    const after = snapshot(item.workspace);
    const effects = [...new Set([...Object.keys(before), ...Object.keys(after)])].filter(path => before[path]?.sha256 !== after[path]?.sha256);
    result.sourceEffects = effects.map(path => ({ path, before: before[path] ?? null, after: after[path] ?? null }));
    if (executionFailed) throw executionFailure;
    if (execution?.stop !== 'final') throw new Error('actual driver did not complete');
    const allowedEffects = item.allowedEffects.map(path => resolve(realpathSync(item.workspace), relative(resolve(item.workspace), resolve(path))));
    if (effects.some(path => !allowedEffects.includes(path))) throw new Error('out-of-scope effect');
    if (item.expectedRelation === 'output-larger' && effects.length === 0) throw new Error('no source effect');
    const first = readJournal(admission.journalPath)[0];
    verifyBindings(first.manifest);
    bound(item.oracle);
    const oracleEnvironment = { ...process.env, COHORT_WORKSPACE: item.workspace };
    delete oracleEnvironment.NODE_TEST_CONTEXT;
    const check = spawnSync(process.execPath, ['--test', item.oracle.path], {
      cwd: dirname(item.oracle.path), timeout: item.timeoutMilliseconds, maxBuffer: 1048576,
      env: oracleEnvironment, encoding: 'utf8',
    });
    verifyBindings(first.manifest);
    result = { accepted: check.status === 0 && !check.error, effects, sourceEffects: result.sourceEffects,
      transcript: execution?.transcript ?? null, answer: execution?.answer ?? null, stop: execution?.stop ?? null,
      check: { oracleSHA256: item.oracle.sha256, exitCode: check.status, signal: check.signal,
        stdout: check.stdout, stderr: check.stderr, error: check.error?.message ?? null },
      usage: { status: 'Unknown', reason: 'host supplied no usage receipt' } };
    if (execution?.usageReceipt !== undefined) result.usage = {
      status: 'CapturedUnnormalized', raw: execution.usageReceipt,
      rawSHA256: digest(JSON.stringify(execution.usageReceipt)),
      reason: 'actual driver receipt requires provider identity and metric normalization',
    };
  } catch (error) { result.failure = { name: error?.name ?? 'Error', message: String(error?.message ?? error) }; }
  appendJournal(admission, 'finished', runId, attemptId, { result });
  return result;
}
export function partitionMeasurements(measurements) {
  return { ordinaryCoding: measurements.filter(item => item.taskKind === 'coding'),
    selfCoding: measurements.filter(item => item.taskKind === 'self-coding'), combined: measurements };
}
export function deriveDeclaredVariants(contract, variants) {
  if (!contract || typeof contract.taskTemplate !== 'string' || !Array.isArray(contract.allowedFields)
      || !Array.isArray(variants) || variants.length > 200) throw new Error('invalid declared contract');
  bound(contract.source);
  const ids = new Set();
  return variants.map(variant => {
    if (!variant || typeof variant.runId !== 'string' || ids.has(variant.runId)) throw new Error('duplicate variant');
    ids.add(variant.runId);
    if (Object.keys(variant.parameters).some(name => !contract.allowedFields.includes(name))) throw new Error('undeclared substitution');
    const task = contract.taskTemplate.replace(/\{\{([a-zA-Z][a-zA-Z0-9]*)\}\}/gu, (_, name) => {
      const value = variant.parameters[name];
      if (typeof value !== 'string' || !value || /[\r\n]/u.test(value)) throw new Error('missing or unsafe substitution');
      return value;
    });
    return { ...variant, task, taskSHA256: digest(task), contractSHA256: contract.source.sha256,
      qualification: 'derived-declaration-only; source fixtures and independent oracle still required' };
  });
}
