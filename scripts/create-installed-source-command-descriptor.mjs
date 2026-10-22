#!/usr/bin/env node
// Explicit caller installation; transitive dependencies remain caller-trusted.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { readBoundedFileBytes } from '../js/provenance/bounded-file.mjs';
import { observeRuntimeExecutable } from '../js/provenance/fresh-runtime.mjs';

export const installedSourceScripts = Object.freeze({
  sourceCstTransformSha256: 'scripts/source-cst-transform-request.mjs',
  unaryConsolidationSha256: 'scripts/consolidate-stateless-unary-request.mjs',
});

export function createInstalledSourceCommandDescriptor(resourceHome) {
  if (typeof resourceHome !== 'string' || !path.isAbsolute(resourceHome) ||
      resourceHome.includes('\0') || fs.realpathSync(resourceHome) !== resourceHome ||
      !fs.statSync(resourceHome).isDirectory()) {
    throw Error('ExplicitCanonicalResourceHomeRequired');
  }
  const executable = fs.realpathSync(process.execPath);
  const state = fs.statSync(executable);
  if (!state.isFile() || state.size <= 0 || state.size > 256 * 1024 * 1024) {
    throw Error('InstalledNodeBounds');
  }
  const observed = observeRuntimeExecutable();
  if (observed.path !== executable || observed.size > 256 * 1024 * 1024) {
    throw Error('InstalledNodeIdentityChanged');
  }
  const descriptor = {
    nodeExecutable: observed.path,
    resourceHome,
    nodeSha256: observed.sha256,
  };
  for (const [key, relative] of Object.entries(installedSourceScripts)) {
    const bytes = readBoundedFileBytes(resourceHome, relative, 1024 * 1024);
    descriptor[key] = createHash('sha256').update(bytes).digest('hex');
  }
  return descriptor;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv.length !== 4 || process.argv[2] !== '--resource-home') {
    throw Error('Usage: --resource-home ABSOLUTE_INSTALLED_ROOT');
  }
  console.log(JSON.stringify(createInstalledSourceCommandDescriptor(process.argv[3])));
}
