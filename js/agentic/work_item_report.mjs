// A work-item terminal record is rendered as a localized answer: what
// stopped the session, which files it read, what it needs and which
// requirements remain unverified. The record itself is kept on the final
// result (`result.record`) for the trace and never becomes the answer
// (issue #1560).
// Native twin: rust/src/agentic_coding/work_item_report.rs.
import { localizedResponse } from './crate/seed.mjs';
import { workItemLanguage } from './general_planner.mjs';
import { resolvedFinalAnswer, observedFinalResult, FinalDisposition } from './final_result.mjs';
import { trim } from './write_str.mjs';
import { requirementNaming } from './work_item_requirements.mjs';

const LISTED_REQUIREMENTS = 8;
const REQUIREMENT_CHARS = 160;

function filled(intent, language, values) {
  let text = localizedResponse(intent, language) ?? '';
  for (const [key, value] of values) text = text.split('{' + key + '}').join(value);
  return text;
}

/** Mirrors `fn requirement_line`: the lines joined into one, at most 160 characters. */
function requirementLine(text) {
  const line = String(text).split('\n').map(trim).filter(part => part !== '').join(' ');
  const characters = Array.from(line);
  return characters.length > REQUIREMENT_CHARS ? characters.slice(0, REQUIREMENT_CHARS).join('') + '…' : line;
}

/** Mirrors `fn refusal`: the path a refusal concerns and the requirement naming it. */
export function refusal(requirements, path) {
  return { path, requirement: requirementNaming(requirements, path) };
}

/**
 * Mirrors `fn headline_intent`: an independently accepted change, a verified
 * no-op, either of them partial while requirements stay unverified, an
 * unaccepted change, or no change (issue #1483).
 */
function headlineIntent(record) {
  // A verified change that leaves requirements unverified is partial, never done.
  if (record.verified === true && record.complete !== true) return record.authored === true ? 'work-item-report-partial' : 'work-item-report-partial-no-change-needed';
  if (record.verified === true) return record.authored === true ? 'work-item-report-accepted' : 'work-item-report-no-change-needed';
  return record.authored === true ? 'work-item-report-changed' : 'work-item-report-unchanged';
}

/** Mirrors `fn changed_paths`: the paths of the record's effects, each once. */
function changedPaths(record) {
  const paths = [];
  for (const effect of Array.isArray(record.effects) ? record.effects : []) {
    if (typeof effect?.path === 'string' && !paths.includes(effect.path)) paths.push(effect.path);
  }
  return paths;
}

/** Mirrors `fn check_intent`: a check passed, failed, or was not run. */
function checkIntent(check) {
  if (check.passed === true || check.status === 'passed') return 'work-item-report-check-passed';
  if (check.passed === false || check.status === 'failed') return 'work-item-report-check-failed';
  return 'work-item-report-check-pending';
}

