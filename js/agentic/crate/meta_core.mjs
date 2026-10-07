// The general recursive meta core recorded ahead of method dispatch (issue
// #559, R1013 server parity): rust/src/meta_core.rs `record_meta_core` and the
// recorders it runs in order - rust/src/meta_frame.rs (js/agentic/crate/
// meta_frame.mjs), rust/src/method_registry.rs `record_method_registry`,
// rust/src/meta_reasoning.rs `record_work_unit_reasoning`,
// rust/src/meta_construction.rs `record_upward_construction`,
// rust/src/solution_evidence.rs `record_solution_evidence`,
// rust/src/selection.rs `record_selection`, rust/src/skill_ledger.rs
// `record_skill_ledger`, rust/src/reasoning_standard/mod.rs
// `record_reasoning_standard` over `open_episode`, and
// rust/src/obligation_ledger.rs `record_obligation_ledger`.
//
// The modes are the defaults every protocol solve runs with (recursion
// `both`, selection `record`, skill `accumulate`, depth 4). The reasoning
// standard is audited over the open episode `record_meta_core` builds - no
// claims, sources, conclusions or actions yet - so only the gates that fire on
// an empty episode are evaluated (the instruction-need gate; the rest are
// `not_triggered`). The rationale sentences live in
// data/meta/agentic-messages.lino.

import { readText } from '../host.mjs';
import { agenticMessage } from '../messages.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { debugString, stableId } from './engine_stable_id.mjs';
import { formatLinoRecord, pushLinoNode } from './links_format.mjs';
import { methodForRoute, recordMethodRegistry } from './method_registry.mjs';
import { ledgerCount, needLedgerLinksNotation, recordNeedLedger, recordProblemFrame, recordWorkUnits } from './meta_frame.mjs';
import { buildObligationTree, collectLeaves } from './obligation_ledger.mjs';
import { DEFAULT_SPLIT_DEPTH_BOUND } from './recursive_execution.mjs';

/** Mirrors `SolverConfig::default().max_decomposition_depth`. */
export const DEFAULT_MAX_DECOMPOSITION_DEPTH = 4;

const utf8Length = (text) => new TextEncoder().encode(text).length;
const methodName = (registry, route) => (route === null ? null : methodForRoute(registry, route)?.name ?? null);

// ------------------------------------------------------------ meta_reasoning.rs

const DECISION_SLUGS = { not_atomic: 'decompose', direct_method: 'direct_method', single_need: 'single_need', depth_bound: 'depth_bound' };

/** Mirrors `fn observe` in rust/src/meta_reasoning.rs. */
function observe(unit) {
  const route = unit.route === null ? agenticMessage('meta_route_none') : agenticMessage('meta_route_recognized', { route: unit.route });
  return agenticMessage('meta_observe', { depth: unit.depth, characters: Array.from(unit.source_span).length, route });
}

/** Mirrors `fn downward_rationale` in rust/src/meta_reasoning.rs. */
function downwardRationale(unit, method) {
  switch (unit.reason) {
    case 'not_atomic': return agenticMessage('meta_downward_not_atomic', { count: unit.children.length });
    case 'direct_method':
      return method === null ? agenticMessage('meta_downward_direct_unresolved') : agenticMessage('meta_downward_direct_method', { method });
    case 'single_need': return agenticMessage('meta_downward_single_need');
    default: return agenticMessage('meta_downward_depth_bound');
  }
}

/** Mirrors `fn upward_rationale` in rust/src/meta_reasoning.rs. */
function upwardRationale(unit) {
  if (!unit.atomic) return agenticMessage('meta_upward_compose', { count: unit.children.length });
  return agenticMessage(unit.reason === 'direct_method' ? 'meta_upward_direct' : 'meta_upward_blocked');
}

/** Mirrors `WorkUnitReasoning::for_unit` + `to_links_notation`: `[lino, steps]`. */
function workUnitReasoning(unit, registry) {
  const children = unit.children.map((child) => workUnitReasoning(child, registry));
  const method = unit.atomic ? methodName(registry, unit.route) : null;
  const pairs = [
    ['record_type', 'work_unit_reasoning'], ['unit_id', unit.unit_id], ['depth', String(unit.depth)],
    ['observation', observe(unit)], ['decision', DECISION_SLUGS[unit.reason]],
    ['downward_rationale', downwardRationale(unit, method)], ['upward_rationale', upwardRationale(unit)],
  ];
  if (method !== null) pairs.push(['method', method]);
  for (const child of unit.children) pairs.push(['child', child.unit_id]);
  let out = formatLinoRecord(unit.unit_id, pairs);
  for (const [lino] of children) out += `\n${lino}`;
  return [out, 1 + children.reduce((sum, [, steps]) => sum + steps, 0)];
}

