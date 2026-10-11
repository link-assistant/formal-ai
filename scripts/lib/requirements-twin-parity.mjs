#!/usr/bin/env node
// Run a Rust gate script and its JavaScript twin and fail when they disagree.
//
// PR #1188 gave the requirement pipeline JavaScript twins
// (`scripts/<name>.mjs` beside `scripts/<name>.rs`) so nobody compiles Rust
// locally. CI keeps the twins honest: each `data/meta/ci-gates/*-js-twin.lino`
// gate runs both commands on the same tree and compares their exit codes,
// stdout and stderr byte for byte. The commands are passed whole, so
// `experiments/formal_ai_subagent/local-gates.mjs` swaps the `rust-script`
// one for its twin locally and nothing is compiled there.
//
// Usage:
//   node scripts/lib/requirements-twin-parity.mjs '<rust command>' '<js command>'
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

/** Run one command through bash, as `scripts/run-ci-gates.rs` runs a gate. */
export function capture(command, cwd = process.cwd()) {
  const result = spawnSync('bash', ['-o', 'pipefail', '-c', command], { cwd, encoding: 'utf8', maxBuffer: 256 << 20 });
  return { status: result.status ?? 128, stdout: result.stdout ?? '', stderr: result.stderr ?? '' };
}

/** The first line where two outputs differ, as `line N: rust «…» / js «…»`. */
export function firstDifference(left, right) {
  const a = left.split('\n');
  const b = right.split('\n');
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return `line ${index + 1}: rust ${JSON.stringify(a[index] ?? null)} / js ${JSON.stringify(b[index] ?? null)}`;
  }
  return null;
}

/** Every way the two runs differ, as messages; empty when they agree. */
export function compareRuns(rust, js) {
  const differences = [];
  if (rust.status !== js.status) differences.push(`exit code: rust ${rust.status} / js ${js.status}`);
  for (const stream of ['stdout', 'stderr']) {
    const difference = firstDifference(rust[stream], js[stream]);
    if (difference) differences.push(`${stream} ${difference}`);
  }
  return differences;
}

export function main(argv = process.argv.slice(2)) {
  if (argv.length !== 2) {
    console.log('::error::usage: requirements-twin-parity.mjs <rust command> <js command>');
    return 2;
  }
  const [rustCommand, jsCommand] = argv;
  const rust = capture(rustCommand);
  const js = capture(jsCommand);
  const differences = compareRuns(rust, js);
  process.stdout.write(rust.stdout);
  process.stderr.write(rust.stderr);
  if (!differences.length) {
    console.log(`\`${jsCommand}\` matches \`${rustCommand}\` (exit ${rust.status}).`);
    return 0;
  }
  for (const difference of differences) console.log(`::error::\`${jsCommand}\` differs from \`${rustCommand}\`: ${difference}`);
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
