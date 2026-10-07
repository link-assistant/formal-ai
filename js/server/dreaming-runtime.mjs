// Cooperative, default-on dreaming for the JavaScript server: the twin of
// rust/src/dreaming_runtime.rs (`ForegroundActivity`, `core_is_idle`,
// `start_core_dreaming`, `run_core_dreaming_once`, `dreaming_disabled`,
// `write_learning_cycle_record`, `learning_cycle_record_path`) and
// rust/src/dreaming.rs `compose_recipe_with_amendments`.
//
// The run never starts while a request is in flight and waits for a real idle
// window after the latest one. It runs on a `worker_threads` thread (the Rust
// worker is an OS thread) so a replay that blocks on the solver realm never
// stalls the event loop, and it reads the same shared foreground counter, so a
// request that arrives mid-run cancels it before anything is written. Without
// persisted issue-494 consent it keeps the learned amendments, patterns and
// trials but strips every deletion: learning is default-on, freeing default-off.

import fs from 'node:fs';
import path from 'node:path';
import { Worker } from 'node:worker_threads';

import { recordedEvents, runIdleAnticipation } from './anticipation.mjs';
import { applyDreamingPlan } from './dreaming-plan.mjs';
import { memoizedReplay, solveIntent } from './dreaming-replay.mjs';
import { loadDataDocument } from './dreaming-support.mjs';
import { googleTrendsLearningCycle, learningCycleLinksNotation } from './learning-cycle.mjs';
import { SyncStore, parseBoolEnv, writeAtomic } from './memory-store.mjs';
import { serverMessage } from './messages.mjs';
import { retainedAmendments } from './standing-requirements.mjs';
import { autoFreeSpaceEnabled, planForRealStorage } from './storage-policy.mjs';

export const DEFAULT_IDLE_SECONDS = 60;
export const DEFAULT_INTERVAL_SECONDS = 6 * 60 * 60;

/** The in-flight foreground count, shared with the dreaming thread. */
export const FOREGROUND = new Int32Array(new SharedArrayBuffer(4));
let lastForegroundMs = Date.now();

/**
 * Mirrors `ForegroundActivity::begin` in rust/src/dreaming_runtime.rs: count
 * one request in. The returned `end` (the Rust `Drop`) counts it out once and
 * restarts the idle clock.
 */
export function beginForegroundActivity() {
  Atomics.add(FOREGROUND, 0, 1);
  lastForegroundMs = Date.now();
  let ended = false;
  return {
    end() {
      if (ended) return;
      ended = true;
      lastForegroundMs = Date.now();
      Atomics.sub(FOREGROUND, 0, 1);
    },
  };
}

/** Mirrors `core_is_idle` in rust/src/dreaming_runtime.rs. */
export function coreIsIdle(idleSeconds, now = Date.now()) {
  if (Atomics.load(FOREGROUND, 0) !== 0) return false;
  return Math.floor((now - lastForegroundMs) / 1000) >= idleSeconds;
}

/** Mirrors `dreaming_disabled` in rust/src/dreaming_runtime.rs: only an explicit false spelling turns it off. */
export function dreamingDisabled(env = process.env) {
  return parseBoolEnv(env.FORMAL_AI_DREAMING) === false;
}

/** Rust `Path::with_extension("recipe.lino")`. */
export function recipePath(memoryPath) {
  const extension = path.extname(memoryPath);
  return `${memoryPath.slice(0, memoryPath.length - extension.length)}.recipe.lino`;
}

/** Mirrors `compose_recipe_with_amendments` in rust/src/dreaming.rs. */
export function composeRecipeWithAmendments(events, env = process.env) {
  let composed = loadDataDocument('dreaming-recipe.lino', env).trimEnd();
  for (const amendment of retainedAmendments(events)) {
    composed += `\n  meta_amendment ${amendment.id}\n    topic: ${amendment.topic}\n    rule: ${amendment.rule}`
      + '\n    applied_by: fn_solve_with_standing_requirements\n';
  }
  return `${composed}\n`;
}

const emptyOutcome = () => ({
  removed_events: 0,
  estimated_reclaimed_bytes: 0,
  learned_amendments: 0,
  learned_patterns: 0,
  recorded_failures: 0,
  recorded_trials: 0,
  learned_algorithm_candidates: 0,
});

