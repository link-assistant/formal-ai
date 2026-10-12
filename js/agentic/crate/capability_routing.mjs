// Capability is a function of the object in the request, the act asked for,
// and where the effect lands (#1138 B10): a port of
// rust/src/capability_routing.rs and rust/src/capability_routing/evidence.rs.
//
// `ObjectType`, `Act` and `Locus` values are their seed slugs. A
// `RoutingOutcome` is `{kind: 'routed', capability}` | `{kind: 'lowered',
// preferred, capability}` | `{kind: 'honest_gap', needed, missing}` |
// `{kind: 'ask', readings}`.

import { cached, readText } from '../host.mjs';
import { normalizePrompt } from './web_engine_core.mjs';
import { mentionsRole, mentionsRoleRaw, wordsForRole } from './seed_meanings.mjs';
import { agenticToolCapabilities } from './seed_agentic_tool_capabilities.mjs';
import { extractConceptQuery } from './concepts_lookup.mjs';
import { detectResponseLanguage } from './translation_language_markers.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';
import { quotedSegmentSpans } from './normal_markov.mjs';
import {
  eqIgnoreAsciiCase, isAlphabetic, isAlphanumeric, isLowercase, isUppercase, rsplitOnce, splitOnce, splitWhitespace,
  trim, trimMatches,
} from './rust_str.mjs';

export const ObjectType = Object.freeze({
  Url: 'url', Path: 'path', PathScope: 'path_scope', Pattern: 'pattern', PathSet: 'path_set',
  QuotedContent: 'quoted_content', TimeExpression: 'time_expression', RelativePeriod: 'relative_period',
  LanguageName: 'language_name', QuantityQuestion: 'quantity_question', TaskList: 'task_list',
  Delegation: 'delegation', BareTerm: 'bare_term', SelfSurface: 'self_surface', None: 'none',
});

const OBJECT_TYPES = ['url', 'path', 'pattern', 'path_set', 'path_scope', 'quoted_content', 'time_expression',
  'relative_period', 'language_name', 'quantity_question', 'task_list', 'delegation', 'bare_term', 'self_surface', 'none'];

const OBJECT_RANK = {
  url: 0, pattern: 1, path_set: 2, path: 3, self_surface: 4, language_name: 5, quantity_question: 6,
  time_expression: 7, relative_period: 7, task_list: 8, delegation: 9, quoted_content: 10, path_scope: 11,
  bare_term: 12, none: 13,
};

export const Act = Object.freeze({
  Retrieve: 'retrieve', Enumerate: 'enumerate', Transform: 'transform', Compose: 'compose', Schedule: 'schedule',
  Explain: 'explain', Demonstrate: 'demonstrate', Record: 'record', Learn: 'learn', Unresolved: 'unresolved',
});

const ACTS_IN_PRECEDENCE = ['schedule', 'demonstrate', 'compose', 'enumerate', 'transform', 'explain', 'record',
  'learn', 'retrieve'];

/**
 * Mirrors `Act::role`.
 * @param {string} act
 * @returns {string}
 */
const actRole = (act) => (act === 'unresolved' ? '' : `capability_act_${act}`);

export const Locus = Object.freeze({
  Workspace: 'workspace', Web: 'web', Dialogue: 'dialogue', SelfSurface: 'self', Unresolved: 'unresolved',
});
const LOCI = ['workspace', 'web', 'dialogue', 'self', 'unresolved'];

const ROLE = {
  assistantMechanismInquiry: 'assistant_mechanism_inquiry',
  calendarDayReference: 'calendar_day_reference',
  calendarHourReference: 'calendar_hour_reference',
  calendarScheduleAction: 'calendar_schedule_action',
  clockReference: 'capability_clock_reference',
  containerScope: 'capability_container_scope',
  contentAssignment: 'capability_content_assignment',
  contentIntroducer: 'capability_content_introducer',
  delegationMarker: 'capability_delegation_marker',
  freshnessLive: 'capability_freshness_live',
  languageReference: 'capability_language_reference',
  priorTurnReference: 'capability_prior_turn_reference',
  quantityInterrogative: 'capability_quantity_interrogative',
  selfSurfaceNoun: 'capability_self_surface_noun',
  taskListNoun: 'capability_task_list_noun',
  webHostSuffix: 'capability_web_host_suffix',
  webScope: 'capability_web_scope',
  workspaceScope: 'capability_workspace_scope',
  scopeCurrent: 'local_path_scope_current',
  scopeDesktop: 'local_path_scope_desktop',
  scopeHome: 'local_path_scope_home',
  translationLanguage: 'translation_language',
  clarificationRequest: 'clarification_request',
};

