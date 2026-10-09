// Deterministic fallback planner for repository change requests
// (rust/src/agentic_coding/general_planner.rs, issue #654).
//
// A `GeneralChangePlan` is `{id, mode, goal, target, content, steps,
// verification_command, terminal_state}`; `mode` is 'literal_file' |
// 'command_output' | 'repository_work_item' (the Rust `GeneralPlanMode::slug`)
// and `terminal_state` is 'executed' | 'planned_not_executed'. A step is
// `{capability, action, expected_evidence, command}`.

import { describesCodeToAuthor as contentRequiresAuthoring, semanticAuthoringLead } from './crate/literal_authoring_contract.mjs';
import { endOfStatement } from './crate/literal_content.mjs';
export { semanticAuthoringLead };
import { Capability } from './capability.mjs';
import { agenticMessage } from './messages.mjs';
import { composedDocumentSpecificationSpan } from './note_composition.mjs';
import { proseSentences } from './shell_command_policy.mjs';
import { traceRoute } from './planner/continuation.mjs';
import {
  CueFamily, actionCueStartAfter, bareSurfaces, cleanContent, cleanCueToken, cleanPathToken,
  rawContentLeadClose, firstActionCueEnd, firstActionCueStart, firstContentLeadEnd, firstPrefixLeadEnd,
  firstRawContentLeadEnd, firstRawPrefixLeadEnd,
  honouringPinnedFirstLine, looksLikeFilePath, payloadContinuesPastItsFirstLine, rankedBindings,
  safeRelativePath, spanOf, tokens,
} from './write_request.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { formalizeIntent } from './crate/intent_formalization.mjs';
import { withoutTrailingKnownModifier } from './crate/implementation_language.mjs';
import { fencedBlock, LINO_FENCE_LANGUAGE } from './crate/issue_report.mjs';
import { detect } from './crate/language.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';
import { resolveReference, workspace } from './crate/self_ast_census.mjs';
import { mentionsRole, roleWordForms } from './write_lexicon.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { quotedSegmentSpans, quotedSegments } from './crate/normal_markov.mjs';
import {
  charIn, isAlphanumeric, isAscii, isAsciiAlphanumeric, isAsciiDigit, isWhitespace, lastChar,
  splitWhitespace, trim, trimEnd, trimEndMatches, trimMatches, trimStart,
} from './write_str.mjs';

export { composeEditRequest, typedWriteTarget } from './write_request.mjs';

/** Mirrors `const PLAN_PATH`. */
export const PLAN_PATH = '.formal-ai/general-change-plan.lino';
const TARGET_PLACEHOLDER = '{target}';

/** `GeneralPlanMode` slugs (`GeneralPlanMode::slug`). */
export const GeneralPlanMode = Object.freeze({
  LiteralFile: 'literal_file',
  CommandOutput: 'command_output',
  RepositoryWorkItem: 'repository_work_item',
});

/** `PlanTerminalState` slugs. */
export const PlanTerminalState = Object.freeze({
  Executed: 'executed',
  PlannedNotExecuted: 'planned_not_executed',
});

const CAPABILITY_SLUGS = Object.freeze({
  search: 'Search', fetch: 'Fetch', read: 'Read', write: 'Write', edit: 'Edit', run: 'Run',
  grep: 'Grep', glob: 'Glob', list_dir: 'ListDir', todo: 'Todo', subagent: 'Subagent',
  read_many: 'ReadMany', multi_edit: 'MultiEdit', ask_user: 'AskUser',
});

/**
 * Mirrors `fn describes_code_to_author`: unquoted content that names a code
 * construct (`a function multiply(a, b)`, seeded `coding_request_object`) is
 * a description of code to write, not the file's bytes — writing it as the
 * whole file destroyed `math.mjs` (PR #1188 dogfooding). Content a seeded
 * content lead introduces (`containing`, `with exactly this content:`) is
 * bytes whatever it mentions.
 */
function describesCodeToAuthor(request, content) {
  return contentRequiresAuthoring(request, content, proseAround(request, content), firstContentLeadEnd(request.toLowerCase()) !== null);
}

