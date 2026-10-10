// Reading a request to find something out about the workspace (issue #1066):
// rust/src/agentic_coding/workspace_inspection.rs.

import { normalizePrompt } from './crate/engine.mjs';
import { toAsciiLowercase, utf8Len } from './crate/rust_str.mjs';
import { mentionsRole, roleWordForms } from './crate/seed_meanings.mjs';
import { codeSearchQueryForTask, codeShapedQuery, searchTokens, validSearchIdentifier } from './shell_command.mjs';
import { isProseWord, sentenceSpans } from './shell_command_policy.mjs';
import { requestBlocks } from './stated_request.mjs';

const ROLE_WORKSPACE_INSPECTION_ACTION = 'workspace_inspection_action';
const ROLE_WEB_SEARCH_SIGNAL = 'web_search_signal';
const ROLE_CODING_DOCUMENTATION_FACT_QUERY = 'coding_documentation_fact_query';
const ROLE_CODING_TEST_ARTIFACT_KIND = 'coding_test_artifact_kind';
const ROLE_CODING_EXPERIMENT_ARTIFACT_KIND = 'coding_experiment_artifact_kind';
const ROLE_CODING_SEARCH_SUBJECT_KIND = 'coding_search_subject_kind';
const ROLE_CODING_CONDITION_SUBJECT_KIND = 'coding_condition_subject_kind';
const ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND = 'coding_source_implementation_subject_kind';
const ROLE_CODING_SEARCH_FACT_QUERY = 'coding_search_fact_query';
const ROLE_CODING_SERIALIZATION_ACTION = 'coding_serialization_action';
const ROLE_CODING_RELATIONSHIP_SUBJECT_KIND = 'coding_relationship_subject_kind';

const tokensOf = (text) => Array.from(searchTokens(text));

/**
 * Mirrors `fn asks_about_the_workspace` in rust/src/agentic_coding/workspace_inspection.rs.
 * @param {string} prompt
 */
export function asksAboutTheWorkspace(prompt) {
  return requestBlocks(prompt).some((block) => {
    const normalized = normalizePrompt(block);
    return mentionsRole(ROLE_WORKSPACE_INSPECTION_ACTION, normalized) && !namesAnExternalSource(normalized);
  });
}

/** Mirrors `fn names_an_external_source`. */
function namesAnExternalSource(normalized) {
  return mentionsRole(ROLE_WEB_SEARCH_SIGNAL, normalized);
}

/**
 * Mirrors `fn workspace_inspection_search_for_task` in
 * rust/src/agentic_coding/workspace_inspection.rs: `{query, pattern, include}`
 * (`include` a string or null), or null.
 * @param {string} prompt
 */
export function workspaceInspectionSearchForTask(prompt) {
  for (const block of requestBlocks(prompt)) {
    if (!asksAboutTheWorkspace(block)) continue;
    const query = codeSearchQueryForTask(block) ?? null;
    if (query !== null) return { query, pattern: query, include: null };
    for (const sentence of sentenceSpans(block)) {
      if (!asksAboutTheWorkspace(sentence)) continue;
      const search = workspaceInspectionSearch(sentence);
      if (search !== null) return search;
    }
    const search = workspaceInspectionSearch(block);
    if (search !== null) return search;
  }
  return null;
}

/** Mirrors `fn workspace_inspection_search`. */
function workspaceInspectionSearch(text) {
  const query = codeShapedQuery(text) ?? null;
  if (query === null) return null;
  const terms = inspectionFactTerms(text, query);
  const canonicalFact = literalInspectionFactQuery(text.toLowerCase()) ?? serializedRelationshipFactQuery(text);
  const include = canonicalFact !== null
    ? canonicalFactFilenameFilter(text)
    : inspectionFilenameFilter(text, query);
  if (canonicalFact !== null) return { query, pattern: canonicalFact, include };
  const patternTerms = [];
  if (include === null) patternTerms.push(query);
  patternTerms.push(...terms);
  return { query, pattern: patternTerms.length ? patternTerms.join('|') : query, include };
}

/** Mirrors `fn canonical_fact_filename_filter`. */
function canonicalFactFilenameFilter(text) {
  if (mentionsRole(ROLE_CODING_DOCUMENTATION_FACT_QUERY, text)) return 'docs/**/*';
  if (mentionsRole(ROLE_CODING_TEST_ARTIFACT_KIND, text)) return 'tests/**/*';
  if (mentionsRole(ROLE_CODING_EXPERIMENT_ARTIFACT_KIND, text)) return 'experiments/**/*';
  return 'src/**/*';
}

/**
 * Mirrors `fn workspace_inspection_terms_for_task` in
 * rust/src/agentic_coding/workspace_inspection.rs.
 * @param {string} prompt
 * @returns {Array<string>}
 */
