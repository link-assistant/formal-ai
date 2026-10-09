// Bound independent source serialization work while preserving input receipt order.
import { Worker } from 'node:worker_threads';
import { availableParallelism } from 'node:os';

export async function mapSourceNetworks(paths, workerUrl, workerData, onProgress = () => {}) {
  if (paths.length === 0) return [];
  const count = Math.min(4, availableParallelism(), paths.length);
  const workers = [], results = new Array(paths.length);
  let next = 0, completed = 0;
  try {
    return await new Promise((resolve, reject) => {
      const assign = (worker) => {
        if (next < paths.length) worker.postMessage({ index: next, path: paths[next++] });
      };
      for (let index = 0; index < count; index += 1) {
        const worker = new Worker(workerUrl, { workerData });
        workers.push(worker);
        worker.on('error', reject);
        worker.on('exit', (code) => {
          if (completed < paths.length) reject(new Error('source serializer exited before completion: ' + code));
        });
        worker.on('message', (message) => {
          if (message.error) { reject(new Error(message.error)); return; }
          if (message.event) { onProgress(message); return; }
          if (!Number.isSafeInteger(message.index) || message.index < 0 || message.index >= paths.length
            || results[message.index] !== undefined || message.receipt?.path !== paths[message.index]) {
            reject(new Error('source serializer returned an invalid receipt identity')); return;
          }
          results[message.index] = message.receipt;
          completed += 1;
          if (completed === paths.length) resolve(results);
          else assign(worker);
        });
        assign(worker);
      }
    });
  } finally {
    await Promise.all(workers.map((worker) => worker.terminate()));
  }
}
