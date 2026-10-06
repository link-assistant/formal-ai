// White-box self-improvement over unknown-path traces: the learner the
// conversation `learn` route stages a reported dialog through, and the
// structured `learning_trace` a chat completion carries when the solver
// verified a synthesized rule (rust/src/self_improvement.rs).
//
// The Rust learner reads the solver's event log flattened into the answer's
// `links_notation` (rust/src/engine.rs `answer_links_notation`). The browser
// worker reports the same solver events as thinking steps
// (js/worker/formal_ai_worker_16.js `writeProgramDiagnosticBundle`), so
// `solverEventsFromThinkingSteps` rebuilds that event log and
// `answerStepsLinks` flattens it the Rust way before the verbatim
// `event_payload` reads it.

import { stableId } from './ids.mjs';
import { readRepoFile } from './lino.mjs';
import { rustLines } from './memory-store.mjs';
import { serverMessage } from './messages.mjs';

const CODING_MODIFICATION_SUITE = 'data/benchmarks/coding-modification-suite.lino';
const DEFAULT_SUITE_ID = 'issue_362_multilingual_coding_modification';
const DEFAULT_RUNNER = 'cargo test --manifest-path rust/Cargo.toml --test unit issue_362_multilingual_multi_turn_coding_modification_ratchet -- --nocapture';
const ABSENT_COUNT = 0;
const LEARNABLE_KINDS = ['selected_rule', 'rule_synthesis_candidate', 'rule_verification', 'write_program_plan'];
const CANDIDATE_FIELDS = ['id', 'source', 'base_task', 'modifier', 'operation', 'operation_modifier', 'target', 'resolved_task'];
const VERIFICATION_FIELDS = ['candidate', 'fixture', 'input', 'expected_order', 'lowering_check', 'render_check', 'status'];

const isObjectValue = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

/** Mirrors rust/src/self_improvement.rs `unquote`. */
export function unquote(value) {
  const inner = value.length >= 2 && value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
  let out = '';
  const chars = [...inner];
  for (let index = 0; index < chars.length; index += 1) {
    if (chars[index] !== '\\') {
      out += chars[index];
      continue;
    }
    index += 1;
    const next = chars[index];
    if (next === 'n') out += '\n';
    else if (next === '"') out += '"';
    else if (next === '\\' || next === undefined) out += '\\';
    else out += `\\${next}`;
  }
  return out;
}

