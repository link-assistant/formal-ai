#!/usr/bin/env node
import {run} from 'node:test';
import {spec} from 'node:test/reporters';
import {pipeline} from 'node:stream/promises';
import {resolve, relative} from 'node:path';

const [option, value, ...files] = process.argv.slice(2);
const concurrency = Number(value);
if (option !== '--concurrency' || !Number.isInteger(concurrency) || concurrency < 1
  || files.length === 0 || new Set(files.map(file => resolve(file))).size !== files.length) {
  console.error('Usage: run-ordered-javascript-tests.mjs --concurrency <positive integer> <test files...>');
  process.exit(2);
}
// Supplying files bypasses the CLI glob discovery that alphabetically sorts
// the duration planner's explicit longest-first list. Keep process isolation
// and inherited Node arguments/environment at the runner's standard defaults.
const selected = new Map(files.map(file => [resolve(file), relative(process.cwd(), resolve(file))]));
const reported = new Set();
// Node may enqueue the initial files synchronously inside run(), before an
// event listener can attach. Record the supplied queue itself before starting.
for (const file of selected.values()) {
  console.error('[test-file] queued ' + file);
  reported.add('queued:' + file);
}
function reportFile(state, event) {
  if (event.nesting !== 0 || typeof event.name !== 'string') return;
  const file = selected.get(resolve(event.name));
  if (file === undefined || reported.has(state + ':' + file)) return;
  reported.add(state + ':' + file);
  console.error('[test-file] ' + state + ' ' + file);
}
const tests = run({files, concurrency});
tests.on('test:enqueue', event => reportFile('queued', event));
tests.on('test:dequeue', event => reportFile('started', event));
// Registered tests emit file summaries. Assertion-only files instead emit a
// file-level pass/fail event. Defer those candidates until the stream ends so
// a case named after its file cannot replace the authoritative file summary.
const summarized = new Set();
const fileResults = new Map();
function rememberFileResult(state, event) {
  if (event.nesting !== 0 || typeof event.name !== 'string' || typeof event.file !== 'string'
    || resolve(event.name) !== resolve(event.file)) return;
  const file = selected.get(resolve(event.file));
  if (file !== undefined) fileResults.set(file, {state, duration_ms: event.details?.duration_ms});
}
function reportCompletion(file, state, milliseconds) {
  if (summarized.has(file)) return;
  summarized.add(file);
  const duration = Number.isFinite(milliseconds) && milliseconds >= 0 ? milliseconds : null;
  console.error('[test-file] ' + state + ' ' + file + ' ' + JSON.stringify({duration_ms: duration}));
}
tests.on('test:pass', event => rememberFileResult('completed', event));
tests.on('test:summary', event => {
  if (typeof event.file !== 'string') return;
  const file = selected.get(resolve(event.file));
  if (file === undefined || summarized.has(file)) return;
  reportCompletion(file, event.success === true ? 'completed' : 'failed', event.duration_ms);
});
tests.on('test:fail', event => {
  rememberFileResult('failed', event);
  if (event.todo === undefined || event.todo === false) {
    process.exitCode = 1;
  }
});
await pipeline(tests.compose(spec), process.stdout);

for (const [file, result] of fileResults) reportCompletion(file, result.state, result.duration_ms);
