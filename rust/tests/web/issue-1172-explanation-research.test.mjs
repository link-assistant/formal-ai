// Issue #1172 R8, browser twin: an explanation request ("Explain the Kotlin
// compiler", a `capability_act_explain` surface opening the prompt) that no
// seeded record or live Wikidata answer covers is researched through web
// search, the result page is captured with its SHA-256 and formalized, and
// the answer quotes each statement that mentions the concept with the page
// URL and digest -- the same fixture and the same expected answer as
// `explanation_answers_from_retrieved_statements_with_citations` in
// rust/tests/unit/issue_1172_live_fact_answer.rs. Every byte comes from a
// fixture fetch; no test reaches the network.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { before, test } from 'node:test';

import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

const DUCKDUCKGO_API = 'https://api.duckduckgo.com/';
const PAGE_URL = 'https://docs.example.org/issue-1172/kotlin-compiler.html';
const PAGE = '<!DOCTYPE html><html lang="en"><body><h1>Kotlin compiler</h1><p>The Kotlin compiler turns Kotlin source files into JVM bytecode.</p><p>Gradle can drive the build as well.</p><ul><li>The Kotlin compiler ships as the kotlinc command.</li></ul></body></html>';
const PAGE_SHA256 = '6bee6584fca407d95ea75b6fab57e68fe1d260cfe732623924ca389816f66ec6';

const fetched = [];
let fixtureOnline = true;

function respond(status, text) {
  return Promise.resolve({ ok: status === 200, status, statusText: '', text: () => Promise.resolve(text) });
}

let host;
before(async () => {
  host = new WorkerHost();
  const context = await host.boot();
  const seedFetch = context.fetch;
  context.fetch = (url, options) => {
    const target = String(url);
    if (!/^https?:\/\//u.test(target) || target.startsWith('http://localhost/')) return seedFetch(url, options);
    fetched.push(target);
    if (!fixtureOnline) return respond(404, '');
    if (target.startsWith(DUCKDUCKGO_API)) {
      return respond(200, JSON.stringify({ AbstractURL: PAGE_URL, Heading: 'Kotlin compiler', AbstractText: 'The Kotlin compiler.', RelatedTopics: [] }));
    }
    if (target === PAGE_URL) return respond(200, PAGE);
    return respond(404, '');
  };
});

test('the explanation concept follows the seeded explain cue', async () => {
  const context = await host.boot();
  assert.equal(evaluate(context, 'explanationConcept("Explain the Kotlin compiler")'), 'kotlin compiler');
  assert.equal(evaluate(context, 'explanationConcept("The Kotlin compiler is fast")'), null);
});

test('the fixture page digest is the one the Rust test pins', () => {
  assert.equal(createHash('sha256').update(PAGE).digest('hex'), PAGE_SHA256);
});

test('an explanation is composed from the retrieved statements, each citing URL and SHA-256', async () => {
  fixtureOnline = true;
  const result = await host.solve('Explain the Kotlin compiler', []);
  assert.equal(result.intent, 'explanation_research');
  assert.equal(
    result.content,
    'What the retrieved pages say about kotlin compiler:\n'
      + `- The Kotlin compiler turns Kotlin source files into JVM bytecode. (source: ${PAGE_URL}, sha256 ${PAGE_SHA256})\n`
      + `- The Kotlin compiler ships as the kotlinc command. (source: ${PAGE_URL}, sha256 ${PAGE_SHA256})`,
  );
  assert.ok(fetched.some((url) => url.startsWith(DUCKDUCKGO_API)));
  assert.ok(fetched.includes(PAGE_URL));
  const events = (result.solverEvents || []).filter((event) => event.kind.startsWith('explanation_research'));
  assert.deepEqual(events, [
    { kind: 'explanation_research:concept', payload: 'kotlin compiler' },
    { kind: 'explanation_research:source', payload: 'web_page' },
  ]);
});

test('a search that cannot be read falls through instead of guessing', async () => {
  fixtureOnline = false;
  const result = await host.solve('Explain the Rust borrow checker', []);
  assert.notEqual(result.intent, 'explanation_research');
  fixtureOnline = true;
});

test('a prompt that does not open with the explain cue is not answered from retrieved pages', async () => {
  const result = await host.solve('The Kotlin compiler is fast', []);
  assert.notEqual(result.intent, 'explanation_research');
});
