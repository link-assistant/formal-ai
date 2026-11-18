// Shared enumeration for the generalized CNL meta test (issue #1584).
// The test and `experiments/issue-1584/feasibility.mjs --write` both use it, so the
// reviewed infeasibility ledger is always computed exactly like the assertion.
import {installHost} from '../../../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../../../js/server/lino.mjs';

installHost({parseLino, readText: readRepoFile, realm: {}});
const {cnlPolicy, outputCandidates, projectOutput, validateOutput} = await import('../../../../js/agentic/crate/cnl_output.mjs');
const {graphKey, utteranceGraphs} = await import('../../../../js/agentic/crate/cnl_grammar.mjs');

export const policy = cnlPolicy();
export const DENSITIES = ['standard', 'primes', 'top-100', 'top-200', 'top-500', 'top-1000', 'max-precision'];
export const LEDGER_PATH = 'data/parity/issue-1584-cnl-infeasible.json';

/** Every fact, its converse and both polarities, as statements and questions. */
export function factGraphs() {
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
export function dialogueGraphs() {
  return [
    {frame: 'reply', slots: {}, polarity: 'positive', mood: 'statement'},
    {frame: 'reply', slots: {}, polarity: 'negative', mood: 'statement'},
    {frame: 'clarify', slots: {count: '2'}, polarity: 'positive', mood: 'statement'},
    {frame: 'choose', slots: {}, polarity: 'positive', mood: 'request'},
  ];
}

// Checks that belong to the lexical density rather than to the named profile.
export const DENSITY_CHECKS = new Set(['non_prime', 'nsm_vocabulary', 'nsm_syntax', 'nsm_semantics',
  'frequency_vocabulary', 'lexical_minimum']);
export const profileViolations = report => report.violations.filter(id => !DENSITY_CHECKS.has(id));

/**
 * Project every graph under every language, profile and density, and validate the
 * whole candidate domain exhaustively as an oracle independent of the ordered search.
 */
export function complianceMatrix(languages = Object.keys(policy.grammars)) {
  const rows = [];
  for (const language of languages) {
    const grammar = policy.grammars[language];
    for (const graph of [...factGraphs(), ...dialogueGraphs()]) {
      for (const profile of policy.profiles) {
        for (const density of DENSITIES) {
          const configuration = {output_standard: profile.id, lexical_density: density};
          const text = projectOutput({language, source: '', parts: [{kind: 'graph', graph}]}, configuration, policy);
          const report = validateOutput(text, language, configuration, policy);
          const domain = outputCandidates(graph, language, configuration, policy)
            .map(candidate => validateOutput(candidate, language, configuration, policy));
          rows.push({language, graph: graphKey(graph), profile: profile.id, density, text, report,
            back: utteranceGraphs(text, grammar).map(graphKey),
            feasible: domain.some(item => item.compliant),
            profileFeasible: domain.some(item => profileViolations(item).length === 0)});
        }
      }
    }
  }
  return rows;
}

/**
 * The infeasibility ledger: for each language/profile/density with no compliant
 * candidate for some graph, how many graphs and which checks the best output still fails.
 */
export function infeasibilityLedger(rows) {
  const ledger = {};
  for (const row of rows.filter(item => !item.feasible)) {
    const key = `${row.language} ${row.profile} ${row.density}`;
    ledger[key] ??= {graphs: 0, failing: {}};
    ledger[key].graphs += 1;
    for (const id of [...row.report.violations, ...row.report.unchecked].sort()) {
      ledger[key].failing[id] = (ledger[key].failing[id] ?? 0) + 1;
    }
  }
  return Object.fromEntries(Object.entries(ledger).sort(([left], [right]) => left.localeCompare(right)));
}
