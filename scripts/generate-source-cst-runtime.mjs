#!/usr/bin/env node
// Derive the small CST-only dependency closure from the canonical frozen web lock.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export function sourceCstRuntimeFiles(root) {
  const text = fs.readFileSync(path.join(root, 'bun.lock'), 'utf8'),
    records = new Map();
  for (const line of text.split('\n')) {
    const match = /^    "([^"]+)": (\[.*\]),?$/u.exec(line);
    if (match) records.set(match[1], JSON.parse(match[2]));
  }
  const desired = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8')).dependencies['@babel/parser'];
  assert.match(desired, /^\d+\.\d+\.\d+$/u, 'the canonical parser must have one exact version');
  const manifest = {
    name: 'formal-ai-source-cst-runtime',
    version: '0.0.0',
    private: true,
    type: 'module',
    dependencies: {
      '@babel/parser': desired
    }
  };
  const packages = {
      '': {
        name: manifest.name,
        version: manifest.version,
        dependencies: manifest.dependencies
      }
    },
    pending = ['@babel/parser'];
  while (pending.length) {
    const name = pending.shift();
    if (packages['node_modules/' + name]) continue;
    const row = records.get(name);
    assert(row, 'missing frozen CST dependency ' + name);
    const spec = row[0],
      version = spec.slice(spec.lastIndexOf('@') + 1);
    assert(spec.startsWith(name + '@'));
    assert.match(row[3], /^sha512-[A-Za-z0-9+/]+=*$/u);
    const entry = {
      version,
      resolved: 'https://registry.npmjs.org/' + name + '/-/' + name.split('/').at(-1) + '-' + version + '.tgz',
      integrity: row[3]
    };
    if (row[2].dependencies) {
      entry.dependencies = row[2].dependencies;
      pending.push(...Object.keys(entry.dependencies));
    }
    if (row[2].bin) entry.bin = typeof row[2].bin === 'string' ? {
      [name.split('/').at(-1)]: row[2].bin
    } : row[2].bin;
    packages['node_modules/' + name] = entry;
  }
  assert.equal(packages['node_modules/@babel/parser'].version, desired);
  const lock = {
    name: manifest.name,
    version: manifest.version,
    lockfileVersion: 3,
    requires: true,
    packages
  };
  return new Map([['package.json', JSON.stringify(manifest, null, 2) + '\n'], ['package-lock.json', JSON.stringify(lock, null, 2) + '\n']]);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert(process.argv.length === 3 && ['--write', '--check'].includes(process.argv[2]));
  const files = sourceCstRuntimeFiles(root),
    directory = path.join(root, 'packages/source-cst-runtime');
  for (const [name, body] of files) {
    const target = path.join(directory, name);
    if (process.argv[2] === '--write') {
      fs.mkdirSync(directory, {
        recursive: true
      });
      fs.writeFileSync(target, body);
    } else assert.equal(fs.readFileSync(target, 'utf8'), body, 'CST runtime drift: ' + name);
  }
  console.log('Complete source CST runtime matches the canonical frozen dependency closure.');
}
