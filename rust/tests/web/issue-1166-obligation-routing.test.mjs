// Issue #1166 (E131) in the JavaScript roots: the request formalized into
// obligations. Twin of rust/tests/unit/issue_1166_obligation_routing.rs over
// the agentic port (js/agentic/crate/intent_formalization_obligations.mjs):
// one node per repeated literal (R1166-2), the never-discarded underivable
// clause (R1166-5), the multilingual (R1166-6) and paraphrase (R1166-7)
// fixtures, and the terminal-routing guard (R1166-8), which the browser
// worker's `leadingShellCommand` (js/worker/formal_ai_worker_wikipedia_and_shell_commands.js) now reads
// exactly as the native `leading_shell_command` does.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, describe, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import {
  ObligationKind, formalizeRequest, gapReport, hasObligation, literalFor, obligationKinds, obligationsOfKind,
  requestCarriesWorkObligations, requestDemands, underivable, uniqueOutputLiteral,
} from '../../../js/agentic/crate/intent_formalization_obligations.mjs';

const fixture = (name) => readFileSync(new URL(`../fixtures/issue-1166/${name}`, import.meta.url), 'utf8');
const CANONICAL = 'hello-world-kotlin-en.txt';

let realm;
before(async () => {
  realm = await installNodeHost(new WorkerHost());
});

describe('issue #1166 obligation graph (agentic JavaScript root)', () => {
  test('the canonical body anchors its output literal once', () => {
    assert.equal(uniqueOutputLiteral(formalizeRequest(fixture(CANONICAL))), 'Hello, World!');
  });

  test('R1166-2: two identical output clauses produce one node', () => {
    const graph = formalizeRequest(fixture(CANONICAL));
    assert.equal(obligationsOfKind(graph, ObligationKind.OutputLiteral).length, 1);
  });

  test('distinct literals stay distinct nodes and have no unique literal', () => {
    const graph = formalizeRequest('First, print exactly: "Alpha"\nThen, print exactly: "Beta"\n');
    assert.equal(obligationsOfKind(graph, ObligationKind.OutputLiteral).length, 2);
    assert.equal(uniqueOutputLiteral(graph), null);
  });

  test('coreference does not merge a filename with an output literal', () => {
    const graph = formalizeRequest('First, print exactly: "test-hello-world.yml"\nAlso, use a meaningful name for the workflow file\n');
    assert.equal(obligationsOfKind(graph, ObligationKind.OutputLiteral).length, 1);
    const naming = obligationsOfKind(graph, ObligationKind.FileNaming);
    assert.equal(naming.length, 1);
    assert.equal(literalFor(graph, naming[0]), null);
  });

  test('the canonical body yields CI, naming, style and badge obligations', () => {
    const graph = formalizeRequest(fixture(CANONICAL));
    for (const kind of [ObligationKind.CiWorkflow, ObligationKind.FileNaming, ObligationKind.CodeStyle, ObligationKind.CiBadge]) {
      assert.ok(hasObligation(graph, kind), kind);
    }
  });

  test('R1166-5: an unknown clause yields an underivable node with its span, reported', () => {
    const graph = formalizeRequest('First, print exactly: "Hi"\nThen, harmonize the quantum flux\n');
    const gaps = underivable(graph);
    assert.ok(gaps.length > 0);
    assert.ok(gaps.every((node) => node.span[1] > node.span[0]));
    assert.ok(gapReport(graph).every((line) => line.includes('underivable')));
  });

  test('R1166-6: ru, hi, zh and es bodies yield the same graph modulo the language tag', () => {
    const en = formalizeRequest(fixture(CANONICAL));
    const ru = formalizeRequest(fixture('hello-world-kotlin-ru.txt'));
    assert.equal(ru.nodes.length, en.nodes.length);
    for (const language of ['ru', 'hi', 'zh', 'es']) {
      const graph = formalizeRequest(fixture(`hello-world-kotlin-${language}.txt`));
      assert.deepEqual(obligationKinds(graph), obligationKinds(en), language);
      assert.equal(uniqueOutputLiteral(graph), uniqueOutputLiteral(en), language);
    }
  });

  test('R1166-7: a paraphrase yields an equivalent graph', () => {
    const canonical = formalizeRequest(fixture(CANONICAL));
    const paraphrase = formalizeRequest(fixture('hello-world-kotlin-paraphrase-en.txt'));
    assert.equal(paraphrase.nodes.length, canonical.nodes.length);
    assert.deepEqual(obligationKinds(paraphrase), obligationKinds(canonical));
    assert.equal(uniqueOutputLiteral(paraphrase), uniqueOutputLiteral(canonical));
  });

  test('R1166-3: request_demands reads the CI clause from the graph', () => {
    assert.ok(requestDemands(fixture(CANONICAL), ObligationKind.CiWorkflow));
    assert.ok(!requestDemands('Write a program that prints "Hi"', ObligationKind.CiWorkflow));
  });
});

describe('issue #1166 R1166-8 routing by the formalized request', () => {
  const WORK = [
    'make a small Kotlin app that prints "Hello, World!"; add a GitHub Actions workflow',
    'сделай приложение на Kotlin, которое выводит "Привет"; добавь workflow GitHub Actions',
  ];
  const COMMANDS = ['git status', 'echo "Hello, World!"'];

  test('work-obligation requests carry work; command lines do not', () => {
    for (const prompt of WORK) assert.ok(requestCarriesWorkObligations(prompt), prompt);
    for (const prompt of COMMANDS) assert.ok(!requestCarriesWorkObligations(prompt), prompt);
  });

  test('the browser worker agrees with the agentic root', () => {
    for (const prompt of [...WORK, ...COMMANDS, fixture(CANONICAL)]) {
      assert.equal(realm.obligationCarriesWork(realm.obligationFormalizeRequest(prompt)), requestCarriesWorkObligations(prompt), prompt);
    }
  });

  test('the worker terminal router keeps commands and refuses sentences about building something', () => {
    assert.equal(realm.detectTerminalCommand('git status'), 'git status');
    assert.equal(realm.detectTerminalCommand('echo "Hello, World!"'), 'echo "Hello, World!"');
    // Every word of this sentence parses as a command argument (no #1175
    // marker word, no trailing punctuation), so only the formalized request
    // tells it apart from `make <target>`.
    const sentence = 'make kotlin app printing "Hello, World!" plus GitHub Actions workflow';
    assert.ok(realm.parsesAsCommandArguments(sentence.slice('make'.length)));
    assert.equal(realm.detectTerminalCommand(sentence), null);
  });
});
