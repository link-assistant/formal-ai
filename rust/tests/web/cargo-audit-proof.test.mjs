// Exercise the actual audit wrapper without resolving or compiling dependencies.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const configuration = readFileSync(join(root, '.cargo/audit.toml'), 'utf8');
function run({missingChrono = false, auditStatus = 0, edit = text => text} = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'audit-proof-'));
  try {
    mkdirSync(join(directory, '.cargo'));
    mkdirSync(join(directory, 'bin'));
    mkdirSync(join(directory, 'rust'));
    writeFileSync(join(directory, 'rust/Cargo.lock'), '# audit protocol fixture\n');
    writeFileSync(join(directory, '.cargo/audit.toml'), edit(configuration));
    const cargo = [
      '#!/usr/bin/env node',
      "const fs = require('node:fs');",
      'const args = process.argv.slice(2);',
      "if (args[0] === 'tree') {",
      "  const spec = args[args.indexOf('--invert') + 1];",
      "  if (spec === 'chrono@0.4.45' && process.env.MISSING_CHRONO === '1') process.exit(1);",
      "  console.log(spec + ' -> introducing dependency');",
      "} else if (args[0] === 'audit') {",
      "  fs.appendFileSync('audit-calls.jsonl', JSON.stringify(args) + '\\n');",
      '  process.exit(Number(process.env.AUDIT_STATUS));',
      '} else process.exit(99);',
    ].join('\n');
    writeFileSync(join(directory, 'bin/cargo'), cargo, {mode: 0o755});
    writeFileSync(join(directory, 'bin/git'), '#!/bin/sh\nprintf "rust/Cargo.lock\\0"\n', {mode: 0o755});
    const result = spawnSync('bash', [join(root, 'scripts/check-rust-dependencies.sh')], {
      cwd: directory, encoding: 'utf8', timeout: 10000,
      env: {...process.env, PATH: join(directory, 'bin') + ':' + process.env.PATH,
        MISSING_CHRONO: missingChrono ? '1' : '0', AUDIT_STATUS: String(auditStatus)},
    });
    let calls = [];
    try { calls = readFileSync(join(directory, 'audit-calls.jsonl'), 'utf8').trim().split('\n').map(JSON.parse); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    return {...result, calls};
  } finally { rmSync(directory, {recursive: true, force: true}); }
}
test('blocked chrono proof allows the warning-denying audit only while the exact crate remains reachable', () => {
  const present = run();
  assert.equal(present.status, 0, present.stderr);
  assert.deepEqual(present.calls, [['audit', '--file', 'rust/Cargo.lock', '--deny', 'warnings']]);
  const absent = run({missingChrono: true});
  assert.equal(absent.status, 1);
  assert.match(absent.stdout, /chrono@0\.4\.45.*no longer reaches the build graph/u);
  assert.deepEqual(absent.calls, []);
});
test('unrelated audit findings remain fatal', () => {
  assert.equal(run({auditStatus: 17}).status, 17);
});
test('an advisory acknowledgement without its filed-report proof fails before auditing', () => {
  const result = run({edit: text => text.split('\n').filter(line => !line.startsWith('# RUSTSEC-2026-0543 blocked-upstream')).join('\n')});
  assert.equal(result.status, 1);
  assert.match(result.stdout, /ignores RUSTSEC-2026-0543 without a proof line/u);
  assert.deepEqual(result.calls, []);
});
