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
  // `seed-files.js` first: it is the generated inventory of `seed/*.lino` files
  // that `seed_loader.js` fetches, kept in its own file so two branches that
  // each add a seed file never edit the same lines. See issue #991 and
  // `data/meta/seed-registry.lino`.
  importScripts(withAssetVersion("seed-files.js"));
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

// The module list lives in its own file so it can be union merged: see
// `data/meta/merge-conflict-policy.lino`. Two branches that each add a worker
// module then produce a superset instead of an add/add conflict, and
// `rust-script scripts/normalize-ordered-lists.rs --write` regenerates the list
// from the directory itself.
importScripts(withAssetVersion("worker-modules.js"));

for (const modulePath of self.FORMAL_AI_WORKER_MODULES) {
  importScripts(withAssetVersion(modulePath));
}