const ROUTING_FILE = 'data/seed/capability-routing.lino';

/** Mirrors `fn table_routing_enabled`. */
export function tableRoutingEnabled() {
  return !parseRoot(readText(ROUTING_FILE)).children
    .some((document) => findChildValue(document, 'routing_enabled') === 'false');
}

/** Mirrors `fn dialogue_utterance_roles`. */
export function dialogueUtteranceRoles() {
  return parseRoot(readText(ROUTING_FILE)).children
    .flatMap((document) => document.children || [])
    .filter((child) => child.name === 'dialogue_utterance_role' && child.value)
    .map((child) => child.value);
}

/** Mirrors `fn words` in rust/src/capability_routing/claim_evidence.rs. */
function claimWords(text) {
  return splitWhitespace(text)
    .map((word) => word.replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, ''))
    .filter((word) => word !== '');
}

/** Mirrors `fn content_beyond_roles` in rust/src/capability_routing/claim_evidence.rs. */
export function contentBeyondRoles(text, roles) {
  const covered = [...roles, 'request_function_word', 'statement_function_word']
    .flatMap((role) => wordsForRole(role))
    .flatMap((surface) => claimWords(surface.toLowerCase()));
  return claimWords(text.toLowerCase()).some((word) => !covered.includes(word));
}

/** Mirrors `fn is_dialogue_utterance`. */
export function isDialogueUtterance(prompt) {
  const roles = dialogueUtteranceRoles();
  const normalized = normalizePrompt(prompt);
  return roles.some((role) => mentionsRole(role, normalized)) && !contentBeyondRoles(normalized, roles);
}

/** Mirrors `fn object_type`: every object the prompt carries, ranked. */
export function objectType(prompt) {
  const normalized = normalizePrompt(prompt);
  const tokens = splitWhitespace(prompt);
  const found = [];
  const note = (object, present) => {
    if (present && !found.includes(object)) found.push(object);
  };
  note('url', tokens.some(isUrl));
  note('path', tokens.some((token) => isPath(token, normalized)));
  note('pattern', tokens.some(isPatternToken));
  note('path_set', pathTokenCount(prompt, normalized) > 1);
  note('self_surface', isSelfSurface(normalized));
  note('language_name',
    (evidences(ROLE.translationLanguage, normalized) || evidences(ROLE.languageReference, normalized))
      && (evidences(actRole('demonstrate'), normalized) || evidences(actRole('transform'), normalized))
      && !isResponseLanguageObligation(prompt, normalized));
  note('time_expression', hasClockTime(prompt) || mentionsRole(ROLE.calendarDayReference, normalized)
    || evidences(ROLE.clockReference, normalized));
  note('relative_period', mentionsRole(ROLE.calendarHourReference, normalized)
    && !mentionsRole(ROLE.calendarDayReference, normalized)
    && !hasClockTime(prompt)
    && !evidences(ROLE.clockReference, normalized)
    && !evidences(ROLE.calendarScheduleAction, normalized));
  note('quantity_question', evidences(ROLE.quantityInterrogative, normalized));
  note('task_list', evidences(ROLE.taskListNoun, normalized));
  note('delegation', evidences(ROLE.delegationMarker, normalized));
  note('quoted_content', hasExplicitContent(prompt, normalized));
  note('path_scope', hasContainerScope(normalized));
  note('bare_term', tokens.some((token) => Array.from(token).some(isAlphanumeric)));
  if (!found.length) return ['none'];
  return found.sort((left, right) => OBJECT_RANK[left] - OBJECT_RANK[right]);
}