/**
 * Mirrors `fn names_an_addition`: `Add <content> to <file>` -- the write verb
 * is the seeded add action and the content comes before the file -- names an
 * addition to that file, never its whole new content. Writing it as the file
 * replaced `m.test.mjs` with the sentence "an assertion that add(2, 2) equals
 * 4" (PR #1188 dogfooding). Content a seeded content lead introduces
 * (`containing`, `with exactly this content:`) is still the file's bytes.
 */
function namesAnAddition(request, content, target) {
  const toks = tokens(request);
  const start = firstActionCueStart(toks);
  const end = firstActionCueEnd(toks);
  if (start === null || end === null || firstContentLeadEnd(request.toLowerCase()) !== null) return false;
  const at = content === '' ? -1 : request.indexOf(content);
  return at >= 0 && at < request.indexOf(target)
    && mentionsRole('coding_member_add_action', normalizePrompt(request.slice(start, end)));
}

/** Mirrors `fn prose_around`: the request without the content and its file paths. */
function proseAround(request, content) {
  return tokens(request.split(content).join(' '))
    .filter((token) => !looksLikeFilePath(cleanPathToken(token.text)))
    .map((token) => token.text).join(' ');
}

/** Mirrors `GeneralChangePlan::links_notation`. */
export function planLinksNotation(plan) {
  let out = 'general_change_plan\n';
  out += field('id', plan.id);
  out += field('execution_mode', plan.mode);
  out += field('terminal_state', plan.terminal_state);
  out += field('goal', plan.goal);
  out += field('target', plan.target);
  plan.steps.forEach((step, index) => {
    out += `  step ${index + 1}\n`;
    out += fieldNested('capability', CAPABILITY_SLUGS[step.capability]);
    out += fieldNested('action', step.action);
    out += fieldNested('expected_evidence', step.expected_evidence);
    if (step.command !== null) out += fieldNested('command', step.command);
  });
  if (plan.verification_command !== '') out += field('verification_command', plan.verification_command);
  return out;
}

/** Mirrors `GeneralChangePlan::planned_not_executed_answer`. */
export function plannedNotExecutedAnswer(plan) {
  const language = detect(plan.goal);
  return (localizedResponse('general_plan_repository_planned', language) ?? '')
    .split(TARGET_PLACEHOLDER).join(plan.target)
    .split('{plan_path}').join(PLAN_PATH)
    .split('{plan}').join(fencedBlock(LINO_FENCE_LANGUAGE, planLinksNotation(plan)));
}

/**
 * Mirrors `fn objective_text`: the request after its first line-anchored
 * objective marker, trimmed; the request itself when it carries none.
 * @param {string} request
 */
export function objectiveText(request) {
  const lead = firstRawPrefixLeadEnd(request, 'request_objective_lead');
  if (!lead || !lineAnchored(request, lead[0])
    || quotedSegmentSpans(request).some((segment) => lead[0] >= segment.start && lead[0] < segment.end)) return request;
  return trim(request.slice(lead[1]));
}

/** Mirrors `fn literal_payload`: a closed literal immediately introduced by a seeded content lead. */
function literalPayload(request) {
  const quoted = quotedSegmentSpans(request);
  const literal = quoted.find((segment) => /^[\s.!?。！？।]*$/u.test(request.slice(segment.end)));
  if (!literal) return null;
  let prefix = request.slice(0, literal.start);
  for (const segment of quoted.filter((segment) => segment.end <= literal.start).reverse()) {
    prefix = prefix.slice(0, segment.start) + ' '.repeat(segment.end - segment.start) + prefix.slice(segment.end);
  }
  const normalized = normalizePrompt(prefix);
  const overwrite = mentionsRole('file_overwrite_consent', normalized);
  const lead = firstRawContentLeadEnd(prefix);
  if (lead === null && !overwrite) return null;
  // An embedded quoted field is not the entire marker-led payload.
  if (lead !== null && !/^[\s:]*$/u.test(prefix.slice(lead[1]))
    && !(overwrite && !prefix.slice(lead[1]).trimEnd().includes('\n'))) return null;
  const words = tokens(prefix);
  const actionStart = firstActionCueStart(words);
  const actionEnd = firstActionCueEnd(words);
  if (actionStart === null || actionEnd === null) return null;
  return overwrite || mentionsRole('file_whole_write_action', normalizePrompt(prefix.slice(actionStart, actionEnd))) ? literal : null;
}

