#!/usr/bin/env node
// Run a `scripts/<name>.rs` check and its JavaScript twin `scripts/<name>.mjs`
// with the same arguments and fail when their stdout, stderr or exit code
// differ (PR #1188, SCRIPTS-B). One CI gate per twin calls this, so a twin
// that drifts from its Rust original fails the pull request that drifted it.
//
// The Rust half compiles the script with rust-script, which only CI does: a
// local run (no `CI` in the environment) runs the twin alone and reports its
// verdict, so `experiments/formal_ai_subagent/local-gates.mjs` never compiles
// Rust. `--with-rust` forces the comparison locally.
//
// Usage:
//   node scripts/lib/checks-twin-parity.mjs <name> [args...]
//   node scripts/lib/checks-twin-parity.mjs --with-rust check-file-size

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

/** Run one command and capture what a diff needs. */
function capture(command, args, env) {
  const result = spawnSync(command, args, { encoding: 'utf8', maxBuffer: 1 << 30, env });
  return {
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
    status: result.error ? `could not start: ${result.error.message}` : result.status,
  };
}

/** The first line where two outputs part, as `line N: rust | js`. */
export function firstDifference(rust, js) {
  const left = rust.split('\n');
  const right = js.split('\n');
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    if (left[index] !== right[index]) {
      return `line ${index + 1}:\n  rust: ${JSON.stringify(left[index] ?? null)}\n  js:   ${JSON.stringify(right[index] ?? null)}`;
    }
  }
  return null;
}

/** Every way two captured runs differ, as messages. */
export function compareRuns(rust, js) {
  const problems = [];
  if (rust.status !== js.status) problems.push(`exit code: rust ${rust.status}, js ${js.status}`);
  for (const stream of ['stdout', 'stderr']) {
    const difference = firstDifference(rust[stream], js[stream]);
    if (difference) problems.push(`${stream} differs at ${difference}`);
  }
  return problems;
}

function main() {
  const argv = process.argv.slice(2);
  const withRust = argv[0] === '--with-rust' ? (argv.shift(), true) : Boolean(process.env.CI);
  const [name, ...args] = argv;
  if (!name || !/^[\w-]+$/.test(name) || !existsSync(`scripts/${name}.mjs`) || !existsSync(`scripts/${name}.rs`)) {
    process.stderr.write('usage: node scripts/lib/checks-twin-parity.mjs [--with-rust] <name> [args...] (run from the repository root; scripts/<name>.rs and scripts/<name>.mjs must exist)\n');
    process.exitCode = 2;
    return;
  }
  // The comparison is about the twins, not about whether the base is set: a
  // gate passes when both agree, so the warning filter env is shared as is.
  const js = capture(process.execPath, [`scripts/${name}.mjs`, ...args], process.env);
  if (!withRust) {
    process.stdout.write(`${name}: JavaScript twin only (exit ${js.status}); the rust-script half runs in CI.\n`);
    process.exitCode = js.status === 0 ? 0 : 1;
    return;
  }
  // rust-script writes cargo's build log to stderr on the run that compiles;
  // a first run builds (or finds the cached binary), the second is compared.
  capture('rust-script', [`scripts/${name}.rs`, ...args], process.env);
  const rust = capture('rust-script', [`scripts/${name}.rs`, ...args], process.env);
  const problems = compareRuns(rust, js);
  if (problems.length === 0) {
    process.stdout.write(`${name}: the JavaScript twin matches the Rust original (exit ${js.status}, ${js.stdout.split('\n').length - 1} stdout line(s)).\n`);
    return;
  }
  for (const problem of problems) {
    process.stdout.write(`::error file=scripts/${name}.mjs::${name}: the JavaScript twin disagrees with scripts/${name}.rs: ${problem.replaceAll('%', '%25').replaceAll('\n', '%0A')}\n`);
  }
  process.stdout.write(`\n--- rust stdout ---\n${rust.stdout}\n--- rust stderr ---\n${rust.stderr}\n`);
  process.exitCode = 1;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
