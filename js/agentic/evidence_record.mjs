// Record an investigation's findings in the file a request names
// (rust/src/agentic_coding/evidence_record.rs).
//
// `planEvidenceRecordStep` is async: it re-plans the residual request through
// the lead planner (`planChatStep`, `planSettledRoutes`) and may consult the
// host solver. Importing ./planner.mjs here is a module cycle; it is safe
// because nothing is called at module evaluation time.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText } from './content.mjs';
import { composeGeneralChangePlan } from './general_planner.mjs';
import { agenticMessage } from './messages.mjs';
import { planOne, writeArguments } from './plan.mjs';
import { instructionEnd } from './positional_edit.mjs';
import { planChatStep, planSettledRoutes } from './planner.mjs';
import { isContinuationCue, traceRoute } from './planner/continuation.mjs';
import { Progress } from './progress.mjs';
import { carriesAuthoringTask } from './shell_command.mjs';
import { sentences } from './shell_command_policy.mjs';
import { observedFileOutcome } from './task_obligations.mjs';
import { commandArgument } from './tool_result.mjs';
import { isVerificationFailureAnswer } from './workspace_change.mjs';
import {
  serializedRelationshipTerm, workspaceInspectionSearchForTask, workspaceInspectionTermsForTask,
} from './workspace_inspection.mjs';
import {
  bareSurfaces, deliveredWriteTarget, firstActionCueStart, pinnedFirstLine, tokens,
} from './write_request.mjs';
import { solve } from './host.mjs';
import { announcesAListItDoesNotMake, defersToTheOpenWeb, isInconclusive } from './crate/engine_answer.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { detect } from './crate/language.mjs';
import { renderResponse } from './crate/seed.mjs';
import { mentionsRole } from './write_lexicon.mjs';
import {
  isAlphanumeric, isAsciiDigit, isWhitespace, lines, splitWhitespace, trim, trimEnd, trimEndMatches,
} from './write_str.mjs';

const DELIVERY_PROBE_TURNS = 4;
const DELIVERY_PROBE_MEMO_CAPACITY = 512;
const OUTPUT_OPTION = '--output';
const deliveryProbeAnswers = new Map();

/** Mirrors `fn parse_obligation`: a `DeliveryBinding` or null. */
/**
 * Mirrors `fn masked_multi_word_quotes`: `text` with every quoted segment that
 * holds whitespace replaced, delimiters included, by as many `x` as it is long,
 * so spans still index the original. A single-token quote — a backticked path
 * or name — stays readable. The lines under a colon-ended first line are the
 * request's payload, whatever they say, so they are masked the same way, line
 * breaks kept (PR #1188 G95: a row that read "append ... to gaps.md" was taken
 * for a delivery, and the answer was written over that file).
 */
function maskedMultiWordQuotes(text) {
  let masked = text;
  for (const segment of quotedSegmentSpans(text)) {
    if (!/\s/u.test(segment.text)) continue;
    masked = masked.slice(0, segment.start) + 'x'.repeat(segment.end - segment.start) + masked.slice(segment.end);
  }
  const end = instructionEnd(text);
  return masked.slice(0, end) + masked.slice(end).replace(/\S/g, 'x');
}

function parseObligation(request) {
  let target = null;
  let firstLine = null;
  let fieldLines = [];
  let residual = '';
  let laterObligation = false;
  // Sentences and delivery cues are read with every multi-word quoted
  // payload masked: `Append "/// Append an empty line to notes.txt." to
  // src/lib.rs.` is one sentence about src/lib.rs, not a delivery to the
  // notes.txt its payload mentions (PR #1188 sub-agent gap 2). The pinned
  // line, field lines and the work before the delivery are read from the
  // request itself, where the quoted text is intact.
  const masked = maskedMultiWordQuotes(request);
  for (const sentence of sentences(masked)) {
    const spanText = request.slice(sentence.span.start, sentence.span.end);
    const at = masked.indexOf(sentence.text, sentence.span.start);
    const text = at < 0 ? sentence.text : request.slice(at, at + sentence.text.length);
    const line = pinnedFirstLine(text);
    if (line !== null && line !== undefined) {
      if (target !== null && !laterObligation) {
        if (firstLine === null) firstLine = line;
        continue;
      }
      residual += spanText;
      continue;
    }
    const delivered = carriesAuthoringTask(normalizePrompt(sentence.text)) ? null : deliveredWriteTarget(sentence.text);
    if (delivered !== null && delivered !== undefined) {
      if (target === null) {
        fieldLines = exactFieldLines(text, delivered);
        target = delivered;
        const work = workBeforeDelivery(text);
        if (work !== null) residual += `${work}. `;
        continue;
      }
      laterObligation = true;
    }
    residual += spanText;
  }
  const trimmed = trim(residual);
  if (trimmed === '' || target === null) return null;
  return { target, first_line: firstLine, field_lines: fieldLines, residual: trimmed };
}

