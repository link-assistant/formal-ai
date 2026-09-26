// Issue #1138 plan 06 leaves 15-16: the browser probes honestly and downloads
// Python only after an explicit, size-labelled user action.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";

const FAMILY = [
  ["en", "Run this and tell me exactly what it prints: print(sum(range(1, 11)))"],
  ["ru", "Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))"],
  ["hi", "इसे चलाओ और मुझे ठीक-ठीक बताओ कि यह क्या छापता है: print(sum(range(1, 11)))"],
  ["zh", "运行这个并准确告诉我它打印了什么：print(sum(range(1, 11)))"],
  ["es", "Ejecuta esto y dime exactamente qué imprime: print(sum(range(1, 11)))"],
];

function wasmDigest() {
  return createHash("sha256")
    .update(readFileSync(path.join(REPO_ROOT, "js/formal_ai_worker.wasm")))
    .digest("hex");
}

test("the browser probes a downloadable runtime and is honest in five languages", async () => {
  const context = createWorkerContext();
  await evaluate(context, "loadSeed()");
  const probe = plain(evaluate(context, 'browserExecutionProbe("python")'));
  assert.equal(probe.status, "available_to_download");
  assert.equal(probe.runtime, "pyodide");
  assert.ok(probe.downloadBytes > 512 * 1024, "runtime bytes stay outside the worker budget");

  for (const [language, prompt] of FAMILY) {
    context.__language = language;
    context.__prompt = prompt;
    const message = evaluate(context, "browserRuntimeMessage(__language)");
    assert.match(message, /20 (?:MB|МБ)/u, `${language}: download size is disclosed`);
    const answer = plain(await evaluate(context, "executeBrowserCodeRequest(__prompt)"));
    assert.equal(answer.runtimeOffer.runtime, "pyodide");
    assert.ok(!answer.content.includes("55"), `${language}: no observation means no answer value`);
  }
});

test("an explicit load enables observed Python output without changing shipped WASM", async () => {
  const before = wasmDigest();
  const context = createWorkerContext();
  await evaluate(context, "loadSeed()");
  await evaluate(context, `loadBrowserPythonRuntime(async () => ({
    setStdout(options) { this.stdout = options.batched; },
    setStderr(options) { this.stderr = options.batched; },
    async runPythonAsync() { this.stdout("55"); return undefined; },
  }))`);
  context.__prompt = FAMILY[0][1];
  const answer = plain(await evaluate(context, "executeBrowserCodeRequest(__prompt)"));
  assert.equal(answer.intent, "browser_code_execution");
  assert.match(answer.content, /\b55\b/u);
  assert.ok(answer.evidence.includes("execution_exit:0"));
  assert.equal(wasmDigest(), before, "lazy runtime loading never modifies the shipped worker binary");

  const app = readFileSync(path.join(REPO_ROOT, "js/app/main.jsx"), "utf8");
  assert.match(app, /data-testid="setting-browser-runtime-load"/u);
  assert.match(app, /onClick=\{loadBrowserRuntime\}/u);
  const catalog = readFileSync(
    path.join(REPO_ROOT, "js/i18n-catalog-messages.lino"),
    "utf8",
  );
  assert.match(catalog, /Load Pyodide \(about 20 MB\)/u);
});
