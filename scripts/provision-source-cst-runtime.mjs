#!/usr/bin/env node
// Only hosted CI installs this small locked parser closure; no lifecycle scripts execute.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
export function provisionSourceCstRuntime(directory, {
  root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  run = spawnSync
} = {}) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Source CST package installation is hosted-only');
  const output = path.resolve(directory);
  assert(!/[\r\n]/u.test(output));
  assert(!fs.existsSync(output), 'fresh grammar runtime directory required');
  const source = path.join(root, 'packages/source-cst-runtime'),
    manifest = fs.readFileSync(path.join(source, 'package.json')),
    lock = fs.readFileSync(path.join(source, 'package-lock.json'));
  fs.mkdirSync(output, {
    recursive: true
  });
  fs.writeFileSync(path.join(output, 'package.json'), manifest, {
    flag: 'wx'
  });
  fs.writeFileSync(path.join(output, 'package-lock.json'), lock, {
    flag: 'wx'
  });
  const result = run('npm', ['ci', '--prefix', output, '--omit=dev', '--ignore-scripts', '--no-audit', '--no-fund'], {
    stdio: 'inherit',
    timeout: 120000
  });
  assert.equal(result.status, 0, 'locked complete-source parser installation failed');
  const expected = JSON.parse(manifest).dependencies['@babel/parser'],
    observed = JSON.parse(fs.readFileSync(path.join(output, 'node_modules/@babel/parser/package.json'), 'utf8')).version;
  assert.equal(observed, expected, 'installed grammar differs from frozen runtime');
  fs.writeFileSync(path.join(output, 'installation.json'), JSON.stringify({
    provider: '@babel/parser',
    version: observed,
    lockSha256: createHash('sha256').update(lock).digest('hex'),
    lifecycleScripts: false
  }, null, 2) + '\n', {
    flag: 'wx'
  });
  assert(process.env.GITHUB_ENV, 'hosted environment sink required');
  fs.appendFileSync(process.env.GITHUB_ENV, 'FORMAL_AI_CST_RUNTIME=' + output + '\n');
  return {
    version: observed
  };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3);
  console.log(JSON.stringify(provisionSourceCstRuntime(process.argv[2])));
}
