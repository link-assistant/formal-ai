import { spawn } from 'node:child_process';
import { performance } from 'node:perf_hooks';
import { StringDecoder } from 'node:string_decoder';

function record(item) {
  const split = item.indexOf('\t');
  if (split < 1 || split === item.length - 1) throw new Error(`invalid coverage item: ${item}`);
  return { target: item.slice(0, split), name: item.slice(split + 1), item };
}

/** Actual process dispatch order, rather than libtest's argument ordering. */
export function executionBatches(selected, listing, executables, secondsOf, maximumBatch = 64) {
  if (!Number.isInteger(maximumBatch) || maximumBatch < 1) throw new Error('invalid batch size');
  const known = new Set(listing);
  if (known.size !== listing.length) throw new Error('duplicate runtime listing item');
  if (new Set(selected).size !== selected.length) throw new Error('duplicate selected coverage item');
  const ordered = selected.map((item) => {
    const row = record(item);
    if (!known.has(item)) throw new Error(`selected item was not listed: ${item}`);
    if (!executables.has(row.target)) throw new Error(`missing executable: ${row.target}`);
    const seconds = secondsOf(item);
    if (!Number.isFinite(seconds) || seconds < 0) throw new Error(`invalid weight: ${item}`);
    return { ...row, seconds };
  }).sort((a, b) => b.seconds - a.seconds || a.item.localeCompare(b.item, 'en'));
  const batches = [];
  for (const row of ordered) {
    const previous = batches.at(-1);
    if (previous && previous.target === row.target && previous.seconds === row.seconds
      && previous.names.length < maximumBatch) {
      previous.names.push(row.name);
    } else {
      batches.push({ target: row.target, seconds: row.seconds, names: [row.name] });
    }
  }
  return batches;
}

function completionCollector(names) {
  const wanted = new Set(names);
  const seen = new Map();
  const problems = [];
  let summary = null;
  return {
    line(line) {
      const completed = line.match(/^test (.+) \.\.\. (ok|FAILED|ignored(?:,.*)?)$/);
      if (completed) {
        const name = completed[1];
        if (!wanted.has(name)) problems.push(`unexpected case: ${name}`);
        if (seen.has(name)) problems.push(`duplicate completion: ${name}`);
        seen.set(name, completed[2].startsWith('ignored') ? 'ignored' : completed[2]);
      }
      const counts = line.match(/^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;/);
      if (counts) {
        if (summary) problems.push('multiple harness summaries');
        summary = { passed: +counts[1], failed: +counts[2], ignored: +counts[3] };
      }
    },
    finish() {
      for (const name of wanted) if (!seen.has(name)) problems.push(`missing completion: ${name}`);
      if (!summary) problems.push('missing harness summary');
      else {
        const actual = { passed: 0, failed: 0, ignored: 0 };
        for (const status of seen.values()) actual[status === 'ok' ? 'passed' : status === 'FAILED' ? 'failed' : 'ignored']++;
        for (const key of Object.keys(actual)) if (actual[key] !== summary[key]) problems.push(`summary ${key} mismatch`);
        if (summary.passed + summary.failed + summary.ignored !== wanted.size) problems.push('summary selected count mismatch');
      }
      return { completed: seen.size, summary, problems };
    },
  };
}

/** Keep executing all batches after a failure; certify only observed completions. */
export async function executeBatches(batches, executables, options = {}) {
  const concurrency = options.concurrency ?? 2;
  const threads = options.threads ?? 2;
  if (!Number.isInteger(concurrency) || concurrency < 1 || concurrency > 2) throw new Error('coverage process concurrency must be 1 or 2');
  if (!Number.isInteger(threads) || threads < 1 || threads > 2) throw new Error('coverage harness threads must be 1 or 2');
  const results = Array(batches.length);
  const launchOrder = [];
  let next = 0;
  async function worker() {
    while (next < batches.length) {
      const index = next++;
      const batch = batches[index];
      const executable = executables.get(batch.target);
      const collector = completionCollector(batch.names);
      const started = performance.now();
      launchOrder.push(index);
      options.onStart?.({ index, target: batch.target, names: batch.names, weight: batch.seconds });
      const outcome = await new Promise((resolve) => {
        let pending = '';
        const decoder = new StringDecoder('utf8');
        const child = spawn(executable, ['--exact', '--color', 'never', '--test-threads', String(threads), ...batch.names], {
          cwd: options.cwd, env: options.env ?? process.env, stdio: ['ignore', 'pipe', 'pipe'],
        });
        child.stdout.on('data', (chunk) => {
          options.onOutput?.(chunk);
          pending += decoder.write(chunk);
          const lines = pending.split('\n');
          pending = lines.pop();
          for (const line of lines) collector.line(line.replace(/\r$/, ''));
        });
        child.stderr.on('data', (chunk) => options.onOutput?.(chunk));
        child.on('error', (error) => resolve({ exitCode: null, signal: null, error: error.message }));
        child.on('close', (exitCode, signal) => {
          pending += decoder.end();
          if (pending) collector.line(pending.replace(/\r$/, ''));
          resolve({ exitCode, signal });
        });
      });
      const observed = collector.finish();
      results[index] = { index, target: batch.target, selected: batch.names.length,
        elapsedSeconds: (performance.now() - started) / 1000, ...outcome, ...observed,
        succeeded: outcome.exitCode === 0 && !outcome.signal && !outcome.error
          && observed.problems.length === 0 && observed.summary?.failed === 0 };
      options.onFinish?.(results[index]);
    }
  }
  await Promise.all(Array.from({ length: concurrency }, worker));
  return { selected: batches.reduce((sum, batch) => sum + batch.names.length, 0),
    completed: results.reduce((sum, batch) => sum + batch.completed, 0),
    succeeded: results.every((batch) => batch.succeeded), launchOrder, batches: results };
}