/** Mirrors rust/src/self_improvement.rs `quote`. */
function quote(value) {
  return value.replace(/\\/g, '\\\\').replace(/"/g, "'").replace(/\n/g, '\\n').replace(/\r/g, '\\r').replace(/\t/g, '\\t');
}

/** Mirrors rust/src/self_improvement.rs `field_value`. */
export function fieldValue(block, name) {
  for (const line of rustLines(block)) {
    const trimmed = line.trim();
    if (!trimmed.startsWith(name)) continue;
    const rest = trimmed.slice(name.length);
    if (!rest) continue;
    if (/^\s/u.test(rest)) return unquote(rest.trim());
  }
  return null;
}

/** Mirrors rust/src/self_improvement.rs `parse_first_record`. */
function parseFirstRecord(text) {
  const block = text.split('\n\n').map((record) => record.trim()).find((record) => record);
  if (block === undefined) return null;
  const fields = [];
  for (const line of rustLines(block).slice(1)) {
    const trimmed = line.trim();
    const space = trimmed.indexOf(' ');
    if (space < 0) continue;
    fields.push([trimmed.slice(0, space), unquote(trimmed.slice(space + 1).trim())]);
  }
  return { field: (name) => fields.find(([key]) => key === name)?.[1] ?? null };
}

/** Mirrors rust/src/self_improvement.rs `BenchmarkGateReport`. */
export class BenchmarkGateReport {
  constructor(suiteId, runner, passed, failed, minimumPassCount, observed = true) {
    this.suite_id = suiteId;
    this.runner = runner;
    this.passed = passed;
    this.failed = failed;
    this.minimum_pass_count = minimumPassCount;
    this.observed = observed;
  }

  /** Mirrors `BenchmarkGateReport::issue_362_from_counts`. */
  static issue362FromCounts(passed, failed) {
    const suite = parseFirstRecord(readRepoFile(CODING_MODIFICATION_SUITE));
    const minimum = Number.parseInt(suite.field('minimum_pass_count') ?? '', 10);
    return new BenchmarkGateReport(
      suite.field('id') ?? DEFAULT_SUITE_ID,
      suite.field('runner') ?? DEFAULT_RUNNER,
      passed,
      failed,
      /^\+?[0-9]+$/.test(suite.field('minimum_pass_count') ?? '') ? minimum : 1,
    );
  }

  /** Mirrors `BenchmarkGateReport::absent_for_issue_362`. */
  static absentForIssue362() {
    const report = BenchmarkGateReport.issue362FromCounts(ABSENT_COUNT, ABSENT_COUNT);
    report.observed = false;
    return report;
  }

  /** Mirrors `BenchmarkGateReport::permits_adoption`. */
  permitsAdoption() {
    return this.observed && this.passed >= this.minimum_pass_count;
  }

  /** Mirrors `BenchmarkGateReport::status_slug`. */
  statusSlug() {
    if (!this.observed) return 'absent';
    return this.permitsAdoption() ? 'passed' : 'failed';
  }
}

/** Mirrors rust/src/event_log.rs `EventLog::append` (content-addressed ids). */
function appendEvent(events, kind, payload) {
  events.push({ id: stableId(kind, `${kind}:${events.length}:${payload}`), kind, payload });
}

/** Mirrors rust/src/self_improvement.rs `UnknownTrace::new`. */
export function unknownTrace(prompt, events) {
  const fingerprint = events.map((event) => `${event.kind}=${event.payload}`).join('\n');
  return { id: stableId('unknown_trace', `${prompt}\n${fingerprint}`), prompt, events };
}

/** Mirrors rust/src/self_improvement.rs `UnknownTrace::from_event_log`. */
export function unknownTraceFromEventLog(prompt, intent, events) {
  const involved = intent === 'unknown' || events.some((event) =>
    event.kind === 'reasoning:unknown' || (event.kind === 'selected_rule' && event.payload.includes('initial unknown')));
  return involved ? unknownTrace(prompt, events.slice()) : null;
}

/** Mirrors rust/src/self_improvement.rs `UnknownTrace::links_notation`. */
export function unknownTraceLinksNotation(trace) {
  let out = 'unknown_trace\n';
  out += `  id "${quote(trace.id)}"\n  prompt "${quote(trace.prompt)}"\n`;
  out += `  event_count "${trace.events.length}"\n`;
  for (const event of trace.events) {
    out += `  event\n    kind "${quote(event.kind)}"\n    id "${quote(event.id)}"\n    payload "${quote(event.payload)}"\n`;
  }
  return out.trimEnd();
}

/** Mirrors rust/src/self_improvement.rs `validate_slug`. */
function validateSlug(name, value) {
  if (value && /^[A-Za-z0-9_-]+$/.test(value)) return null;
  return serverMessage('learning_invalid_slug', { name, value });
}

/** Mirrors rust/src/self_improvement.rs `learned_program_rule_lino`. */
function learnedProgramRuleLino(ruleId, baseTask, modifier, resolvedTask) {
  return [
    'substitution_rules',
    '  id "learned_program_plan_rules"',
    `  rule "${ruleId}"`,
    '    order "90"',
    '    event "learned"',
    `    when "request:modifier -> ${modifier}"`,
    `    replace "request:task -> ${baseTask}"`,
    `      with "request:task -> ${resolvedTask}"`,
  ].join('\n');
}

/**
 * Mirrors rust/src/self_improvement.rs `propose_rule_from_trace`: the
 * proposal record, or `{ rejection }` with the reason. The learned rule's
 * `SubstitutionRuleSet::from_links_notation` check cannot fail here: every
 * value it interpolates passed `validate_slug` first.
 */
export function proposeRuleFromTrace(trace, gate) {
  const reversed = [...trace.events].reverse();
  const candidate = reversed.find((event) => event.kind === 'rule_synthesis_candidate');
  if (!candidate) return { rejection: serverMessage('learning_no_event', { kind: 'rule_synthesis_candidate' }) };
  const verification = reversed.find((event) => event.kind === 'rule_verification');
  if (!verification) return { rejection: serverMessage('learning_no_event', { kind: 'rule_verification' }) };
  const status = fieldValue(verification.payload, 'status') ?? '';
  if (status !== 'passed') return { rejection: serverMessage('learning_verification_failed', { status }) };
  const fields = {};
  for (const name of ['id', 'base_task', 'modifier', 'resolved_task']) {
    fields[name] = fieldValue(candidate.payload, name);
    if (fields[name] === null) return { rejection: serverMessage('learning_missing_field', { name }) };
  }
  const fixture = fieldValue(verification.payload, 'fixture') ?? 'unknown';
  for (const [name, key] of [['rule_id', 'id'], ['base_task', 'base_task'], ['modifier', 'modifier'], ['resolved_task', 'resolved_task']]) {
    const invalid = validateSlug(name, fields[key]);
    if (invalid !== null) return { rejection: invalid };
  }
  const { id: ruleId, base_task: baseTask, modifier, resolved_task: resolvedTask } = fields;
  const summary = serverMessage('learning_proposal_summary', {
    modifier,
    base_task: baseTask,
    resolved_task: resolvedTask,
    fixture,
    suite: gate.suite_id,
    status: gate.statusSlug(),
    passed: gate.passed,
    minimum: gate.minimum_pass_count,
  });
  return {
    proposal: {
      id: stableId('learned_rule', `${trace.id}:${ruleId}:${baseTask}:${modifier}:${resolvedTask}`),
      trace_id: trace.id,
      rule_id: ruleId,
      base_task: baseTask,
      modifier,
      resolved_task: resolvedTask,
      fixture,
      summary,
      seed_rule_lino: learnedProgramRuleLino(ruleId, baseTask, modifier, resolvedTask),
      adoption: gate.permitsAdoption() ? 'adoptable' : 'blocked_by_benchmark',
    },
  };
}

/** Mirrors rust/src/self_improvement.rs `LearningRun::new`. */
function learningRun(gate, traceCount, proposals, rejections) {
  const fingerprint = `${gate.suite_id}:${gate.passed}:${gate.failed}:${traceCount}:${proposals.map((proposal) => proposal.id).join(',')}`;
  return { id: stableId('self_improvement_run', fingerprint), trace_count: traceCount, gate, proposals, rejections };
}

/** Mirrors rust/src/self_improvement.rs `learn_rules_from_unknown_traces`. */
export function learnRulesFromUnknownTraces(traces, gate) {
  const proposals = [];
  const rejections = [];
  for (const trace of traces) {
    const outcome = proposeRuleFromTrace(trace, gate);
    if (outcome.proposal) proposals.push(outcome.proposal);
    else rejections.push({ trace_id: trace.id, reason: outcome.rejection });
  }
  return learningRun(gate, traces.length, proposals, rejections);
}

/** Mirrors rust/src/self_improvement.rs `LearningRun::links_notation`. */
export function learningRunLinksNotation(run) {
  const field = (key, value) => `  ${key} "${quote(String(value))}"\n`;
  const nested = (key, value) => `    ${key} "${quote(String(value))}"\n`;
  let out = 'self_improvement_run\n';
  out += field('id', run.id);
  out += `  trace_count "${run.trace_count}"\n`;
  out += field('benchmark_suite', run.gate.suite_id);
  out += field('benchmark_runner', run.gate.runner);
  out += `  benchmark_passed "${run.gate.passed}"\n  benchmark_failed "${run.gate.failed}"\n`;
  out += `  benchmark_minimum_pass_count "${run.gate.minimum_pass_count}"\n`;
  out += field('benchmark_status', run.gate.statusSlug());
  for (const proposal of run.proposals) {
    out += '  learned_rule\n';
    out += nested('id', proposal.id) + nested('trace', proposal.trace_id) + nested('rule', proposal.rule_id);
    out += nested('base_task', proposal.base_task) + nested('modifier', proposal.modifier);
    out += nested('resolved_task', proposal.resolved_task) + nested('fixture', proposal.fixture);
    out += nested('adoption', proposal.adoption) + nested('summary', proposal.summary);
    out += nested('seed_rule', proposal.seed_rule_lino);
  }
  for (const rejection of run.rejections) {
    out += `  rejected_trace\n${nested('trace', rejection.trace_id)}${nested('reason', rejection.reason)}`;
  }
  return out.trimEnd();
}

/** Mirrors rust/src/self_improvement.rs `find_learning_trace`: `[prompt|null, events]` or null. */
export function findLearningTrace(value) {
  if (isObjectValue(value)) {
    const trace = value.learning_trace;
    if (isObjectValue(trace) && Array.isArray(trace.events)) {
      return [typeof trace.prompt === 'string' ? trace.prompt : null, trace.events];
    }
    // serde_json maps iterate in sorted key order.
    for (const key of Object.keys(value).sort()) {
      const found = findLearningTrace(value[key]);
      if (found) return found;
    }
    return null;
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      const found = findLearningTrace(item);
      if (found) return found;
    }
    return null;
  }
  if (typeof value === 'string') {
    let nested;
    try {
      nested = JSON.parse(value);
    } catch {
      return null;
    }
    return findLearningTrace(nested);
  }
  return null;
}