function lineAnchored(text, start) {
  const before = Array.from(text.slice(0, start)).reverse();
  for (const character of before) {
    if (character === '\n') return true;
    if (!isWhitespace(character)) return false;
  }
  return true;
}

/**
 * Mirrors `fn compose_general_change_plan`: a `GeneralChangePlan` or null.
 * @param {string} fullRequest
 */
export function composeGeneralChangePlan(fullRequest) {
  const request = objectiveText(fullRequest);
  // An additive edit (append, prepend) never rewrites the whole file: the
  // workspace-change arm owns it, and a request it cannot ground is declined.
  const literal = literalPayload(request);
  const instruction = literal === null ? request : request.slice(0, literal.start) + request.slice(literal.end);
  const lowered = instruction.toLowerCase();
  if (mentionsRole('file_edit_position_end', lowered) || mentionsRole('file_edit_position_start', lowered)) return null;
  const commandOutput = parseCommandOutputRequest(request);
  const fileRequest = commandOutput ? [commandOutput[0], ''] : parseWriteRequest(request);
  if (!fileRequest) return composeRepositoryWorkPlan(request);
  const target = fileRequest[0];
  let content = literal === null ? withoutTrailingKnownModifier(fileRequest[1]) ?? fileRequest[1] : fileRequest[1];
  const repaired = honouringPinnedFirstLine(instruction, content);
  if (repaired !== null) {
    traceRoute('general_change_plan', 'repaired_pinned_first_line');
    content = repaired;
  }
  if (!safeRelativePath(target)) return null;
  if (!commandOutput && describesCodeToAuthor(request, content)) return null;
  if (!commandOutput && namesAnAddition(request, content, target)) return null;
  const responseLanguage = detect(request);
  const intent = formalizeIntent(request, responseLanguage);
  const verificationCommand = `cat ${target}`;
  const steps = [{
    capability: Capability.Write,
    action: (localizedResponse('general-plan-append-action', 'en') ?? '').replace('{plan_path}', PLAN_PATH),
    expected_evidence: agenticMessage('general_planner_plan_event_evidence', { impulse_id: intent.impulse_id }),
    command: null,
  }];
  if (commandOutput) {
    steps.push({
      capability: Capability.Run,
      action: commandPlanText('general_plan_command_capture_action', responseLanguage, target),
      expected_evidence: commandPlanText('general_plan_command_output_evidence', responseLanguage, target),
      command: `${commandOutput[1]} > ${shellQuote(target)}`,
    });
  } else {
    steps.push({
      capability: Capability.Write,
      action: agenticMessage('general_planner_write_content_action', { target }),
      expected_evidence: agenticMessage('general_planner_workspace_file_evidence', { target }),
      command: null,
    });
  }
  steps.push({
    capability: Capability.Run,
    action: agenticMessage('general_planner_verification_action'),
    expected_evidence: commandOutput
      ? commandPlanText('general_plan_command_verification_evidence', responseLanguage, target)
      : content,
    command: verificationCommand,
  });
  return {
    id: stableId('general_change_plan', `${intent.impulse_id}:${target}:${content}:${commandOutput ? commandOutput[1] : ''}`),
    mode: commandOutput ? GeneralPlanMode.CommandOutput : GeneralPlanMode.LiteralFile,
    goal: intent.source_text,
    target,
    content,
    steps,
    verification_command: verificationCommand,
    terminal_state: PlanTerminalState.Executed,
  };
}

function composeRepositoryWorkPlan(request) {
  const target = repositoryWorkReference(request);
  if (target === null) return null;
  // Mirrors `opens_with_reference`: Hive Mind's objective field (`Issue to
  // solve: <url>`, the prepared branch, "Proceed.") carries its verb in the
  // delimiter `objectiveText` already stripped, so an objective that opens
  // with the work-item reference is that field (issues #1154 and #1155).
  const first = splitWhitespace(request)[0];
  const opensWithReference = first !== undefined && repositoryWorkReference(first) !== null;
  if (!opensWithReference && !mentionsBareRole(request, 'software_authoring_action')) return null;
  const responseLanguage = detect(request);
  const intent = formalizeIntent(request, responseLanguage);
  return {
    id: stableId('repository_work_item_plan', `${intent.impulse_id}:${target}`),
    mode: GeneralPlanMode.RepositoryWorkItem,
    goal: intent.source_text,
    target,
    content: '',
    steps: [
      workItemStep(Capability.Fetch, 'read', responseLanguage, target),
      workItemStep(Capability.Write, 'action', responseLanguage, PLAN_PATH),
    ],
    verification_command: '',
    terminal_state: PlanTerminalState.PlannedNotExecuted,
  };
}