/** Mirrors `fn work_item_report`. */
export function workItemReport(record, task) {
  const language = workItemLanguage(task);
  const subject = typeof record.workItem === 'string' && record.workItem !== ''
    ? record.workItem : filled('work-item-report-request', language, []);
  const reason = typeof record.reason === 'string' ? record.reason : 'Unknown';
  const paragraphs = [filled(headlineIntent(record), language, [['subject', subject], ['reason', reason]])];
  const read = [];
  for (const observation of Array.isArray(record.observations) ? record.observations : []) {
    if (observation?.complete === true && typeof observation.path === 'string' && !read.includes(observation.path)) read.push(observation.path);
  }
  paragraphs.push(read.length
    ? filled('work-item-report-read', language, [['paths', read.map(path => '`' + path + '`').join(', ')]])
    : filled('work-item-report-read-none', language, []));
  const changed = record.authored === true ? changedPaths(record) : [];
  if (changed.length) paragraphs.push(filled('work-item-report-effects', language, [['paths', changed.map(path => '`' + path + '`').join(', ')]]));
  const checks = (Array.isArray(record.checks) ? record.checks : []).filter(check => typeof check?.kind === 'string');
  if (checks.length) {
    paragraphs.push(filled('work-item-report-checks', language,
      [['checks', checks.map(check => filled(checkIntent(check), language, [['kind', check.kind]])).join('; ')]]));
  }
  const unresolved = (Array.isArray(record.unresolved) ? record.unresolved : []).filter(row => typeof row?.reference === 'string');
  if (unresolved.length) {
    const lines = [filled('work-item-report-unresolved', language, [['count', String(unresolved.length)]])];
    for (const row of unresolved.slice(0, LISTED_REQUIREMENTS)) {
      lines.push('- `' + requirementLine(row.reference) + '`: `' + (typeof row.reason === 'string' ? row.reason : 'Unknown') + '`');
    }
    if (unresolved.length > LISTED_REQUIREMENTS) {
      lines.push(filled('work-item-report-more', language, [['count', String(unresolved.length - LISTED_REQUIREMENTS)]]));
    }
    paragraphs.push(lines.join('\n'));
  }
  // A named gap is the narrowest reason no change was made (issue #1483).
  const gap = typeof record.gap === 'string' ? record.gap : null;
  if (gap !== null) paragraphs.push(filled('work-item-report-gap', language, [['gap', gap]]));
  // A partial result, or a named gap, names the work it leaves open; requirement rows are listed below.
  const remaining = ((record.verified === true && record.complete !== true) || gap !== null) && Array.isArray(record.remainingWork)
    ? record.remainingWork.filter(item => typeof item === 'string' && item !== 'independent-requirement-acceptance') : [];
  if (remaining.length) {
    const lines = [filled('work-item-report-remaining', language, [['count', String(remaining.length)]])];
    for (const item of remaining.slice(0, LISTED_REQUIREMENTS)) lines.push('- `' + requirementLine(item) + '`');
    if (remaining.length > LISTED_REQUIREMENTS) {
      lines.push(filled('work-item-report-more', language, [['count', String(remaining.length - LISTED_REQUIREMENTS)]]));
    }
    paragraphs.push(lines.join('\n'));
  }
  // A file the solver selected from a citation alone was not written (issue #1562).
  if (typeof record.unboundWrite?.path === 'string') {
    paragraphs.push(filled('work-item-report-unbound-write', language, [['path', record.unboundWrite.path]]));
  }
  const requirements = Array.isArray(record.requirements) ? record.requirements : [];
  if (typeof record.refused?.path === 'string') {
    let refused = filled('work-item-report-refused', language, [['path', record.refused.path]]);
    const index = record.refused.requirement;
    if (Number.isInteger(index) && index >= 0 && index < requirements.length) {
      refused += ' ' + filled('work-item-report-concerns', language,
        [['number', String(index + 1)], ['requirement', requirementLine(requirements[index].text)]]);
    }
    paragraphs.push(refused);
  } else if (reason === 'MissingReadTool') {
    paragraphs.push(filled('work-item-report-missing-read-tool', language, []));
  }
  // Requirements keep their numbers; only those not yet verified are listed.
  // A clause that only cites evidence is no obligation (issue #1562).
  const obliged = requirements.filter(requirement => requirement?.status !== 'citation').length;
  const open = requirements.map((requirement, index) => ({ requirement, index })).filter(row => row.requirement?.status !== 'verified' && row.requirement?.status !== 'citation');
  if (obliged && !open.length) {
    paragraphs.push(filled('work-item-report-requirements-verified', language, [['count', String(obliged)]]));
  } else if (open.length) {
    const lines = [filled('work-item-report-requirements', language, [['count', String(open.length)]])];
    for (const { requirement, index } of open.slice(0, LISTED_REQUIREMENTS)) {
      lines.push(String(index + 1) + '. ' + requirementLine(requirement?.text ?? ''));
    }
    if (open.length > LISTED_REQUIREMENTS) {
      lines.push(filled('work-item-report-more', language, [['count', String(open.length - LISTED_REQUIREMENTS)]]));
    }
    paragraphs.push(lines.join('\n'));
  }
  const schema = typeof record.schema === 'string' ? record.schema : 'work-item-record';
  paragraphs.push(filled('work-item-report-trace', language, [['schema', schema]]));
  return paragraphs.join('\n\n');
}

/**
 * Mirrors `fn work_item_record_final`: the localized report is the answer and
 * the record stays on the final result for the trace.
 */
export function workItemRecordFinal(record, task, disposition, origin) {
  const final = resolvedFinalAnswer(workItemReport(record, task), disposition, origin);
  final.result.record = record;
  return final;
}

/**
 * Mirrors `reported_work_item_record`: the work-item record behind a final
 * plan, or `null` (trace and tests).
 */
export function workItemRecordOf(plan) {
  return observedFinalResult(plan)?.record ?? null;
}

/**
 * Mirrors `fn reported_work_item_final`: a work-item session never answers
 * with a JSON object. A final whose answer is one is rendered as a report and
 * its object kept as the record; any other plan is returned unchanged.
 */
export function reportedWorkItemFinal(plan, task) {
  if (plan?.kind !== 'final' || workItemRecordOf(plan) !== null || !parsesAsJsonObject(plan.answer)) return plan;
  const result = observedFinalResult(plan);
  const disposition = result?.disposition === FinalDisposition.Failure ? FinalDisposition.Failure : FinalDisposition.Gap;
  return workItemRecordFinal(JSON.parse(plan.answer), task, disposition, result?.origin ?? 'work_item_record');
}

/** Mirrors `fn parses_as_json_object`. */
export function parsesAsJsonObject(text) {
  try {
    const value = JSON.parse(text);
    return value !== null && typeof value === 'object' && !Array.isArray(value);
  } catch {
    return false;
  }
}
