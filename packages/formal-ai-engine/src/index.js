/** Browser-only engine client. Serve the packaged assets on the worker's origin. */
export async function createEngine(options = {}) {
  const base = new URL(options.assetBase || "../assets/", import.meta.url);
  if (typeof Worker !== "function") throw new Error("This package requires a browser Worker host");
  const worker = new Worker(new URL("worker/formal_ai_worker.js", base));
  let sequence = 0;
  let closed = false;
  const pending = new Map();
  let readyResolve, readyReject;
  const ready = new Promise((resolve, reject) => { readyResolve = resolve; readyReject = reject; });
  const readyTimeout = setTimeout(() => readyReject(new Error("Engine startup timed out")), options.timeoutMs || 60000);
  function fail(error) {
    readyReject(error);
    for (const entry of pending.values()) { clearTimeout(entry.timer); entry.reject(error); }
    pending.clear();
  }
  worker.onerror = event => fail(new Error(event.message || "Engine worker failed"));
  worker.onmessage = ({ data }) => {
    if (data.kind === "ready") {
      clearTimeout(readyTimeout);
      if (data.mode === "engine unavailable") readyReject(new Error(data.engineError || data.mode));
      else readyResolve(data);
      return;
    }
    if (data.kind === "engine_unavailable") { fail(new Error(data.error || "Engine unavailable")); return; }
    const entry = pending.get(data.requestId);
    if (entry) { pending.delete(data.requestId); clearTimeout(entry.timer); entry.resolve(data); }
  };
  function request(payload) {
    if (closed) return Promise.reject(new Error("Engine disposed"));
    const requestId = `engine-${++sequence}`;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => { pending.delete(requestId); reject(new Error("Engine request timed out")); }, options.timeoutMs || 60000);
      pending.set(requestId, { resolve, reject, timer });
      worker.postMessage({ ...payload, requestId });
    });
  }
  try { await ready; } catch (error) { clearTimeout(readyTimeout); worker.terminate(); throw error; }
  // Classic shared bindings publish browser memory and seed APIs without a second solver.
  await import(new URL("memory.js", base).href);
  const memory = globalThis.FormalAiMemory;
  if (!memory) { worker.terminate(); throw new Error("Memory bindings unavailable"); }
  return {
    async solve(prompt, context = {}) {
      return request({ prompt, history: context.history || [], prefs: context.preferences || {},
        memoryEvents: await memory.listEvents(), userContext: context.userContext || {} });
    },
    memory: {
      list: () => memory.listEvents(),
      append: event => memory.appendEvent(event),
      clear: () => memory.clearEvents(),
      async exportBundle() {
        const seed = await request({ kind: "seed_dump" });
        return memory.exportFullMemory({ seed: { raw: seed.raw || {} }, events: await memory.listEvents() });
      },
      async importBundle(text) {
        const bundle = memory.importFullMemory(text);
        await memory.importEvents(bundle.events || []);
        return bundle;
      },
    },
    dispose() { closed = true; clearTimeout(readyTimeout); fail(new Error("Engine disposed")); worker.terminate(); },
  };
}
