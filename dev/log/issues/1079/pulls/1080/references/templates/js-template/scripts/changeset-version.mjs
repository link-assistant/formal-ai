#!/usr/bin/env bun

/**
 * Custom changeset version script that ensures package-lock.json is synchronized
 * with package.json after version bumps.
 *
 * This script:
 * 1. Detects the JavaScript package root (supports both single-language and multi-language repos)
 * 2. Runs `changeset version` to update package versions
 * 3. Runs `npm install` to synchronize package-lock.json with the new versions
 *
 * Configuration:
 * - CLI: --js-root <path> to explicitly set JavaScript root
 * - Environment: JS_ROOT=<path>
 *
 * Uses link-foundation libraries:
 * - use-m: Dynamic package loading without package.json dependencies
 * - command-stream: Modern shell command execution with streaming support
 *
 * Repository layout: single-language and multi-language repositories are both
 * supported through a configurable JavaScript package root.
 */

import { getJsRoot, needsCd, parseJsRootConfig } from './js-paths.mjs';
import { bootstrapDependencies } from './bootstrap-dependencies.mjs';
import { loadCommandStream } from './use-module.mjs';

// Import command-stream for shell command execution
// Loaded through bootstrapDependencies: when the use-m CDN is unreachable,
// this script reports the failure and writes its outputs, and the process
// does not die inside module initialisation.
const [{ $ }] = await bootstrapDependencies([loadCommandStream]);

// Store the original working directory to restore after cd commands
// IMPORTANT: command-stream's cd is a virtual command that calls process.chdir()
const originalCwd = process.cwd();

try {
  // Get JavaScript package root (auto-detect or use explicit config)
  const jsRootConfig = parseJsRootConfig();
  const jsRoot = getJsRoot({ jsRoot: jsRootConfig, verbose: true });

  console.log('Running changeset version...');

  // IMPORTANT: cd is a virtual command that calls process.chdir(), so we restore after
  if (needsCd({ jsRoot })) {
    await $`cd ${jsRoot} && npx changeset version`;
    process.chdir(originalCwd);
  } else {
    await $`npx changeset version`;
  }

  console.log('\nSynchronizing package-lock.json...');

  // Use --legacy-peer-deps to handle peer dependency conflicts
  // --legacy-peer-deps keeps a peer-dependency conflict from failing the
  // lockfile refresh with npm ERESOLVE.
  if (needsCd({ jsRoot })) {
    await $`cd ${jsRoot} && npm install --package-lock-only --legacy-peer-deps`;
    process.chdir(originalCwd);
  } else {
    await $`npm install --package-lock-only --legacy-peer-deps`;
  }

  console.log('\n✅ Version bump complete with synchronized package-lock.json');
} catch (error) {
  // Restore cwd on error
  process.chdir(originalCwd);
  console.error('Error during version bump:', error.message);
  if (process.env.DEBUG) {
    console.error('Stack trace:', error.stack);
  }
  process.exit(1);
}