function workItemStep(capability, slug, language, target) {
  const evidence = slug === 'read' ? 'general_plan_repository_read_evidence' : 'general_plan_repository_evidence';
  return {
    capability,
    action: commandPlanText(`general_plan_repository_${slug}`, language, target),
    expected_evidence: commandPlanText(evidence, language, target),
    command: null,
  };
}

const URL_EDGE = charIn('<>()[]{},;."\'。，、；：（）「」«»।');

/** Mirrors `fn repository_work_reference`. @param {string} request @returns {string|null} */
export function repositoryWorkReference(request) {
  for (const token of splitWhitespace(request)) {
    const url = trimMatches(token, URL_EDGE);
    const path = url.startsWith('https://github.com/')
      ? url.slice('https://github.com/'.length)
      : url.startsWith('http://github.com/') ? url.slice('http://github.com/'.length) : null;
    if (path === null) continue;
    const segments = path.split('/');
    if (segments.length === 4 && segments[0] && segments[1] && (segments[2] === 'issues' || segments[2] === 'pull')
      && Array.from(segments[3]).every(isAsciiDigit)) return url;
  }
  return null;
}

function parseCommandOutputRequest(request) {
  const toks = tokens(request);
  const runVerbs = terminalCommandVocabulary().run_verbs;
  const actions = bareSurfaces('file_write_action_cue');
  const targets = bareSurfaces('file_write_target_cue');
  const destinations = bareSurfaces('file_write_destination_cue');
  for (const run of toks.filter((token) => runVerbs.includes(cleanCueToken(token.text)))) {
    const tail = request.slice(run.end);
    const leading = tail.length - trimStart(tail).length;
    const quoted = tail.slice(leading);
    const quote = Array.from(quoted)[0];
    if (quote === undefined) return null;
    if (quote !== "'" && quote !== '"' && quote !== '`') continue;
    const body = quoted.slice(1);
    const close = body.indexOf(quote);
    if (close < 0) continue;
    const command = trim(body.slice(0, close));
    if (!command || /[\n\r\0]/.test(command)) continue;
    const suffix = request.slice(run.end + leading + 1 + close + 1);
    if (!mentionsBareRole(suffix, 'file_write_command_output_reference')) continue;
    const suffixTokens = tokens(suffix);
    if (!suffixTokens.some((token) => actions.includes(cleanCueToken(token.text)))) continue;
    let target = null;
    for (let index = 1; index < suffixTokens.length; index += 1) {
      const cleaned = cleanPathToken(suffixTokens[index].text);
      const cue = cleanCueToken(suffixTokens[index - 1].text);
      if (looksLikeFilePath(cleaned) && safeRelativePath(cleaned) && (targets.includes(cue) || destinations.includes(cue))) {
        target = cleaned;
        break;
      }
    }
    if (target !== null) return [target, command];
  }
  return null;
}

/** Mirrors `fn mentions_bare_role`. */
export function mentionsBareRole(text, role) {
  const lowerText = text.toLowerCase();
  return roleWordForms(role).filter((form) => form.slot === 'bare').some((form) => {
    const needle = form.text.toLowerCase();
    const start = lowerText.indexOf(needle);
    if (start < 0) return false;
    if (!isAscii(needle)) return true;
    const end = start + needle.length;
    const beforeOk = start === 0 || !isAlphanumeric(lastChar(lowerText.slice(0, start)));
    const afterOk = end === lowerText.length || !isAlphanumeric(Array.from(lowerText.slice(end, end + 2))[0]);
    return beforeOk && afterOk;
  });
}

/** Mirrors `fn shell_quote`. @param {string} value */
export function shellQuote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function commandPlanText(intent, language, target) {
  return (localizedResponse(intent, language) ?? intent).split(TARGET_PLACEHOLDER).join(target);
}

/** Mirrors `fn has_file_write_intent`. @param {string} lower */
export function hasFileWriteIntent(lower) {
  return parseWriteRequest(lower) !== null;
}

