// Multi-step web research for agentic clients (issue #687):
// rust/src/agentic_coding/web_research.rs.
//
// The routes that consult the engine (`FormalAiEngine.answer`,
// `crate::solve_with_history`) go through the host solver and are therefore
// async: webResearchQueryFor, unresolvedWebResearchQueryFor,
// midResearchWebQueryFor, definitionFollowupTopic.

import { Capability, registryId } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { chatMessageToTurn, latestUserRequest, userRequestText } from './content.mjs';
import { calculationExpressionCandidates } from './crate/calculation.mjs';
import { route, routingTable } from './crate/capability_routing.mjs';
import { containsCjk } from './crate/coding_cjk.mjs';
import { extractConceptQuery, lookupConceptQuery } from './crate/concepts_lookup.mjs';
import { asksForClarification, defersToTheOpenWeb, isInconclusive } from './crate/engine_answer.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect } from './crate/language.mjs';
import { symbolicCosineSimilarity } from './crate/probability.mjs';
import { isAsciiPunctuation, isWhitespace, replaceAllLiteral, splitWhitespace, stripPrefix, stripSuffix,
  toAsciiLowercase, trim, trimEnd, trimEndMatches } from './crate/rust_str.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { agentInfoValue } from './crate/seed_agent_info.mjs';
