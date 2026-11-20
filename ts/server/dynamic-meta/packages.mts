import { serverMessage } from '../messages.mjs';
import { createHash } from 'node:crypto';

import { linksToLino, typeMarkers, typeSystem } from '@link-foundation/ui-links';
import { readRecipe } from './recipe.mjs';
import { EVENT_KINDS, linkContext, parseLinkSubstitution } from './substitutions.mjs';

const IDENTIFIER_PATTERN = /^[a-z][a-z0-9_-]{0,63}$/u;
const QUALIFIED_IDENTIFIER_PATTERN = /^[a-z][a-z0-9_-]{0,63}\/[a-z][a-z0-9_-]{0,63}$/u;
const PACKAGE_KINDS = ['npm', 'crate', 'github', 'local'];
const MANIFEST_LIMIT = 128 * 1024;
const PACKAGE_VERSION = /^\d+\.\d+\.\d+(?:-[0-9a-z.-]+)?(?:\+[0-9a-z.-]+)?$/iu;
const GIT_REVISION = /^[0-9a-f]{7,64}$/iu;
const SHA256 = /^[0-9a-f]{64}$/u;

export const sha256 = (text) => createHash('sha256').update(text).digest('hex');

function sourceProvenance(provenance) {
  if (provenance === undefined) return {};
  const { url, sha256: digest, line } = provenance ?? {};
  const parsed = typeof url === 'string' && url.length <= 2048 ? URL.parse(url) : null;
  if (!parsed || parsed.protocol !== 'https:' || parsed.username || parsed.password || parsed.hash ||
      typeof digest !== 'string' || !SHA256.test(digest) ||
      (line !== undefined && (!Number.isSafeInteger(line) || line < 1))) {
    throw new Error(serverMessage('dynamic-meta-invalid-provenance'));
  }
  return { provenance: { url: parsed.href, sha256: digest, ...(line === undefined ? {} : { line }) } };
}

export function boundedManifest(source) {
  if (typeof source !== 'string' || Buffer.byteLength(source) > MANIFEST_LIMIT) {
    throw new Error(serverMessage('dynamic-meta-manifest-size'));
  }
  const data = JSON.parse(source);
  if (data?.schema !== 'formal-ai-plugin/v1' || typeof data.id !== 'string' || !IDENTIFIER_PATTERN.test(data.id) ||
      !PACKAGE_KINDS.includes(data.origin?.kind)) {
    throw new Error(serverMessage('dynamic-meta-invalid-manifest'));
  }
  const { kind, locator, revision } = data.origin;
  if (typeof locator !== 'string' || !locator.trim() || locator.length > 256 ||
      typeof revision !== 'string' || !revision.trim() || revision.length > 128) {
    throw new Error(serverMessage('dynamic-meta-origin-locator'));
  }
  if ((kind === 'github' && !GIT_REVISION.test(revision)) ||
      ((kind === 'npm' || kind === 'crate') && !PACKAGE_VERSION.test(revision))) {
    throw new Error(serverMessage('dynamic-meta-origin-revision'));
  }

  const seen = new Set();
  const exportsOf = (key, parse) => {
    const entries = data[key] ?? [];
    if (!Array.isArray(entries) || entries.length > 64) throw new Error(serverMessage('dynamic-meta-export-limit', { collection: key }));
    return entries.map((entry) => {
      if (typeof entry?.id !== 'string' || !IDENTIFIER_PATTERN.test(entry.id) || seen.has(entry.id)) {
        throw new Error(serverMessage('dynamic-meta-unique-export'));
      }
      seen.add(entry.id);
      return { id: entry.id, ...parse(entry) };
    });
  };

  const rules = exportsOf('rules', (entry) => {
    const { event, source, target } = entry.when ?? {};
    if (!EVENT_KINDS.includes(event) ||
        [source, target].some((value) => value !== undefined && (!Number.isSafeInteger(value) || value < 0))) {
      throw new Error(serverMessage('dynamic-meta-invalid-trigger'));
    }
    const when = { event, source, target };
    if (entry.substitute !== undefined && entry.setRule === undefined) {
      const substitution = parseLinkSubstitution(entry.substitute, { template: true });
      if (event === 'create' && /"@old-(?:source|target)"/u.test(JSON.stringify(substitution))) {
        throw new Error(serverMessage('dynamic-meta-create-old-endpoints'));
      }
      return { when, substitution };
    }
    const change = entry.setRule;
    if (entry.substitute !== undefined || typeof change?.target !== 'string' ||
        !IDENTIFIER_PATTERN.test(change.target) || typeof change.enabled !== 'boolean') {
      throw new Error(serverMessage('dynamic-meta-trigger-action'));
    }
    return { when, setRule: { target: `${data.id}/${change.target}`, enabled: change.enabled } };
  });
  const recipes = exportsOf('recipes', ({ language, source, provenance }) => ({
    program: readRecipe(language, source), ...sourceProvenance(provenance),
  }));
  const datasets = exportsOf('datasets', ({ substitute, provenance }) => ({
    substitution: parseLinkSubstitution(substitute), ...sourceProvenance(provenance),
  }));
  const integrations = exportsOf('integrations', ({ adapter }) => {
    if (typeof adapter !== 'string' || !IDENTIFIER_PATTERN.test(adapter)) throw new Error(serverMessage('dynamic-meta-invalid-adapter'));
    return { adapter };
  });

  for (const rule of rules) {
    if (rule.setRule && !rules.some(({ id }) => `${data.id}/${id}` === rule.setRule.target)) {
      throw new Error(serverMessage('dynamic-meta-missing-rule'));
    }
  }
  return {
    id: data.id,
    origin: { kind, locator, revision, digest: sha256(source) },
    rules,
    recipes,
    datasets,
    integrations,
  };
}

