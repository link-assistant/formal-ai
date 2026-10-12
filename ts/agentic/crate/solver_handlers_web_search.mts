// Natural-language web-search intent recognition: the JavaScript twin of
// rust/src/solver_handlers/web_search_intent.rs, the marker projection of
// rust/src/web_search_markers.rs, and `normalize_url_candidate` /
// `looks_like_hostname` from rust/src/solver_handlers/web_requests/url_parse.rs.
//
// Every surface cue comes from the meaning lexicon by role and slot; offsets
// are UTF-16 indices into one string and its ASCII-lowercased twin, which share
// lengths, so the byte arithmetic of the Rust original carries over unchanged.

import { cached } from '../host.mjs';
import { agenticMessage } from '../messages.mjs';
import { extractConceptQuery, lookupConceptQuery } from './concepts_lookup.mjs';
import { normalizePrompt } from './engine.mjs';
import { factStoreResolves } from './solver_handlers_benchmark_prompts.mjs';
import { handlerMatches } from './rule_interpreter.mjs';
import { policyLeadClause } from './seed_caller_context.mjs';
import {
  afterSlot, beforeSlot, containsCjk, mentionsRole, roleWordForms, slotOf, wordsForRole,
} from './seed_meanings.mjs';
import { vocabularyMatches } from './seed_operation_vocabulary.mjs';
import { recognizesReachabilityProblem } from './solver_search.mjs';
import {
  isAlphanumeric, isAsciiAlphanumeric, isWhitespace, splitWhitespace, stripPrefix, stripSuffix,
  toAsciiLowercase, trim, trimEnd, trimEndMatches, trimMatches,
} from './rust_str.mjs';

/** Mirrors `enum WebSearchQueryKind` (`as_str` slugs). */
export const WebSearchQueryKind = Object.freeze({
  ExplicitPrefix: 'explicit_prefix',
  SemanticAction: 'semantic_action',
  LatestNews: 'latest_news',
  RecordsInformationRequest: 'records_information_request',
  ImplicitResearchQuestion: 'implicit_research_question',
  EnumerationResearchRequest: 'enumeration_research_request',
  UnresolvedBareTerm: 'unresolved_bare_term',
  UnknownReasoningFallback: 'unknown_reasoning_fallback',
  DocumentOriginalityCheck: 'document_originality_check',
});

const LOCAL_FILE_EXTENSIONS = new Set([
  'txt', 'md', 'json', 'yaml', 'yml', 'toml', 'rs', 'py', 'js', 'ts', 'tsx', 'jsx', 'css', 'html', 'xml', 'csv',
  'lino', 'log', 'sh',
]);

/** Mirrors `fn probable_local_file_name` in rust/src/solver_handlers/web_search_intent.rs. */
export function probableLocalFileName(candidate) {
  const at = candidate.lastIndexOf('.');
  if (at < 0) return false;
  return LOCAL_FILE_EXTENSIONS.has(toAsciiLowercase(candidate.slice(at + 1)));
}

/** Mirrors `fn looks_like_hostname` in rust/src/solver_handlers/web_requests/url_parse.rs. */
export function looksLikeHostname(value) {
  const host = trim(value);
  if (!host.includes('.') || host.startsWith('.') || host.endsWith('.')) return false;
  const labels = host.split('.');
  if (labels.some((label) => label === '')) return false;
  if (labels[labels.length - 1].length < 2) return false;
  return labels.every((label) => Array.from(label).every((character) => isAsciiAlphanumeric(character) || character === '-')
    && !label.startsWith('-') && !label.endsWith('-'));
}

