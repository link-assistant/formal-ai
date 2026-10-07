// Issue #1165 R1165-10 parity: the JavaScript root keeps the same
// rediscoverable procedure cache as rust/src/discovery_production.rs, case for
// case with rust/tests/unit/issue_1165_discovery_production.rs — the same
// kotlin hello-world row, the same refusal, the same content address recomputed
// on store and checked on load, the same hit/miss routing. Cache files live
// under the OS temp directory through an injected `io`, as the Rust tests
// isolate theirs, so the committed cache is never touched.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { pushLinoNode } from '../../../js/agentic/crate/links_format.mjs';
import { findChildValue, parseRoot } from '../../../js/agentic/crate/seed_parser.mjs';
import {
  CachedOrDiscovered,
  DEFAULT_CACHE_FILE,
  ENTRY_REFUSAL,
  answerRecipe,
  bootstrapCacheActive,
  cacheFileHeader,
  cachedOrResearch,
  contentAddress,
  deleteAll,
  fnv1a64,
  grammarExists,
  isValidCacheEntry,
  knowsLanguage,
  loadAt,
  oracleKnowsLanguage,
  lookup,
  recipes,
  rediscoverableRecipe,
  requiredEntryFields,
  store,
  wasCached,
} from '../../../js/agentic/crate/discovery_production.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

/** Node filesystem IO for an absolute cache path (the Rust `std::fs` calls). */
const nodeIo = {
  readText: (file) => readFileSync(file, 'utf8'),
  writeText: (file, text) => {
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
  },
};

/** Mirrors `fn isolated_cache`: a fresh per-test path under the temp dir. */
function isolatedCache(name) {
  const directory = path.join(tmpdir(), 'formal-ai-issue-1165-js', name);
  rmSync(directory, { recursive: true, force: true });
  return directory;
}

/** Mirrors `fn kotlin_hello_recipe`. */
const kotlinHelloRecipe = () => rediscoverableRecipe({
  language: 'kotlin',
  task: 'hello_world',
  rediscovery_query: 'kotlin hello_world verified coding procedure SPDX license',
  rediscovery_source: 'http://helloworldcollection.de/#Kotlin',
  entry: 'fun main() {\n    println("Hello, World!")\n}',
  verified_output: 'Hello, World!',
  content_id: 0n,
});

test('fnv1a64 is the 64-bit FNV-1a, bit for bit (published reference vectors)', () => {
  assert.equal(fnv1a64(new Uint8Array()), 0xcbf29ce484222325n);
  assert.equal(fnv1a64('a'), 0xaf63dc4c8601ec8cn);
  assert.equal(fnv1a64('foobar'), 0x85944171f73967e8n);
  assert.equal(fnv1a64(new TextEncoder().encode('foobar')), fnv1a64('foobar'));
});

test('the required fields come from the policy seed, plus the cache key', () => {
  const policy = parseRoot(readText('data/seed/program-cache-policy.lino')).children[0];
  const requires = policy.children.find((node) => node.name === 'entry_requires');
  const declared = requires.children.filter((node) => node.value === 'required').map((node) => node.name);
  assert.deepEqual([...requiredEntryFields()], ['language', 'task', ...declared]);
  assert.equal(findChildValue(policy, 'cache_file'), DEFAULT_CACHE_FILE);
  assert.ok(cacheFileHeader().startsWith('# The rediscoverable coding-procedure cache'));
  assert.ok(cacheFileHeader().includes('\n#\n'), 'an empty header line is a bare #');
});

test('store rejects a procedure without rediscovery fields (R1165-3)', () => {
  const cache = loadAt(isolatedCache('reject'), nodeIo);
  const recipe = { ...kotlinHelloRecipe(), rediscovery_query: '', rediscovery_source: '' };
  assert.equal(isValidCacheEntry(recipe), false);
  const result = store(cache, recipe);
  assert.equal(result.ok, false, 'policy must refuse the row');
  assert.equal(result.error, ENTRY_REFUSAL);
  assert.ok(result.error.includes('rediscovery'), 'the refusal names the missing policy fields');
  assert.deepEqual(recipes(cache), []);
});

test('store, reload from disk, look up case-insensitively', () => {
  const file = isolatedCache('roundtrip');
  const cache = loadAt(file, nodeIo);
  assert.deepEqual(store(cache, kotlinHelloRecipe()), { ok: true }, 'policy-complete row stores');
  const reloaded = loadAt(file, nodeIo);
  const recipe = lookup(reloaded, 'Kotlin', 'hello_world');
  assert.ok(recipe, 'lookup is case-insensitive on the language');
  assert.equal(recipe.entry, kotlinHelloRecipe().entry);
  assert.equal(recipe.verified_output, 'Hello, World!');
  assert.equal(recipe.content_id, contentAddress(recipe.entry), 'the stored address matches the program');
  assert.ok(readFileSync(file, 'utf8').startsWith(cacheFileHeader()), 'the file opens with the seed header');
  assert.equal(readFileSync(file, 'utf8'), cache.file_text, 'file_text is the bytes written');
});

