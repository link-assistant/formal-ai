// Dreaming replay through the production amendment path, synchronously:
// rust/src/dreaming_application.rs `replay_answer_with_amendments`.
//
// Rust replays a stored task input through `UniversalSolver` inline. The
// JavaScript solver is the worker realm, whose `solve` is asynchronous, while
// `SyncStore::import_links_notation` (and so the auto-free-space pass on
// import) is synchronous. The bridge runs one solver realm on a
// `worker_threads` thread, booted lazily on the first replay a pass really
// needs, and blocks on a shared flag (`Atomics.wait`) until the answer is
// posted, so the planner sees the same answer the live surfaces would give.

import { MessageChannel, Worker, receiveMessageOnPort } from 'node:worker_threads';

import { serverMessage } from './messages.mjs';

const REPLAY_TIMEOUT_MS = 120000;

let bridge = null;

function boot() {
  if (bridge) return bridge;
  const { port1, port2 } = new MessageChannel();
  const signal = new Int32Array(new SharedArrayBuffer(4));
  const worker = new Worker(new URL('./dreaming-replay-worker.mjs', import.meta.url), {
    workerData: { port: port2, signal },
    transferList: [port2],
  });
  // An idle bridge never keeps the process alive.
  worker.unref();
  port1.unref();
  bridge = { worker, port: port1, signal };
  return bridge;
}

/**
 * Mirrors rust/src/dreaming_application.rs `replay_answer_with_amendments`.
 * @param {string} input
 * @param {Array<{id: string, topic: string, rule: string}>} amendments
 * @returns {string}
 */
export function replayAnswerWithAmendments(input, amendments) {
  const { port, signal } = boot();
  Atomics.store(signal, 0, 0);
  port.postMessage({ input, amendments });
  const deadline = Date.now() + REPLAY_TIMEOUT_MS;
  let received;
  for (;;) {
    Atomics.wait(signal, 0, 0, Math.max(deadline - Date.now(), 1));
    received = receiveMessageOnPort(port);
    if (received || Date.now() >= deadline) break;
    // The flag rose before the message landed; give the port a moment.
    if (Atomics.load(signal, 0) === 1) Atomics.wait(signal, 0, 1, 5);
  }
  if (!received) {
    throw new Error(serverMessage('dreaming_replay_timeout', { seconds: REPLAY_TIMEOUT_MS / 1000 }));
  }
  if (received.message.error !== undefined) throw new Error(received.message.error);
  return received.message.answer;
}

/**
 * A replay function for one planning pass: identical `(input, amendments)`
 * requests are answered once, so `apply_dreaming_plan`'s re-plan of an
 * unchanged store does not solve every task twice.
 * @param {(input: string, amendments: Array<object>) => string} [replay]
 */
export function memoizedReplay(replay = replayAnswerWithAmendments) {
  const cache = new Map();
  return (input, amendments) => {
    const key = JSON.stringify([input, amendments.map(({ id, topic, rule }) => [id, topic, rule])]);
    if (!cache.has(key)) cache.set(key, replay(input, amendments));
    return cache.get(key);
  };
}