function exactFieldLines(sentence, target) {
  const fields = sentence.split('`')
    .filter((_, index) => index % 2 === 1)
    .map(trim)
    .filter((quoted) => quoted !== target && !Array.from(quoted).some(isWhitespace))
    .filter((quoted) => {
      const equals = quoted.indexOf('=');
      return equals >= 0 && /^[A-Za-z][A-Za-z0-9_]*$/.test(quoted.slice(0, equals));
    });
  return fields.filter((field, index) => !fields.slice(index + 1).includes(field));
}

function workBeforeDelivery(sentence) {
  const start = firstActionCueStart(tokens(sentence));
  if (start === null || start === undefined) return null;
  const work = withoutTrailingSeparator(trim(sentence.slice(0, start)));
  return Array.from(work).some(isAlphanumeric) ? work : null;
}

function withoutTrailingSeparator(work) {
  const lowered = work.toLowerCase();
  let best = null;
  for (const separator of bareSurfaces('skill_procedure_clause_separator')) {
    if (!lowered.endsWith(separator)) continue;
    const kept = work.slice(0, work.length - separator.length);
    const last = Array.from(kept).pop();
    if (last === undefined || !isWhitespace(last)) continue;
    const trimmedKept = trimEnd(kept);
    if (best === null || trimmedKept.length < best.length) best = trimmedKept;
  }
  return best ?? work;
}

