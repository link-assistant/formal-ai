// Issue #1168 R1168-8 parity: js/agentic/crate/version_resolution.mjs resolves
// the generated-workflow pins exactly as rust/src/version_resolution.rs does,
// case for case with rust/tests/unit/issue_1168_latest_versions.rs — the same
// publisher fixtures, the same live → cache → baseline ladder, the same
// derivation lines, the same placeholder fill. The Rust tests drive a mocked
// `SourceTransport` behind a `CachedSourceClient`; the JavaScript root takes
// the fetch as an injected function, so the fixture below plays both: a live
// transport that records each capture, and an offline replay of those
// captures marked `cached`, as `CachedSourceClient` replays them.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import {
  baselineVersionSet,
  fillWorkflowVersions,
  provenanceNote,
  recordVersionSet,
  resolveVersionSet,
} from '../../../js/agentic/crate/version_resolution.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

/** Mirrors `MockTransport::seeded`: one fixture body per publisher URL. */
const FIXTURES = new Map([
  ['https://api.github.com/repos/actions/checkout/releases/latest', '{"tag_name":"v7.0.1"}'],
  ['https://api.github.com/repos/actions/checkout/commits/v7.0.1', '{"sha":"3d3c42e5aac5ba805825da76410c181273ba90b1"}'],
  ['https://api.github.com/repos/actions/setup-java/releases/latest', '{"tag_name":"v6.0.1"}'],
  ['https://api.github.com/repos/actions/setup-java/commits/v6.0.1', '{"sha":"de7274f081f381c8f8158605e0321c36c376e2e6"}'],
  ['https://api.github.com/repos/fwilhe2/setup-kotlin/releases/latest', '{"tag_name":"v2.0"}'],
  ['https://api.github.com/repos/fwilhe2/setup-kotlin/commits/v2.0', '{"sha":"ee9692514da313706b193d808526812102a344e4"}'],
  ['https://api.github.com/repos/actions/setup-python/releases/latest', '{"tag_name":"v7.0.0"}'],
  ['https://api.github.com/repos/actions/setup-python/commits/v7.0.0', '{"sha":"5fda3b95a4ea91299a34e894583c3862153e4b97"}'],
  ['https://api.github.com/repos/python/cpython/releases/latest', '{"tag_name":"v3.14.7"}'],
  ['https://api.github.com/repos/python/cpython/commits/v3.14.7', '{"sha":"0000000000000000000000000000000000000000"}'],
  ['https://api.github.com/repos/JetBrains/kotlin/releases/latest', '{"tag_name":"v2.4.20"}'],
  ['https://api.github.com/repos/JetBrains/kotlin/commits/v2.4.20', '{"sha":"0000000000000000000000000000000000000000"}'],
  ['https://api.adoptium.net/v3/info/available_releases', '{"most_recent_lts":25}'],
  ['https://api.github.com/repos/coursier/setup-action/releases/latest', '{"tag_name":"v3.0.4"}'],
  ['https://api.github.com/repos/coursier/setup-action/commits/v3.0.4', '{"sha":"648df969f41ef15fda2baba8b37f9fa3d16390a3"}'],
  ['https://api.github.com/repos/scala/scala/releases/latest', '{"tag_name":"v2.13.18"}'],
  ['https://api.github.com/repos/scala/scala/commits/v2.13.18', '{"sha":"0000000000000000000000000000000000000000"}'],
]);

/** Mirrors `const fn fixed_clock`. */
const FIXED_CLOCK = '1800000000';

/** Mirrors `MockTransport::online`: live captures, each kept in `cache`. */
function onlineFetch(cache) {
  return (url) => {
    const text = FIXTURES.get(url);
    if (text === undefined) return null;
    const capture = { source_url: url, text, fetched_at: FIXED_CLOCK, sha256: sha256Hex(text), cached: false };
    cache.set(url, capture);
    return capture;
  };
}

/** Mirrors an offline `CachedSourceClient`: replay of what `cache` holds. */
function offlineFetch(cache) {
  return (url) => {
    const capture = cache.get(url);
    return capture ? { ...capture, cached: true } : null;
  };
}