// ------------------------------------------------------------ meta_construction.rs

/** Mirrors `fn visit_post_order` in rust/src/meta_construction.rs. */
function constructionSteps(unit, registry, steps = []) {
  for (const child of unit.children) constructionSteps(child, registry, steps);
  const order = steps.length + 1;
  if (!unit.children.length) {
    const method = methodName(registry, unit.route);
    const rationale = method === null
      ? agenticMessage('meta_construction_leaf_blocked')
      : agenticMessage('meta_construction_leaf_method', { method });
    steps.push({ unit, order, kind: 'leaf_method', method, inputs: [], rationale });
  } else {
    const inputs = unit.children.map((child) => child.unit_id);
    steps.push({ unit, order, kind: 'compose', method: null, inputs, rationale: agenticMessage('meta_construction_compose', { count: inputs.length }) });
  }
  return steps;
}

/** Mirrors `UpwardConstruction::to_links_notation`. */
function upwardConstructionLinksNotation(root, steps) {
  let out = formatLinoRecord('upward_construction', [
    ['record_type', 'upward_construction'], ['root_id', root.unit_id], ['step_count', String(steps.length)],
  ]);
  for (const step of steps) {
    const pairs = [
      ['record_type', 'construction_step'], ['unit_id', step.unit.unit_id], ['depth', String(step.unit.depth)],
      ['order', String(step.order)], ['kind', step.kind], ['rationale', step.rationale],
    ];
    if (step.method !== null) pairs.push(['method', step.method]);
    for (const input of step.inputs) pairs.push(['input', input]);
    out += `\n${formatLinoRecord(step.unit.unit_id, pairs)}`;
  }
  return out;
}

// ------------------------------------------------------------ solution_evidence.rs

/** Mirrors `SolutionEvidence::assemble`. */
function solutionEvidence(frame, ledger, registry) {
  return {
    frame_id: frame.frame_id,
    trails: ledger.rows.map((row) => {
      const method = methodName(registry, row.route);
      return {
        need_id: row.need_id,
        source_span: row.source_span,
        work_unit_id: row.unit_id,
        status: row.status,
        route: row.route,
        method,
        method_via_alias: row.route !== null && method !== null && row.route !== method,
        connected: row.unit_id !== null && row.status !== 'pending',
      };
    }),
  };
}

const accountedFor = (evidence) => evidence.trails.length > 0 && evidence.trails.every((trail) => trail.connected);

/** Mirrors `SolutionEvidence::to_links_notation`. */
function solutionEvidenceLinksNotation(evidence) {
  const fullyResolved = evidence.trails.length > 0 && evidence.trails.every((trail) => trail.connected && trail.status === 'satisfied');
  const pairs = [
    ['record_type', 'solution_evidence'], ['frame_id', evidence.frame_id], ['trail_count', String(evidence.trails.length)],
    ['accounted_for', String(accountedFor(evidence))], ['fully_resolved', String(fullyResolved)],
    ['resolved_to_method', String(evidence.trails.filter((trail) => trail.method !== null).length)],
  ];
  for (const trail of evidence.trails) pairs.push(['trail', trail.need_id]);
  let out = formatLinoRecord('solution_evidence', pairs);
  for (const trail of evidence.trails) {
    const trailPairs = [
      ['record_type', 'evidence_trail'], ['need_id', trail.need_id], ['source_span', trail.source_span],
      ['status', trail.status], ['connected', String(trail.connected)],
    ];
    if (trail.work_unit_id !== null) trailPairs.push(['work_unit', trail.work_unit_id]);
    if (trail.route !== null) trailPairs.push(['route', trail.route]);
    if (trail.method !== null) {
      trailPairs.push(['method', trail.method]);
      if (trail.method_via_alias) trailPairs.push(['method_via_alias', 'true']);
    }
    out += `\n${formatLinoRecord(trail.need_id, trailPairs)}`;
  }
  return out;
}

// ------------------------------------------------------------ selection.rs