/** Mirrors `fn has_authoritative_literal_write`. @param {string} request */
export function hasAuthoritativeLiteralWrite(request) {
  return (firstPrefixLeadEnd(request.toLowerCase(), 'file_write_authoritative_content_lead') !== null
    || literalPayload(request) !== null) && composeGeneralChangePlan(request)?.mode === GeneralPlanMode.LiteralFile;
}

function parseWriteRequest(request) {
  const toks = tokens(request);
  const literal = literalPayload(request);
  if (literal !== null) {
    // A closed whole-file payload has already supplied the content. Only
    // target cues before it bind the file; its internal prose is authored data.
    const binding = rankedBindings(toks).find((candidate) =>
      toks[candidate.index].end <= literal.start && candidate.cue_end <= literal.start
      && bindingHasWriteInstruction(request, toks, candidate));
    if (binding) return [binding.path, cleanContent(request.slice(literal.start, literal.end)) ?? literal.text];
  }
  for (const binding of rankedBindings(toks)) {
    const parsed = parseWriteRequestBound(request, toks, binding);
    if (parsed) return parsed;
  }
  return null;
}

/** Mirrors `fn binding_has_write_instruction`: a write cue cannot authorize another statement. */
function bindingHasWriteInstruction(request, toks, binding) {
  const clauseStart = binding.cue_precedes ? binding.cue_start : toks[binding.index].start;
  const scoped = statementScope(request);
  const sentence = proseSentences(scoped).find((item) => {
    const span = spanOf(item);
    return clauseStart >= span.start && clauseStart < span.end;
  });
  if (sentence === undefined) return false;
  const span = spanOf(sentence);
  const target = toks[binding.index];
  if (target.start < span.start || target.end > span.end
    || binding.cue_start < span.start || binding.cue_end > span.end) return false;
  const statement = slice(scoped, span.start, span.end);
  if (statement === null) return false;
  const localAction = firstActionCueStart(tokens(statement));
  const actionStart = localAction === null ? null : span.start + localAction;
  const before = slice(request, span.start, actionStart ?? toks[binding.index].start);
  return before !== null && !mentionsRole('file_read_action_cue', normalizePrompt(before));
}

const slice = (text, start, end) => (start <= end && end <= text.length ? text.slice(start, end) : null);

function parseWriteRequestBound(request, toks, binding) {
  if (!bindingHasWriteInstruction(request, toks, binding)) return null;
  const lowered = request.toLowerCase();
  const destCues = bareSurfaces('file_write_destination_cue');
  const fileIndex = binding.index;
  const target = binding.path;
  const clauseStart = binding.cue_precedes ? binding.cue_start : toks[fileIndex].start;
  const cueIsDestination = binding.family === CueFamily.Destination;
  const specification = composedDocumentSpecificationSpan(request);
  const specificationSpan = specification ? spanOf({ span: specification }) : null;
  const lead = firstRawContentLeadEnd(request);
  if (lead) {
    const markerEnd = lead[1];
    const insideSpecification = specificationSpan && markerEnd >= specificationSpan.start && markerEnd < specificationSpan.end;
    if (!insideSpecification && positionsShareStatement(request, markerEnd, clauseStart)) {
      const markerLeads = markerEnd <= clauseStart;
      const statementEnd = markerLeads
        ? endOfStatement(request, markerEnd, clauseStart)
        : endOfStatement(request, markerEnd, request.length);
      const close = rawContentLeadClose(request, markerEnd);
      const payloadEnd = close === null ? statementEnd : Math.min(close, statementEnd);
      const markerSpan = slice(request, markerEnd, payloadEnd);
      if (!markerLeads || firstActionCueEnd(toks) !== null) {
        const content = markerSpan === null ? null : cleanContent(markerSpan);
        if (content !== null && isLiteralContent(content, markerSpan, mentionsRole('file_write_content_qualifier', normalizePrompt(request.slice(lead[0], markerEnd))))
          && (!namesDeferredWorkProduct(content)
            || firstPrefixLeadEnd(lowered, 'file_write_authoritative_content_lead') !== null
            || literalPayload(request) !== null)) {
          return [target, content];
        }
      }
    }
  }
  let contentSpan;
  if (cueIsDestination && binding.cue_precedes) {
    const actionEnd = firstActionCueEnd(toks);
    if (actionEnd === null) return null;
    if (!(actionEnd <= clauseStart && positionsShareStatement(request, actionEnd, clauseStart))) return null;
    contentSpan = slice(request, actionEnd, clauseStart);
  } else if (cueIsDestination) {
    const actionStart = actionCueStartAfter(toks, binding.cue_end);
    if (actionStart === null) return null;
    if (!(binding.cue_end <= actionStart && positionsShareStatement(request, binding.cue_end, actionStart))) return null;
    contentSpan = slice(request, binding.cue_end, actionStart);
  } else {
    const valueLead = toks.slice(fileIndex + 1).find((token) => destCues.includes(cleanCueToken(token.text)));
    if (!valueLead) return null;
    const actionEnd = firstActionCueEnd(toks);
    if (actionEnd === null) return null;
    if (!(actionEnd <= clauseStart
      && positionsShareStatement(request, actionEnd, clauseStart)
      && positionsShareStatement(request, clauseStart, valueLead.start))) return null;
    contentSpan = request.slice(valueLead.end);
  }
  if (contentSpan === null) return null;
  const content = cleanContent(contentSpan);
  if (content === null) return null;
  if (isNonReferentialContent(content) || namesDeferredWorkProduct(content) || !isLiteralContent(content, contentSpan)) return null;
  return [target, content];
}