/**
 * Mirrors `fn plan_evidence_record_step`.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export async function planEvidenceRecordStep(task, messages, toolNames) {
  const obligation = parseObligation(task);
  if (!obligation) return null;
  const reason = await laterRouteDelivering(task, obligation, toolNames);
  if (reason !== null) {
    traceRoute('evidence_record', reason);
    return null;
  }
  const writeTool = toolFor(toolNames, Capability.Write);
  if (!writeTool) return null;
  const progress = Progress.scan(messages);
  const outcome = observedFileOutcome(task, obligation.target, messages);
  if (outcome.kind === 'satisfied') {
    traceRoute('evidence_record', 'already_written');
    const content = progress.successfulWriteContentFor(obligation.target);
    const written = content === null || content === undefined ? '' : writtenObservation(obligation, content);
    return { kind: 'final', answer: written !== '' ? written : agenticMessage('evidence_record_recorded', { target: obligation.target }) };
  }
  if (outcome.kind === 'refuted' || outcome.kind === 'unsatisfiable') {
    traceRoute('evidence_record', 'write_not_observed');
    return { kind: 'final', answer: agenticMessage('evidence_record_write_failed', { target: obligation.target }) };
  }
  let observed = null;
  if (parseObligation(obligation.residual) === null && workspaceInspectionSearchForTask(obligation.residual) !== null) {
    const latest = progress.latestSuccessfulOutput(Capability.Grep);
    if (latest !== null && latest !== undefined && substantiveResultLine(latest, obligation.residual) !== '') observed = latest;
  }
  const residualMessages = withResidualRequest(messages, obligation.residual);
  if (residualMessages === null) return null;
  const isWorkspaceObservation = observed !== null;
  let answer = observed;
  if (answer === null) {
    const plan = await planChatStep(residualMessages, toolNames);
    if (plan && plan.kind === 'tool_calls') {
      traceRoute('evidence_record', 'investigating');
      return plan;
    }
    if (plan && plan.kind === 'final') {
      if (isVerificationFailureAnswer(obligation.residual, plan.answer)) {
        traceRoute('evidence_record', 'residual_reported_failure');
        return plan;
      }
      // A failed step's output is never a file's content: the answer that
      // reports it is the answer, and nothing is written (PR #1188 G102).
      if (progress.latestFailure() !== null) {
        traceRoute('evidence_record', 'residual_step_failed');
        return plan;
      }
      answer = plan.answer;
    } else {
      traceRoute('evidence_record', 'symbolic_residual');
      answer = await symbolicAnswer(obligation.residual);
      if (answer === null) return null;
    }
  }
  traceRoute('evidence_record', obligation.target);
  const deliveredAnswer = isWorkspaceObservation ? substantiveResultLine(answer, obligation.residual) : answer;
  return planOne(writeTool, writeArguments(obligation.target, renderObligation(obligation, deliveredAnswer)));
}

async function laterRouteDelivering(task, obligation, toolNames) {
  if (obligation.first_line !== null || obligation.field_lines.length) return null;
  const plan = composeGeneralChangePlan(task);
  if (plan !== null && plan.target === obligation.target) return 'declined_composed_target';
  return (await settledRouteDelivers(task, obligation.target, toolNames)) ? 'declined_settled_route' : null;
}

async function settledRouteDelivers(task, target, toolNames) {
  const key = JSON.stringify([task, target, toolNames.join('\u001f')]);
  if (deliveryProbeAnswers.has(key)) return deliveryProbeAnswers.get(key);
  const answer = await probeSettledRoutes(task, target, toolNames);
  if (deliveryProbeAnswers.size >= DELIVERY_PROBE_MEMO_CAPACITY) deliveryProbeAnswers.clear();
  deliveryProbeAnswers.set(key, answer);
  return answer;
}

async function probeSettledRoutes(task, target, toolNames) {
  const probe = [{ role: 'user', content: task }];
  for (let turn = 0; turn < DELIVERY_PROBE_TURNS; turn += 1) {
    const plan = await planSettledRoutes(task, probe, toolNames);
    if (!plan || plan.kind !== 'tool_calls') return false;
    for (const [index, call] of plan.calls.entries()) {
      if (plannedDestination(call.arguments) === target) return true;
      const id = `delivery-probe-${turn}-${index}`;
      probe.push({
        role: 'assistant',
        content: '',
        tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }],
      });
      probe.push({ role: 'tool', tool_call_id: id, name: call.tool, content: 'ok' });
    }
  }
  return false;
}

function plannedDestination(args) {
  return plannedWritePath(args) ?? plannedCommandOutputPath(args);
}

function plannedCommandOutputPath(args) {
  const command = commandArgument(args);
  if (command === null || command === undefined) return null;
  const parts = splitWhitespace(command);
  const index = parts.indexOf(OUTPUT_OPTION);
  return index < 0 || parts[index + 1] === undefined ? null : parts[index + 1];
}

function plannedWritePath(args) {
  let value;
  try {
    value = JSON.parse(args);
  } catch {
    return null;
  }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  const key = ['path', 'filePath', 'file_path', 'absolute_path'].find((candidate) => typeof value[candidate] === 'string');
  return key === undefined ? null : value[key];
}

function renderObligation(obligation, answer) {
  if (obligation.field_lines.length) {
    const result = substantiveResultLine(answer, obligation.residual);
    const observedResults = exactFieldValues(answer, 'result');
    let next = 0;
    let rendered = '';
    for (const field of obligation.field_lines) {
      rendered += field;
      if (field.endsWith('=')) {
        if (field === 'result=') {
          rendered += nonHollowResult(combinedObservation(observedResults, obligation.residual) ?? result, obligation.residual);
        } else {
          rendered += next < observedResults.length ? observedResults[next++] : result;
        }
      }
      rendered += '\n';
    }
    return rendered;
  }
  const text = trimEnd(nonHollowResult(answer, obligation.residual));
  return obligation.first_line === null ? `${text}\n` : `${obligation.first_line}\n\n${text}\n`;
}

function exactFieldValues(answer, key) {
  const prefix = `${key}=`;
  return lines(answer).map(trim).filter((line) => line.startsWith(prefix)).map((line) => line.slice(prefix.length));
}

function combinedObservation(values, request) {
  if (!values.length) return null;
  if (values.length === 1) return values[0];
  const joined = values.join('; ');
  return renderResponse('evidence_exact_composition', detect(request), [['values', joined]]) ?? joined;
}

function nonHollowResult(raw, task) {
  const result = trimEnd(raw);
  const language = detect(task);
  let intent;
  if (result.endsWith(':') || result.endsWith('：')) intent = 'coding_repository_source_observation';
  else if (splitWhitespace(result).length >= 4) return result;
  else intent = 'coding_repository_result_observation';
  return renderResponse(intent, language, [['result', result]]) ?? result;
}

function writtenObservation(obligation, content) {
  const all = lines(content);
  if (obligation.first_line !== null && all.length && trim(all[0]) === obligation.first_line) all.shift();
  return trim(all.join('\n'));
}

function compareTuples(left, right) {
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) return left[index] - right[index];
  }
  return 0;
}

function substantiveResultLine(answer, task) {
  let currentSourceAuthority = 0;
  const candidates = [];
  const completedSuffix = agenticMessage('evidence_record_command_output_suffix');
  for (const rawLine of lines(answer)) {
    const line = trim(rawLine);
    if (looksLikeResultPathHeading(line)) {
      currentSourceAuthority = sourceAuthority(line);
      continue;
    }
    if (line !== '' && line !== '```text' && line !== '```' && !line.endsWith(completedSuffix) && !isMatchCount(line)) {
      candidates.push([line, Math.max(currentSourceAuthority, sourceAuthority(line))]);
    }
  }
  const terms = workspaceInspectionTermsForTask(task);
  const conditionRequested = mentionsRole('coding_condition_subject_kind', task);
  const implementationRequested = mentionsRole('coding_source_implementation_subject_kind', task);
  const relationship = serializedRelationshipTerm(task) ?? null;
  traceRoute('evidence_record_relationship', relationship ?? 'none');
  traceRoute('evidence_record_terms', terms.join(','));
  if (!candidates.length) return '';
  const score = ([line, authority]) => relevanceScore(line, terms, conditionRequested, implementationRequested, relationship, authority);
  let [selected] = candidates[0];
  let selectedScore = score(candidates[0]);
  for (const candidate of candidates.slice(1)) {
    const candidateScore = score(candidate);
    if (compareTuples(candidateScore, selectedScore) > 0) {
      [selected] = candidate;
      selectedScore = candidateScore;
    }
  }
  traceRoute('evidence_record_selected', selected);
  return selected;
}

function relevanceScore(line, terms, conditionRequested, implementationRequested, relationship, authority) {
  const words = line.split(/[^A-Za-z0-9]/u).filter((word) => word !== '').map((word) => word.toLowerCase());
  const matches = (term) => words.includes(term) || term.split('_').every((part) => words.includes(part));
  const overlap = terms.filter(matches).length;
  const requestedProperty = terms.length && matches(terms[terms.length - 1]) ? 1 : 0;
  const declarationShape = looksLikeDeclaration(line)
    ? 1 + (looksLikeLocalBinding(line) ? 0 : 1) + (looksLikeExposedDeclaration(line) ? 1 : 0)
    : 0;
  const boundRequested = terms.some((term) => mentionsRole('coding_bound_cue', term));
  const semanticOverlap = boundRequested && words.some((word) => mentionsRole('coding_bound_cue', word)) ? 1 : 0;
  const codeShape = /[{}();=<>]/u.test(line) ? 1 : 0;
  const directIdentity = relationship !== null && words.some((word, index) => word === relationship && words[index + 1] === 'id') ? 1 : 0;
  return [
    implementationRequested && !isQuotedOrCommentedSource(sourceText(line)) ? 1 : 0,
    directIdentity,
    requestedProperty,
    conditionRequested && looksLikeCondition(line) ? 1 : 0,
    authority,
    conditionRequested && looksLikeInstanceCondition(line) ? 1 : 0,
    declarationShape,
    semanticOverlap,
    overlap,
    codeShape,
  ];
}

function looksLikeCondition(line) {
  const source = sourceText(line);
  if (isQuotedOrCommentedSource(source)) return false;
  return ['if ', 'while ', 'match ', 'when '].some((prefix) => source.startsWith(prefix))
    || ['&&', '||', '==', '!=', '>=', '<='].some((operator) => source.includes(operator))
    || source.startsWith('!');
}

function looksLikeInstanceCondition(line) {
  const source = sourceText(line);
  return ['self.', 'this.', 'self->', 'this->', '$this->'].some((receiver) => source.includes(receiver));
}

function looksLikeLocalBinding(line) {
  const source = sourceText(line);
  return ['let ', 'var ', 'auto ', 'local '].some((prefix) => source.startsWith(prefix));
}

function looksLikeExposedDeclaration(line) {
  const keyword = splitWhitespace(sourceText(line))[0];
  return keyword !== undefined && (['pub', 'public', 'export', 'exported'].includes(keyword) || keyword.startsWith('pub('));
}

function looksLikeDeclaration(line) {
  const source = sourceText(line);
  if (isQuotedOrCommentedSource(source)) return false;
  const colon = source.indexOf(':');
  if (colon < 0) return false;
  const left = source.slice(0, colon);
  const right = source.slice(colon + 1);
  const name = splitWhitespace(left).pop();
  return !right.startsWith(':') && !/[()=+]/u.test(left) && name !== undefined && /^[A-Za-z0-9_]+$/.test(name);
}

function sourceText(line) {
  const split = line.indexOf(': ');
  return trim(split < 0 ? line : line.slice(split + 2));
}

function isQuotedOrCommentedSource(source) {
  return source.startsWith('//') || source.startsWith('/*') || source.startsWith('*') || source.startsWith('#')
    || source.startsWith('<!--') || ['"', "'", '`'].includes(Array.from(source)[0]);
}

function isMatchCount(line) {
  if (!line.startsWith('Found ') || !line.endsWith(' matches') || line.length < 'Found  matches'.length) return false;
  return Array.from(line.slice('Found '.length, line.length - ' matches'.length)).every(isAsciiDigit);
}

function looksLikeResultPathHeading(line) {
  return line.endsWith(':') && (line.startsWith('/') || line.startsWith('./'));
}

function sourceAuthority(pathOrLine) {
  return trimEndMatches(pathOrLine, (character) => character === ':').split(/[/\\]/u).includes('src') ? 1 : 0;
}

/**
 * Mirrors `fn symbolic_answer` through the host solver (async).
 * @param {string} residual
 */
/**
 * The suffix of an intent that reports a gap -- `capability_gap`,
 * `write_program_skill_gap`: a refusal, never content to record (PR #1188
 * G102: a request describing a test file wrote the refusal into it).
 */
const GAP_INTENT_SUFFIX = '_gap';

async function symbolicAnswer(residual) {
  const answer = await solve(residual);
  if (!answer || answer.intent.endsWith(GAP_INTENT_SUFFIX) || isInconclusive(answer) || defersToTheOpenWeb(answer)
    || announcesAListItDoesNotMake(answer)) return null;
  const text = trim(answer.answer ?? '');
  return text === '' ? null : text;
}

function withResidualRequest(messages, residual) {
  let index = -1;
  for (let at = messages.length - 1; at >= 0; at -= 1) {
    if (messages[at].role === 'user' && !isContinuationCue(plainText(messages[at].content))) {
      index = at;
      break;
    }
  }
  if (index < 0) return null;
  const out = messages.slice(0, index + 1).map((message) => ({ ...message }));
  out[index].content = residual;
  for (const message of messages.slice(index + 1)) {
    if (message.role.toLowerCase() !== 'user') out.push({ ...message });
  }
  return out;
}