/** Mirrors `MethodSelection::for_unit` + `to_links_notation`. */
function selectionLinksNotation(root, registry) {
  const leaves = [];
  const collect = (unit) => {
    if (!unit.children.length) leaves.push({ unit, method: methodName(registry, unit.route) });
    else unit.children.forEach(collect);
  };
  collect(root);
  const resolved = leaves.filter((leaf) => leaf.method !== null).length;
  let out = formatLinoRecord('selection', [
    ['record_type', 'selection'], ['root_id', root.unit_id], ['leaf_count', String(leaves.length)],
    ['resolved_count', String(resolved)], ['unresolved_count', String(leaves.length - resolved)],
  ]);
  for (const { unit, method } of leaves) {
    const pairs = [['record_type', 'leaf_selection'], ['unit_id', unit.unit_id], ['depth', String(unit.depth)]];
    if (unit.route !== null) pairs.push(['route', unit.route]);
    pairs.push(['method', method ?? 'unresolved']);
    out += `\n${formatLinoRecord(unit.unit_id, pairs)}`;
  }
  return out;
}

// ------------------------------------------------------------ skill_ledger.rs

/** Mirrors `fn curriculum_reason` in rust/src/skill_ledger.rs. */
function curriculumReason(hasMethod, connected) {
  if (!hasMethod) return agenticMessage('skill_curriculum_no_method');
  return agenticMessage(connected ? 'skill_curriculum_unsatisfied' : 'skill_curriculum_disconnected');
}

/** Mirrors `SkillLedger::from_evidence` + `to_links_notation`: `[lino, promotable]`. */
function skillLedgerLinksNotation(evidence) {
  const skills = [];
  const curriculum = [];
  for (const trail of evidence.trails) {
    if (trail.connected && trail.status === 'satisfied' && trail.method !== null) {
      skills.push({ skill_id: stableId('candidate_skill', `${trail.method}:${trail.source_span}`), trail });
    } else {
      curriculum.push({ item_id: stableId('curriculum_item', trail.need_id), trail, reason: curriculumReason(trail.method !== null, trail.connected) });
    }
  }
  const pairs = [
    ['record_type', 'skill_ledger'], ['frame_id', evidence.frame_id], ['skill_count', String(skills.length)],
    ['proposed', String(skills.length)], ['stable', '0'], ['promotable', '0'], ['curriculum_count', String(curriculum.length)],
  ];
  for (const skill of skills) pairs.push(['skill', skill.skill_id]);
  for (const item of curriculum) pairs.push(['curriculum', item.item_id]);
  let out = formatLinoRecord(stableId('skill_ledger', evidence.frame_id), pairs);
  for (const { skill_id: id, trail } of skills) {
    const skillPairs = [
      ['record_type', 'candidate_skill'], ['skill_id', id], ['method', trail.method], ['source_span', trail.source_span],
      ['status', 'proposed'], ['has_tests', 'false'], ['has_benchmark_delta', 'false'], ['promotable', 'false'],
    ];
    if (trail.route !== null) skillPairs.push(['route', trail.route]);
    if (trail.work_unit_id !== null) skillPairs.push(['work_unit', trail.work_unit_id]);
    out += `\n${formatLinoRecord(id, skillPairs)}`;
  }
  for (const { item_id: id, trail, reason } of curriculum) {
    out += `\n${formatLinoRecord(id, [
      ['record_type', 'curriculum_item'], ['item_id', id], ['need_id', trail.need_id],
      ['source_span', trail.source_span], ['status', trail.status], ['reason', reason],
    ])}`;
  }
  return [out, 0];
}

// ------------------------------------------------------------ reasoning_standard

/** Mirrors `fn standard` in rust/src/reasoning_standard/mod.rs: `{gates, thresholds}` (the shipped standard is valid). */
function reasoningStandard() {
  const root = parseLinoRoot(readText('data/meta/reasoning-standard.lino')).children.find((node) => node.name === 'reasoning_standard');
  const value = (node, name) => (node.children || []).find((child) => child.name === name)?.value ?? '';
  const thresholds = {};
  for (const node of root.children.filter((child) => child.name === 'threshold')) thresholds[node.value] = Number(value(node, 'value'));
  const gates = root.children.filter((child) => child.name === 'gate')
    .map((node) => ({ slug: node.value, order: Number(value(node, 'order')), trigger: value(node, 'trigger') }))
    .sort((left, right) => left.order - right.order);
  return { gates, thresholds };
}

/**
 * Mirrors `fn audit` over `open_episode(formalization)` and
 * `ReasoningAudit::to_links_notation`: `[lino, gates, verdict]`.
 */
