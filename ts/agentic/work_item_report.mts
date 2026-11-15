// A work-item terminal record is rendered as a localized answer: what
// stopped the session, which files it read, what it needs and which
// requirements remain unverified. The record itself is kept on the final
// result (`result.record`) for the trace and never becomes the answer
// (issue #1560).
// Native twin: rust/src/agentic_coding/work_item_report.rs.
import { localizedResponse } from './crate/seed.mjs';
import { detect } from './crate/language.mjs';
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

/** Mirrors `fn requirement_line`: the first line, at most 160 characters. */
function requirementLine(text) {
  const first = trim(String(text).split('\n')[0]);
  const characters = Array.from(first);
  return characters.length > REQUIREMENT_CHARS ? characters.slice(0, REQUIREMENT_CHARS).join('') + '…' : first;
}

/** Mirrors `fn refusal`: the path a refusal concerns and the requirement naming it. */
export function refusal(requirements, path) {
  return { path, requirement: requirementNaming(requirements, path) };
}

/** Mirrors `fn work_item_report`. */
export function workItemReport(record, task) {
  const language = detect(task);
  const subject = typeof record.workItem === 'string' && record.workItem !== ''
    ? record.workItem : filled('work_item_report_request', language, []);
  const reason = typeof record.reason === 'string' ? record.reason : 'Unknown';
  const paragraphs = [filled(record.authored === true ? 'work_item_report_changed' : 'work_item_report_unchanged',
    language, [['subject', subject], ['reason', reason]])];
  const read = [];
  for (const observation of Array.isArray(record.observations) ? record.observations : []) {
    if (observation?.complete === true && typeof observation.path === 'string' && !read.includes(observation.path)) read.push(observation.path);
  }
  paragraphs.push(read.length
    ? filled('work_item_report_read', language, [['paths', read.map(path => '`' + path + '`').join(', ')]])
    : filled('work_item_report_read_none', language, []));
  const requirements = Array.isArray(record.requirements) ? record.requirements : [];
  if (typeof record.refused?.path === 'string') {
    let refused = filled('work_item_report_refused', language, [['path', record.refused.path]]);
    const index = record.refused.requirement;
    if (Number.isInteger(index) && index >= 0 && index < requirements.length) {
      refused += ' ' + filled('work_item_report_concerns', language,
        [['number', String(index + 1)], ['requirement', requirementLine(requirements[index].text)]]);
    }
    paragraphs.push(refused);
  } else if (reason === 'MissingReadTool') {
    paragraphs.push(filled('work_item_report_missing_read_tool', language, []));
  }
  if (requirements.length) {
    const lines = [filled('work_item_report_requirements', language, [['count', String(requirements.length)]])];
    requirements.slice(0, LISTED_REQUIREMENTS).forEach((requirement, index) => {
      lines.push(String(index + 1) + '. ' + requirementLine(requirement?.text ?? ''));
    });
    if (requirements.length > LISTED_REQUIREMENTS) {
      lines.push(filled('work_item_report_more', language, [['count', String(requirements.length - LISTED_REQUIREMENTS)]]));
    }
    paragraphs.push(lines.join('\n'));
  }
  const schema = typeof record.schema === 'string' ? record.schema : 'work-item-record';
  paragraphs.push(filled('work_item_report_trace', language, [['schema', schema]]));
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

/** The work-item record behind a final plan, or `null` (trace and tests). */
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
