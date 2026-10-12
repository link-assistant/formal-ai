// Seed data the server reads directly: the public model aliases, the agent
// info record, and the merged seed bundle (`/v1/bundle`). Mirrors
// rust/src/seed/model_aliases.rs and rust/src/seed.rs `bundle_from_files`.

import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

let aliases = null;

function modelAliases() {
  if (aliases) return aliases;
  const root = parseLino(readRepoFile('data/seed/model-aliases.lino'));
  const canonical = childValue(root, 'canonical');
  const list = childrenNamed(root, 'alias').map((node) => node.value);
  if (!list.some((alias) => normalizeModelId(alias) === normalizeModelId(canonical))) {
    list.unshift(canonical);
  }
  aliases = { canonical, aliases: list };
  return aliases;
}

function normalizeModelId(model) {
  return String(model).trim().toLowerCase();
}

/** `canonical_model_id`. @returns {string} */
export function canonicalModelId() {
  return modelAliases().canonical;
}

/**
 * `try_resolve_model_id`: the canonical id for an accepted alias (or an absent
 * model), null for a model this server does not serve.
 * @param {string|null|undefined} model
 * @returns {string|null}
 */
export function tryResolveModelId(model) {
  const trimmed = typeof model === 'string' ? model.trim() : '';
  if (!trimmed) return canonicalModelId();
  const normalized = normalizeModelId(trimmed);
  return modelAliases().aliases.some((alias) => normalizeModelId(alias) === normalized)
    ? canonicalModelId()
    : null;
}

/** `resolve_model_id`. @param {string|null|undefined} model @returns {string} */
export function resolveModelId(model) {
  return tryResolveModelId(model) || canonicalModelId();
}

let agentInfoCache = null;

/**
 * `seed::agent_info()`: the `field name / value` pairs of agent-info.lino.
 * @returns {Map<string, string>}
 */
export function agentInfo() {
  if (agentInfoCache) return agentInfoCache;
  agentInfoCache = new Map();
  const root = parseLino(readRepoFile('data/seed/agent-info.lino'));
  for (const field of childrenNamed(root, 'field')) {
    agentInfoCache.set(field.value, childValue(field, 'value'));
  }
  return agentInfoCache;
}

/** The registry files flagged `bundle true`, in registry order. @returns {Array<string>} */
export function bundledSeedFiles() {
  const root = parseLino(readRepoFile('data/meta/seed-registry.lino'));
  return childrenNamed(root, 'seed')
    .filter((seed) => childValue(seed, 'bundle') === 'true')
    .map((seed) => `data/seed/${seed.value}.lino`);
}

function escapeValue(raw) {
  return raw.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

/** `bundle_from_files`. @param {Array<[string, string]>} files @returns {string} */
export function bundleFromFiles(files) {
  let out = 'formal_ai_seed_bundle\n';
  for (const [name, contents] of files) {
    out += `  file "${escapeValue(name)}"\n`;
    for (const line of contents.split(/\r?\n/)) {
      if (line.length === 0) continue;
      out += `    ${line}\n`;
    }
  }
  return out;
}

let bundleCache = null;

/** `merged_bundle`. @returns {string} */
export function mergedBundle() {
  if (bundleCache === null) {
    bundleCache = bundleFromFiles(bundledSeedFiles().map((file) => [file, readRepoFile(file)]));
  }
  return bundleCache;
}

/** The crate version, from rust/Cargo.toml (`env!("CARGO_PKG_VERSION")`). @returns {string} */
export function packageVersion() {
  const match = /^version\s*=\s*"([^"]+)"/m.exec(readRepoFile('rust/Cargo.toml'));
  return match ? match[1] : '';
}
