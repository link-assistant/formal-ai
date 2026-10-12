import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import { quotedSegmentSpans, quoteFault } from "../crate/normal_markov.mjs";
import assert from "./source-invariants.mjs";
import { observeSourceCallables, observedConditionalGraphs } from "./callable_catalog.mjs";
import { parseLinoRoot, findChildValue } from "../write_lino.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";
import { clausesWithSpans } from "../crate/obligation_ledger.mjs";
import { mentionsRole } from "../write_lexicon.mjs";
import { bindAcceptedImport } from "./accepted-import-authority.mjs";
/** A seeded goal is a conditional source-to-source composition, never an expected-output fixture. */
export function qualifySeededComposition(request, observations, acceptance, operation, seed) {
  const root = parseLinoRoot(seed).children.find(node => node.name === 'source-callable-composition');
  assert.ok(root, sourceContractDiagnostic("missing-composition-contract"));
  const goals = root.children.filter(node => node.name === 'goal');
  assert.equal(request.identity, sha256Hex(request.text), 'request drift');
  const goalScope = sourceGoalScope(request.text);
  assert.notEqual(goalScope, null, sourceContractDiagnostic("unproved-request-scope"));
  const clauses = clausesWithSpans(goalScope).map(([text, span]) => ({
    text: text.toLowerCase(),
    span
  }));
  const returned = clauses.filter(clause => mentionsRole('coding_return_action', clause.text));
  const matching = goals.filter(goal => goal.children.filter(node => node.name === 'lexeme' && node.id === 'en').some(node => node.children.filter(value => value.name === 'surface').some(surface => surface.children.some(value => value.name === 'text' && value.id !== '' && returned.some(clause => {
    const at = clause.text.indexOf(value.id.toLowerCase());
    return at >= 0 && !mentionsRole('statement_negation_cue', clause.text.slice(0, at));
  })))));
  // The Lino reader stores scalar lexemes in child values; do not guess a goal from export names.
  assert.equal(matching.length, 1, sourceContractDiagnostic("ambiguous-composition-goal"));
  const goal = matching[0];
  assert.equal(findChildValue(goal, 'producer-result'), 'optional-record');
  assert.equal(findChildValue(goal, 'consumer-result'), 'text');
  assert.equal(findChildValue(goal, 'absent-result'), 'null');
  const authorized = [];
  for (const source of observations) {
    const catalog = observeSourceCallables(source.content, source.path);
    for (const exported of catalog.exports) {
      try {
        const binding = {
          specifier: source.path,
          contentId: catalog.contentId,
          exported: exported.exposed
        };
        authorized.push({
          source,
          binding,
          receipt: bindAcceptedImport(acceptance, source, binding, operation)
        });
      } catch {}
    }
  }
  const graphs = observedConditionalGraphs(observations).filter(graph => graph.kind === 'conditional-schema-graph' && graph.schemas.produced.kind === 'optional' && graph.schemas.produced.value.kind === 'record' && graph.schemas.result.kind === 'text');
  const candidates = graphs.flatMap(graph => {
    const pair = graph.bindings.map(binding => authorized.find(value => value.source.path === binding.path && value.binding.exported === binding.exported));
    return pair.every(Boolean) ? [{
      graph,
      pair
    }] : [];
  });
  assert.equal(candidates.length, 1, sourceContractDiagnostic("ambiguous-source-composition"));
  return Object.freeze({
    kind: 'source-qualified-seeded-composition-candidate',
    goal: goal.id,
    graph: candidates[0].graph,
    imports: candidates[0].pair.map(value => value.receipt),
    requestIdentity: request.identity,
    goalClauseCoverage: 'partial',
    writeAuthority: false,
    authored: false,
    verified: false,
    callEffects: 'conditional',
    moduleEffects: 'unknown',
    rustEquivalence: 'unknown'
  });
}

/** Mirrors source_goal_scope in the copied Rust source contract. */
export function sourceGoalScope(source) {
  if (typeof source !== 'string' || quoteFault(source) !== null) return null;
  for (const character of source) {
    const point = character.codePointAt(0);
    if (point >= 0xD800 && point <= 0xDFFF) return null;
  }
  const encoder = new TextEncoder();
  let result = '',
    cursor = 0;
  for (const span of quotedSegmentSpans(source)) {
    result += source.slice(cursor, span.start);
    result += ' '.repeat(encoder.encode(source.slice(span.start, span.end)).length);
    cursor = span.end;
  }
  return result + source.slice(cursor);
}
