// Version pins for generated workflows (rust/src/version_resolution.rs).
//
// Only the shipped baseline is resolved here: live and cached GitHub /
// Adoptium captures are native-only. A `ResolvedVersion` is `{tag, sha,
// origin: 'live'|'cache'|'baseline', source_url, fetched_at,
// response_sha256}`.

import { cached, readText } from '../host.mjs';
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

/**
 * Mirrors `VersionSet::for_generation`.
 * native-only: rust/src/version_resolution.rs VersionSet::resolve; the live
 * and cached release captures need the network / source cache, so every pin
 * comes from the shipped baseline (as Rust does when both are unavailable).
 */
export function forGeneration() {
  return baselineVersionSet();
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