test('content_id is fnv1a of the entry, recomputed on store and checked on load', () => {
  const recipe = kotlinHelloRecipe();
  assert.equal(contentAddress(recipe.entry), fnv1a64(new TextEncoder().encode(recipe.entry)));
  const cache = loadAt(isolatedCache('content-id'), nodeIo);
  assert.equal(store(cache, { ...kotlinHelloRecipe(), content_id: 0xffffffffffffffffn }).ok, true);
  assert.equal(
    lookup(cache, 'kotlin', 'hello_world').content_id,
    fnv1a64(recipe.entry),
    'store recomputes the address from the entry',
  );

  const file = isolatedCache('content-id-drift');
  const drifted = loadAt(file, nodeIo);
  assert.equal(store(drifted, kotlinHelloRecipe()).ok, true);
  const text = readFileSync(file, 'utf8');
  writeFileSync(file, text.replace('content_id "0x', 'content_id "0xf'));
  assert.equal(lookup(loadAt(file, nodeIo), 'kotlin', 'hello_world'), null, 'a drifted address must not answer');
});

test('delete_all empties the cache file (R1165-7, offline half)', () => {
  const file = isolatedCache('delete-all');
  const cache = loadAt(file, nodeIo);
  assert.equal(store(cache, kotlinHelloRecipe()).ok, true);
  deleteAll(cache);
  assert.deepEqual(recipes(cache), []);
  assert.equal(lookup(loadAt(file, nodeIo), 'kotlin', 'hello_world'), null, 'no row survives deletion');
  assert.ok(readFileSync(file, 'utf8').includes('coding_procedure_cache'), 'the empty cache keeps its document head');
});

test('the miss path runs research and stores the verified procedure; the second ask is a hit', () => {
  const candidate = 'def main\n  __COUNT_TO_THREE__\nend\n';
  const verified = 'def main\n  1.upto(3) { |number| puts number }\nend\n';
  const source = 'https://research.invalid/ruby-count';
  // A gap with the Rust `CodingResearchGap::links_notation` shape.
  const gap = (task, language) => ({
    nextQuery: () => `${language} ${task} verified coding procedure`,
    linksNotation() {
      let out = pushLinoNode('', 0, 'coding_research_gap', `${task}_${language}`);
      out = pushLinoNode(out, 2, 'task', task);
      out = pushLinoNode(out, 2, 'language', language);
      out = pushLinoNode(out, 2, 'status', 'open');
      return pushLinoNode(out, 2, 'next_query', this.nextQuery());
    },
  });
  let rounds = 0;
  // Stands in for `research_coding_skill_gap`: the verified rewrite of the candidate.
  const research = () => {
    rounds += 1;
    return { ok: true, execution: { output: candidate.replace('__COUNT_TO_THREE__', '1.upto(3) { |number| puts number }'), cycle: '' } };
  };

  const cache = loadAt(isolatedCache('miss-path'), nodeIo);
  const first = cachedOrResearch(cache, gap('count_to_three', 'ruby'), research, verified, source);
  assert.equal(first.ok, true, 'research verifies the candidate');
  assert.equal(wasCached(first.answer), false, 'an empty cache must route to research');
  assert.equal(first.answer.kind, CachedOrDiscovered.Discovered);
  const recipe = answerRecipe(first.answer);
  assert.equal(recipe.entry, verified, 'the row reuses the verified produced program');
  assert.equal(recipe.verified_output, verified);
  assert.equal(recipe.rediscovery_source, source);
  assert.equal(recipe.rediscovery_query, 'ruby count_to_three verified coding procedure', 'captured before research');
  assert.equal(recipe.content_id, fnv1a64(verified));
  assert.ok(lookup(cache, 'ruby', 'count_to_three'), 'the verified procedure is stored');
  assert.equal(rounds, 1);

  const second = cachedOrResearch(cache, gap('count_to_three', 'ruby'), research, verified, source);
  assert.equal(second.ok, true, 'lookup answers');
  assert.equal(second.answer.kind, CachedOrDiscovered.Cached);
  assert.equal(rounds, 1, 'a cache hit must not re-run the research round');
});

test('a failed research round is the error, and nothing is stored', () => {
  const cache = loadAt(isolatedCache('miss-failure'), nodeIo);
  const gap = {
    nextQuery: () => 'q',
    linksNotation: () => 'coding_research_gap "g"\n  task "count_to_three"\n  language "ruby"\n',
  };
  const error = { reason: 'coding_research_verification_failed', cycle: 'c' };
  const result = cachedOrResearch(cache, gap, () => ({ ok: false, error }), 'x', 'https://research.invalid/x');
  assert.deepEqual(result, { ok: false, error });
  assert.deepEqual(recipes(cache), []);
});

test('grammar_exists reads the CST seed', () => {
  for (const language of ['kotlin', 'php', 'swift', 'scala', 'rust', 'r']) {
    assert.ok(grammarExists(language), `${language} carries a CST grammar`);
  }
  assert.equal(grammarExists('latin-vulgate'), false);
  assert.equal(grammarExists('  '), false);
});

test('knows_language requires a grammar and a procedure', () => {
  assert.ok(knowsLanguage('kotlin'), 'grammar plus a bootstrap-recorded discovery answers');
  assert.equal(knowsLanguage('rust'), false, 'a grammar with no recorded procedure is a rediscovery away');
  assert.equal(knowsLanguage('latin-vulgate'), false, 'no grammar means no amount of caching answers');
});

test('the bootstrap tier is governed by the policy seed', () => {
  assert.ok(bootstrapCacheActive(), 'the committed policy keeps the bootstrap active');
  assert.ok(oracleKnowsLanguage('kotlin'), 'the snapshots answer while the bootstrap is active');
});
