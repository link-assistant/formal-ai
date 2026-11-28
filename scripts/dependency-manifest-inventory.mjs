#!/usr/bin/env node
// Every tracked Cargo manifest is active or has a specific preserved purpose.
import {isUtf8} from 'node:buffer';
import {spawnSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

export function trackedDependencyFiles(repository, git = spawnSync) {
  const result = git('git', ['ls-files', '-z'], {cwd: repository, maxBuffer: 32 * 1024 * 1024});
  if (result.error || result.signal || result.status !== 0) {
    const error = new Error('Cannot enumerate complete tracked dependency manifests: ' + (result.error?.message ?? result.signal ?? result.stderr ?? result.status));
    error.exitCode = Number.isInteger(result.status) && result.status > 0 && result.status <= 255 ? result.status : 1;
    throw error;
  }
  const bytes = result.stdout;
  if (!Buffer.isBuffer(bytes) || (bytes.length && bytes.at(-1) !== 0)) throw new Error('Incomplete tracked-file inventory');
  if (!isUtf8(bytes)) throw new Error('Tracked-file inventory contains a non-UTF-8 path');
  const raw = bytes.toString('utf8');
  const files = raw ? raw.slice(0, -1).split('\0') : [];
  if (new Set(files).size !== files.length) throw new Error('Duplicate tracked-file inventory');
  if (files.some(name => path.posix.isAbsolute(name) || name.split('/').some(part => !part || part === '.' || part === '..'))) {
    throw new Error('Invalid tracked-file inventory path');
  }
  return files;
}

export function dependencyManifests(repository, git = spawnSync) {
  const files = trackedDependencyFiles(repository, git);
  const tracked = new Set(files);
  const policy = JSON.parse(readFileSync(path.join(repository, 'scripts/dependency-manifests.json'), 'utf8'));
  if (policy.schemaVersion !== 1 || !Array.isArray(policy.activeCargo) ||
      !policy.preservedCargo || typeof policy.preservedCargo !== 'object' || Array.isArray(policy.preservedCargo)) {
    throw new Error('Malformed dependency manifest policy');
  }
  const active = policy.activeCargo;
  const preserved = Object.keys(policy.preservedCargo);
  const classified = [...active, ...preserved];
  if (!active.length || new Set(classified).size !== classified.length) throw new Error('Empty or duplicate Cargo classification');
  for (const name of classified) {
    if (typeof name !== 'string' || path.posix.isAbsolute(name) || name.includes('\\') ||
        name.split('/').some(part => !part || part === '.' || part === '..') ||
        path.posix.basename(name) !== 'Cargo.toml' || !tracked.has(name)) {
      throw new Error('Invalid or untracked Cargo classification: ' + name);
    }
    readFileSync(path.join(repository, name));
  }
  for (const name of preserved) {
    const entry = policy.preservedCargo[name];
    if (!entry || !['historical-reproducer', 'empty-fixture'].includes(entry.kind) ||
        typeof entry.reason !== 'string' || !entry.reason.trim() ||
        typeof entry.issue !== 'string' || !/^https:\/\/github\.com\/[^/]+\/[^/]+\/issues\/[1-9][0-9]*$/u.test(entry.issue)) {
      throw new Error('Preserved manifest requires its purpose and issue: ' + name);
    }
    if (entry.kind === 'empty-fixture' && readFileSync(path.join(repository, name), 'utf8').trim()) {
      throw new Error('Formerly empty fixture needs reclassification: ' + name);
    }
  }
  const cargo = files.filter(name => path.posix.basename(name) === 'Cargo.toml').sort();
  const covered = new Set(classified);
  const missing = cargo.filter(name => !covered.has(name));
  if (missing.length) throw new Error('Unclassified Cargo manifests: ' + missing.join(', '));
  const npm = files.filter(name => path.posix.basename(name) === 'package.json').sort();
  for (const name of npm) JSON.parse(readFileSync(path.join(repository, name), 'utf8'));
  return {activeCargo: [...active].sort(), preservedCargo: [...preserved].sort(), npm, cargo};
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    let repository = process.cwd();
    let cargoOnly = false;
    while (args.length) {
      const flag = args.shift();
      if (flag === '--cargo') cargoOnly = true;
      else if (flag === '--repo' && args.length) repository = path.resolve(args.shift());
      else throw new Error('Usage: dependency-manifest-inventory.mjs [--repo directory] [--cargo]');
    }
    const result = dependencyManifests(repository);
    process.stdout.write(cargoOnly ? result.activeCargo.join('\n') + '\n' : JSON.stringify(result, null, 2) + '\n');
  } catch (error) {
    console.error('dependency manifest coverage: ' + error.message);
    process.exitCode = 1;
  }
}
