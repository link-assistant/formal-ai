// Issue #665's offline installation precaches every asset of the app, a
// second copy of each worker module and seed. Registered on page load, it
// competed with the worker's cold start and roughly doubled the time to a
// ready engine under load (PR #1188: the local web E2E legs saw the composer
// stay disabled past their 5 s window). js/pwa-register.js now registers the
// service worker only once the page has loaded and the app has announced the
// engine ready; this suite runs the script against stub browser globals.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");

/** Runs js/pwa-register.js against stub globals; returns its controls. */
function loadRegister() {
  const listeners = new Map();
  const registrations = [];
  const window = {
    isSecureContext: true,
    addEventListener(type, listener) {
      listeners.set(type, [...(listeners.get(type) || []), listener]);
    },
  };
  const navigator = {
    serviceWorker: {
      register: async (url, options) => {
        registrations.push({ url: String(url), scope: options.scope });
        return { scope: options.scope };
      },
    },
  };
  const document = { baseURI: "https://example.test/formal-ai/" };
  vm.runInNewContext(read("js/pwa-register.js"), { window, navigator, document, URL, Promise, console });
  const fire = async (type) => {
    for (const listener of listeners.get(type) || []) listener();
    await new Promise((resolve) => setImmediate(resolve));
  };
  return { window, registrations, fire };
}

test("the offline installation waits for the engine, not only the page load", async () => {
  const page = loadRegister();
  await page.fire("load");
  assert.deepEqual(page.registrations, [], "registered before the engine was ready");
  await page.fire("formal-ai-ready");
  assert.deepEqual(page.registrations, [
    { url: "https://example.test/formal-ai/service-worker.js", scope: "/formal-ai/" },
  ]);
  assert.equal(page.window.FORMAL_AI_OFFLINE_REGISTRATION.scope, "/formal-ai/");
});

test("an engine ready before the page load registers on the load", async () => {
  const page = loadRegister();
  await page.fire("formal-ai-ready");
  assert.deepEqual(page.registrations, []);
  await page.fire("load");
  assert.equal(page.registrations.length, 1);
});

test("the app announces the ready engine the moment the worker reports it", () => {
  const hooks = read("js/app/application-worker-hooks.jsx");
  const ready = hooks.slice(hooks.indexOf('data.kind === "ready"'));
  assert.match(ready.slice(0, ready.indexOf("return;")), /dispatchEvent\(new Event\("formal-ai-ready"\)\)/);
});

test("the listener is in place before the app can start the worker", () => {
  const html = read("js/app/index.html");
  const register = html.indexOf('<script src="pwa-register.js?v=__FORMAL_AI_ASSET_VERSION__"></script>');
  assert.ok(register > 0, "pwa-register.js must be a classic, versioned script");
  assert.ok(register < html.indexOf('<script src="app.js?'), "pwa-register.js must run before app.js");
});
