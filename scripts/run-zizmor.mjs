#!/usr/bin/env node
import {spawnSync} from 'node:child_process';
import {setTimeout} from 'node:timers/promises';

// The digest is the zizmor-action v0.6.4 entry for 1.29.0. Both audit
// personas retain online advisory checks, GitHub annotations and the live scope.
export const image = 'ghcr.io/zizmorcore/zizmor:1.29.0@sha256:863026d54f91271b10b60b67ad8054cb37120167e162482597db102b3026a284';
export function advisoryTransportFailure(result) {
  const diagnostic = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.replace(/\x1b\[[0-9;]*m/gu, '');
  return result.status === 1 && !result.error && !diagnostic.includes('::error') &&
    diagnostic.includes('fatal: no audit was performed') &&
    diagnostic.includes("'known-vulnerable-actions' audit failed") &&
    diagnostic.includes('request error while accessing GitHub API') &&
    /Cache error:.*error sending request for url \(https:\/\/api\.github\.com\/advisories\?/u.test(diagnostic);
}
export async function runAudit(persona, {execute = spawnSync, pause = setTimeout, output = process} = {}) {
  if (!['regular', 'pedantic'].includes(persona)) throw new Error('Expected regular or pedantic audit persona');
  const run = args => {
    const result = execute('docker', args, {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
    output.stdout.write(result.stdout ?? '');
    output.stderr.write(result.stderr ?? '');
    if (result.error) output.stderr.write(`${result.error.message}\n`);
    return result;
  };
  const pulled = run(['pull', image]);
  if (pulled.status !== 0 || pulled.error) return pulled.status || 1;
  const args = ['run', '--rm', '--volume', `${process.cwd()}:/workspace:ro`, '--workdir', '/workspace',
    '--env', 'GH_TOKEN', image, `--persona=${persona}`, '--format=github', '--color=never',
    '--config=.github/zizmor.yml', `--min-confidence=${persona === 'regular' ? 'medium' : 'high'}`];
  if (persona === 'pedantic') args.push('--min-severity=high');
  args.push('--', '.github/workflows', '.github/actions');
  for (let attempt = 0; attempt < 3; attempt++) {
    const result = run(args);
    if (!advisoryTransportFailure(result) || attempt === 2) return result.error ? 1 : (result.status ?? 1);
    output.stdout.write(`::notice::Retrying the complete online Zizmor ${persona} audit after an advisory transport failure (${attempt + 1}/2).\n`);
    await pause([2000, 5000][attempt]);
  }
}
if (process.argv[1] && import.meta.url === new URL(process.argv[1], 'file:').href) {
  runAudit(process.argv[2]).then(status => { process.exitCode = status; }).catch(error => {
    console.error(error.message); process.exitCode = 1;
  });
}
