// Issue #1168 R1168-1..5 on the JavaScript generation path: the version pins of
// a generated workflow come from the publishers' APIs through the native
// source-cache layout (js/agentic/crate/source_cache.mjs, the twin of
// `CachedSourceClient::fetch`), are recorded with their provenance, and fall
// back to the cached capture and then the shipped baseline, each said out
// loud. `forGeneration()` reads the installed host's `sourceFetch`, which
// js/agentic/node-host.mjs builds from `FORMAL_AI_SOURCE_CACHE_DIR` and the
// `FORMAL_AI_LIVE_FETCH` opt-in exactly as `VersionSet::for_generation` does.
// No test here reaches the network: the transport is a fixture.

import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { cacheKey, cachedSourceFetch } from '../../../js/agentic/crate/source_cache.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import {
  baselineVersionSet, forGeneration, provenanceNote, recordVersionSet, resolveVersionSet,
} from '../../../js/agentic/crate/version_resolution.mjs';

const RELEASES = [
  ['actions/checkout', 'v7.0.1', '3d3c42e5aac5ba805825da76410c181273ba90b1'],
  ['actions/setup-java', 'v6.0.1', 'de7274f081f381c8f8158605e0321c36c376e2e6'],
  ['fwilhe2/setup-kotlin', 'v2.0', 'ee9692514da313706b193d808526812102a344e4'],
  ['actions/setup-python', 'v7.0.0', '5fda3b95a4ea91299a34e894583c3862153e4b97'],
  ['python/cpython', 'v3.14.7', '0000000000000000000000000000000000000000'],
  ['JetBrains/kotlin', 'v2.4.20', '0000000000000000000000000000000000000000'],
  ['coursier/setup-action', 'v3.0.4', '648df969f41ef15fda2baba8b37f9fa3d16390a3'],
  ['scala/scala', 'v2.13.18', '0000000000000000000000000000000000000000'],
];

// The publishers' own answer shapes: api.github.com pretty-prints.
const BODIES = new Map([['https://api.adoptium.net/v3/info/available_releases', '{\n  "most_recent_lts": 25\n}']]);
for (const [repository, tag, sha] of RELEASES) {
  BODIES.set(`https://api.github.com/repos/${repository}/releases/latest`, `{\n  "tag_name": "${tag}"\n}`);
  BODIES.set(`https://api.github.com/repos/${repository}/commits/${tag}`, `{\n  "sha": "${sha}"\n}`);
}

const scratch = mkdtempSync(path.join(tmpdir(), 'issue-1168-source-cache-'));
const requested = [];
const fsIo = (transport) => ({
  readText: (file) => { try { return readFileSync(file, 'utf8'); } catch { return null; } },
  readBytes: (file) => { try { return new Uint8Array(readFileSync(file)); } catch { return null; } },
  writeBytes: (file, bytes) => writeFileSync(file, bytes),
  createDirAll: (directory) => mkdirSync(directory, { recursive: true }),
  get: transport,
});
const fixtureTransport = (url) => {
  requested.push(url);
  return BODIES.has(url) ? new TextEncoder().encode(BODIES.get(url)) : null;
};
const unreachable = () => null;

before(async () => {
  await installNodeHost(new WorkerHost());
});
after(() => rmSync(scratch, { recursive: true, force: true }));

test('the cache key is the native FNV-1a 64 of the URL', () => {
  assert.equal(cacheKey(''), 'cbf29ce484222325');
  assert.equal(cacheKey('a'), 'af63dc4c8601ec8c');
});