/** Mirrors `fn evidences_retrieve_act`. */
export function evidencesRetrieveAct(prompt) {
  return evidenceStrength(actRole('retrieve'), normalizePrompt(prompt)) > 0;
}

/** Mirrors `fn is_response_language_obligation`. */
function isResponseLanguageObligation(prompt, normalized) {
  return extractConceptQuery(prompt) !== null && detectResponseLanguage(normalized) !== null;
}

/** Mirrors `fn acts`. */
export function acts(prompt) {
  const normalized = normalizePrompt(prompt);
  if (!normalized) return ['unresolved'];
  const scored = ACTS_IN_PRECEDENCE
    .map((candidate, order) => [evidenceStrength(actRole(candidate), normalized), order, candidate])
    .filter(([strength]) => strength > 0);
  scored.sort((left, right) => right[0] - left[0] || left[1] - right[1]);
  const ordered = scored.map(([, , candidate]) => candidate);
  if (!ordered.includes('retrieve')) ordered.push('retrieve');
  return ordered;
}

/** Mirrors `fn act`. */
export function act(prompt) {
  return acts(prompt)[0] ?? 'unresolved';
}

/** Mirrors `fn locus_of`. */
export function locusOf(object, prompt) {
  const normalized = normalizePrompt(prompt);
  switch (object) {
    case 'url':
      return 'web';
    case 'path':
    case 'path_scope':
    case 'pattern':
    case 'path_set':
      return 'workspace';
    case 'task_list':
    case 'delegation':
    case 'language_name':
    case 'none':
      return 'dialogue';
    case 'self_surface':
      return isPriorTurnReference(normalized) ? 'dialogue' : 'self';
    case 'time_expression':
    case 'relative_period':
      return hasWorkspaceScope(normalized) ? 'workspace' : 'dialogue';
    case 'quoted_content':
      if (hasWorkspaceScope(normalized)) return 'workspace';
      return hasWebScope(normalized) ? 'web' : 'dialogue';
    default:
      return hasWorkspaceScope(normalized) ? 'workspace' : 'web';
  }
}

/** Mirrors `fn locus`. */
export function locus(prompt) {
  return locusOf(objectType(prompt)[0] ?? 'none', prompt);
}

/** Mirrors `fn routing_table`. */
export function routingTable() {
  return cached('capability-routing-table', () => {
    const rows = routingTableFrom(readText(ROUTING_FILE));
    return rows.ok ? rows.rows : [];
  });
}

/** Mirrors `fn routing_table_from`: `{ok: true, rows}` or `{ok: false, error}`. */
export function routingTableFrom(text) {
  const rows = [];
  for (const document of parseRoot(text).children) {
    for (const record of document.children || []) {
      if (record.name !== 'route') continue;
      const objectSlug = findChildValue(record, 'object');
      const actSlug = findChildValue(record, 'act');
      const locusSlug = findChildValue(record, 'locus');
      if (!OBJECT_TYPES.includes(objectSlug)) return { ok: false, error: `route:object:${objectSlug}` };
      if (![...ACTS_IN_PRECEDENCE, 'unresolved'].includes(actSlug)) return { ok: false, error: `route:act:${actSlug}` };
      if (!LOCI.includes(locusSlug)) return { ok: false, error: `route:locus:${locusSlug}` };
      const capability = findChildValue(record, 'capability');
      if (!capability) return { ok: false, error: `route:${objectSlug}:${actSlug}:capability` };
      const fallback = findChildValue(record, 'fallback');
      rows.push({
        object: objectSlug,
        act: actSlug,
        locus: locusSlug,
        capability,
        fallback: fallback || null,
        because: findChildValue(record, 'because'),
      });
    }
  }
  return { ok: true, rows };
}

/** Mirrors `fn route`. */
export function route(prompt, advertised) {
  return routeDecision(prompt, advertised).outcome;
}

