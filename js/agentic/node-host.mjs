// The Node host for the agentic planner: repository files from disk, the
// shared Links Notation parser, and a booted worker realm (js/agentic/host.mjs).
// Used by the JavaScript server and by node:test suites; a browser host would
// install the same shape from the worker's own globals.

import { statSync } from 'node:fs';

import { parseLino, readRepoFile } from '../server/lino.mjs';
import { symbolicFromWorker } from '../server/solve.mjs';
import { installHost } from './host.mjs';

function stat(path) {
  try {
    return statSync(path);
  } catch {
    return null;
  }
}

/**
 * Boot `worker` (a js/server/worker-host.mjs `WorkerHost`) and install the
 * planner host over its realm.
 * @param {{boot: () => Promise<object>}} worker
 * @returns {Promise<object>} the worker realm
 */
export async function installNodeHost(worker) {
  const context = await worker.boot();
  installHost({
    readText: readRepoFile,
    parseLino,
    realm: context,
    solve: async (prompt, history) => symbolicFromWorker(await worker.solve(prompt, history)),
    isDirectory: (path) => Boolean(stat(path)?.isDirectory()),
    isFile: (path) => Boolean(stat(path)?.isFile()),
    currentDirectory: () => process.cwd(),
  });
  return context;
}