const firstSegment = (text) => text.split(/[/?#]/)[0];

/** Mirrors `fn normalize_url_candidate` in rust/src/solver_handlers/web_requests/url_parse.rs. */
export function normalizeUrlCandidate(raw) {
  const candidate = trim(raw);
  if (candidate === '' || Array.from(candidate).some(isWhitespace) || candidate.includes('@')) return null;
  const lower = candidate.toLowerCase();
  let url;
  if (lower.startsWith('http://') || lower.startsWith('https://')) {
    url = candidate;
  } else {
    const hostCandidate = firstSegment(candidate);
    if (probableLocalFileName(hostCandidate)) return null;
    if (lower.startsWith('www.') || looksLikeHostname(hostCandidate)) url = `https://${candidate}`;
    else return null;
  }
  const at = url.indexOf('://');
  if (at < 0) return null;
  const host = firstSegment(url.slice(at + 3)).split(':')[0];
  return looksLikeHostname(host) ? url : null;
}

// ---------------------------------------------------------------------------
// rust/src/web_search_markers.rs

const prefixLiterals = (role) => roleWordForms(role).filter((form) => slotOf(form) === 'prefix').map(beforeSlot);
const suffixLiterals = (role) => roleWordForms(role).filter((form) => slotOf(form) === 'suffix').map(afterSlot);
const circumfixLiterals = (role) => roleWordForms(role)
  .filter((form) => slotOf(form) === 'circumfix')
  .map((form) => [beforeSlot(form), afterSlot(form)]);
const bareLiterals = (role) => roleWordForms(role).filter((form) => slotOf(form) === 'bare').map((form) => form.text);
const sourceLiterals = (role) => wordsForRole(role).map((word) => trim(word).toLowerCase());

function informationLiterals() {
  const sources = bareLiterals('web_search_source_only');
  return bareLiterals('web_search_signal').filter((marker) => !sources.some((source) => trim(source) === trim(marker)));
}

/** Mirrors `fn markers` in rust/src/web_search_markers.rs. */
export function markers() {
  return cached('web-search-markers', () => ({
    explicit_prefixes: prefixLiterals('web_search_explicit_prefix'),
    explicit_suffixes: suffixLiterals('web_search_explicit_prefix'),
    explicit_circumfixes: circumfixLiterals('web_search_explicit_prefix'),
    action_markers: bareLiterals('web_search_action'),
    strong_action_markers: bareLiterals('web_search_strong_action'),
    strong_imperative_lead_markers: prefixLiterals('web_search_strong_action'),
    signal_markers: bareLiterals('web_search_signal'),
    topic_after_markers: prefixLiterals('web_search_topic_marker'),
    topic_before_markers: suffixLiterals('web_search_topic_marker'),
    imperative_lead_markers: prefixLiterals('web_search_imperative_lead'),
    imperative_tail_markers: suffixLiterals('web_search_imperative_lead'),
    leading_noise: prefixLiterals('web_search_query_leading_noise'),
    trailing_noise: suffixLiterals('web_search_query_trailing_noise'),
    source_only: sourceLiterals('web_search_source_only'),
    source_markers: bareLiterals('web_search_source_only'),
    source_medium_markers: bareLiterals('web_medium'),
    information_markers: informationLiterals(),
    news_subject_markers: bareLiterals('web_search_news_subject'),
    news_recency_markers: bareLiterals('web_search_news_recency'),
    records_subject_markers: bareLiterals('web_search_records_subject'),
    public_event_subject_markers: bareLiterals('web_search_public_event_subject'),
    followup_verbs: bareLiterals('followup_instruction_verb'),
    continuation_markers: bareLiterals('clause_continuation_marker'),
    term_information_prefixes: prefixLiterals('term_information_request_opener'),
    term_information_suffixes: suffixLiterals('term_information_request_opener'),
    term_information_circumfixes: circumfixLiterals('term_information_request_opener'),
    research_question_prefixes: prefixLiterals('research_question_opener'),
    research_modifiers: bareLiterals('research_superlative_modifier'),
    research_evidence_domains: bareLiterals('research_evidence_domain'),
    research_evaluation_domains: bareLiterals('research_evaluation_domain'),
    enumeration_prefixes: prefixLiterals('enumeration_request_opener'),
    enumeration_constraint_markers: bareLiterals('enumeration_constraint'),
  }));
}

// ---------------------------------------------------------------------------
// rust/src/solver_handlers/web_search_intent.rs

const conversationSearchOpeners = () => agenticMessage('web_search_conversation_search_openers')
  .split('|')
  .map((opener) => `${opener} `);

/**
 * Mirrors `fn extract_web_search_request` in
 * rust/src/solver_handlers/web_search_intent.rs: `{query, kind}` or null.
 * @param {string} prompt
 * @param {string} normalized the lowercased prompt
 */
export function extractWebSearchRequest(prompt, normalized) {
  const normalizedWords = normalizePrompt(prompt);
  if (conversationSearchOpeners().some((opener) => normalizedWords.startsWith(opener))
    || isPersonalFactFilterRequest(normalizedWords)) {
    return null;
  }
  const explicit = extractExplicitWebSearchQuery(normalized) ?? extractExplicitWebSearchQuery(normalizedWords);
  if (explicit !== null) return { query: explicit, kind: WebSearchQueryKind.ExplicitPrefix };
  if (isTextExtractionRequest(normalizedWords)) return null;
  if (recognizesReachabilityProblem(normalizedWords)) return null;
  if (factStoreResolves(prompt)) return null;
  const cascade = [
    [() => extractSemanticWebSearchQuery(normalizedWords), WebSearchQueryKind.SemanticAction],
    [() => extractSourceGroundedQuestion(prompt, normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
    [() => extractCurrentSourceInformationRequest(normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
    [() => extractLatestNewsSearchRequest(normalizedWords), WebSearchQueryKind.LatestNews],
    [() => extractRecordsInformationRequest(normalizedWords), WebSearchQueryKind.RecordsInformationRequest],
    [() => extractEnumerationResearchRequest(normalizedWords), WebSearchQueryKind.EnumerationResearchRequest],
    [() => extractCurrentPublicEventQuestion(normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
    [() => extractTermInformationRequest(prompt, normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
    [() => extractImplicitResearchQuestion(normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
    [() => extractExternallyVerifiableQuestion(prompt, normalizedWords), WebSearchQueryKind.ImplicitResearchQuestion],
  ];
  for (const [extract, kind] of cascade) {
    const query = extract();
    if (query !== null) return { query, kind };
  }
  return null;
}

/** Mirrors `fn web_search_query_for` in rust/src/solver_handlers/web_search_intent.rs. */
export function webSearchQueryFor(prompt) {
  return extractWebSearchRequest(prompt, prompt.toLowerCase())?.query ?? null;
}

/** Mirrors `fn detect_web_search_query` in rust/src/solver_handlers/web_requests.rs. */
export function detectWebSearchQuery(prompt) {
  return extractWebSearchRequest(prompt, prompt.toLowerCase())?.query ?? null;
}

const isPersonalFactFilterRequest = (normalized) => mentionsRole('personal_facts_listing_request', normalized);

const URL_WRAPPERS = new Set(['<', '>', '(', ')', '[', ']', '{', '}', '"', "'", '`', '«', '»']);
const isUrlWrapperPunctuation = (character) => URL_WRAPPERS.has(character);
const URL_TRAILING = new Set(['.', ',', '!', '?', ';', ':', '…']);
const isUrlTrailingPunctuation = (character) => URL_TRAILING.has(character);
const SENTENCE_BOUNDARY = new Set(['.', '?', '!', ';', ':', '。', '？', '！', '；', '：']);
const isSentenceBoundary = (character) => SENTENCE_BOUNDARY.has(character);

/** Mirrors `fn clean_search_query` in rust/src/solver_handlers/web_search_intent.rs. */
export function cleanSearchQuery(value) {
  const stripped = trimEndMatches(trimMatches(trim(value), isUrlWrapperPunctuation), isUrlTrailingPunctuation);
  return splitWhitespace(stripped).join(' ');
}

function extractSemanticWebSearchQuery(normalized) {
  const m = markers();
  const imperativeCandidate = imperativeLeadCandidate(normalized, m.imperative_lead_markers, m)
    ?? imperativeTailCandidate(normalized, m.imperative_tail_markers);
  const hasImperativeLead = imperativeCandidate !== null;
  const hasAction = hasImperativeLead || containsAnySearchMarker(normalized, m.action_markers);
  if (!hasAction) return null;
  const hasStrongAction = imperativeLeadCandidate(normalized, m.strong_imperative_lead_markers, m) !== null
    || imperativeTailCandidate(normalized, m.imperative_tail_markers) !== null
    || containsAnySearchMarker(normalized, m.strong_action_markers);
  if (!hasStrongAction && !containsAnySearchMarker(normalized, m.signal_markers)) return null;
  for (const marker of m.topic_after_markers) {
    const index = normalized.indexOf(marker);
    if (index < 0) continue;
    const topic = normalized.slice(index + marker.length);
    if (statesWhenToSearch(topic)) continue;
    const query = validSearchQuery(topic);
    if (query !== null) return query;
  }
  for (const marker of m.topic_before_markers) {
    const index = normalized.indexOf(marker);
    if (index < 0) continue;
    const query = validSearchQuery(normalized.slice(0, index));
    if (query !== null) return query;
  }
  if (imperativeCandidate !== null && !statesWhenToSearch(imperativeCandidate)) {
    const query = validSearchQuery(imperativeCandidate);
    if (query !== null) return query;
  }
  return null;
}

/** Mirrors `fn imperative_tail_candidate`. */
function imperativeTailCandidate(normalized, tails) {
  for (const tail of tails) {
    const candidate = stripSuffix(normalized, tail);
    if (candidate !== null) return candidate;
  }
  return null;
}

/** Mirrors `fn imperative_lead_candidate`. */
function imperativeLeadCandidate(normalized, leads, m) {
  for (const lead of leads) {
    const candidate = stripPrefix(normalized, lead);
    if (candidate !== null) return candidate;
    const index = normalized.indexOf(lead);
    if (index < 0) continue;
    const introducer = normalized.slice(0, index);
    const questionLed = startsWithAny(normalized, m.research_question_prefixes);
    const sourceLed = containsAnySearchMarker(introducer, m.source_markers);
    if (questionLed || sourceLed) return normalized.slice(index + lead.length);
  }
  return null;
}

function extractSourceGroundedQuestion(prompt, normalized) {
  if (!questionIsInterrogative(prompt, normalized)
    || !containsAnySearchMarker(normalized, markers().source_medium_markers)) {
    return null;
  }
  return extractTopicSubject(normalized);
}

function extractCurrentSourceInformationRequest(normalized) {
  const m = markers();
  if (!containsAnySearchMarker(normalized, m.source_medium_markers)
    || !containsAnySearchMarker(normalized, m.news_recency_markers)
    || !containsAnySearchMarker(normalized, m.information_markers)) {
    return null;
  }
  return extractTopicSubject(normalized);
}

function extractTopicSubject(normalized) {
  const m = markers();
  for (const marker of m.topic_after_markers) {
    const index = normalized.indexOf(marker);
    if (index < 0) continue;
    const topic = normalized.slice(index + marker.length);
    if (statesWhenToSearch(topic)) continue;
    return validSearchQuery(topic);
  }
  for (const marker of m.topic_before_markers) {
    const index = normalized.indexOf(marker);
    if (index >= 0) return validSearchQuery(normalized.slice(0, index));
  }
  return null;
}

function extractExplicitWebSearchQuery(normalized) {
  const m = markers();
  for (const prefix of m.explicit_prefixes) {
    const rest = stripPrefix(normalized, prefix);
    if (rest === null) continue;
    const query = validSearchQuery(rest);
    if (query !== null) return query;
  }
  for (const [prefix, suffix] of m.explicit_circumfixes) {
    const rest = stripPrefix(normalized, prefix);
    if (rest === null) continue;
    const candidate = stripSuffix(rest, suffix) ?? stripSuffix(trimEndMatches(rest, isUrlTrailingPunctuation), suffix);
    if (candidate === null) continue;
    const query = validSearchQuery(candidate);
    if (query !== null) return query;
  }
  for (const suffix of m.explicit_suffixes) {
    const candidate = stripSuffix(normalized, suffix)
      ?? stripSuffix(trimEndMatches(normalized, isUrlTrailingPunctuation), suffix);
    if (candidate === null) continue;
    const query = validSearchQuery(candidate);
    if (query !== null) return query;
  }
  return null;
}

const isTextExtractionRequest = (normalized) => vocabularyMatches('extract_url', normalized)
  || vocabularyMatches('extract_email', normalized)
  || vocabularyMatches('extract_number', normalized);

function extractLatestNewsSearchRequest(normalized) {
  const m = markers();
  if (!containsAnySearchMarker(normalized, m.news_subject_markers)
    || !containsAnySearchMarker(normalized, m.news_recency_markers)) {
    return null;
  }
  return validNewsSearchQuery(normalized);
}

function extractRecordsInformationRequest(normalized) {
  const m = markers();
  if (!containsAnySearchMarker(normalized, m.records_subject_markers)) return null;
  const hasTopicMarker = [...m.topic_after_markers, ...m.topic_before_markers]
    .some((marker) => containsSearchMarker(normalized, marker));
  if (!hasTopicMarker) return null;
  return validNewsSearchQuery(normalized);
}

function extractCurrentPublicEventQuestion(normalized) {
  const m = markers();
  if (!startsWithAny(normalized, m.research_question_prefixes)) return null;
  if (!containsAnySearchMarker(normalized, m.public_event_subject_markers)
    || !containsAnySearchMarker(normalized, m.news_recency_markers)) {
    return null;
  }
  return validSearchQuery(stripImplicitResearchPrefix(normalized));
}

function extractTermInformationRequest(prompt, normalized) {
  if (conceptLookupResolves(prompt) || termInformationPromptIsLocalContext(normalized)) return null;
  const m = markers();
  const candidates = [
    ...m.term_information_prefixes.map((prefix) => stripPrefix(normalized, prefix)),
    ...m.term_information_suffixes.map((suffix) => stripSuffix(normalized, suffix)),
    ...m.term_information_circumfixes.map(([before, after]) => {
      const rest = stripPrefix(normalized, before);
      return rest === null ? null : stripSuffix(rest, after);
    }),
  ].filter((candidate) => candidate !== null);
  for (const candidate of candidates) {
    if (termInformationQueryIsLocalContext(candidate)) return null;
    const query = validSearchQuery(candidate);
    if (query !== null) return query;
  }
  return null;
}

function conceptLookupResolves(prompt) {
  const query = extractConceptQuery(prompt);
  return query !== null && query !== undefined && lookupConceptQuery(query) !== null;
}

const termInformationPromptIsLocalContext = (normalized) => mentionsRole('self_introduction_request', normalized)
  || mentionsRole('capability_query', normalized)
  || mentionsRole('capability_query_more', normalized);

function termInformationQueryIsLocalContext(raw) {
  const query = cleanSearchQuery(raw).toLowerCase();
  return mentionsRole('non_referential_subject', query) || mentionsRole('assistant_self_reference', query);
}

function extractImplicitResearchQuestion(normalized) {
  const m = markers();
  if (!startsWithAny(normalized, m.research_question_prefixes)) return null;
  const padded = ` ${normalized} `;
  const hasModifier = m.research_modifiers.some((marker) => padded.includes(marker));
  const hasEvidenceDomain = m.research_evidence_domains.some((marker) => padded.includes(marker));
  const hasEvaluationDomain = m.research_evaluation_domains.some((marker) => padded.includes(marker));
  if (!(hasModifier || (hasEvidenceDomain && hasEvaluationDomain))) return null;
  return validSearchQuery(stripImplicitResearchPrefix(normalized));
}

function extractExternallyVerifiableQuestion(prompt, normalized) {
  if (!questionIsInterrogative(prompt, normalized)) return null;
  if (!promptNamesEngineeredBrand(prompt)) return null;
  if (conceptLookupResolves(prompt)
    || termInformationPromptIsLocalContext(normalized)
    || handlerMatches('docs_method_explanation', prompt)) {
    return null;
  }
  const query = stripImplicitResearchPrefix(normalized);
  if (termInformationQueryIsLocalContext(query)) return null;
  return validSearchQuery(query);
}

function questionIsInterrogative(prompt, normalized) {
  if (startsWithAny(normalized, markers().research_question_prefixes)) return true;
  const tail = trimEnd(prompt);
  return tail.endsWith('?') || tail.endsWith('？');
}

function promptNamesEngineeredBrand(prompt) {
  let previousIsLowerLatin = false;
  for (const character of prompt) {
    if (previousIsLowerLatin && /^[A-Z]$/.test(character)) return true;
    previousIsLowerLatin = /^[a-z]$/.test(character);
  }
  return false;
}

function extractEnumerationResearchRequest(normalized) {
  const query = stripEnumerationResearchPrefix(normalized);
  if (query === null || !looksLikeEnumerationResearchQuery(query)) return null;
  return validSearchQuery(query);
}

const startsWithAny = (value, prefixes) => prefixes.some((prefix) => value.startsWith(prefix));

function stripImplicitResearchPrefix(value) {
  for (const prefix of markers().research_question_prefixes) {
    const stripped = stripPrefix(value, prefix);
    if (stripped !== null) return stripped;
  }
  return value;
}

function stripEnumerationResearchPrefix(value) {
  for (const prefix of markers().enumeration_prefixes) {
    const stripped = stripPrefix(value, prefix);
    if (stripped !== null) return stripped;
  }
  return null;
}

function looksLikeEnumerationResearchQuery(query) {
  if (splitWhitespace(query).length < 3) return false;
  return containsAnySearchMarker(query, markers().enumeration_constraint_markers);
}

const containsAnySearchMarker = (normalized, list) => list.some((marker) => containsSearchMarker(normalized, marker));

/**
 * Whether `normalized` holds `marker`; a marker padded with spaces matches whole words only.
 * @param {string} normalized
 * @param {string} marker
 * @returns {boolean}
 */
function containsSearchMarker(normalized, marker) {
  if (marker.startsWith(' ') || marker.endsWith(' ')) return ` ${normalized} `.includes(marker);
  return normalized.includes(marker);
}

/** Mirrors `fn states_when_to_search`. */
function statesWhenToSearch(topic) {
  const condition = policyLeadClause(topic);
  return condition !== null && opensWithNonReferentialSubject(condition);
}

function opensWithNonReferentialSubject(clause) {
  const subject = splitWhitespace(clause)[0];
  if (subject === undefined) return false;
  return roleWordForms('non_referential_subject').some((form) => slotOf(form) === 'bare' && subject === form.text);
}

const validSearchQuery = (value) => validCleanSearchQuery(cleanSemanticSearchQuery(value));

const validNewsSearchQuery = (value) => validCleanSearchQuery(cleanSearchQuery(truncateSearchInstructionTail(value)));

function validCleanSearchQuery(query) {
  const queryKey = query.toLowerCase();
  if (query === '' || markers().source_only.some((word) => word === queryKey) || normalizeUrlCandidate(query) !== null) {
    return null;
  }
  return query;
}

const lastChar = (text) => (text === '' ? null : Array.from(text.slice(-2)).pop());
const firstChar = (text) => (text === '' ? null : String.fromCodePoint(text.codePointAt(0)));

/** Mirrors `fn truncate_search_instruction_tail`. */
function truncateSearchInstructionTail(value) {
  const m = markers();
  const lower = toAsciiLowercase(value);
  let cut = value.length;
  for (const verb of m.followup_verbs) {
    const cjk = containsCjk(verb);
    let from = 0;
    for (;;) {
      const start = lower.indexOf(verb, from);
      if (start < 0) break;
      const end = start + verb.length;
      from = end;
      if (!cjk && (!isTokenStart(lower, start) || !isTokenEnd(lower, end))) continue;
      const boundary = boundaryBefore(lower, start, m);
      if (boundary !== null) cut = Math.min(cut, boundary);
      if (verb === '') break;
    }
  }
  return trim(value.slice(0, cut));
}

function isTokenStart(text, index) {
  const previous = lastChar(text.slice(0, index));
  return !(previous !== null && isAlphanumeric(previous));
}

function isTokenEnd(text, index) {
  const next = firstChar(text.slice(index));
  return !(next !== null && isAlphanumeric(next));
}

/** Mirrors `fn boundary_before`. */
function boundaryBefore(text, verbStart, m) {
  const head = trimEnd(text.slice(0, verbStart));
  if (head === '') return null;
  if (isSentenceBoundary(lastChar(head))) return head.length;
  let cursor = head;
  let matched = false;
  for (;;) {
    const trimmed = trimEnd(cursor);
    const marker = m.continuation_markers.find((candidate) => endsWithToken(trimmed, candidate));
    if (marker === undefined) break;
    cursor = trimmed.slice(0, trimmed.length - marker.length);
    matched = true;
  }
  return matched ? trimEnd(cursor).length : null;
}

/** Mirrors `fn ends_with_token`. */
function endsWithToken(haystack, marker) {
  if (containsCjk(marker)) return haystack.endsWith(marker);
  if (haystack === marker) return true;
  const head = stripSuffix(haystack, marker);
  const tail = head === null ? null : lastChar(head);
  return tail !== null && isWhitespace(tail);
}

/** Mirrors `fn clean_semantic_search_query`. */
function cleanSemanticSearchQuery(value) {
  const m = markers();
  let query = cleanSearchQuery(truncateSearchInstructionTail(value));
  for (;;) {
    const before = query;
    for (const prefix of m.leading_noise) {
      const stripped = stripPrefix(query, prefix);
      if (stripped !== null) query = cleanSearchQuery(stripped);
    }
    for (const suffix of m.trailing_noise) {
      const stripped = stripSuffix(query, suffix);
      if (stripped !== null) query = cleanSearchQuery(stripped);
    }
    if (query === before) return query;
  }
}