/** Mirrors `fn route_decision`: `{object, act, locus, outcome}`. */
export function routeDecision(prompt, advertised) {
  const table = routingTable();
  const objects = objectType(prompt);
  const evidenced = acts(prompt);
  for (const object of objects) {
    const objectLocus = locusOf(object, prompt);
    for (const candidate of evidenced) {
      const outcome = routeWith(table, object, candidate, objectLocus, advertised);
      if (outcome.kind !== 'ask') return { object, act: candidate, locus: objectLocus, outcome };
    }
  }
  const highest = objects[0] ?? 'none';
  const first = evidenced[0] ?? 'unresolved';
  const highestLocus = locusOf(highest, prompt);
  return { object: highest, act: first, locus: highestLocus, outcome: routeWith(table, highest, first, highestLocus, advertised) };
}

/** Mirrors `fn route_placed`. */
export function routePlaced(prompt, advertised) {
  const decision = routeDecision(prompt, advertised);
  if (decision.outcome.kind === 'ask') return null;
  const defaultWeb = decision.locus === 'web'
    && (decision.object === 'bare_term' || decision.object === 'quantity_question')
    && !hasWebScope(normalizePrompt(prompt));
  return defaultWeb ? null : decision.outcome;
}

/** Mirrors `fn advertised_provides`. */
function advertisedProvides(slug, advertised) {
  return advertised.some((name) => eqIgnoreAsciiCase(name, slug)
    || agenticToolCapabilities().some((entry) => {
      const namesSlug = entry.id === slug || entry.aliases.includes(slug);
      const namesTool = eqIgnoreAsciiCase(entry.id, name) || entry.aliases.some((alias) => eqIgnoreAsciiCase(alias, name));
      return namesSlug && namesTool;
    }));
}

/** Mirrors `fn route_with`. */
export function routeWith(table, object, candidate, rowLocus, advertised) {
  const matched = table.filter((row) => row.object === object && row.act === candidate && row.locus === rowLocus);
  if (matched.length !== 1) {
    return { kind: 'ask', readings: readingsFor(table, object, candidate, rowLocus, matched) };
  }
  const [row] = matched;
  if (advertisedProvides(row.capability, advertised)) return { kind: 'routed', capability: row.capability };
  if (row.fallback !== null && advertisedProvides(row.fallback, advertised)) {
    return { kind: 'lowered', preferred: row.capability, capability: row.fallback };
  }
  return { kind: 'honest_gap', needed: row.capability, missing: row.fallback ?? row.capability };
}

const DEFAULT_OUTCOME = 'ask';

/** Mirrors `fn readings_for`. */
function readingsFor(table, object, candidate, rowLocus, matched) {
  const readings = [`${object}:${candidate}:${rowLocus}`];
  for (const row of matched) if (!readings.includes(row.capability)) readings.push(row.capability);
  for (const row of table) {
    if (row.object !== object && !(row.act === candidate && row.locus === rowLocus)) continue;
    if (!readings.includes(row.capability)) readings.push(row.capability);
  }
  if (readings.length < 2) readings.push(DEFAULT_OUTCOME);
  return readings;
}

// ---- evidence.rs ----

/** Mirrors `fn evidences` in rust/src/capability_routing/evidence.rs. */
export function evidences(role, normalized) {
  return mentionsRole(role, normalized) || mentionsRoleRaw(role, normalized);
}

/** Mirrors `fn evidence_strength`. */
export function evidenceStrength(role, normalized) {
  return wordsForRole(role)
    .filter((word) => normalized.includes(word))
    .reduce((max, word) => Math.max(max, Array.from(word).length), 0);
}

/** Mirrors `fn is_url`. */
export function isUrl(token) {
  const trimmed = trimMatches(token, (character) => !isAlphanumeric(character));
  const split = splitOnce(trimmed, '://');
  return split !== null && (split[0] === 'http' || split[0] === 'https') && split[1] !== '';
}

/** Mirrors `fn is_path`. */
export function isPath(token, normalized) {
  const trimmed = trimMatches(token, (character) => ',;"\''.includes(character));
  if (isUrl(trimmed)) return false;
  if (trimmed.includes('/') || trimmed.includes('\\')) return true;
  const split = rsplitOnce(trimmed, '.');
  if (!split) return false;
  const [stem, extension] = split;
  const chars = Array.from(extension);
  return stem !== ''
    && chars.length >= 1 && chars.length <= 5
    && chars.every(isAlphanumeric)
    && chars.some(isAlphabetic)
    && !isWebHostSuffix(extension)
    && (!looksLikeQualifiedMember(stem, extension) || hasWorkspaceScope(normalized));
}

