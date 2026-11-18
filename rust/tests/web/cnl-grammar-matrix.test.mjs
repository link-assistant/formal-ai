// Generalized CNL meta test (issue #1584): instead of listing outputs by hand,
// enumerate every declared meaning graph, every language grammar, every
// template style and every output configuration, and check the properties
// that must hold for all of them:
//   1. coverage   - every output style can say every answerable graph;
//   2. inversion  - every realization parses back to the graph it came from;
//   3. compliance - every profile x density projection is compliant and keeps the meaning;
//   4. round trip - free-form -> Links Notation -> dense -> primes -> cnl-default keeps the graph;
//   5. dialogue   - every phrasing of one question gets the same answer, ambiguity asks first.
import test from 'node:test';
import assert from 'node:assert/strict';
import {installHost} from '../../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../../js/server/lino.mjs';
import {
  cnlPolicy, controlledDialogue, formalizeOutput, outputCandidates, outputLinksNotation, outputMeaning,
  projectOutput, renderOutput, validateOutput,
} from '../../../js/agentic/crate/cnl_output.mjs';
import {graphHolds, graphKey, realizeGraph, utteranceGraphs} from '../../../js/agentic/crate/cnl_grammar.mjs';

installHost({parseLino, readText: readRepoFile, realm: {}});
const policy = cnlPolicy();
const DENSITIES = ['standard', 'primes', 'top-100', 'top-200', 'top-500', 'top-1000', 'max-precision'];
const answerable = policy.shared.frames.filter(frame => frame.answerable);

/** Every fact, its converse and both polarities, as statements and questions. */
function factGraphs() {
  const graphs = new Map();
  for (const fact of policy.shared.facts) {
    const frame = policy.shared.frames.find(item => item.id === fact.frame);
    for (const args of [fact.args, [...fact.args].reverse()]) {
      for (const polarity of ['positive', 'negative']) {
        for (const mood of ['statement', 'question']) {
          const graph = {frame: frame.id, slots: Object.fromEntries(frame.slots.map((slot, index) => [slot, args[index]])),
            polarity, mood};
          graphs.set(graphKey(graph), graph);
        }
      }
    }
  }
  return [...graphs.values()];
}

/** Dialogue frames that every output style must also realize. */
function dialogueGraphs() {
  return [
    {frame: 'reply', slots: {}, polarity: 'positive', mood: 'statement'},
    {frame: 'reply', slots: {}, polarity: 'negative', mood: 'statement'},
    {frame: 'clarify', slots: {count: '2'}, polarity: 'positive', mood: 'statement'},
    {frame: 'choose', slots: {}, polarity: 'positive', mood: 'request'},
  ];
}

const grammars = Object.values(policy.grammars);
const graphs = factGraphs();

test('every supported language declares a grammar with the same prime inventory', () => {
  assert.deepEqual(Object.keys(policy.grammars).sort(), policy.languages.map(language => language.id).sort());
  const primeIds = grammar => grammar.primes.map(([id]) => id).join('|');
  for (const grammar of grammars) {
    assert.equal(grammar.primes.length, 65, grammar.id);
    assert.equal(primeIds(grammar), primeIds(policy.grammars.en), grammar.id);
    for (const term of policy.grammars.en.terms) assert.ok(grammar.termById.has(term.id), grammar.id + ' ' + term.id);
  }
});

test('explications are unique: no two concepts share a non-colloquial form', () => {
  for (const grammar of grammars) {
    for (const register of policy.shared.registers.filter(name => name !== 'forms')) {
      const owners = new Map();
      for (const term of grammar.terms) {
        for (const form of term.registers[register] ?? []) {
          const owner = owners.get(form.toLowerCase());
          assert.ok(owner === undefined || owner === term.id,
            `${grammar.id}/${register}: "${form}" names both ${owner} and ${term.id}`);
          owners.set(form.toLowerCase(), term.id);
        }
      }
    }
  }
});

test('coverage: every output style realizes every answerable graph and dialogue frame', () => {
  for (const grammar of grammars) {
    for (const style of grammar.outputStyles) {
      for (const graph of [...graphs, ...dialogueGraphs()]) {
        assert.ok(realizeGraph(graph, style, 'forms', grammar) !== null, `${grammar.id}/${style}: ${graphKey(graph)}`);
      }
    }
    assert.ok(answerable.length >= 3);
  }
});

test('inversion: every realization parses back to the graph it came from', () => {
  for (const grammar of grammars) {
    for (const graph of [...graphs, ...dialogueGraphs()]) {
      for (const style of grammar.outputStyles) {
        for (const register of policy.shared.registers) {
          const text = realizeGraph(graph, style, register, grammar);
          if (text === null) continue;
          const keys = utteranceGraphs(text, grammar).map(graphKey);
          assert.ok(keys.includes(graphKey(graph)), `${grammar.id}/${style}/${register}: "${text}" -> ${keys}`);
        }
      }
    }
  }
});