function openEpisodeAudit(formalization) {
  const { gates, thresholds } = reasoningStandard();
  const episodeId = formalization.impulse_id;
  const taskClass = formalization.kind;
  const required = thresholds.minimum_instruction_sources ?? 0;
  const outcomes = gates.map((gate) => {
    // `GateTrigger::fires` on an episode with no claims, sources, conclusions or actions.
    const fires = gate.trigger === 'instruction_need_present' && taskClass.trim() !== '';
    if (!fires) return { ...gate, status: 'not_triggered', findings: [] };
    const findings = gate.slug === 'instruction_formalization'
      ? [`${taskClass}:no_instructions_gathered`, ...(required > 0 ? [`instruction_sources:0:required:${required}`] : [])]
      : [`${gate.slug}:no_evaluator`];
    return { ...gate, status: findings.length ? 'violated' : 'satisfied', findings };
  });
  const blockers = outcomes.filter((outcome) => outcome.status === 'violated')
    .flatMap((outcome) => outcome.findings.map((finding) => `${outcome.slug}:${finding}`));
  blockers.push(`${episodeId}:no_conclusion_recorded`);
  const verdict = 'not_confirmed_not_refuted';
  let out = pushLinoNode('', 0, 'reasoning_standard_audit', episodeId);
  out = pushLinoNode(out, 2, 'record_type', 'reasoning_standard_audit');
  out = pushLinoNode(out, 2, 'verdict', verdict);
  for (const outcome of outcomes) {
    out = pushLinoNode(out, 2, 'gate', outcome.slug);
    out = pushLinoNode(out, 4, 'order', String(outcome.order));
    out = pushLinoNode(out, 4, 'trigger', outcome.trigger);
    out = pushLinoNode(out, 4, 'status', outcome.status);
    for (const finding of outcome.findings) out = pushLinoNode(out, 4, 'finding', finding);
  }
  for (const blocker of blockers) out = pushLinoNode(out, 2, 'blocker', blocker);
  return [out, outcomes.length, verdict];
}

// ------------------------------------------------------------ obligation_ledger.rs

/** Rust `{:?}` of an `ObligationExpectation`. */
function expectationDebug(expectation) {
  switch (expectation.kind) {
    case 'file_bytes':
      return `FileBytes { path: ${debugString(expectation.path)}, sha256: ${expectation.sha256 === null || expectation.sha256 === undefined ? 'None' : `Some(${debugString(expectation.sha256)})`} }`;
    case 'command_exit': return `CommandExit { command: ${debugString(expectation.command)}, expected_exit: ${expectation.expected_exit} }`;
    case 'output_hash': return `OutputHash { command: ${debugString(expectation.command)}, sha256: ${debugString(expectation.sha256)} }`;
    case 'symbolic_check': return `SymbolicCheck { check_id: ${debugString(expectation.check_id)} }`;
    default: return `Underivable { reason: ${debugString(expectation.reason)} }`;
  }
}

/** Mirrors `ObligationExpectation::to_links_notation`. */
function expectationLinksNotation(expectation) {
  const pairs = [['record_type', 'obligation_expectation'], ['shape', expectation.kind]];
  if (expectation.kind === 'file_bytes') {
    pairs.push(['path', expectation.path]);
    if (expectation.sha256 !== null && expectation.sha256 !== undefined) pairs.push(['sha256', expectation.sha256]);
  } else if (expectation.kind === 'command_exit') {
    pairs.push(['command', expectation.command], ['expected_exit', String(expectation.expected_exit)]);
  } else if (expectation.kind === 'output_hash') {
    pairs.push(['command', expectation.command], ['sha256', expectation.sha256]);
  } else if (expectation.kind === 'symbolic_check') {
    pairs.push(['check_id', expectation.check_id]);
  } else {
    pairs.push(['reason', expectation.reason]);
  }
  return formatLinoRecord(stableId('obligation_expectation', expectationDebug(expectation)), pairs);
}

/** Mirrors `ObligationNode::to_links_notation` for a node no observation has reached (`Unattempted`). */
function obligationNodeLinksNotation(node) {
  const pairs = [['record_type', 'obligation_node'], ['node_id', node.node_id]];
  if (node.parent !== null) pairs.push(['parent', node.parent]);
  pairs.push(['clause', node.clause], ['span_start', String(node.span[0])], ['span_end', String(node.span[1])]);
  if (node.need_id !== null) pairs.push(['need_id', node.need_id]);
  pairs.push(['depth', String(node.depth)], ['expectation', node.expectation.kind], ['outcome', node.outcome.kind]);
  if (node.expectation.kind === 'underivable') pairs.push(['underivable_reason', node.expectation.reason]);
  let out = formatLinoRecord(node.node_id, pairs);
  out += `\n${expectationLinksNotation(node.expectation)}`;
  out += `\n${formatLinoRecord(stableId('obligation_outcome', 'Unattempted'), [['record_type', 'obligation_outcome'], ['outcome', 'unattempted']])}`;
  for (const child of node.children) out += `\n${obligationNodeLinksNotation(child)}`;
  return out;
}

