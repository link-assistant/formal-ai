// @ts-check
// How long a spec waits for the chat composer to become enabled, which
// happens when the browser worker reports ready (`workerReady` in
// js/app/application-worker-hooks.jsx). Every composer-enabled wait uses this one
// bound. The worker's cold start loads its modules, the seeds and the WASM
// engine; with four Playwright workers on a four-vCPU runner it measured
// about 7 s at the worst in CI (PR #1188), so 20 s leaves room for a loaded
// runner while staying well inside the 30 s per-test timeout.
const { test } = require('@playwright/test');

const WORKER_READY_TIMEOUT_MS = 20_000;

/**
 * Give the running test room for one more cold worker start. A test that
 * reloads the page (to switch the UI language, say) boots the worker again
 * each time, so a fixed per-test timeout would shrink with every reload; call
 * this before such a reload.
 */
function allowWorkerBoot() {
  const info = test.info();
  info.setTimeout(info.timeout + WORKER_READY_TIMEOUT_MS);
}

module.exports = { WORKER_READY_TIMEOUT_MS, allowWorkerBoot };