// The oracle validates the whole search domain exhaustively, independently of
// the projection's ordered search: if any candidate is compliant, the
// projection must be compliant; if none is, the combination is infeasible and
// the projection must still never break the named profile when some candidate
// keeps it. Infeasible combinations are listed in the case study.
const DENSITY_CHECKS = new Set(['non_prime', 'nsm_vocabulary', 'nsm_syntax', 'nsm_semantics',
  'frequency_vocabulary', 'lexical_minimum']);
const profileViolations = report => report.violations.filter(id => !DENSITY_CHECKS.has(id));

export function complianceMatrix() {
  const rows = [];
  for (const grammar of grammars) {
    for (const graph of [...graphs, ...dialogueGraphs()]) {
      for (const profile of policy.profiles) {
        for (const density of DENSITIES) {
          const configuration = {output_standard: profile.id, lexical_density: density};
          const text = projectOutput({language: grammar.id, source: '', parts: [{kind: 'graph', graph}]}, configuration);
          const report = validateOutput(text, grammar.id, configuration);
          const domain = outputCandidates(graph, grammar.id, configuration).map(candidate => validateOutput(candidate, grammar.id, configuration));
          rows.push({language: grammar.id, graph: graphKey(graph), profile: profile.id, density, text, report,
            back: utteranceGraphs(text, grammar).map(graphKey),
            feasible: domain.some(item => item.compliant),
            profileFeasible: domain.some(item => profileViolations(item).length === 0)});
        }
      }
    }
  }
  return rows;
}

test('compliance: every feasible profile and density projection is compliant and keeps its graph', () => {
  const failures = [];
  const rows = complianceMatrix();
  for (const row of rows) {
    const label = `${row.language} ${row.profile} ${row.density} "${row.text}"`;
    if (row.back.length !== 1 || row.back[0] !== row.graph) failures.push(`${label} reads as ${row.back}`);
    if (row.feasible && !row.report.compliant) failures.push(`${label} ${JSON.stringify(row.report)}`);
    if (row.profileFeasible && profileViolations(row.report).length) failures.push(`${label} breaks the profile`);
  }
  assert.deepEqual(failures.slice(0, 20), []);
  // Most of the matrix must be feasible; the infeasible remainder is documented, not hidden.
  const feasible = rows.filter(row => row.feasible).length;
  assert.ok(feasible / rows.length > 0.8, `${feasible}/${rows.length}`);
});

test('round trip: free-form -> Links Notation -> dense -> primes -> cnl-default keeps every graph', () => {
  for (const grammar of grammars) {
    for (const graph of graphs) {
      const freeForm = realizeGraph(graph, 'plain', 'precise', grammar);
      let document = formalizeOutput(freeForm, grammar.id);
      const notation = outputLinksNotation(document);
      assert.deepEqual(outputMeaning(document), [['graph', graphKey(graph)]], `${grammar.id}: ${freeForm}`);
      for (const configuration of [{lexical_density: 'max-precision'}, {lexical_density: 'primes'}, {}]) {
        const output = renderOutput(document, configuration);
        document = formalizeOutput(output.text, grammar.id);
        assert.equal(outputLinksNotation(document), notation, `${grammar.id}: ${freeForm} -> ${output.text}`);
      }
    }
  }
});

test('dialogue: every phrasing of one question is answered identically and from declared facts', () => {
  for (const grammar of grammars) {
    for (const graph of graphs.filter(item => item.mood === 'question')) {
      const answers = new Set();
      for (const style of grammar.outputStyles) {
        const prompt = realizeGraph(graph, style, 'precise', grammar);
        const turn = controlledDialogue(prompt, [], {}, policy);
        // Ambiguous wording (for example a bare polysemous noun) must ask instead of guessing.
        if (utteranceGraphs(prompt, grammar).length > 1) {
          assert.equal(turn?.intent, 'cnl_clarification', prompt);
          continue;
        }
        assert.equal(turn?.intent, 'cnl_answer', `${grammar.id}: ${prompt}`);
        assert.ok(turn.outputProjection.report.compliant, `${grammar.id}: ${prompt} -> ${turn.content}`);
        answers.add(turn.content);
      }
      assert.equal(answers.size, 1, `${grammar.id}: ${[...answers].join(' | ')}`);
      const expected = graphHolds(graph, policy.shared) === (graph.polarity === 'positive');
      const reply = realizeGraph({frame: 'reply', slots: {}, polarity: expected ? 'positive' : 'negative', mood: 'statement'},
        policy.profiles[0].styles[0], 'forms', grammar);
      assert.ok([...answers][0].startsWith(reply), `${grammar.id}: ${[...answers][0]} should start with ${reply}`);
    }
  }
});

test('in-dialog switching is deterministic for every configuration', () => {
  const prompt = 'Is a cat an animal?';
  for (const profile of policy.profiles) {
    for (const density of DENSITIES) {
      const configuration = {output_standard: profile.id, lexical_density: density};
      const first = controlledDialogue(prompt, [], configuration, policy).content;
      assert.equal(controlledDialogue(prompt, [], {}, policy).intent, 'cnl_answer');
      assert.equal(controlledDialogue(prompt, [], configuration, policy).content, first);
    }
  }
});
