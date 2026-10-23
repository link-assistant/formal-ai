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

// Every address the chain loads — `seed/*.lino`, the generated module list,
// `formal_ai_worker.wasm` — is web-root relative, but this entry lives one
// directory down in `js/worker/` (issue #1138). Resolve `importScripts` and
// `fetch` against the parent directory so the rest of the chain is unchanged
// wherever the entry file sits.
try {
  const webRoot = new URL("..", self.location.href);
  const fromWebRoot = (url) =>
    typeof url === "string" && url ? new URL(url, webRoot).href : url;
  const nativeImportScripts = importScripts;
  const networkFailures = new Map();
  importScripts = (...urls) => {
    for (const url of urls.map(fromWebRoot)) {
      const previous = networkFailures.get(url);
      if (previous && previous.attempts >= 2) throw previous.error;
      for (let attempt = (previous?.attempts || 0) + 1; attempt <= 2; attempt++) {
        try {
          nativeImportScripts(url);
          networkFailures.delete(url);
          break;
        } catch (error) {
          // A failed network import has not executed the script. Syntax and
          // runtime failures may have side effects and must never be replayed.
          if (error?.name !== "NetworkError" || !String(error.message).includes(url)) throw error;
          networkFailures.set(url, { attempts: attempt, error });
          if (attempt === 2) {
            console.error("WorkerAssetImportFailed", { url, attempts: attempt, error: String(error) });
            throw error;
          }
          console.warn("WorkerAssetImportRetry", { url, attempt, maximumAttempts: 2 });
        }
      }
    }
  };
  const nativeFetch = fetch;
  fetch = (url, options) => nativeFetch(fromWebRoot(url), options);
} catch (_error) {
  // Hosts without a usable location keep plain worker-relative resolution.
}

importScripts(withAssetVersion("worker-script-prerequisites.js"));

// The completed set is private to this actual import closure, not caller globals.
const loadWorkerAsset = createWorkerAssetLoader(importScripts, withAssetVersion);

try {
  // `seed-files.js` first: it is the generated inventory of `seed/*.lino` files
  // that `seed_loader.js` fetches, kept in its own file so two branches that
  // each add a seed file never edit the same lines. See issue #991 and
  // `data/meta/seed-registry.lino`; shipped responses also boot offline.
  loadWorkerAsset("worker-bootstrap-responses.js");
  loadWorkerAsset("seed-files.js");
  loadWorkerAsset("parser-cache-primordials.js");
  loadWorkerAsset("seed_loader.js");
} catch (error) {
  // The seed parser is required by dependent modules. Report a failed boot
  // instead of leaving the application waiting for a ready event forever.
  console.error("WorkerBootstrapFailed", error);
  throw error;
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

importScripts(withAssetVersion("worker-modules.js"));

loadWorkerModules(self.FORMAL_AI_WORKER_MODULES, loadWorkerAsset);
