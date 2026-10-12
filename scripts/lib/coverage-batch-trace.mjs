function denseStrings(values, nonempty = false) {
  if (!Array.isArray(values)) return false;
  for (let index = 0; index < values.length; index++) {
    if (!Object.hasOwn(values, index) || typeof values[index] !== 'string'
        || (nonempty && values[index].length === 0)) return false;
  }
  return true;
}

/** Data-only completed process trace. Group elapsed time is never divided among cases. */
export function completedBatchTrace(context, batch) {
  if (!denseStrings(batch.names, true) || batch.names.length !== batch.selected
      || new Set(batch.names).size !== batch.names.length
      || !denseStrings(batch.argv)
      || !Number.isFinite(batch.elapsedSeconds) || batch.elapsedSeconds < 0) {
    throw new Error('invalid coverage batch trace');
  }
  const singleCaseMeasured = batch.selected === 1 && batch.succeeded === true
    && batch.completed === 1 && batch.exitCode === 0 && !batch.signal && !batch.error
    && batch.summary?.passed === 1 && batch.summary?.failed === 0
    && batch.summary?.ignored === 0 && batch.problems?.length === 0;
  return { schema: 'coverage-completed-batch-v1', context, batch,
    timingScope: singleCaseMeasured ? 'single-completed-case-process' : 'batch-process-only',
    executionSourceAuthentication: 'Unknown' };
}
export function batchTraceLine(context, batch) {
  return 'COVERAGE_BATCH_TRACE ' + JSON.stringify(completedBatchTrace(context, batch)) + '\n';
}
