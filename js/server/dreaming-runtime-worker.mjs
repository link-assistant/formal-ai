// The dreaming thread js/server/dreaming-runtime.mjs starts for one idle run:
// the twin of the worker thread rust/src/dreaming_runtime.rs
// `start_core_dreaming` spawns. It drops itself to the lowest scheduling
// priority where that is a per-thread setting (Linux, where libuv's
// `setpriority(PRIO_PROCESS, 0)` reaches only the calling thread), then runs
// `runCoreDreamingOnce` against the server's shared foreground counter.

import os from 'node:os';
import { parentPort, workerData } from 'node:worker_threads';

import { runCoreDreamingOnce } from './dreaming-runtime.mjs';

if (process.platform === 'linux') {
  try {
    os.setPriority(0, os.constants.priority.PRIORITY_LOWEST);
  } catch {
    // best effort, like `set_current_thread_priority`: the idle gate still applies
  }
}

try {
  const outcome = runCoreDreamingOnce(workerData.memoryPath, {
    env: workerData.env,
    foreground: new Int32Array(workerData.foreground),
  });
  parentPort.postMessage({ outcome });
} catch (error) {
  parentPort.postMessage({ error: String(error?.message || error) });
}
