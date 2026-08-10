// Universal solver implementation for the demo worker.
//
// Every reasoning path here mirrors the Rust `FormalAiEngine` in
// `src/solver.rs` so the website, CLI, Telegram bot, library, and HTTP server
// all produce the same answers for the same prompts. The answer the user
// sees is always a projection of an append-only event log — there is no
// hardcoded prompt→answer table.
//
// All multilingual phrases, concept summaries, and the tool registry are
// loaded from `seed/*.lino` files at startup via `seed_loader.js`. Editing a
// `.lino` file is enough to retune the agent — no JavaScript change required.

function currentAssetVersion() {
  try {
    const search = self.location && self.location.search;
    const match = search && /[?&]v=([^&]+)/.exec(search);
    return match ? decodeURIComponent(match[1].replace(/\+/g, " ")) : "";
  } catch (_error) {
    return "";
  }
}

function withAssetVersion(url) {
  const version = currentAssetVersion();
  if (!version) return url;
  return `${url}${url.includes("?") ? "&" : "?"}v=${encodeURIComponent(
    version,
  )}`;
}

try {
  importScripts(withAssetVersion("seed_loader.js"));
} catch (_error) {
  // Seed loader is optional: tests that mock the worker may exclude it.
}

try {
  importScripts(withAssetVersion("web-search-component.bundle.js"));
} catch (_error) {
  // Keep the bounded local implementation available when a downstream embed
  // deliberately omits the optional browser component bundle.
  self.FormalAIWebSearchComponent = Object.freeze({
    package: "unavailable",
    defaultProviderIds: [],
    orderProviders: (providers) => providers.slice(),
    fuseResults: (_results, _k, evidence) => {
      evidence.push("web_search:component_error:@link-assistant/web-search:unavailable");
      return null;
    },
    fetchUrl: async (url, evidence) => {
      evidence.push("http_fetch:component_error:unavailable");
      return fetch(url, { method: "GET", mode: "cors" });
    },
  });
}

const FORMAL_AI_WORKER_MODULES = [
  "worker/formal_ai_worker_00.js",
  "worker/formal_ai_worker_01.js",
  "worker/formal_ai_worker_02.js",
  "worker/formal_ai_worker_03.js",
  "worker/formal_ai_worker_04.js",
  "worker/formal_ai_worker_05.js",
  "worker/formal_ai_worker_06.js",
  "worker/formal_ai_worker_07.js",
  "worker/formal_ai_worker_08.js",
  "worker/formal_ai_worker_09.js",
  "worker/formal_ai_worker_10.js",
  "worker/formal_ai_worker_11.js",
  "worker/formal_ai_worker_12.js",
  "worker/formal_ai_worker_13.js",
  "worker/formal_ai_worker_14.js",
  "worker/formal_ai_worker_15.js",
  "worker/formal_ai_worker_16.js",
  "worker/formal_ai_worker_17.js",
  "worker/formal_ai_worker_18.js",
  "worker/formal_ai_worker_19.js",
  "worker/formal_ai_worker_20.js",
  "worker/formal_ai_worker_21.js",
  "worker/formal_ai_worker_22.js",
  "worker/formal_ai_worker_23.js",
];

for (const modulePath of FORMAL_AI_WORKER_MODULES) {
  importScripts(withAssetVersion(modulePath));
}
