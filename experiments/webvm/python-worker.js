/* Issue #670: isolated measurement worker; no host shell or host filesystem. */
let runtime;
self.onmessage = async ({ data }) => {
  const started = performance.now();
  try {
    if (data.kind === "load") {
      importScripts("../../js/worker/formal_ai_worker_browser_runtime.js");
      runtime = await loadBrowserPythonRuntime();
      postMessage({ kind: "ready", startupMs: performance.now() - started,
        runtime: BROWSER_PYTHON_RUNTIME, scope: "browser-worker" });
      return;
    }
    if (!runtime) throw new Error("Load Python explicitly before running code");
    const output = [];
    runtime.setStdout({ batched: line => output.push(String(line)) });
    runtime.setStderr({ batched: line => output.push(String(line)) });
    const result = await runtime.runPythonAsync(String(data.code));
    if (result !== undefined && result !== null && typeof result.destroy === "function") result.destroy();
    postMessage({ kind: "observed", elapsedMs: performance.now() - started,
      output, assertionsPassed: true, scope: "browser-worker" });
  } catch (error) {
    postMessage({ kind: "failed", error: String(error.message || error),
      elapsedMs: performance.now() - started, assertionsPassed: false,
      scope: "browser-worker" });
  }
};