const PATTERN_EDGES = new Set([',', ';', '"', '\'', '`', ':', '(', ')', '.', '。', '?', '!', '¿', '¡']);

/** Mirrors `fn trim_pattern_token`. */
export function trimPatternToken(token) {
  return trimMatches(token, (character) => PATTERN_EDGES.has(character));
}

/** Mirrors `fn is_pattern_token`. */
export function isPatternToken(token) {
  const trimmed = trimPatternToken(token);
  if (!trimmed) return false;
  if (!(trimmed.includes('*') || trimmed.includes('?') || trimmed.includes('['))) return false;
  return trimmed.includes('/') || trimmed.includes('\\')
    || (trimmed.includes('.') && Array.from(trimmed).some(isAlphanumeric));
}

/** Mirrors `fn path_token_count`. */
export function pathTokenCount(prompt, normalized) {
  return splitWhitespace(prompt).filter((token) => isPath(token, normalized)).length;
}

/** Mirrors `fn looks_like_qualified_member`. */
export function looksLikeQualifiedMember(stem, member) {
  const identifier = (text) => Array.from(text).every((character) => isAlphanumeric(character) || character === '_');
  if (stem.includes('.') || !identifier(stem) || !identifier(member)) return false;
  const chars = Array.from(stem);
  return chars.some((left, index) => index + 1 < chars.length && isLowercase(left) && isUppercase(chars[index + 1]));
}

/** Mirrors `fn is_web_host_suffix`. */
export function isWebHostSuffix(extension) {
  return wordsForRole(ROLE.webHostSuffix).includes(extension.toLowerCase());
}

/** Mirrors `fn has_clock_time`. */
export function hasClockTime(prompt) {
  const chars = Array.from(prompt);
  const digit = (character) => character !== undefined && /^[0-9]$/.test(character);
  return chars.some((character, index) => character === ':' && index > 0 && digit(chars[index - 1]) && digit(chars[index + 1]));
}

const QUOTE_PAIRS = [['"', '"'], ['“', '”'], ['«', '»']];

/** Mirrors `fn quoted_span`. */
export function quotedSpan(prompt) {
  for (const [delimiter, closing] of QUOTE_PAIRS) {
    const open = prompt.indexOf(delimiter);
    if (open < 0) continue;
    const rest = prompt.slice(open + delimiter.length);
    const close = rest.indexOf(closing);
    if (close < 0) continue;
    const inner = rest.slice(0, close);
    if (splitWhitespace(inner).length > 1) return inner;
  }
  return null;
}

/** Mirrors `fn has_quoted_span`. */
export function hasQuotedSpan(prompt) {
  return quotedSpan(prompt) !== null;
}

/** Mirrors `fn has_explicit_content`. */
export function hasExplicitContent(prompt, normalized) {
  return hasQuotedSpan(prompt) || evidences(ROLE.contentIntroducer, normalized);
}

/** Mirrors `fn has_container_scope`. */
export function hasContainerScope(normalized) {
  return [ROLE.scopeDesktop, ROLE.scopeHome, ROLE.scopeCurrent, ROLE.containerScope]
    .some((role) => evidences(role, normalized));
}

/** Mirrors `fn has_workspace_scope`. */
export function hasWorkspaceScope(normalized) {
  return hasContainerScope(normalized) || evidences(ROLE.workspaceScope, normalized);
}

/** Mirrors `fn has_web_scope`. */
export function hasWebScope(normalized) {
  return evidences(ROLE.webScope, normalized) || isFreshnessLive(normalized);
}

/** Mirrors `fn names_open_web`. */
export function namesOpenWeb(prompt) {
  return hasWebScope(normalizePrompt(prompt));
}

/** Mirrors `fn is_freshness_live`. */
export function isFreshnessLive(normalized) {
  return evidences(ROLE.freshnessLive, normalized);
}