/** Mirrors `fn need_for_clause` in rust/src/obligation_ledger.rs. */
function needForClause(frame, clause) {
  let best = null;
  for (const need of frame.needs) {
    if (!(clause.includes(need.source_span) || need.source_span.includes(clause))) continue;
    if (best === null || utf8Length(need.source_span) >= utf8Length(best.source_span)) best = need;
  }
  return best ? best.need_id : null;
}

/** Mirrors `ObligationLedger::for_frame` + `fn record_obligation_ledger` before any observation. */
function recordObligationLedger(log, frame, ledger, request) {
  const root = buildObligationTree(request, DEFAULT_SPLIT_DEPTH_BOUND);
  const leaves = collectLeaves(root);
  for (const leaf of leaves) leaf.need_id = needForClause(frame, leaf.clause);
  const count = (slug) => String(leaves.filter((leaf) => leaf.outcome.kind === slug).length);
  const header = formatLinoRecord(stableId('obligation_ledger', frame.frame_id), [
    ['record_type', 'obligation_ledger'], ['frame_id', frame.frame_id], ['node_count', String(leaves.length)],
    ['satisfied', count('satisfied')], ['refuted', count('refuted')], ['unsatisfiable', count('unsatisfiable')],
    ['unattempted', count('unattempted')], ['discharged', 'false'],
  ]);
  log.push({ kind: 'obligation_ledger', payload: `${header}\n${obligationNodeLinksNotation(root)}` });
  log.push({ kind: 'obligation_ledger:discharged', payload: 'false' });
  // `need_ledger_with_execution`: nothing is satisfied before dispatch, so the executed ledger is the planned one.
  log.push({ kind: 'need_ledger:executed', payload: needLedgerLinksNotation(ledger) });
  log.push({ kind: 'need_ledger:executed_satisfied', payload: String(ledgerCount(ledger, 'satisfied')) });
}

/**
 * Mirrors `fn record_meta_core` in rust/src/meta_core.rs with the default
 * modes: every recorder's events, in order, appended to `log`.
 * @param {Array<{kind: string, payload: string}>} log
 * @param {object} formalization the `formalizeIntentRecord` record
 */
export function recordMetaCore(log, formalization, maxDepth = DEFAULT_MAX_DECOMPOSITION_DEPTH) {
  const frame = recordProblemFrame(log, formalization);
  const root = recordWorkUnits(log, formalization, maxDepth);
  const ledger = recordNeedLedger(log, frame, root);
  const registry = recordMethodRegistry(log);
  const [reasoning, reasoningSteps] = workUnitReasoning(root, registry);
  log.push({ kind: 'work_unit_reasoning', payload: reasoning });
  log.push({ kind: 'work_unit_reasoning:steps', payload: String(reasoningSteps) });
  const steps = constructionSteps(root, registry);
  log.push({ kind: 'upward_construction', payload: upwardConstructionLinksNotation(root, steps) });
  log.push({ kind: 'upward_construction:steps', payload: String(steps.length) });
  const evidence = solutionEvidence(frame, ledger, registry);
  log.push({ kind: 'solution_evidence', payload: solutionEvidenceLinksNotation(evidence) });
  log.push({ kind: 'solution_evidence:accounted_for', payload: String(accountedFor(evidence)) });
  log.push({ kind: 'selection', payload: selectionLinksNotation(root, registry) });
  const [skills, promotable] = skillLedgerLinksNotation(evidence);
  log.push({ kind: 'skill_ledger', payload: skills });
  log.push({ kind: 'skill_ledger:promotable', payload: String(promotable) });
  const [audit, gates, verdict] = openEpisodeAudit(formalization);
  log.push({ kind: 'reasoning_standard', payload: audit });
  log.push({ kind: 'reasoning_standard:gates', payload: String(gates) });
  log.push({ kind: 'reasoning_standard:verdict', payload: verdict });
  recordObligationLedger(log, frame, ledger, formalization.source_text);
}