export function workspaceInspectionTermsForTask(prompt) {
  for (const block of requestBlocks(prompt)) {
    if (!asksAboutTheWorkspace(block)) continue;
    for (const sentence of sentenceSpans(block)) {
      if (!asksAboutTheWorkspace(sentence)) continue;
      const query = codeShapedQuery(sentence) ?? null;
      if (query !== null) return inspectionFactTerms(sentence, query);
    }
    const query = codeShapedQuery(block) ?? null;
    if (query !== null) return inspectionFactTerms(block, query);
  }
  return [];
}

/** Mirrors `fn inspection_fact_terms`. */
function inspectionFactTerms(text, query) {
  const terms = [];
  const scoped = inspectionSubjectAndFollowing(text, query);
  const normalizedQuery = query.toLowerCase();
  for (const token of tokensOf(scoped)) {
    const normalized = token.replaceAll('-', '_').toLowerCase();
    if (utf8Len(normalized) < 3
      || normalized === normalizedQuery
      || /^[0-9]*$/.test(normalized)
      || isProseWord(normalized)
      || mentionsRole(ROLE_WORKSPACE_INSPECTION_ACTION, normalized)
      || mentionsRole(ROLE_CODING_SEARCH_SUBJECT_KIND, normalized)
      || terms.includes(normalized)) continue;
    terms.push(normalized);
  }
  const canonical = literalInspectionFactQuery(scoped.toLowerCase());
  if (canonical !== null && !terms.includes(canonical)) terms.push(canonical);
  return terms;
}

/** Mirrors `fn inspection_subject_and_following`. */
function inspectionSubjectAndFollowing(text, query) {
  const hyphenated = query.replaceAll('_', '-');
  const offsets = [query, hyphenated]
    .map((spelling) => asciiCaseInsensitiveOffset(text, spelling))
    .filter((offset) => offset !== null);
  return offsets.length ? text.slice(Math.min(...offsets)) : text;
}

/** Mirrors `fn ascii_case_insensitive_offset` (UTF-16 offsets on character boundaries). */
function asciiCaseInsensitiveOffset(text, needle) {
  if (!/^[\x00-\x7f]*$/.test(needle)) return null;
  const wanted = toAsciiLowercase(needle);
  let offset = 0;
  for (const character of text) {
    if (offset + needle.length <= text.length
      && toAsciiLowercase(text.slice(offset, offset + needle.length)) === wanted) return offset;
    offset += character.length;
  }
  return null;
}

/** Mirrors `fn module_filename_filter`. */
function moduleFilenameFilter(query) {
  return query.includes('_') && /^[a-z0-9_]*$/.test(query) ? `*${query}*` : null;
}

/** Mirrors `fn inspection_filename_filter`. */
function inspectionFilenameFilter(text, query) {
  const module = moduleFilenameFilter(query);
  if (module !== null) return module;
  return mentionsRole(ROLE_CODING_CONDITION_SUBJECT_KIND, text)
    || mentionsRole(ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND, text)
    ? 'src/**/*'
    : null;
}

/** Mirrors `fn literal_inspection_fact_query` (`max_by_key` keeps the last longest form). */
function literalInspectionFactQuery(normalized) {
  let best = null;
  for (const form of roleWordForms(ROLE_CODING_SEARCH_FACT_QUERY)) {
    if (!normalized.includes(form.text.toLowerCase())) continue;
    if (best === null || Array.from(form.text).length >= Array.from(best.text).length) best = form;
  }
  return best !== null && best.action ? best.action : null;
}

/** Mirrors `fn serialized_relationship_fact_query`. */
function serializedRelationshipFactQuery(text) {
  const term = serializedRelationshipTerm(text);
  return term === null ? null : `"${term}"`;
}

/**
 * Mirrors `fn serialized_relationship_term` in
 * rust/src/agentic_coding/workspace_inspection.rs.
 * @param {string} text
 * @returns {string|null}
 */
export function serializedRelationshipTerm(text) {
  if (!mentionsRole(ROLE_CODING_SERIALIZATION_ACTION, normalizePrompt(text))) return null;
  const tokens = tokensOf(text);
  for (let index = 0; index + 1 < tokens.length; index += 1) {
    if (validSearchIdentifier(tokens[index])
      && mentionsRole(ROLE_CODING_RELATIONSHIP_SUBJECT_KIND, tokens[index + 1])) {
      return toAsciiLowercase(tokens[index]);
    }
  }
  return null;
}

// Discovery remains distinct from a qualified workspace inspection finding.
export { workspaceDiscoveryContract, workspaceDiscoveryStep } from './workspace_discovery.mjs';