/** Mirrors fn statement_scope: preserve UTF-16 positions while masking literal punctuation. */
function statementScope(request) {
  let scoped = request;
  for (const segment of quotedSegmentSpans(request)) {
    scoped = scoped.slice(0, segment.start) + ' '.repeat(segment.end - segment.start) + scoped.slice(segment.end);
  }
  return scoped;
}

function positionsShareStatement(request, left, right) {
  const [from, limit] = left <= right ? [left, right] : [right, left];
  return from === limit || endOfStatement(request, from, limit) === limit;
}

/** Mirrors fn is_literal_content: punctuation needs a closed operand or explicit qualifier. */
function isLiteralContent(content, raw, explicitlyQualified = false) {
  if (Array.from(content).some(isAlphanumeric)) return true;
  if (content.length === 0 || raw === null) return false;
  const [only, ...others] = quotedSegmentSpans(raw);
  if (only && others.length === 0 && only.text === content
    && /^[\s:—–-]*$/u.test(raw.slice(0, only.start))
    && /^[\s.!?。！？।]*$/u.test(raw.slice(only.end))) return true;
  return explicitlyQualified && !/[\x60"'«»“”‘’„‚「」『』]/u.test(content);
}

function isNonReferentialContent(content) {
  const lowered = content.toLowerCase();
  return roleWordForms('non_referential_subject').some((form) => form.slot === 'bare' && lowered === form.text);
}

function namesDeferredWorkProduct(content) {
  const lowered = trimEnd(trimEndMatches(trim(content), charIn('.!?。！？'))).toLowerCase();
  return roleWordForms('file_write_deferred_content_reference').some((form) => {
    if (form.slot === 'bare') return lowered === form.text;
    if (form.slot === 'suffix') return endsWithHeadNoun(lowered, trimStart(form.after));
    return false;
  });
}

function endsWithHeadNoun(content, noun) {
  if (!noun || !content.endsWith(noun)) return false;
  const before = content.slice(0, content.length - noun.length);
  return before === '' || !isAsciiAlphanumeric(lastChar(before));
}

/**
 * Mirrors `fn resolve_census_target`: a `CensusResolution` or null.
 * @param {string} reference
 */
export function resolveCensusTarget(reference) {
  const addressesWorkspace = reference.includes('/') || (reference.includes(':') && !reference.includes('://'));
  if (!addressesWorkspace) return null;
  return resolveReference(workspace(), reference);
}

/** Mirrors `fn mentions_software_authoring`. */
export function mentionsSoftwareAuthoring(text) {
  return mentionsBareRole(text, 'software_authoring_action');
}

function escape(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('"', '\\"').replaceAll('\n', '\\n');
}

function field(name, value) {
  return `  ${name} "${escape(value)}"\n`;
}

function fieldNested(name, value) {
  return `    ${name} "${escape(value)}"\n`;
}
