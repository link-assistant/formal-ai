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
tests.on('test:pass', event => reportFile('completed', event));
tests.on('test:fail', event => {
  if (event.todo === undefined || event.todo === false) {
    process.exitCode = 1;
    reportFile('failed', event);
  }
});
await pipeline(tests.compose(spec), process.stdout);