/**
 * Mirrors rust/src/self_improvement.rs `learn_from_reported_conversation`:
 * `{ trace, learning, awaiting_human_review, promoted_ledger }` or null.
 */
export function learnFromReportedConversation(context) {
  const found = findLearningTrace(context);
  if (!found) return null;
  const [tracePrompt, reported] = found;
  let prompt = tracePrompt;
  if (prompt === null) {
    const messages = isObjectValue(context) && Array.isArray(context.messages) ? context.messages : [];
    const user = [...messages].reverse().find((message) => isObjectValue(message) && message.role === 'user');
    prompt = user && typeof user.content === 'string' ? user.content : null;
  }
  if (prompt === null) return null;
  const events = [];
  for (const event of reported) {
    if (!isObjectValue(event) || typeof event.kind !== 'string' || typeof event.payload !== 'string') return null;
    if (LEARNABLE_KINDS.includes(event.kind)) appendEvent(events, event.kind, event.payload);
  }
  const trace = unknownTraceFromEventLog(prompt, 'unknown', events);
  if (!trace) return null;
  return {
    trace,
    learning: learnRulesFromUnknownTraces([trace], BenchmarkGateReport.absentForIssue362()),
    awaiting_human_review: true,
    promoted_ledger: null,
  };
}

// ---- learning_trace_from_symbolic_answer ---------------------------------

