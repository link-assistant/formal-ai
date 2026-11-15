// Issue #1487 coding acceptance: operand order, comparison strength,
// connective, negation and an explicit count survive the path from issue text
// to obligation graph, to candidate program, to an independent test.
//
// A requirement whose structure no discovered coding part reads (`twice`,
// `not`, `or`) keeps that structure as an unresolved obligation, and the
// program for the requirement without it is withheld rather than offered.
// The unread-structure readings are held to the shared fixture
// rust/tests/fixtures/proposition-structure-parity.json, which the native test
// rust/tests/unit/proposition_structure_parity.rs reads too.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import { before, describe, it } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { formalizeRequest, gapReport } from '../../../js/agentic/crate/intent_formalization_obligations.mjs';
import { blankIdentifiers, blankQuoted, splitCompounds, unreadStructure } from '../../../js/agentic/crate/unread_structure.mjs';

const FIXTURE = JSON.parse(fs.readFileSync(new URL('../fixtures/proposition-structure-parity.json', import.meta.url), 'utf8'));
const UNRESOLVED = 'synthesis:obligation:unresolved_structure=';
const signature = 'Implement Python function count_vowels(text: str) -> int.';

let host;
before(async () => {
  host = new WorkerHost();
  await installNodeHost(host);
  host.context.fetch = () => { throw new Error('offline test: no network'); };
});

const python = spawnSync('python3', ['--version'], { encoding: 'utf8' }).status === 0;

/** Runs `source` and then `check` under python3; true when every assertion holds. */
const passes = (source, check) => spawnSync('python3', ['-c', `${source}\n${check}\n`], { encoding: 'utf8', timeout: 20000 }).status === 0;

describe('issue #1487 unread requirement structure', () => {
  it('reads every shared fixture entry', () => {
    assert.ok(FIXTURE.unread.length > 0);
    for (const entry of FIXTURE.unread) assert.equal(unreadStructure(entry.text, entry.ids), entry.unread, entry.text);
  });

  it('keeps operand order, comparison strength and count distinct', () => {
    const read = (text, ids = []) => unreadStructure(text, ids);
    assert.notEqual(read('Return a - b.'), read('Return b - a.'));
    assert.notEqual(read('Return whether a < b.'), read('Return whether a <= b.'));
    const vowels = ['reduce_count', 'vowel_character_class'];
    assert.equal(read('Return the number of vowels in the text.', vowels), '');
    assert.equal(read('Return twice the number of vowels in the text.', vowels), 'repetition=repetition-twice');
  });

  it('does not read quoted literals, identifiers or hyphenated compounds as structure', () => {
    assert.equal(blankQuoted('prints "a or b" and «x < y»'), 'prints   and  ');
    assert.equal(blankQuoted('an unclosed "quote stays'), 'an unclosed "quote stays');
    assert.equal(blankIdentifiers('below_zero(x) or not_a'), ' (x) or  ');
    assert.equal(splitCompounds('reverse-sorted a-b x-1'), 'reverse sorted a-b x-1');
  });
});

describe('issue #1487 obligation graph', () => {
  it('gives each distinct coding request its own obligation', () => {
    const pairs = [
      ['Write a JavaScript function that returns a - b.', 'Write a JavaScript function that returns b - a.'],
      ['Write a JavaScript function that returns whether a < b.', 'Write a JavaScript function that returns whether a <= b.'],
      ['Write a JavaScript function that returns a and b.', 'Write a JavaScript function that returns a or b.'],
      ['Write a Python program that prints "hello".', 'Write a Python program that prints "hello" twice.'],
    ];
    const ids = (text) => formalizeRequest(text).classifications.map(([id]) => id);
    for (const [left, right] of pairs) {
      assert.equal(ids(left).length, 1, left);
      assert.equal(ids(right).length, 1, right);
      assert.notDeepEqual(ids(left), ids(right), `${left} / ${right}`);
    }
    const check = (text) => formalizeRequest(text).nodes[0].expectation.check_id;
    assert.notEqual(check('Write a JavaScript function that returns a - b.'), check('Write a JavaScript function that returns b - a.'));
  });

  it('keeps an unrecognized clause as an unresolved obligation, never an empty proof', async () => {
    const prompt = 'Frobnicate the quux so that it glorps.';
    const graph = formalizeRequest(prompt);
    assert.equal(graph.nodes.length, 1);
    assert.equal(graph.nodes[0].expectation.kind, 'underivable');
    assert.equal(gapReport(graph).length, 1);
    const response = await host.solve(prompt);
    assert.ok(!response.evidence.some((entry) => /proof_outcome:proven|tests passed/u.test(entry)), JSON.stringify(response.evidence));

    const coding = await host.solve('Implement Python function frob(text: str) -> int. Return the glorp of the quux in the text.');
    assert.equal(coding.intent, 'write_program');
    assert.equal(coding.synthesizedProgram, undefined);
    assert.ok(coding.evidence.includes('synthesis:discovered_parts:'), JSON.stringify(coding.evidence));
    assert.ok(coding.evidence.includes('synthesis:verification:unverified_browser_boundary'));
  });
});

describe('issue #1487 candidate program and independent test', () => {
  it('composes the plain requirement and withholds a program that would drop twice, not or or', async () => {
    const plain = await host.solve(`${signature} Return the number of vowels in the text.`);
    assert.equal(plain.intent, 'write_program');
    assert.ok(plain.synthesizedProgram, 'the plain requirement composes');
    assert.ok(!plain.evidence.some((entry) => entry.startsWith(UNRESOLVED)), JSON.stringify(plain.evidence));

    const withheld = [
      ['Return twice the number of vowels in the text.', 'repetition=repetition-twice'],
      ['Return the number of characters that are not vowels in the text.', 'scope=_ not _'],
      ['Return the number of vowels or digits in the text.', 'shape=_ disjunction _'],
    ];
    for (const [requirement, unread] of withheld) {
      const response = await host.solve(`${signature} ${requirement}`);
      assert.equal(response.intent, 'write_program', requirement);
      assert.equal(response.synthesizedProgram, undefined, `${requirement} must not receive the plain program`);
      assert.ok(response.evidence.includes(`${UNRESOLVED}${unread}`), `${requirement}: ${JSON.stringify(response.evidence)}`);
      assert.ok(!response.evidence.includes('synthesis:ir:type_checked'), requirement);
      assert.doesNotMatch(response.content, /```python/u, requirement);
      assert.match(response.content, /Unresolved requirement structure/u, requirement);
    }
  });

  it('an independent test tells the plain program from the twice requirement', { skip: !python && 'python3 is unavailable' }, async () => {
    const plain = await host.solve(`${signature} Return the number of vowels in the text.`);
    const source = plain.synthesizedProgram.source;
    assert.ok(passes(source, "assert count_vowels('hello') == 2\nassert count_vowels('sky') == 0"), source);
    // The program a silent drop of `twice` would have offered fails the
    // independent test of the twice requirement, which is why it is withheld.
    assert.equal(passes(source, "assert count_vowels('hello') == 4"), false, source);
  });
});