test('resolves_every_pin_from_live_fixtures', () => {
  const versions = resolveVersionSet(onlineFetch(new Map()));
  assert.equal(versions.checkout.tag, 'v7.0.1');
  assert.equal(versions.checkout.sha, '3d3c42e5aac5ba805825da76410c181273ba90b1');
  assert.equal(versions.checkout.origin, 'live');
  assert.equal(versions.setup_java.tag, 'v6.0.1');
  assert.equal(versions.setup_kotlin.tag, 'v2.0');
  assert.equal(versions.setup_python.tag, 'v7.0.0');
  assert.equal(versions.python_interpreter.tag, '3.14.7');
  assert.equal(versions.kotlin.tag, 'v2.4.20');
  assert.equal(versions.java_lts.tag, '25');
  assert.equal(versions.java_lts.sha, '');
  assert.equal(versions.java_lts.origin, 'live');
  assert.equal(versions.setup_coursier.tag, 'v3.0.4');
  assert.equal(versions.setup_coursier.sha, '648df969f41ef15fda2baba8b37f9fa3d16390a3');
  assert.equal(versions.scala.tag, '2.13.18');
  assert.deepEqual(provenanceNote(versions), []);
});

test('records_the_resolution_in_the_derivation', () => {
  const versions = resolveVersionSet(onlineFetch(new Map()));
  const joined = recordVersionSet(versions).map((event) => `${event.kind} ${event.payload}`).join('\n');
  assert.equal(joined.includes('version_resolution pin=actions_checkout tag=v7.0.1'), true);
  assert.equal(joined.includes('java_lts tag=25 sha= origin=live'), true);
  assert.equal(joined.includes('pin=scala_compiler tag=2.13.18 sha=0000000000000000000000000000000000000000 origin=live'), true);
  assert.equal(
    joined.split('\n')[0],
    `source:http https://api.github.com/repos/actions/checkout/commits/v7.0.1 fetched_at=1800000000 sha256=${sha256Hex('{"sha":"3d3c42e5aac5ba805825da76410c181273ba90b1"}')} cached=false`,
  );
});

test('offline_replays_the_cached_capture_and_says_so', () => {
  const cache = new Map();
  assert.equal(resolveVersionSet(onlineFetch(cache)).checkout.origin, 'live');
  const versions = resolveVersionSet(offlineFetch(cache));
  assert.equal(versions.checkout.tag, 'v7.0.1');
  assert.equal(versions.checkout.origin, 'cache');
  assert.equal(versions.kotlin.tag, 'v2.4.20');
  assert.equal(
    provenanceNote(versions)[0],
    'actions/checkout v7.0.1: (version resolved from cache; fetched_at=1800000000)',
  );
  assert.equal(recordVersionSet(versions).filter((event) => event.kind === 'cache_hit').length, 9);
});

test('empty_cache_falls_back_to_the_shipped_baseline', () => {
  const versions = resolveVersionSet(offlineFetch(new Map()));
  assert.equal(versions.checkout.origin, 'baseline');
  assert.equal(versions.checkout.tag, 'v7.0.1');
  assert.equal(versions.setup_java.tag, 'v6.0.1');
  assert.equal(versions.setup_kotlin.tag, 'v2.0');
  assert.equal(versions.kotlin.tag, 'v2.4.20');
  assert.equal(versions.java_lts.tag, '25');
  assert.equal(versions.scala.tag, '2.13.18');
  assert.deepEqual(versions, { ...baselineVersionSet(), captures: [] });
});

test('fills_the_scala_ci_setup_placeholders', () => {
  const template = '      - uses: coursier/setup-action@{setup_coursier_ref} # {setup_coursier_tag}\n        with:\n          jvm: temurin:{java_lts}\n          apps: scala:{scala_version} scalac:{scala_version}\n';
  assert.equal(
    fillWorkflowVersions(template, baselineVersionSet()),
    '      - uses: coursier/setup-action@648df969f41ef15fda2baba8b37f9fa3d16390a3  # v3.0.4\n        with:\n          jvm: temurin:25\n          apps: scala:2.13.18 scalac:2.13.18',
  );
});
