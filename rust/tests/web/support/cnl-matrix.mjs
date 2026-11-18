// Shared enumeration for the generalized CNL meta test (issue #1584).
// The test and `experiments/issue-1584/feasibility.mjs --write` both use it, so the
// reviewed infeasibility ledger is always computed exactly like the assertion.
import {installHost} from '../../../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../../../js/server/lino.mjs';

// CNL_MATRIX_LANGUAGES=en,ru hides every other grammar draft (used by
// experiments/issue-1584/check-language.mjs so one draft cannot break another).
const visible = process.env.CNL_MATRIX_LANGUAGES?.split(',');
installHost({parseLino, realm: {}, readText: file => {
  const match = /cnl-language-([a-z]+)\.lino$/u.exec(file);
  if (visible && match && !visible.includes(match[1])) throw new Error('cnl_language_hidden');
  return readRepoFile(file);
}});
const {
  cnlPolicy, controlledDialogue, formalizeOutput, outputCandidates, outputLinksNotation, outputMeaning, projectOutput,
  renderOutput, validateOutput,
} = await import('../../../../js/agentic/crate/cnl_output.mjs');
const {graphHolds, graphKey, realizeGraph, utteranceGraphs} = await import('../../../../js/agentic/crate/cnl_grammar.mjs');

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
export const DENSITY_CHECKS = new Set(['non-prime', 'nsm-vocabulary', 'nsm-syntax', 'nsm-semantics',
  'frequency-vocabulary', 'lexical-minimum']);
export const profileViolations = report => report.violations.filter(id => !DENSITY_CHECKS.has(id));

/**
 * Project every graph under every language, profile and density, and validate the
 * whole candidate domain exhaustively as an oracle independent of the ordered search.
 */