/**
 * Mirrors `run_core_dreaming_once` in rust/src/dreaming_runtime.rs: one idle
 * run over the memory log at `memoryPath`. A missing log is an empty store
 * that is never created; an incompatible one throws (the Rust `io::Result`
 * error) before anything is written. `foreground` is the shared request
 * counter checked between planning and application. After the dreaming
 * plan, the run anticipates the next requests (js/server/anticipation.mjs,
 * writing `<memory>.anticipation.lino`) and leaves the proposal-only
 * learning-cycle record (`<memory>.learning-cycle.lino`). `classify` is the
 * offline solver's intent for a prompt.
 */
export function runCoreDreamingOnce(memoryPath, {
  env = process.env, replay = memoizedReplay(), foreground = FOREGROUND, classify = solveIntent,
} = {}) {
  const store = fs.existsSync(memoryPath) ? SyncStore.open({ ...env, FORMAL_AI_MEMORY_PATH: memoryPath }, memoryPath) : null;
  if (store && !store.compatible) throw new Error(serverMessage('memory_schema_write_refused'));
  let events = store ? store.events : [];
  const plan = planForRealStorage(events, memoryPath, 0, replay);
  if (!autoFreeSpaceEnabled(memoryPath)) {
    plan.actions = [];
    plan.selected_reclaim_bytes = 0;
  }
  // Mid-run cancellation point: planning is the expensive half.
  if (Atomics.load(foreground, 0) !== 0) return emptyOutcome();
  const dreamed = applyDreamingPlan(events, plan, replay);
  if (Object.values(dreamed.outcome).some((count) => count > 0)) {
    events = dreamed.events;
    persist(store, events);
    writeAtomic(recipePath(memoryPath), composeRecipeWithAmendments(events, env));
  }
  const anticipated = runIdleAnticipation(memoryPath, events, { env, classify });
  if (recordedEvents(anticipated.outcome) > 0) persist(store, anticipated.events);
  writeLearningCycleRecord(memoryPath);
  return dreamed.outcome;
}

/** Persist `events` through the opened store (a missing log never gains records). */
function persist(store, events) {
  if (!store) return;
  store.events = events;
  store.persist();
}

/** Mirrors `learning_cycle_record_path`: `Path::with_extension("learning-cycle.lino")`. */
export function learningCycleRecordPath(memoryPath) {
  const extension = path.extname(memoryPath);
  return `${memoryPath.slice(0, memoryPath.length - extension.length)}.learning-cycle.lino`;
}

/** Mirrors `write_learning_cycle_record`: the Google Trends cycle, proposal-only, beside the log. */
export function writeLearningCycleRecord(memoryPath) {
  writeAtomic(learningCycleRecordPath(memoryPath), `${learningCycleLinksNotation(googleTrendsLearningCycle())}\n`);
}

/** Run one pass on the dreaming thread; resolves with its outcome or rejects with its error. */
export function runOnDreamingThread(memoryPath, env = process.env) {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL('./dreaming-runtime-worker.mjs', import.meta.url), {
      workerData: { memoryPath, env, foreground: FOREGROUND.buffer },
    });
    worker.unref();
    worker.once('message', (message) => (message.error === undefined ? resolve(message.outcome) : reject(new Error(message.error))));
    worker.once('error', reject);
  });
}

/**
 * Mirrors `start_core_dreaming` in rust/src/dreaming_runtime.rs: unless
 * FORMAL_AI_DREAMING is an explicit false spelling, check every idle period
 * and, once idle, run one pass, then rest for the interval. Every timer is
 * unrefed, so dreaming never keeps the server process alive. Returns the
 * handle whose `stop()` ends the loop, or null when dreaming is off.
 */
export function startCoreDreaming({
  env = process.env, memoryPath, idleSeconds = DEFAULT_IDLE_SECONDS, intervalSeconds = DEFAULT_INTERVAL_SECONDS,
  run = runOnDreamingThread,
} = {}) {
  if (dreamingDisabled(env)) return null;
  lastForegroundMs = Date.now();
  let timer = null;
  let stopped = false;
  const schedule = (seconds) => {
    if (stopped) return;
    timer = setTimeout(tick, seconds * 1000);
    timer.unref();
  };
  async function tick() {
    if (!coreIsIdle(idleSeconds)) {
      schedule(idleSeconds);
      return;
    }
    try {
      await run(memoryPath, env);
    } catch (error) {
      if (parseBoolEnv(env.FORMAL_AI_DREAMING_DEBUG) === true) {
        process.stderr.write(`${serverMessage('dreaming_background_failed', { error: error?.message || error })}\n`);
      }
    }
    schedule(intervalSeconds);
  }
  schedule(idleSeconds);
  return {
    stop() {
      stopped = true;
      if (timer) clearTimeout(timer);
    },
  };
}
