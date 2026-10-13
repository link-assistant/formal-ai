#!/usr/bin/env node
// Overlap full retranslation and full compilation; neither check is optional.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { BLOCKERS_FILE, CALLS_FILE, LEDGER_FILE, projectionPath, scopeModules } from './translate-js-rust.mjs';

export async function runParallelChecks(commands) {
  const children = [];
  const cancellations = [];
  const stopped = new Set();
  let interrupted = false;
  const stop = () => {
    for (const child of children) {
      if (!child.pid || stopped.has(child.pid)) continue;
      stopped.add(child.pid);
      const kill = (signal) => {
        try {
          if (process.platform === 'win32') child.kill(signal);
          else process.kill(-child.pid, signal);
        } catch (error) { if (error.code !== 'ESRCH') throw error; }
      };
      kill('SIGTERM');
      cancellations.push(new Promise((resolve, reject) => setTimeout(() => {
        try { kill('SIGKILL'); resolve(); } catch (error) { reject(error); }
      }, 250)));
    }
  };
  const interrupt = () => { interrupted = true; stop(); };
  process.on('SIGTERM', interrupt);
  process.on('SIGINT', interrupt);
  try {
    const results = await Promise.all(commands.map(([command, ...args]) => new Promise((resolve) => {
      const child = spawn(command, args, { stdio: 'inherit', detached: process.platform !== 'win32' });
      children.push(child);
      child.once('error', (error) => { console.error(error.message); stop(); resolve(1); });
      child.once('close', (code, signal) => {
        const status = code === 0 && !signal ? 0 : 1;
        if (status !== 0) stop();
        resolve(status);
      });
    })));
    return !interrupted && results.every((status) => status === 0) ? 0 : 1;
  } catch (error) {
    stop();
    throw error;
  } finally {
    await Promise.all(cancellations);
    process.removeListener('SIGTERM', interrupt);
    process.removeListener('SIGINT', interrupt);
  }
}

async function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const modules = scopeModules(root);
  const paths = [...modules, ...modules.map(projectionPath), LEDGER_FILE, BLOCKERS_FILE, CALLS_FILE];
  const snapshot = () => paths.map((path) => createHash('sha256').update(readFileSync(join(root, path))).digest('hex'));
  const before = snapshot();
  const translator = join(root, 'scripts/translate-js-rust.mjs');
  const status = await runParallelChecks([
    [process.execPath, translator, '--check', '--meta-language', join(root, '.meta-language')],
    [process.execPath, translator, '--compile'],
  ]);
  if (status !== 0) return status;
  if (JSON.stringify(modules) !== JSON.stringify(scopeModules(root))) throw new Error("Translation module inventory changed during verification");
  if (JSON.stringify(before) !== JSON.stringify(snapshot())) throw new Error('Translation inputs or projections changed during verification');
  return 0;
}
if (process.argv[1] === fileURLToPath(import.meta.url)) process.exitCode = await main();