import { afterSlot, beforeSlot, mentionsRole, roleWordForms, slotOf, wordsForRole } from './crate/seed_meanings.mjs';
import { cleanSearchQuery, detectWebSearchQuery, webSearchQueryFor } from './crate/solver_handlers_web_search.mjs';
import { formalize } from './crate/summarization.mjs';
import { solve } from './host.mjs';
import { fetchArguments, finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { isReportIntent } from './report_issue.mjs';
import { requestBlocks } from './stated_request.mjs';
import { renderFailure } from './tool_result.mjs';

const ROLE_DEFINITION_EXAMPLE_REQUEST = 'definition_example_request';
const ROLE_DEFINITION_ANTECEDENT_FOLLOWUP = 'definition_antecedent_followup';
const ROLE_CLAUSE_CONTINUATION_MARKER = 'clause_continuation_marker';
const ROLE_RESEARCH_QUESTION_OPENER = 'research_question_opener';
const ROLE_DEFINITION_COMMAND = 'definition_command';
const ROLE_WEB_SEARCH_IMPERATIVE_LEAD = 'web_search_imperative_lead';
const ROLE_NON_REFERENTIAL_SUBJECT = 'non_referential_subject';

/** `FormalAiEngine.answer(text)`: the host solver with no history. */
const engineAnswer = (text) => solve(text, []);

const byCharsDescending = (left, right) => Array.from(right).length - Array.from(left).length;

/**
 * Mirrors `fn web_research_query_for` in rust/src/agentic_coding/web_research.rs.
 * @param {Array<object>} messages
 * @returns {Promise<string|null>}
 */
export async function webResearchQueryFor(messages) {
  const task = latestUserRequest(messages);
  if (task === null) return null;
  let query = null;
  for (const block of requestBlocks(task)) {
    query = seedResearchSubject(block)
      ?? seedSlottedSubject(block, ROLE_DEFINITION_EXAMPLE_REQUEST)
      ?? detectWebSearchQuery(block)
      ?? seedDefinitionSubject(block)
      ?? await seedUnresolvedQuestionSubject(block);
    if (query !== null) break;
  }
  if (query === null) return null;
  if (isContextReference(query)) {
    const topic = await topicFromHistory(messages);
    return topic === null ? null : trimQuestionPunctuation(topic);
  }
  return trimQuestionPunctuation(query);
}

/**
 * Mirrors `fn unresolved_web_research_query_for` in rust/src/agentic_coding/web_research.rs.
 * @param {Array<object>} messages
 * @returns {Promise<string|null>}
 */
export function unresolvedWebResearchQueryFor(messages) {
  return unresolvedResearchQueryWith(messages, async (text) => {
    const { intent } = await engineAnswer(text);
    return (intent === 'unknown' || intent === 'web_search') && tableLeavesRequestToResearch(text);
  });
}

/** Mirrors `fn table_leaves_request_to_research`. */
function tableLeavesRequestToResearch(text) {
  const slugs = [];
  for (const row of routingTable()) {
    if (!slugs.includes(row.capability)) slugs.push(row.capability);
    if (row.fallback !== null && row.fallback !== undefined && !slugs.includes(row.fallback)) slugs.push(row.fallback);
  }
  const outcome = route(text, slugs);
  switch (outcome.kind) {
    case 'routed':
    case 'lowered':
      return isResearchCapability(outcome.capability);
    case 'honest_gap':
      return isResearchCapability(outcome.needed);
    default:
      return true;
  }
}

/** Mirrors `fn is_research_capability`. */
function isResearchCapability(capability) {
  return capability === 'web_search' || capability === 'web_fetch';
}

/**
 * Mirrors `fn mid_research_web_query_for` in rust/src/agentic_coding/web_research.rs.
 * @param {Array<object>} messages
 * @returns {Promise<string|null>}
 */
export async function midResearchWebQueryFor(messages) {
  return recordedSearchQueryForTask(messages)
    ?? await webResearchQueryFor(messages)
    ?? await unresolvedResearchQueryWith(messages, async (text) => {
      const answer = await engineAnswer(text);
      return (!asksForClarification(answer) && isInconclusive(answer)) || defersToTheOpenWeb(answer);
    });
}

/** Mirrors `fn recorded_search_query_for_task`. */
function recordedSearchQueryForTask(messages) {
  const task = latestUserRequest(messages);
  if (task === null) return null;
  const progress = Progress.scan(messages);
  if (!progress.latestSuccessUnstalled(Capability.Search)) return null;
  const args = progress.latestSuccessfulArguments(Capability.Search);
  const query = args === null || args === undefined ? null : searchQueryArgument(args);
  if (query === null) return null;
  const queryText = normalizePrompt(query);
  return queryText !== '' && normalizePrompt(task).includes(queryText) ? query : null;
}

/** Mirrors `fn search_query_argument`. */
function searchQueryArgument(args) {
  let value;
  try {
    value = JSON.parse(args);
  } catch {
    return null;
  }
  return value && typeof value === 'object' && !Array.isArray(value) && typeof value.query === 'string'
    ? value.query
    : null;
}

/** Mirrors `fn unresolved_research_query_with` (an async `unresolved` predicate). */
async function unresolvedResearchQueryWith(messages, unresolved) {
  const task = latestUserRequest(messages);
  if (task === null) return null;
  const preceding = messages.slice(0, Math.max(0, messages.length - 1));
  if (!(await unresolved(task)) || await isConversationMetaRequest(task, preceding)) return null;
  let stated = task;
  for (const block of requestBlocks(task)) {
    if (await unresolved(block)) {
      stated = block;
      break;
    }
  }
  return trimQuestionPunctuation(trim(stated));
}

/**
 * Mirrors `fn has_successful_search_result` in rust/src/agentic_coding/web_research.rs.
 * @param {Array<object>} messages
 */
export function hasSuccessfulSearchResult(messages) {
  const output = Progress.scan(messages).latestSuccessfulOutput(Capability.Search);
  return output !== null && output !== undefined;
}

/**
 * Mirrors `fn is_definition_followup` in rust/src/agentic_coding/web_research.rs.
 * @param {string} task
 */
export function isDefinitionFollowup(task) {
  return mentionsRole(ROLE_DEFINITION_ANTECEDENT_FOLLOWUP, normalizePrompt(task));
}

/**
 * Mirrors `fn definition_followup_topic` in rust/src/agentic_coding/web_research.rs.
 * @param {Array<object>} messages
 * @param {string} task
 * @returns {Promise<string|null>}
 */
export async function definitionFollowupTopic(messages, task) {
  const normalized = normalizePrompt(task);
  const forms = [...wordsForRole(ROLE_DEFINITION_ANTECEDENT_FOLLOWUP)].sort(byCharsDescending);
  let prefix = null;
  for (const raw of forms) {
    const form = normalizePrompt(raw);
    const position = normalized.indexOf(form);
    if (position < 0) continue;
    const candidate = trim(normalized.slice(0, position));
    if (candidate !== '') {
      prefix = candidate;
      break;
    }
  }
  if (prefix !== null) {
    let antecedent = trim(trimEndMatches(prefix, (character) =>
      isWhitespace(character) || isAsciiPunctuation(character) || character === '？' || character === '。'));
    const continuations = [...wordsForRole(ROLE_CLAUSE_CONTINUATION_MARKER)].sort(byCharsDescending);
    for (const raw of continuations) {
      const marker = normalizePrompt(raw);
      if (antecedent === marker) {
        antecedent = '';
        break;
      }
      if (antecedent.endsWith(` ${marker}`)) {
        antecedent = antecedent.slice(0, antecedent.length - marker.length - 1);
        break;
      }
    }
    if (antecedent !== '') {
      return detectWebSearchQuery(antecedent)
        ?? seedPrefixSubject(antecedent, ROLE_RESEARCH_QUESTION_OPENER)
        ?? antecedent;
    }
  }
  return topicFromHistory(messages);
}

/**
 * Mirrors `fn definition_followup_clarification` in rust/src/agentic_coding/web_research.rs.
 * @param {string} task
 */
export function definitionFollowupClarification(task) {
  return localizedResponse('definition_followup_clarify', detect(task)) ?? '';
}

/**
 * Mirrors `fn contextual_reference_clarification` in rust/src/agentic_coding/web_research.rs.
 * @param {string} task
 * @returns {string|null}
 */
export function contextualReferenceClarification(task) {
  const query = extractConceptQuery(task);
  if (query === null || query.context === null) return null;
  return isContextReference(query.term) ? definitionFollowupClarification(task) : null;
}

/** Mirrors `fn seed_research_subject`. */
function seedResearchSubject(task) {
  return seedPrefixSubject(task, ROLE_WEB_SEARCH_IMPERATIVE_LEAD);
}

/** Mirrors `fn seed_definition_subject`. */
function seedDefinitionSubject(task) {
  const normalized = normalizePrompt(task);
  for (const form of roleWordForms(ROLE_RESEARCH_QUESTION_OPENER)) {
    if (slotOf(form) !== 'prefix') continue;
    if (!mentionsRole(ROLE_DEFINITION_COMMAND, normalizePrompt(beforeSlot(form)))) continue;
    const rest = stripPrefix(normalized, normalizePrompt(beforeSlot(form)));
    if (rest === null) continue;
    const subject = trimQuestionPunctuation(rest);
    if (trim(subject) !== '') return subject;
  }
  return null;
}

/** Mirrors `fn seed_slotted_subject`. */
function seedSlottedSubject(task, role) {
  const normalized = normalizePrompt(task);
  for (const form of roleWordForms(role)) {
    const before = normalizePrompt(beforeSlot(form));
    const after = normalizePrompt(afterSlot(form));
    let subject = null;
    switch (slotOf(form)) {
      case 'prefix':
        subject = stripPrefix(normalized, before);
        break;
      case 'suffix':
        subject = stripSuffix(normalized, after);
        break;
      case 'circumfix': {
        const body = stripPrefix(normalized, before);
        subject = body === null ? null : stripSuffix(body, after);
        break;
      }
      default:
        subject = null;
    }
    if (subject === null) continue;
    const trimmed = trimQuestionPunctuation(subject);
    if (trimmed !== '') return trimmed;
  }
  return null;
}

/** Mirrors `fn seed_unresolved_question_subject` (async: consults the engine). */
async function seedUnresolvedQuestionSubject(task) {
  const subject = seedPrefixSubject(task, ROLE_RESEARCH_QUESTION_OPENER);
  if (subject === null) return null;
  const { intent } = await engineAnswer(task);
  return intent === 'unknown' || intent === 'web_search' ? subject : null;
}

/** Mirrors `fn seed_prefix_subject`. */
function seedPrefixSubject(task, role) {
  const normalized = normalizePrompt(task);
  for (const form of roleWordForms(role)) {
    if (slotOf(form) !== 'prefix') continue;
    const rest = stripPrefix(normalized, normalizePrompt(beforeSlot(form)));
    if (rest === null) continue;
    const subject = trimQuestionPunctuation(rest);
    if (subject !== '') return subject;
  }
  return null;
}

/** Mirrors `const MAX_RESEARCH_ROUNDS`. */
const MAX_RESEARCH_ROUNDS = 3;

/**
 * Mirrors `fn plan_web_research_step` in rust/src/agentic_coding/web_research.rs:
 * one round of the search -> fetch research recipe, or null.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {string} query
 * @param {boolean} corroborate
 */
export function planWebResearchStep(messages, toolNames, query, corroborate) {
  const progress = Progress.scan(messages);
  const failure = progress.latestFailure();
  if (failure && (failure.capability === Capability.Search || failure.capability === Capability.Fetch)) {
    return finalAnswer(renderFailure(registryId(failure.capability), failure.detail, query));
  }
  const last = progress.last() ?? null;
  if (last === null) {
    const tool = toolFor(toolNames, Capability.Search) ?? null;
    return tool === null ? null : planOne(tool, jsonText({ query }));
  }
  if (last === Capability.Search) {
    return planFetches(toolNames, progress) ?? finalAnswer(composeFinalAnswer(query, progress));
  }
  if (last === Capability.Fetch) {
    if (!corroborate && progress.fetched_pages.length && !uncoveredAspects(query, progress).length) {
      return finalAnswer(composeFinalAnswer(query, progress));
    }
    return planFetches(toolNames, progress)
      ?? planDeeperRound(toolNames, progress, query)
      ?? finalAnswer(composeFinalAnswer(query, progress));
  }
  return planDeeperRound(toolNames, progress, query);
}

/** Mirrors `fn plan_fetches`. */
function planFetches(toolNames, progress) {
  const tool = toolFor(toolNames, Capability.Fetch) ?? null;
  if (tool === null) return null;
  const output = progress.search_output ?? null;
  if (output === null) return null;
  const already = new Set(progress.attempted_fetches);
  const url = researchUrls(output).find((candidate) => !already.has(candidate));
  return url === undefined ? null : planOne(tool, fetchArguments(url));
}

/** Mirrors `fn plan_deeper_round`. */
function planDeeperRound(toolNames, progress, query) {
  if (progress.count(Capability.Search) >= MAX_RESEARCH_ROUNDS) return null;
  const open = uncoveredAspects(query, progress);
  if (open.length !== 1 || aspectsOf(query).length < 3) return null;
  const tool = toolFor(toolNames, Capability.Search) ?? null;
  return tool === null ? null : planOne(tool, jsonText({ query: open.join(' ') }));
}

/** Mirrors `fn uncovered_aspects`. */
function uncoveredAspects(query, progress) {
  if (!progress.fetched_pages.length) return [];
  const evidence = progress.fetched_pages.map(([, text]) => text.toLowerCase()).join(' ');
  return aspectsOf(query).filter((aspect) => !evidence.includes(aspect.toLowerCase()));
}

const MIN_ASPECT_CHARS = 2;
const ALPHANUMERIC = /^[\p{Alphabetic}\p{N}]$/u;

/** Split on non-alphanumeric characters (Rust `split(|c| !c.is_alphanumeric())`, empties kept). */
function splitNonAlphanumeric(text) {
  const parts = [''];
  for (const character of text) {
    if (ALPHANUMERIC.test(character)) parts[parts.length - 1] += character;
    else parts.push('');
  }
  return parts;
}

/** Mirrors `fn aspects_of`. */
function aspectsOf(query) {
  if (containsCjk(query)) return Array.from(query).filter((character) => ALPHANUMERIC.test(character));
  const seen = new Set();
  const out = [];
  for (const token of splitNonAlphanumeric(query)) {
    if (Array.from(token).length < MIN_ASPECT_CHARS) continue;
    const lower = token.toLowerCase();
    if (seen.has(lower)) continue;
    seen.add(lower);
    out.push(lower);
  }
  return out;
}

const MAX_EXTRACT_CHARS = 600;
const EXTRACT_SENTENCES = 3;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/web_research.rs. */
function composeFinalAnswer(query, progress) {
  if (progress.fetched_pages.length) {
    return progress.fetched_pages
      .map(([url, evidence]) => `${extractAnswer(query, trim(evidence))}\n\n${seedText('web_research_source_label')}: ${url}`)
      .join('\n\n');
  }
  if (progress.attempted_fetches.length) return renderSeedText('web_research_no_content', 'query', query);
  const evidence = trim(progress.fetched_text ?? progress.search_output ?? '');
  if (evidence === '') return renderSeedText('web_research_no_content', 'query', query);
  const url = progress.search_output === null || progress.search_output === undefined
    ? null
    : preferredUrl(progress.search_output);
  const source = url === null ? '' : `\n\n${seedText('web_research_source_label')}: ${url}`;
  return `${extractAnswer(query, evidence)}${source}`;
}

/** Mirrors `fn extract_answer`. */
function extractAnswer(query, rawEvidence) {
  const evidence = structurallyCompleteProse(rawEvidence);
  const scored = formalize(evidence)
    .map((statement, position) => [position, relevance(query, statement.text), statement.text])
    .filter(([, score]) => score > 0);
  if (!scored.length) return truncateChars(evidence, MAX_EXTRACT_CHARS);
  scored.sort((left, right) => (right[1] - left[1]) || (left[0] - right[0]));
  const kept = scored.slice(0, EXTRACT_SENTENCES);
  kept.sort((left, right) => left[0] - right[0]);
  return kept.map(([, , text]) => text).join(' ');
}

/** Mirrors `fn structurally_complete_prose`. */
function structurallyCompleteProse(evidence) {
  const blocks = evidence.split('\n\n').map(trim).filter((block) => block !== '');
  const prose = blocks.filter((block) => {
    const chars = Array.from(block);
    return chars.length && isSentenceTerminal(chars[chars.length - 1]);
  });
  return prose.length ? prose.join('\n') : trim(evidence);
}

/** Mirrors `const fn is_sentence_terminal`. */
function isSentenceTerminal(character) {
  return ['.', '!', '?', '。', '…', '।', '॥'].includes(character);
}

/** Mirrors `fn relevance`. */
function relevance(query, sentence) {
  const cosine = symbolicCosineSimilarity(query, sentence);
  if (cosine > 0 || !containsCjk(query)) return cosine;
  return characterOverlap(query, sentence);
}

/** Mirrors `fn character_overlap`. */
function characterOverlap(query, sentence) {
  const inSentence = new Set(Array.from(sentence).filter((character) => ALPHANUMERIC.test(character)));
  const inQuery = new Set(Array.from(query).filter((character) => ALPHANUMERIC.test(character)));
  if (!inQuery.size) return 0;
  let shared = 0;
  for (const character of inQuery) if (inSentence.has(character)) shared += 1;
  return Math.fround(Math.fround(shared) / Math.fround(inQuery.size));
}

/** Mirrors `fn truncate_chars`. */
function truncateChars(value, max) {
  const text = trim(value);
  const chars = Array.from(text);
  if (chars.length <= max) return text;
  return `${trimEnd(chars.slice(0, Math.max(0, max - 1)).join(''))}…`;
}

/** Mirrors `fn seed_text`: the agent-info field, or the key itself. */
function seedText(key) {
  return agentInfoValue(key) ?? key;
}

/** Mirrors `fn render_seed_text`. */
function renderSeedText(key, name, value) {
  return replaceAllLiteral(seedText(key), `{${name}}`, value);
}

/**
 * Mirrors `fn preferred_url` in rust/src/agentic_coding/web_research.rs.
 * @param {string} text
 * @returns {string|null}
 */
export function preferredUrl(text) {
  return researchUrls(text)[0] ?? null;
}

const MAX_RESEARCH_SOURCES = 3;

/** Mirrors `fn research_urls`. */
function researchUrls(text) {
  const seen = new Set();
  const urls = urlsIn(text).filter((url) => {
    if (seen.has(url)) return false;
    seen.add(url);
    return true;
  });
  const position = urls.findIndex(authoritativeHost);
  if (position >= 0) [urls[0], urls[position]] = [urls[position], urls[0]];
  return urls.slice(0, MAX_RESEARCH_SOURCES);
}

/**
 * Mirrors `fn urls_in` in rust/src/agentic_coding/web_research.rs.
 * @param {string} text
 * @returns {Array<string>}
 */
export function urlsIn(text) {
  return splitWhitespace(text)
    .filter((token) => token.startsWith('http://') || token.startsWith('https://'))
    .map((token) => trimEndMatches(token, (character) => '.,;)]"\''.includes(character)));
}

/** Mirrors `fn authoritative_host`. */
function authoritativeHost(url) {
  const at = url.indexOf('://');
  const rest = at < 0 ? url : url.slice(at + 3);
  const host = toAsciiLowercase(rest.split('/')[0].split(':')[0]);
  const labels = host.split('.').reverse();
  const terminal = labels[0];
  return terminal === 'gov' || terminal === 'edu' || (terminal === 'uk' && labels[1] === 'gov');
}

/** Mirrors `fn is_context_reference`. */
function isContextReference(query) {
  const normalized = normalizePrompt(query);
  return roleWordForms(ROLE_NON_REFERENTIAL_SUBJECT).some((form) => {
    switch (slotOf(form)) {
      case 'bare':
        return normalizePrompt(form.text) === normalized;
      case 'prefix':
        return normalized.startsWith(normalizePrompt(beforeSlot(form)));
      default:
        return false;
    }
  });
}

/**
 * Mirrors `fn topic_from_history`: the research topic of the nearest earlier
 * user turn that states one (the `ContextHierarchy` chain resolves a
 * reference at the nearest ancestor context asserting it).
 */
async function topicFromHistory(messages) {
  let latest = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      latest = index;
      break;
    }
  }
  if (latest < 0) return null;
  for (let index = latest - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message.role.toLowerCase() !== 'user') continue;
    const text = userRequestText(message.content);
    if (isReportIntent(text) || await isConversationMetaRequest(text, messages.slice(0, index))) continue;
    const topic = detectWebSearchQuery(text)
      ?? seedPrefixSubject(text, ROLE_RESEARCH_QUESTION_OPENER)
      ?? trimQuestionPunctuation(text);
    if (trim(topic) !== '' && !isContextReference(topic)) return topic;
  }
  return null;
}