export function namedExport(packages, collection, reference) {
  if (typeof reference !== 'string' || !QUALIFIED_IDENTIFIER_PATTERN.test(reference)) throw new Error(serverMessage('dynamic-meta-export-reference'));
  const [packageId, exportId] = reference.split('/');
  const entry = packages.find(({ id }) => id === packageId)?.[collection].find(({ id }) => id === exportId);
  if (!entry) throw new Error(serverMessage('dynamic-meta-unknown-export', { collection, reference }));
  return entry;
}

export function packageLinks(packages) {
  const context = linkContext();
  const { links, names } = context;
  const markers = typeMarkers(links, names);
  const types = typeSystem(links, markers);
  const name = (address, label) => { names.setName(address, label); return address; };
  const define = (label, parent) => name(types.define(parent), label);
  const packageType = define('Package', markers.root);
  const kinds = new Map(PACKAGE_KINDS.map((kind) => [kind, define(`Package/${kind}`, packageType)]));
  const exportType = define('PackageExport', markers.root);
  const exports = new Map(['rules', 'recipes', 'datasets', 'integrations'].map((kind) => [kind, define(kind, exportType)]));
  const owns = define('Owns', markers.root);
  const payloadType = define('Payload', markers.root);
  for (const pkg of packages) {
    const pkgName = `package/${pkg.id}`;
    const origin = name(types.instance(payloadType, 0), `sha256/${pkg.origin.digest}/${pkg.id}`);
    const owner = name(types.instance(kinds.get(pkg.origin.kind), origin), pkgName);
    for (const [kind, type] of exports) {
      for (const entry of pkg[kind]) {
        const digest = sha256(JSON.stringify(entry));
        const payload = name(types.instance(payloadType, 0), `sha256/${digest}/${pkg.id}/${entry.id}`);
        const value = name(types.instance(type, payload), `${pkgName}/${entry.id}`);
        const pair = links.create();
        links.update(pair, owner, value);
        types.instance(owns, pair);
      }
    }
  }
  return linksToLino(links, { names });
}
