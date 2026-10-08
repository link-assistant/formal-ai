// Version pins for generated workflows (rust/src/version_resolution.rs).
//
// A `ResolvedVersion` is `{tag, sha, origin: 'live'|'cache'|'baseline',
// source_url, fetched_at, response_sha256}`.
//
// `resolveVersionSet` mirrors `VersionSet::resolve` over an injected `fetch`
// (the JavaScript root has no `CachedSourceClient` twin): `fetch(url)` returns
// a capture `{source_url, text, fetched_at, sha256, cached}` — the fields of
// Rust's `SourceCapture` the resolution reads — or null where Rust's
// `client.fetch(url)` errs. `forGeneration(fetch)` walks the same ladder over
// a host-supplied `fetch`; by default the installed host's `sourceFetch`
// (crate/source_cache.mjs over the Node io), and from the shipped baseline
// when the host has none.

import { cached, hasHost, host, readText } from '../host.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';
import { reportText } from './seed_reports.mjs';
import { lines, trimStart } from '../write_str.mjs';

const TOOLCHAINS = 'data/seed/toolchains.lino';

function nodes() {
  return cached('write:toolchains_nodes', () => {
    const out = [];
    const descend = (node) => {
      out.push(node);
      for (const child of node.children || []) descend(child);
    };
    descend(parseLinoRoot(readText(TOOLCHAINS)));
    return out;
  });
}

/** Mirrors `ResolvedVersion::pinned_ref`: the SHA when known, else the tag. */
export function pinnedRef(pin) {
  return pin.sha === '' ? pin.tag : pin.sha;
}

/** Mirrors `fn baseline`: the shipped baseline for `id`, or null. */
function baseline(id) {
  const node = nodes().find((candidate) => candidate.name === 'generated_version' && candidate.id === id);
  if (!node) return null;
  return {
    tag: findChildValue(node, 'latest_tag'),
    sha: findChildValue(node, 'latest_sha'),
    origin: 'baseline',
    source_url: findChildValue(node, 'releases_url'),
    fetched_at: findChildValue(node, 'measured_at'),
    response_sha256: '',
  };
}

const PIN_IDS = [
  ['checkout', 'actions_checkout'],
  ['setup_java', 'actions_setup_java'],
  ['setup_kotlin', 'setup_kotlin_action'],
  ['setup_python', 'actions_setup_python'],
  ['python_interpreter', 'python_interpreter'],
  ['kotlin', 'kotlin_compiler'],
  ['java_lts', 'java_lts'],
  ['setup_coursier', 'setup_coursier_action'],
  ['scala', 'scala_compiler'],
];

/** Mirrors `VersionSet::baseline`. */
export function baselineVersionSet() {
  const set = { captures: [] };
  for (const [field, id] of PIN_IDS) {
    const pin = baseline(id);
    if (!pin) return null;
    set[field] = pin;
  }
  return set;
}

/** Mirrors `fn json_string`: a top-level JSON string field, or null. */
function jsonString(capture, key) {
  try {
    const value = JSON.parse(capture.text)[key];
    return typeof value === 'string' ? value : null;
  } catch {
    return null;
  }
}

/** Mirrors `str::trim_start_matches('v')` on a resolved pin's tag. */
function bareTag(resolved) {
  return resolved ? { ...resolved, tag: trimStartV(resolved.tag) } : null;
}

/** Mirrors `VersionSet::resolve_github`. */
function resolveGithub(fetch, publisher, repository, captures) {
  const releasesUrl = `https://api.github.com/repos/${publisher}/${repository}/releases/latest`;
  const release = fetch(releasesUrl);
  if (!release) return null;
  const origin = release.cached ? 'cache' : 'live';
  const tag = jsonString(release, 'tag_name');
  if (tag === null) return null;
  const commit = fetch(`https://api.github.com/repos/${publisher}/${repository}/commits/${tag}`);
  if (!commit) return null;
  const sha = jsonString(commit, 'sha');
  if (sha === null) return null;
  captures.push(commit);
  return { tag, sha, origin, source_url: releasesUrl, fetched_at: commit.fetched_at, response_sha256: commit.sha256 };
}