/** Mirrors `fn is_conversation_meta_request` (async: consults the solver). */
async function isConversationMetaRequest(prompt, preceding) {
  const history = preceding.map(chatMessageToTurn).filter((turn) => turn !== null);
  return (await solve(prompt, history)).intent === 'summarize_conversation';
}

/** Mirrors `fn trim_question_punctuation`. */
function trimQuestionPunctuation(text) {
  return trim(trimEndMatches(trim(text), (character) => '?？؟¿.!。'.includes(character)));
}

/**
 * Mirrors `fn open_web_query_for_block` in rust/src/agentic_coding/web_research.rs.
 * @param {string} block
 * @returns {string|null}
 */
export function openWebQueryForBlock(block) {
  const query = extractConceptQuery(block);
  if (query !== null) return replaceAllLiteral(query.term.toLowerCase(), '-', ' ');
  return statedWebSearchQueryForBlock(block);
}

/**
 * Mirrors `fn stated_web_search_query_for_block` in rust/src/agentic_coding/web_research.rs.
 * @param {string} block
 * @returns {string|null}
 */
export function statedWebSearchQueryForBlock(block) {
  const query = webSearchQueryFor(block);
  return query === null || query === undefined ? null : cleanSearchQuery(query);
}

/**
 * Mirrors `fn concept_lookup_leaves_unknown` in rust/src/agentic_coding/web_research.rs.
 * @param {string} prompt
 */
export function conceptLookupLeavesUnknown(prompt) {
  const query = extractConceptQuery(prompt);
  return query !== null && lookupConceptQuery(query) === null
    && calculationExpressionCandidates(prompt).length === 0;
}