/** Mirrors rust/src/links_format.rs `flatten_lino_value`. */
function flattenLinoValue(value) {
  return value.replace(/\r/g, '\\r').replace(/\n/g, '\\n').replace(/\t/g, '\\t');
}

/**
 * The solver events behind the worker's thinking steps
 * (js/worker/formal_ai_worker_16.js `writeProgramDiagnosticBundle`): a route
 * attempt carries `selected_rule <payload>`, a rule construction carries the
 * `rule_synthesis_request` record followed by the `rule_synthesis_candidate`
 * one, a program plan carries `write_program_plan` and its body.
 */
export function solverEventsFromThinkingSteps(steps) {
  const events = [];
  for (const step of steps || []) {
    const detail = String(step.detail ?? '');
    const name = String(step.step ?? '');
    const lead = detail.split(/\s/u, 1)[0];
    if (name === 'rule_construction') {
      const at = detail.search(/(^|\n)rule_synthesis_candidate(\n|$)/u);
      if (at < 0) {
        events.push({ kind: name, payload: detail });
        continue;
      }
      const split = detail[at] === '\n' ? at + 1 : at;
      if (split > 0) events.push({ kind: 'rule_synthesis_request', payload: detail.slice(0, split).replace(/\n$/u, '') });
      events.push({ kind: 'rule_synthesis_candidate', payload: detail.slice(split) });
    } else if (lead === 'selected_rule' || lead === 'write_program_plan') {
      events.push({ kind: lead, payload: detail.slice(lead.length).replace(/^[ \n]/u, '') });
    } else if (name === 'rule_verification') {
      events.push({ kind: name, payload: detail });
    } else {
      events.push({ kind: name, payload: detail });
    }
  }
  return events;
}