/** Mirrors `fn resolve_java_lts`: Adoptium's `most_recent_lts`, no SHA. */
function resolveJavaLts(fetch, captures) {
  const url = 'https://api.adoptium.net/v3/info/available_releases';
  const capture = fetch(url);
  if (!capture) return null;
  let lts = null;
  try {
    lts = JSON.parse(capture.text).most_recent_lts;
  } catch {
    return null;
  }
  if (!Number.isInteger(lts) || lts < 0) return null;
  captures.push(capture);
  return {
    tag: String(lts),
    sha: '',
    origin: capture.cached ? 'cache' : 'live',
    source_url: url,
    fetched_at: capture.fetched_at,
    response_sha256: capture.sha256,
  };
}

/**
 * Mirrors `VersionSet::resolve`: every pin through `fetch`, each falling back
 * to the shipped baseline when the publisher cannot be reached (R5), in the
 * Rust order so the capture list matches.
 */
export function resolveVersionSet(fetch) {
  const captures = [];
  const set = { captures };
  set.checkout = resolveGithub(fetch, 'actions', 'checkout', captures) ?? baseline('actions_checkout');
  set.setup_java = resolveGithub(fetch, 'actions', 'setup-java', captures) ?? baseline('actions_setup_java');
  set.setup_kotlin = resolveGithub(fetch, 'fwilhe2', 'setup-kotlin', captures) ?? baseline('setup_kotlin_action');
  set.setup_python = resolveGithub(fetch, 'actions', 'setup-python', captures) ?? baseline('actions_setup_python');
  set.python_interpreter = bareTag(resolveGithub(fetch, 'python', 'cpython', captures)) ?? baseline('python_interpreter');
  set.kotlin = resolveGithub(fetch, 'JetBrains', 'kotlin', captures) ?? baseline('kotlin_compiler');
  set.java_lts = resolveJavaLts(fetch, captures) ?? baseline('java_lts');
  set.setup_coursier = resolveGithub(fetch, 'coursier', 'setup-action', captures) ?? baseline('setup_coursier_action');
  set.scala = bareTag(resolveGithub(fetch, 'scala', 'scala', captures)) ?? baseline('scala_compiler');
  return set;
}

/**
 * Mirrors `VersionSet::record`: the derivation events, as `{kind, payload}`
 * rows — each capture's `source:http` (and `cache_hit`) line, then one
 * `version_resolution` line per pin.
 */
export function recordVersionSet(versions) {
  const events = [];
  for (const capture of versions.captures) {
    events.push({
      kind: 'source:http',
      payload: `${capture.source_url} fetched_at=${capture.fetched_at} sha256=${capture.sha256} cached=${capture.cached ? 'true' : 'false'}`,
    });
    if (capture.cached) events.push({ kind: 'cache_hit', payload: capture.source_url });
  }
  for (const [field, id] of PIN_IDS) {
    const entry = versions[field];
    events.push({
      kind: 'version_resolution',
      payload: `pin=${id} tag=${entry.tag} sha=${entry.sha} origin=${entry.origin}`,
    });
  }
  return events;
}

/**
 * Mirrors `VersionSet::for_generation`: every pin through the same
 * live → cache → baseline ladder as `resolveVersionSet`, over an injected
 * `fetch` that stands in for Rust's `CachedSourceClient` (a live capture
 * carries `cached: false`, a replayed one `cached: true`, a miss is null).
 * A host that owns no source cache passes no `fetch`, and every pin then
 * comes from the shipped baseline, as Rust's ladder ends when both the
 * network and the cache are unavailable.
 * @param {((url: string) => object|null)} [fetch]
 */
export function forGeneration(fetch = hostSourceFetch()) {
  return typeof fetch === 'function' ? resolveVersionSet(fetch) : baselineVersionSet();
}

/**
 * The installed host's source fetch (`host().sourceFetch()`), the twin of
 * the `CachedSourceClient` `for_generation` builds: `FORMAL_AI_SOURCE_CACHE_DIR`
 * (default `data`), online only under `FORMAL_AI_LIVE_FETCH`. A host without
 * one (the browser) answers from the shipped baseline.
 */
function hostSourceFetch() {
  if (!hasHost()) return null;
  const make = host().sourceFetch;
  return typeof make === 'function' ? make() : null;
}

function pin(versions, id) {
  const entry = PIN_IDS.find(([, name]) => name === id);
  return entry ? versions[entry[0]] : null;
}

