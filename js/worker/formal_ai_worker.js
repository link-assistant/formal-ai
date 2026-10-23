// Browser entry for shared solver modules and trusted seeds (data/meta/seed-registry.lino).

function withAssetVersion(url) {
  try {
    const raw = /[?&]v=([^&]+)/.exec(self.location?.search || "")?.[1];
    if (raw) return `${url}${url.includes("?") ? "&" : "?"}v=${encodeURIComponent(decodeURIComponent(raw.replace(/\+/g, " ")))}`;
  } catch (_error) {}
  return url;
}

// This entry lives in worker/; shared scripts, seeds and WASM are web-root relative.
// Resolve imports and fetches from its parent (issue #1138).
try {
  const webRoot = new URL("..", self.location.href);
  const fromWebRoot = (url) =>
    typeof url === "string" && url ? new URL(url, webRoot).href : url;
  const nativeImportScripts = importScripts;
  const networkFailures = new Map(), maximumImportAttempts = 2;
  importScripts = (...urls) => {
    for (const url of urls.map(fromWebRoot)) {
      const previous = networkFailures.get(url);
      if (previous && previous.attempts >= maximumImportAttempts) throw previous.error;
      for (let attempt = (previous?.attempts || 0) + 1; attempt <= maximumImportAttempts; attempt++) {
        try {
          nativeImportScripts(url);
          networkFailures.delete(url);
          break;
        } catch (error) {
          // A failed network import has not executed the script. Syntax and
          // runtime failures may have side effects and must never be replayed.
          if (error?.name !== "NetworkError" || !String(error.message).includes(url)) throw error;
          networkFailures.set(url, { attempts: attempt, error });
          if (attempt === maximumImportAttempts) {
            console.error("WorkerAssetImportFailed", { url, attempts: attempt, error: String(error) });
            throw error;
          }
          console.warn("WorkerAssetImportRetry", { url, attempt, maximumAttempts: maximumImportAttempts });
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
  // Load the seed inventory and parser before dependent modules (issue #991).
  // Bootstrap responses also support offline answers.
  loadWorkerAsset("worker-bootstrap-responses.js");
  loadWorkerAsset("seed-files.js");
  loadWorkerAsset("parser-cache-primordials.js");
  loadWorkerAsset("seed_loader.js");
} catch (error) {
  // Required bootstrap failures must reach the application's worker error handler.
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