/**
 * The solver event log behind an answer: the native log the worker records
 * (`solver_events`, js/worker/formal_ai_worker_solver_events.js) once it
 * carries the rule-synthesis events, else the log rebuilt from the worker's
 * own trace (`worker_steps`, or the projected `thinking_steps`).
 */
function answerSolverEvents(answer) {
  const native = Array.isArray(answer?.solver_events) ? answer.solver_events : [];
  if (native.some((event) => event?.kind === 'rule_synthesis_candidate')) {
    return native.map((event) => ({ kind: String(event.kind), payload: String(event.payload ?? '') }));
  }
  return solverEventsFromThinkingSteps(Array.isArray(answer?.worker_steps) ? answer.worker_steps : answer?.thinking_steps);
}

/** Mirrors rust/src/engine.rs `answer_links_notation`'s `steps` value. */
export function answerStepsLinks(events) {
  return events.map((event, index) => `step_${index} ${event.kind} ${flattenLinoValue(event.payload)}`).join('; ');
}

/** Mirrors rust/src/self_improvement.rs `event_payload`. */
export function eventPayload(links, kind) {
  const marker = ` ${kind} `;
  const found = links.indexOf(marker);
  if (found < 0) return null;
  const tail = links.slice(found + marker.length);
  const ends = ['; step_', "'\n", '"\n'].map((needle) => tail.indexOf(needle));
  const end = ends.find((at) => at >= 0) ?? tail.length;
  const flattened = tail.slice(0, end).trim();
  if (kind === 'selected_rule') return flattened;
  const normalized = flattened.split('\\\\n').join('\n').split('\\n').join('\n');
  const fields = kind === 'rule_synthesis_candidate' ? CANDIDATE_FIELDS : VERIFICATION_FIELDS;
  let payload = kind;
  fields.forEach((field, index) => {
    const needle = ` ${field} `;
    const at = normalized.indexOf(needle);
    if (at < 0) return;
    const fieldTail = normalized.slice(at + needle.length);
    const nexts = fields.slice(index + 1).map((next) => fieldTail.indexOf(` ${next} `)).filter((position) => position >= 0);
    const value = fieldTail.slice(0, nexts.length ? Math.min(...nexts) : fieldTail.length).trim();
    if (value) payload += `\n  ${field} ${value}`;
  });
  return payload;
}

/**
 * Mirrors rust/src/self_improvement.rs `learning_trace_from_symbolic_answer`,
 * reading the solver events the worker reports as thinking steps.
 * @returns {object|null} a `json!` value (keys sorted)
 */
export function learningTraceFromSymbolicAnswer(prompt, answer) {
  const links = answerStepsLinks(answerSolverEvents(answer));
  const candidate = eventPayload(links, 'rule_synthesis_candidate');
  if (candidate === null) return null;
  const verification = eventPayload(links, 'rule_verification');
  if (verification === null) return null;
  const selected = eventPayload(links, 'selected_rule');
  if (selected === null) return null;
  return {
    events: [
      { kind: 'selected_rule', payload: selected },
      { kind: 'rule_synthesis_candidate', payload: candidate },
      { kind: 'rule_verification', payload: verification },
    ],
    prompt,
  };
}