export function complianceMatrix(languages = Object.keys(policy.grammars)) {
  const rows = [];
  for (const language of languages) {
    const grammar = policy.grammars[language];
    if (!grammar) continue;
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

/**
 * The generalized properties of one language grammar. Each entry lists the
 * counterexamples found; an empty list means the property holds for every
 * enumerated graph, style, register and configuration.
 */
export function languageProperties(language) {
  const grammar = policy.grammars[language];
  const english = policy.grammars.en;
  const graphs = factGraphs();
  const all = [...graphs, ...dialogueGraphs()];
  const failures = {inventory: [], uniqueness: [], coverage: [], inversion: [], roundTrip: [], dialogue: []};
  if (!grammar) return {...failures, inventory: [`${language}: no grammar`]};
  const primeIds = item => item.primes.map(([id]) => id).join('|');
  if (primeIds(grammar) !== primeIds(english)) failures.inventory.push(`${language}: prime ids differ from en`);
  if (grammar.primes.length !== 65) failures.inventory.push(`${language}: ${grammar.primes.length} primes`);
  for (const term of english.terms) if (!grammar.termById.has(term.id)) failures.inventory.push(`${language}: missing term ${term.id}`);
  for (const style of english.outputStyles) {
    if (!grammar.outputStyles.includes(style)) failures.inventory.push(`${language}: missing output style ${style}`);
  }
  for (const register of policy.shared.registers.filter(name => name !== 'forms')) {
    const owners = new Map();
    for (const term of grammar.terms) {
      for (const form of term.registers[register] ?? []) {
        const owner = owners.get(form.toLowerCase());
        if (owner !== undefined && owner !== term.id) failures.uniqueness.push(`${language}/${register}: "${form}" names ${owner} and ${term.id}`);
        owners.set(form.toLowerCase(), term.id);
      }
    }
  }
  for (const graph of all) {
    for (const style of grammar.outputStyles) {
      if (realizeGraph(graph, style, 'forms', grammar) === null) failures.coverage.push(`${language}/${style}: ${graphKey(graph)}`);
      for (const register of policy.shared.registers) {
        const text = realizeGraph(graph, style, register, grammar);
        if (text === null) continue;
        const keys = utteranceGraphs(text, grammar).map(graphKey);
        if (!keys.includes(graphKey(graph))) failures.inversion.push(`${language}/${style}/${register}: "${text}" -> ${keys}`);
      }
    }
  }
  for (const graph of graphs) {
    const freeForm = realizeGraph(graph, 'plain', 'precise', grammar);
    if (freeForm === null) continue;
    let document = formalizeOutput(freeForm, language);
    const notation = outputLinksNotation(document);
    if (JSON.stringify(outputMeaning(document)) !== JSON.stringify([['graph', graphKey(graph)]])) {
      failures.roundTrip.push(`${language}: "${freeForm}" formalizes as ${JSON.stringify(outputMeaning(document))}`);
      continue;
    }
    for (const configuration of [{lexical_density: 'max-precision'}, {lexical_density: 'primes'}, {}]) {
      const output = renderOutput(document, configuration);
      document = formalizeOutput(output.text, language);
      if (outputLinksNotation(document) !== notation) {
        failures.roundTrip.push(`${language}: "${freeForm}" -> ${JSON.stringify(configuration)} "${output.text}" changes the meaning`);
        break;
      }
    }
  }
  for (const graph of graphs.filter(item => item.mood === 'question')) {
    const answers = new Set();
    for (const style of grammar.outputStyles) {
      const prompt = realizeGraph(graph, style, 'precise', grammar);
      const turn = controlledDialogue(prompt, [], {}, policy);
      if (utteranceGraphs(prompt, grammar).length > 1) {
        // Ambiguous wording (for example a bare polysemous noun) must ask instead of guessing.
        if (turn?.intent !== 'cnl_clarification') failures.dialogue.push(`${language}: "${prompt}" must clarify, got ${turn?.intent}`);
        else if (!turn.outputProjection.report.compliant) failures.dialogue.push(`${language}: clarification "${turn.content}" is not compliant`);
        continue;
      }
      if (turn?.intent !== 'cnl_answer' || turn.language !== language) {
        failures.dialogue.push(`${language}: "${prompt}" -> ${turn?.intent} ${turn?.language}`);
        continue;
      }
      if (!turn.outputProjection.report.compliant) failures.dialogue.push(`${language}: "${prompt}" -> "${turn.content}" ${JSON.stringify(turn.outputProjection.report)}`);
      answers.add(turn.content);
    }
    if (answers.size > 1) failures.dialogue.push(`${language}: one question, several answers: ${[...answers].join(' | ')}`);
    const expected = graphHolds(graph, policy.shared) === (graph.polarity === 'positive');
    const reply = realizeGraph({frame: 'reply', slots: {}, polarity: expected ? 'positive' : 'negative', mood: 'statement'},
      policy.profiles[0].styles[0], 'forms', grammar);
    for (const answer of answers) if (!answer.startsWith(reply)) failures.dialogue.push(`${language}: "${answer}" should start with "${reply}"`);
  }
  return failures;
}

/** Oracle counterexamples of one language: non-compliant feasible rows, broken profiles, standard infeasibility. */
export function complianceFailures(rows) {
  const failures = [];
  for (const row of rows) {
    const label = `${row.language} ${row.profile} ${row.density} ${row.graph} "${row.text}"`;
    if (row.back.length !== 1 || row.back[0] !== row.graph) failures.push(`${label} reads as ${row.back}`);
    if (row.feasible && !row.report.compliant) failures.push(`${label} ${JSON.stringify(row.report)}`);
    if (row.profileFeasible && profileViolations(row.report).length) failures.push(`${label} breaks the profile`);
    if (!row.feasible && row.report.compliant) failures.push(`${label} is compliant outside the oracle domain`);
    // Every named profile must be satisfiable at standard density for every graph.
    if (row.density === 'standard' && !row.feasible) failures.push(`${label} standard density is infeasible: ${JSON.stringify(row.report)}`);
  }
  return failures;
}
