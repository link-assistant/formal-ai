import { openSync, readSync, closeSync, fstatSync, readFileSync, writeFileSync, mkdirSync, lstatSync } from 'node:fs';
import { resolve, sep } from 'node:path';
import { createHash } from 'node:crypto';
import { completedBatchTrace } from './coverage-batch-trace.mjs';

export function inputSnapshots(paths) {
  return new Map(paths.map(path => [path, readFileSync(path)]));
}

function digest(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function executableObservation(path) {
  const fd = openSync(path, 'r');
  try {
    const before = fstatSync(fd);
    if (!before.isFile()) throw new Error('coverage executable is not a regular file');
    const buffer = Buffer.alloc(1024 * 1024);
    const hash = createHash('sha256');
    let bytes = 0;
    for (;;) {
      const count = readSync(fd, buffer, 0, buffer.length, null);
      if (!count) break;
      hash.update(buffer.subarray(0, count));
      bytes += count;
    }
    const after = fstatSync(fd);
    if (bytes !== before.size || before.dev !== after.dev || before.ino !== after.ino
        || before.size !== after.size || before.mtimeMs !== after.mtimeMs
        || before.ctimeMs !== after.ctimeMs) throw new Error('coverage executable changed during observation');
    return { path, bytes, sha256: hash.digest('hex'), sourceAuthentication: 'Unknown' };
  } finally {
    closeSync(fd);
  }
}

export function createCoverageCheckpoints(directory, inputs, context, batches, executables) {
  const names = ['manifest.tsv', 'tests.tsv', 'plan.tsv', 'native-durations.lino', 'coverage-weights.lino'];
  if (!(inputs instanceof Map) || inputs.size !== names.length) throw new Error('invalid coverage capture inputs');
  const rows = [...inputs.entries()];
  const inputPaths = new Set(rows.map(([path]) => resolve(path)));
  if (inputPaths.has(resolve(directory))) throw new Error('coverage capture collides with an input');
  const selected = rows[2][1].toString('utf8').split('\n').filter(Boolean);
  const planned = batches.flatMap(batch => batch.names.map(name => `${batch.target}\t${name}`));
  if (new Set(planned).size !== planned.length || selected.length !== planned.length
      || selected.some(item => !planned.includes(item))) throw new Error('coverage batch capture does not conserve selected cases');
  for (const [path, bytes] of rows) {
    if (!Buffer.isBuffer(bytes) || !readFileSync(path).equals(bytes)) throw new Error('coverage input changed before capture');
  }
  const executableInputs = [...executables].map(([target, path]) => {
    try { return { target, status: 'Observed', ...executableObservation(path) }; }
    catch (error) {
      return { target, path, status: 'Missing', sourceAuthentication: 'Unknown',
        observationError: { code: error.code ?? null, message: error.message } };
    }
  });
  mkdirSync(directory);
  const write = (name, value) => writeFileSync(resolve(directory, name), JSON.stringify(value) + '\n', { flag: 'wx' });
  const inputRecords = rows.map(([path, bytes], index) => {
    writeFileSync(resolve(directory, names[index]), bytes, { flag: 'wx' });
    return { path, capturedAs: names[index], bytes: bytes.length, sha256: digest(bytes) };
  });
  const boundContext = { ...context, inputs: inputRecords, executableInputs };
  const capture = { schema: 'coverage-execution-capture-v1', context: boundContext,
    batches: batches.map((batch, index) => ({ index, target: batch.target,
      names: [...batch.names], weight: batch.seconds })), selected: selected.length,
    executionSourceAuthentication: 'Unknown', namespaceRaceGuarantee: 'Unknown' };
  write('capture.json', capture);
  const started = new Set(), completed = new Set();
  return {
    context: boundContext,
    start(batch) {
      const plannedBatch = capture.batches[batch.index];
      if (!plannedBatch || started.has(batch.index) || batch.target !== plannedBatch.target
          || JSON.stringify(batch.names) !== JSON.stringify(plannedBatch.names)) {
        throw new Error('coverage start is not the captured batch');
      }
      write(`start-${batch.index}.json`, { schema: 'coverage-batch-start-v1', context: boundContext, batch });
      started.add(batch.index);
    },
    finish(batch) {
      if (!started.has(batch.index) || completed.has(batch.index)) throw new Error('coverage completion has no unique captured start');
      const trace = completedBatchTrace(boundContext, batch);
      const plannedBatch = capture.batches[batch.index];
      if (batch.target !== plannedBatch.target
          || JSON.stringify(batch.names) !== JSON.stringify(plannedBatch.names)) throw new Error('coverage completion changed batch names');
      write(`completed-${batch.index}.json`, trace);
      completed.add(batch.index);
      return trace;
    },
  };
}

/** Refuse reused outputs and input collisions before a harness can run. */
export function assertFreshCoverageOutputs(resultPath, directory, inputs) {
  const inputPaths = new Set([...inputs.keys()].map(path => resolve(path)));
  for (const path of [resultPath, directory]) {
    if (typeof path !== 'string' || path.length === 0) throw new Error('invalid coverage output path');
    if (inputPaths.has(resolve(path))) throw new Error('coverage output collides with an input');
    let existing;
    try { existing = lstatSync(path); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    if (existing) throw new Error('coverage output already exists');
  }
  const result = resolve(resultPath), receipt = resolve(directory);
  if (result === receipt || result.startsWith(receipt + sep)
      || receipt.startsWith(result + sep)) throw new Error('coverage output namespaces overlap');
}