/** Mirrors `VersionSet::provenance_note`. */
export function provenanceNote(versions) {
  return [
    ['actions/checkout', versions.checkout],
    ['actions/setup-java', versions.setup_java],
    ['fwilhe2/setup-kotlin', versions.setup_kotlin],
    ['actions/setup-python', versions.setup_python],
    ['python', versions.python_interpreter],
    ['kotlin', versions.kotlin],
    ['java', versions.java_lts],
    ['coursier/setup-action', versions.setup_coursier],
    ['scala', versions.scala],
  ]
    .filter(([, entry]) => entry.origin !== 'live')
    .map(([name, entry]) => reportText(
      entry.origin === 'cache' ? 'version_provenance_cache' : 'version_provenance_baseline',
      [['name', name], ['tag', entry.tag], ['fetched_at', entry.fetched_at]],
    ));
}

function workflowActions(versions) {
  return nodes()
    .filter((node) => node.name === 'generated_version')
    .flatMap((node) => {
      const uses = findChildValue(node, 'workflow_uses');
      if (uses === '') return [];
      const own = pin(versions, node.id);
      if (!own) return [];
      const key = findChildValue(node, 'workflow_version_key');
      const scopedPin = pin(versions, findChildValue(node, 'workflow_version_pin'));
      const scoped = scopedPin && key !== '' ? [key, scopedPin] : null;
      return [{ uses, marker: `${uses}@`, pin: own, scoped }];
    });
}

function workflowStepMarker() {
  const node = nodes().find((candidate) => candidate.name === 'generated_workflow_step');
  const marker = node ? findChildValue(node, 'marker') : '';
  return marker === '' ? null : marker;
}

function trimStartV(tag) {
  let out = tag;
  while (out.startsWith('v')) out = out.slice(1);
  return out;
}

/** Mirrors `fn fill_workflow_versions`. */
export function fillWorkflowVersions(template, versions) {
  const replacements = [
    ['{checkout_ref}', pinnedRef(versions.checkout)],
    ['{checkout_tag}', versions.checkout.tag],
    ['{setup_java_ref}', pinnedRef(versions.setup_java)],
    ['{setup_java_tag}', versions.setup_java.tag],
    ['{setup_kotlin_ref}', pinnedRef(versions.setup_kotlin)],
    ['{setup_kotlin_tag}', versions.setup_kotlin.tag],
    ['{setup_python_ref}', pinnedRef(versions.setup_python)],
    ['{setup_python_tag}', versions.setup_python.tag],
    ['{python_version}', versions.python_interpreter.tag],
    ['{kotlin_version}', versions.kotlin.tag],
    ['{java_lts}', versions.java_lts.tag],
    ['{setup_coursier_ref}', pinnedRef(versions.setup_coursier)],
    ['{setup_coursier_tag}', versions.setup_coursier.tag],
    ['{scala_version}', versions.scala.tag],
  ];
  const text = replacements.reduce((current, [from, to]) => current.split(from).join(to), template);
  const actions = workflowActions(versions);
  const stepMarker = workflowStepMarker();
  let block = null;
  let out = '';
  for (const raw of lines(text)) {
    let line = raw;
    let entered = false;
    for (const action of actions) {
      if (line.includes(action.marker)) {
        line = replaceActionRef(line, action.uses, action.pin);
        if (action.scoped) {
          block = action.scoped;
          entered = true;
        }
      }
    }
    if (!entered && stepMarker !== null && trimStart(line).startsWith(stepMarker)) block = null;
    if (block && line.includes(block[0])) line = replaceQuotedValue(line, trimStartV(block[1].tag));
    out += `${line}\n`;
  }
  return out.replace(/\n+$/u, '');
}

function replaceActionRef(line, action, entry) {
  const marker = `${action}@`;
  const start = line.indexOf(marker);
  if (start < 0) return line;
  const refStart = start + marker.length;
  const offset = line.slice(refStart).search(/[^A-Za-z0-9._-]/u);
  const refEnd = offset < 0 ? line.length : refStart + offset;
  const rest = trimStart(line.slice(refEnd)).startsWith('#') ? '' : line.slice(refEnd);
  return `${line.slice(0, refStart)}${pinnedRef(entry)}  # ${entry.tag}${rest}`;
}

function replaceQuotedValue(line, value) {
  const colon = line.indexOf(':');
  if (colon < 0) return line;
  const head = line.slice(0, colon + 1);
  const first = trimStart(line.slice(colon + 1))[0];
  const quote = first === "'" || first === '"' ? first : '';
  return `${head} ${quote}${value}${quote}`;
}
