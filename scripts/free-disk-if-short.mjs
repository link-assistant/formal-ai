#!/usr/bin/env node
// Issue #1580: run a disk cleanup only on a runner that lacks room for what comes next.
//
// Hive Mind draft run 43738765561 spent 4 of its 30 minutes in Hive Mind's
// `free-disk-space.mjs` (deleting .NET, Android, GHC and CodeQL, then pruning
// every Docker image) on a runner that already had 80 GB free; the ~9 GB image
// it then pulled used 21 GB. Those minutes came out of the job that also runs
// the 20-minute attempt.
//
// Usage: free-disk-if-short.mjs <required-gigabytes> <cleanup command> [args...]
// Set FORMAL_AI_FREE_DISK_DEBUG=1 to print the measurement that decided it.
import {spawnSync} from 'node:child_process';
import {statfsSync} from 'node:fs';
import {pathToFileURL} from 'node:url';

const GIB = 1024 ** 3;

/** True when the filesystem has less than `requiredGigabytes` GiB available. */
export function shortOfRoom(availableBytes, requiredGigabytes) {
  if (!Number.isSafeInteger(availableBytes) || availableBytes < 0) throw Error('available bytes must be a non-negative integer');
  if (!Number.isInteger(requiredGigabytes) || requiredGigabytes <= 0) throw Error('required gigabytes must be a positive integer');
  return availableBytes < requiredGigabytes * GIB;
}

export function availableBytes(path = '/') {
  const stats = statfsSync(path);
  return Number(stats.bavail) * Number(stats.bsize);
}

export function main(argv = process.argv.slice(2), {measure = availableBytes, run = spawnSync, log = console.log, env = process.env} = {}) {
  const [required, command, ...args] = argv;
  if (!/^[1-9][0-9]*$/.test(required ?? '') || !command) throw Error('usage: free-disk-if-short.mjs <required-gigabytes> <cleanup command> [args...]');
  const available = measure('/');
  const gigabytes = (available / GIB).toFixed(1);
  if (env.FORMAL_AI_FREE_DISK_DEBUG === '1') log(`free-disk-if-short: / has ${available} bytes available; required ${required} GiB; cleanup: ${[command, ...args].join(' ')}`);
  if (!shortOfRoom(available, Number(required))) {
    log(`${gigabytes} GiB available on /, at least the ${required} GiB required; skipping the cleanup.`);
    return 0;
  }
  log(`${gigabytes} GiB available on /, less than the ${required} GiB required; running the cleanup.`);
  const result = run(command, args, {stdio: 'inherit'});
  if (result.error) throw result.error;
  return result.status ?? 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try { process.exitCode = main(); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
