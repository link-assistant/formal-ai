import { spawn } from 'node:child_process';
import { mkdtempSync, realpathSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const active = new Map();
const signals = ['SIGINT', 'SIGTERM', 'SIGHUP'];
let terminatingSignal = null;

function forwardSignal(signal) {
  terminatingSignal ??= signal;
  for (const child of active.keys()) child.kill(signal);
}

const handlers = new Map(signals.map(signal => [signal, () => forwardSignal(signal)]));

function release(child) {
  const directory = active.get(child);
  if (!directory) return;
  active.delete(child);
  rmSync(directory, { recursive: true, force: true });
  if (active.size) return;
  for (const [signal, handler] of handlers) process.removeListener(signal, handler);
  if (terminatingSignal) process.kill(process.pid, terminatingSignal);
}

// Both native execution lanes share this boundary; listing never enters it.
// The child retains its real PID, streams, argv, working directory and profile.
export function spawnWithIsolatedTestCache(executable, argumentsList, options = {}) {
  if (terminatingSignal) throw new Error('test execution interrupted');
  const directory = mkdtempSync(join(realpathSync(tmpdir()), 'formal-ai-test-cache-'));
  let child;
  try {
    child = spawn(executable, argumentsList, {
      ...options,
      env: { ...(options.env ?? process.env),
        FORMAL_AI_SOURCE_CACHE_DIR: directory,
        FORMAL_AI_CACHE_DIR: directory },
    });
  } catch (error) {
    rmSync(directory, { recursive: true, force: true });
    throw error;
  }
  if (!active.size) {
    for (const [signal, handler] of handlers) process.on(signal, handler);
  }
  active.set(child, directory);
  child.once('close', () => release(child));
  return child;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [executable, ...argumentsList] = process.argv.slice(2);
  if (!executable) throw new Error('usage: test-cache-isolation.mjs executable [arguments...]');
  const child = spawnWithIsolatedTestCache(executable, argumentsList, { stdio: 'inherit' });
  child.once('error', error => {
    console.error(error.message);
    process.exitCode = 1;
  });
  child.once('close', (code, signal) => {
    if (signal && !terminatingSignal) process.kill(process.pid, signal);
    else process.exitCode = code ?? 1;
  });
}
