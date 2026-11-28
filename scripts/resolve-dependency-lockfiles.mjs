#!/usr/bin/env node
// Resolve only after a complete Git observation; pass paths directly as argv/cwd.
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {dependencyManifests, trackedDependencyFiles} from './dependency-manifest-inventory.mjs';

export function resolveDependencyLockfiles(repository, kind, run = spawnSync) {
  let resolutions;
  if (kind === '--cargo') {
    resolutions = dependencyManifests(repository).activeCargo.map(manifest => ({
      tool: 'cargo', args: ['update', '--workspace', '--manifest-path', manifest], cwd: repository,
    }));
  } else if (kind === '--javascript') {
    resolutions = trackedDependencyFiles(repository).flatMap(name => {
      const basename = path.posix.basename(name);
      const cwd = path.join(repository, path.posix.dirname(name));
      if (basename === 'bun.lock') return [{tool: 'bun', args: ['install', '--lockfile-only'], cwd}];
      if (basename === 'package-lock.json') return [{
        tool: 'npm', args: ['install', '--package-lock-only', '--ignore-scripts', '--no-audit', '--no-fund'], cwd,
      }];
      return [];
    });
  } else {
    throw new Error('Usage: resolve-dependency-lockfiles.mjs --cargo|--javascript');
  }
  for (const resolution of resolutions) {
    if (process.env.FORMAL_AI_DEPENDENCY_DEBUG === '1') {
      console.error('dependency lock resolution: ' + JSON.stringify(resolution));
    }
    const result = run(resolution.tool, resolution.args, {cwd: resolution.cwd, stdio: 'inherit'});
    if (result.error || result.signal || result.status !== 0) {
      const error = new Error('Dependency resolver failed: ' + JSON.stringify(resolution) + ': ' +
        (result.error?.message ?? result.signal ?? result.status));
      error.exitCode = Number.isInteger(result.status) && result.status > 0 && result.status <= 255 ? result.status : 1;
      throw error;
    }
  }
  return resolutions.length;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) throw new Error('Exactly one dependency resolver kind is required');
    resolveDependencyLockfiles(process.cwd(), process.argv[2]);
  } catch (error) {
    console.error('dependency lock resolution: ' + JSON.stringify(error.message));
    process.exitCode = error.exitCode ?? 1;
  }
}