test('R1168-1/2/3: online, every pin is read from its publisher and captured in the native layout', () => {
  const cacheDir = path.join(scratch, 'live');
  const fetch = cachedSourceFetch({ cacheDir, online: true, io: fsIo(fixtureTransport), now: () => 1800000000 });
  const versions = resolveVersionSet(fetch);
  assert.equal(versions.checkout.tag, 'v7.0.1');
  assert.equal(versions.checkout.sha, '3d3c42e5aac5ba805825da76410c181273ba90b1');
  assert.equal(versions.checkout.origin, 'live');
  assert.equal(versions.kotlin.tag, 'v2.4.20');
  assert.equal(versions.java_lts.tag, '25');
  assert.equal(versions.java_lts.origin, 'live');
  assert.equal(versions.scala.tag, '2.13.18');
  assert.deepEqual(provenanceNote(versions), []);
  assert.ok(requested.includes('https://api.github.com/repos/actions/checkout/releases/latest'));
  assert.ok(requested.includes('https://api.github.com/repos/actions/checkout/commits/v7.0.1'));
  const url = 'https://api.github.com/repos/actions/checkout/commits/v7.0.1';
  const meta = readFileSync(path.join(cacheDir, 'source-cache', `${cacheKey(url)}.meta`), 'utf8');
  const body = BODIES.get(url);
  assert.equal(meta, `source-capture-v2\nurl=${url}\nfetched_at=1800000000\nsha256=${sha256Hex(body)}\n`);
  assert.equal(readFileSync(path.join(cacheDir, 'source-cache', 'objects', `${sha256Hex(body)}.body`), 'utf8'), body);
});

test('R1168-4: the resolution is recorded with each capture\'s provenance', () => {
  const fetch = cachedSourceFetch({ cacheDir: path.join(scratch, 'record'), online: true, io: fsIo(fixtureTransport), now: () => 1800000000 });
  const lines = recordVersionSet(resolveVersionSet(fetch)).map((event) => `${event.kind} ${event.payload}`).join('\n');
  assert.ok(lines.includes('source:http https://api.github.com/repos/actions/checkout/commits/v7.0.1 fetched_at=1800000000'));
  assert.ok(lines.includes('version_resolution pin=java_lts tag=25 sha= origin=live'));
});

test('R1168-5: unreachable, the cached capture answers and says so; with no cache, the baseline does', () => {
  const cacheDir = path.join(scratch, 'live');
  const offline = cachedSourceFetch({ cacheDir, online: true, io: fsIo(unreachable), now: () => 1800000000 + 61 * 24 * 3600 });
  const replayed = resolveVersionSet(cachedSourceFetch({ cacheDir, online: false, io: fsIo(unreachable) }));
  assert.equal(replayed.checkout.origin, 'cache');
  assert.equal(replayed.checkout.tag, 'v7.0.1');
  assert.ok(provenanceNote(replayed).join('\n').includes('(version resolved from cache; fetched_at=1800000000)'));
  // A stale capture with the network down is not replayed online (the native
  // TTL rule); the pin then comes from the shipped baseline, said out loud.
  const stale = resolveVersionSet(offline);
  assert.equal(stale.checkout.origin, 'baseline');
  const empty = resolveVersionSet(cachedSourceFetch({ cacheDir: path.join(scratch, 'empty'), online: false, io: fsIo(unreachable) }));
  assert.deepEqual(empty, { ...baselineVersionSet(), captures: [] });
  assert.ok(provenanceNote(empty).join('\n').includes('(version resolved from the shipped baseline measured'));
});

test('forGeneration reads the host source cache named by FORMAL_AI_SOURCE_CACHE_DIR', () => {
  const previous = process.env.FORMAL_AI_SOURCE_CACHE_DIR;
  const live = process.env.FORMAL_AI_LIVE_FETCH;
  try {
    delete process.env.FORMAL_AI_LIVE_FETCH;
    process.env.FORMAL_AI_SOURCE_CACHE_DIR = path.join(scratch, 'live');
    const versions = forGeneration();
    assert.equal(versions.checkout.origin, 'cache');
    assert.equal(versions.checkout.sha, '3d3c42e5aac5ba805825da76410c181273ba90b1');
    process.env.FORMAL_AI_SOURCE_CACHE_DIR = path.join(scratch, 'nothing-here');
    assert.equal(forGeneration().checkout.origin, 'baseline');
  } finally {
    if (previous === undefined) delete process.env.FORMAL_AI_SOURCE_CACHE_DIR;
    else process.env.FORMAL_AI_SOURCE_CACHE_DIR = previous;
    if (live !== undefined) process.env.FORMAL_AI_LIVE_FETCH = live;
  }
});