/** Mirrors `fn is_self_surface`. */
export function isSelfSurface(normalized) {
  return evidences(ROLE.selfSurfaceNoun, normalized)
    || evidences(ROLE.assistantMechanismInquiry, normalized)
    || isPriorTurnReference(normalized);
}

/** Mirrors `fn is_bare_clarification`. */
export function isBareClarification(prompt) {
  const cleaned = normalizePrompt(prompt);
  return cleaned !== '' && wordsForRole(ROLE.clarificationRequest).some((surface) => surface === cleaned);
}

/** Mirrors `fn is_prior_turn_reference`. */
export function isPriorTurnReference(normalized) {
  return evidences(ROLE.priorTurnReference, normalized);
}

/** Mirrors `fn explicit_content`. */
export function explicitContent(prompt) {
  const span = quotedSpan(prompt);
  if (span !== null) return span;
  const characters = Array.from(prompt);
  const lowered = characters.flatMap((character) => Array.from(character.toLowerCase()));
  if (lowered.length !== characters.length) return null;
  let best = null;
  for (const surface of wordsForRole(ROLE.contentIntroducer)) {
    const needle = Array.from(surface);
    if (!needle.length || needle.length > lowered.length) continue;
    for (let start = 0; start <= lowered.length - needle.length; start += 1) {
      if (needle.every((character, offset) => lowered[start + offset] === character)) {
        const end = start + needle.length;
        if (best === null || end > best) best = end;
      }
    }
  }
  if (best === null) return assignedContent(prompt);
  const tail = trimMatches(trim(characters.slice(best).join('')), (character) => ':"“”.。'.includes(character));
  return trim(tail) ? trim(tail) : null;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function byteFind(haystack, needle) {
  outer: for (let start = 0; start + needle.length <= haystack.length; start += 1) {
    for (let offset = 0; offset < needle.length; offset += 1) if (haystack[start + offset] !== needle[offset]) continue outer;
    return start;
  }
  return -1;
}

const isCharBoundary = (bytes, index) => index === 0 || index === bytes.length
  || (index < bytes.length && (bytes[index] & 0xc0) !== 0x80);

/** Mirrors `fn assigned_content` (UTF-8 byte offsets, as Rust reads them). */
export function assignedContent(prompt) {
  const path = firstPath(prompt);
  if (path === null) return null;
  const split = splitOnce(prompt, path);
  if (!split) return null;
  const after = encoder.encode(split[1]);
  const lowered = encoder.encode(split[1].toLowerCase());
  let best = null;
  for (const surface of wordsForRole(ROLE.contentAssignment)) {
    const needle = encoder.encode(surface);
    const position = byteFind(lowered, needle);
    if (position < 0) continue;
    const end = position + needle.length;
    if ((best === null || end < best) && isCharBoundary(after, end)) best = end;
  }
  if (best === null) return null;
  const content = trimMatches(trim(decoder.decode(after.slice(best))), (character) => character === '"');
  return content || null;
}

/** Mirrors `fn first_path`. */
export function firstPath(prompt) {
  const normalized = normalizePrompt(prompt);
  // A path the request leaves unquoted names the file before one inside a
  // quoted payload does (`Insert the line "path '/v1/x'" after … in r.lino`).
  const outside = quotedSegmentSpans(prompt)
    .reduceRight((text, segment) => `${text.slice(0, segment.start)} ${text.slice(segment.end)}`, prompt);
  // The sentence's closing period is not the path's (`… in r.lino.`).
  const peeled = (candidate) => candidate.replace(/^[,;"'()。]+/u, '').replace(/[,;"'()。.]+$/u, '');
  const token = [outside, prompt]
    .map((text) => splitWhitespace(text).find((candidate) => isPath(candidate, normalized) || isPath(peeled(candidate), normalized)))
    .find((candidate) => candidate !== undefined);
  return token === undefined ? null : peeled(token) || null;
}

/** Mirrors `fn first_url`. */
export function firstUrl(prompt) {
  const token = splitWhitespace(prompt).find(isUrl);
  return token === undefined ? null : trimMatches(token, (character) => ',;"\'.'.includes(character));
}
