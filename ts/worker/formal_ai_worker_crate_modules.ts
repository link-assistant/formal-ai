// The crate modules in the browser worker (R1188-U18, U19, U21).
//
// scripts/generate-worker-crate-modules.mjs writes every module of
// js/agentic/crate/ that a chat route calls, with its imports, to a
// formal_ai_worker_crate_*.js factory. `crateModule(name)` runs a factory once,
// on first use, and hands it the modules it imports, so the browser runs the
// same code as the JavaScript root instead of a second port.
//
// js/agentic/host.mjs is installed over the worker's own seed:
// - a repository seed path reads the text the worker loaded (`SEED_RAW`), or,
//   for a meaning seed the worker itself does not load, the text
//   `loadCrateSeeds` fetched beside it (`CRATE_SEED_RAW`), so the crate modules
//   read the whole meaning lexicon, as the Rust engine does;
// - the seed registry is the meaning seeds of
//   formal_ai_worker_crate_seed_registry.js, the only part a crate module reads;
// - Links Notation is parsed by the seed loader the worker already runs
//   (`FormalAiSeed.parse`), the parser the Node host uses.
// The host is installed again whenever the worker reloads its seed (each load
// sets `CRATE_SEED_RAW` anew), which clears the caches of the crate modules.

const CRATE_HOST_MODULE = "host.mjs";
const CRATE_SEED_REGISTRY = "data/meta/seed-registry.lino";
const crateInstances = new Map();
let crateHostSeed = null;
let CRATE_SEED_RAW = {};

/** The crate meaning seeds `present(fileName)` does not hold, as seed paths. */
function missingCrateSeeds(present) {
  const files = (self.FORMAL_AI_CRATE_MEANING_SEEDS || []).map((seed) => `seed/${seed}.lino`);
  return files.filter((file) => !present(seedFileBaseName(file)));
}

/**
 * Fetch the meaning seeds the worker's own seed list leaves out.
 * `loadSeed` awaits it once the worker seed is loaded; the first load takes the
 * fetch started at boot (below), so it adds no round trip before ready.
 * @param {Record<string, string>} raw the worker's seed texts
 * @returns {Promise<void>}
 */
async function loadCrateSeeds(raw) {
  const missing = missingCrateSeeds((file) => Boolean(seedRawText(raw, file)));
  const prefetch = crateSeedPrefetch;
  crateSeedPrefetch = null;
  const fetchNow = () => (missing.length > 0 ? self.FormalAiSeed.loadAll(missing) : null);
  const reuse = prefetch && prefetch.files.join("\n") === missing.join("\n");
  const loaded = await (reuse ? prefetch.promise : fetchNow());
  CRATE_SEED_RAW = (loaded && loaded.raw) || {};
}

// Once every module has loaded, fetch the meaning seeds missing from the
// worker's list (FORMAL_AI_SEED_FILES, what `loadAll()` fetches) beside it.
let crateSeedPrefetch = null;
Promise.resolve().then(() => {
  const listed = new Set((self.FORMAL_AI_SEED_FILES || []).map(seedFileBaseName));
  const files = self.FormalAiSeed ? missingCrateSeeds((file) => listed.has(file)) : [];
  if (files.length === 0) return;
  crateSeedPrefetch = { files, promise: self.FormalAiSeed.loadAll(files) };
  crateSeedPrefetch.promise.catch(() => {}); // reported where `loadSeed` awaits it
});

/**
 * The seed registry as the crate modules read it: its meaning seeds, in order.
 * @returns {string}
 */
function crateSeedRegistryText() {
  const seeds = self.FORMAL_AI_CRATE_MEANING_SEEDS || [];
  const records = seeds.map((seed) => `  seed ${seed}\n    lexicon meaning`);
  return ["seed_registry", ...records].join("\n");
}

/**
 * A repository file as the crate modules read it.
 * @param {string} relative
 * @returns {string}
 */
function crateHostReadText(relative) {
  if (relative === CRATE_SEED_REGISTRY) return crateSeedRegistryText();
  const file = seedFileBaseName(relative);
  return seedRawText(SEED_RAW, file) || seedRawText(CRATE_SEED_RAW, file);
}

/**
 * The exports of the crate module `name`, its factory run on first use.
 * @param {string} name
 * @returns {object}
 */
function crateInstance(name) {
  if (!crateInstances.has(name)) {
    const factory = (self.FORMAL_AI_CRATE_FACTORIES || {})[name];
    if (typeof factory !== "function") throw new Error(`crate_module_missing:${name}`);
    crateInstances.set(name, factory(crateInstance));
  }
  return crateInstances.get(name);
}

/**
 * The exports of the crate module `name` (`crate/dependency_summarization.mjs`),
 * with the worker installed as its host.
 * @param {string} name
 * @returns {object}
 */
function crateModule(name) {
  if (crateHostSeed !== CRATE_SEED_RAW) {
    crateInstance(CRATE_HOST_MODULE).installHost({
      readText: crateHostReadText,
      parseLino: (text) => self.FormalAiSeed.parse(text),
      realm: self,
    });
    crateHostSeed = CRATE_SEED_RAW;
  }
  return crateInstance(name);
}
