#!/usr/bin/env node
// Hosted source checks only. Main's native gates and real base ratchet remain required.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const nodeArguments = ['--max-old-space-size=384'];
export const governanceChecks = Object.freeze([
  {name: 'language-parity', command: process.execPath, args: [...nodeArguments, 'scripts/check-language-parity.mjs', '--check']},
  {name: 'minimal-core', command: process.execPath, args: [...nodeArguments, 'scripts/check-minimal-core-boundary.mjs']},
  {name: 'current-debt-ceilings', command: process.execPath, args: [...nodeArguments, 'scripts/check-fast-governance.mjs', '--current-debt-ceilings']},
  {name: 'honest-closure', command: 'python3', args: ['scripts/check-closure-audit.py']},
  {name: 'role-namespace', command: 'python3', args: ['scripts/generate-role-registry.py', '--check']},
  {name: 'seed-metadata', command: process.execPath, args: [...nodeArguments, 'scripts/audit-seed-metadata.mjs']},
  {name: 'response-language-parity', command: process.execPath, args: [...nodeArguments, 'scripts/generate-response-parity-debt.mjs', '--check']}
]);

function hostedEnvironment(environment) {
  if (environment.GITHUB_ACTIONS !== 'true' || environment.RUNNER_OS !== 'Linux') {
    throw Error('Fast governance census is restricted to hosted Linux GitHub Actions');
  }
}

export function executeCheck(check, timeoutMs, cwd) {
  hostedEnvironment(process.env);
  // GNU timeout owns a process group, including the closure audit's Python child.
  return spawnSync('timeout', ['--signal=TERM', '--kill-after=1s', `${timeoutMs / 1000}s`, check.command, ...check.args], {
    cwd, env: process.env, encoding: 'utf8', maxBuffer: 128 * 1024,
    timeout: timeoutMs + 2000, killSignal: 'SIGKILL'
  });
}

export function runGovernanceChecks({cwd = root, execute = executeCheck, now = Date.now, budgetMs = 60000} = {}) {
  const deadline = now() + budgetMs;
  const results = [];
  for (const check of governanceChecks) {
    const started = now();
    const remaining = deadline - started;
    const timeoutMs = Math.min(15000, remaining - 2000);
    let result;
    if (timeoutMs <= 0) result = {status: null, error: {message: 'Shared governance deadline exhausted'}};
    else {
      try { result = execute(check, timeoutMs, cwd); }
      catch (error) { result = {status: null, error}; }
    }
    const passed = result.status === 0 && !result.error && now() <= deadline;
    results.push({name: check.name, passed, status: result.status, durationMs: now() - started});
    console.log(`fast-governance ${check.name}: ${passed ? 'PASS' : 'FAIL'} (${now() - started}ms)`);
    for (const output of [result.stdout, result.stderr, result.error?.message]) {
      if (output) console.log(String(output).slice(-16384));
    }
  }
  console.log(JSON.stringify({scope: 'current source governance; not base nonincrease or tests-as-docs parity', results}));
  return results.every(result => result.passed) && now() <= deadline ? 0 : 1;
}

export async function main(argv = process.argv.slice(2), environment = process.env) {
  hostedEnvironment(environment);
  if (argv.length === 1 && argv[0] === '--current-debt-ceilings') {
    const {parseRatchet, measure, checkMeasured} = await import('./check-debt-ratchet.mjs');
    const ratchet = parseRatchet(fs.readFileSync(path.join(root, 'data/meta/debt-ratchet.lino'), 'utf8'));
    const failures = checkMeasured(ratchet, measure(root));
    if (failures.length) { console.error(failures.join('\n')); return 1; }
    console.log('Current debt ceilings match measured source; base nonincrease is checked by the existing separate gate');
    return 0;
  }
  if (argv.length) throw Error('Unsupported fast-governance option');
  return runGovernanceChecks();
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try { process.exitCode = await main(); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
