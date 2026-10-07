// Issue #1166 R1166-3/R1166-4 parity: the program executor binds its output
// operands from the obligation graph's clauses, and every executor that reads
// the graph reports the same `obligation_gap` lines in both roots. The cases
// are rust/tests/fixtures/issue-1166/executor-gaps.json, which
// rust/tests/unit/issue_1166_executor_gaps.rs reads too.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { explicitStdout, programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import {
  ObligationKind,
  boundOutputLiterals,
  formalizeRequest,
  obligationGapLines,
  obligationsOfKind,
  unboundOutputReport,
} from '../../../js/agentic/crate/intent_formalization_obligations.mjs';

const fixture = JSON.parse(readFileSync(new URL('../fixtures/issue-1166/executor-gaps.json', import.meta.url), 'utf8'));
const canonical = readFileSync(new URL('../fixtures/issue-1166/hello-world-kotlin-en.txt', import.meta.url), 'utf8');

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('output_literals_are_bound_from_the_obligation_clauses', () => {
  assert.equal(fixture.bindingCases.length, 4);
  for (const { id, prompt, expected } of fixture.bindingCases) {
    assert.deepEqual(boundOutputLiterals(prompt), expected, id);
    assert.equal(explicitStdout(prompt), expected.length ? expected.join('\n') : null, id);
  }
});

test('executors_record_every_obligation_gap_line', () => {
  assert.equal(fixture.gapCases.length, 2);
  for (const { id, prompt, expected } of fixture.gapCases) {
    assert.deepEqual(obligationGapLines(prompt), expected, id);
  }
});

test('unbound_output_literal_is_reported_against_its_node', () => {
  const prompt = 'Write a Python program that prints "A"\nThen write "B"';
  const graph = formalizeRequest(prompt);
  const bound = boundOutputLiterals(prompt);
  assert.deepEqual(bound, ['A']);
  const unbound = unboundOutputReport(graph, bound);
  assert.equal(unbound.length, 1);
  const node = obligationsOfKind(graph, ObligationKind.OutputLiteral)
    .find((candidate) => graph.literals.some(([id, value]) => id === candidate.node_id && value === 'B'));
  assert.ok(node, 'the graph demands the second literal');
  assert.ok(unbound[0].includes(node.node_id), unbound[0]);
  assert.deepEqual(unboundOutputReport(graph, ['A', 'B']), []);
});

test('program_contract_run_carries_the_gap_lines', () => {
  const prompt = 'Write a Python program that prints "A"\nThen write "B"';
  const answer = programContractAnswer(prompt);
  assert.ok(answer && answer.execution_recipe);
  assert.deepEqual(answer.obligation_gaps, obligationGapLines(prompt));
});

test('authoring_kinds_are_read_from_seed_roles', () => {
  const graph = formalizeRequest('First, add clear comments\nAlso, use a meaningful name for the workflow file\nFinally, a CI badge\n');
  assert.deepEqual(graph.classifications.map(([, kind]) => kind), [ObligationKind.CodeStyle, ObligationKind.FileNaming, ObligationKind.CiBadge]);
});

test('canonical_issue_body_formalizes_like_the_native_graph', () => {
  // rust/tests/unit/issue_1166_obligation_routing.rs: one output-literal node
  // after coreference, and every authoring kind present.
  const graph = formalizeRequest(canonical);
  assert.equal(obligationsOfKind(graph, ObligationKind.OutputLiteral).length, 1);
  assert.deepEqual(graph.literals.map(([, value]) => value), ['Hello, World!']);
  for (const kind of Object.values(ObligationKind).filter((kind) => kind !== ObligationKind.ProgramFile)) {
    assert.ok(graph.classifications.some(([, present]) => present === kind), kind);
  }
  assert.equal(explicitStdout(canonical), 'Hello, World!');
});
