// Issue #386 — route the JS worker's web-search / research / enumeration
// recogniser through the meaning lexicon, exactly like the Rust handler
// src/solver_handlers/web_search_intent.rs.
//
// BEFORE: seventeen hardcoded const arrays (WEB_SEARCH_EXPLICIT_PREFIXES,
// SEARCH_QUERY_AFTER_MARKERS, IMPLICIT_RESEARCH_MODIFIERS, …) listed the surface
// cues — and only in English/Russian, lagging the Rust seed which already covers
// Hindi and Chinese. AFTER: the cues are projected out of the language-
// independent meaning lexicon (data/seed/meanings-web-search*.lino,
// meanings-web-research.lino, meanings-web-followup.lino — embedded in
// MEANINGS_LINO) by semantic *role* and *slot*, so adding a language or synonym
// is a pure data edit. The follow-up truncation becomes the same universal
// boundary algorithm Rust uses (an instruction verb after a sentence boundary or
// a clause-continuation run), not a list of memorised "". compare""-style regexes.
//
// This transform is idempotent: it no-ops once the worker already references
// webSearchMarkers(). Run after any data/seed/meanings-web-*.lino change:
//   node experiments/issue-386-web-search-to-meanings.mjs
// Parity with the Rust solver is guarded by
// experiments/issue-386-js-web-search.mjs.

import fs from "node:fs";

const workerPath = new URL("../src/web/formal_ai_worker.js", import.meta.url);
let worker = fs.readFileSync(workerPath, "utf8");

if (worker.includes("function webSearchMarkers()")) {
  console.log("already routed through webSearchMarkers(); nothing to do");
  process.exit(0);
}

function replaceOnce(anchor, replacement, label) {
  const index = worker.indexOf(anchor);
  if (index === -1) throw new Error(`anchor not found: ${label}`);
  if (worker.indexOf(anchor, index + anchor.length) !== -1) {
    throw new Error(`anchor not unique: ${label}`);
  }
  worker = worker.slice(0, index) + replacement + worker.slice(index + anchor.length);
}

// --- 1: replace the seventeen hardcoded arrays with a role/slot projection ----
// The block spans the first array through the last; everything between the
// `const WEB_SEARCH_EXPLICIT_PREFIXES = [` opener and the
// `ENUMERATION_RESEARCH_CONSTRAINT_MARKERS` closer (just before
// `function containsSearchMarker`) is one contiguous run of array literals.
const arraysRe =
  /const WEB_SEARCH_EXPLICIT_PREFIXES = \[[\s\S]*?\n\];\n\n(?=function containsSearchMarker)/;
if (!arraysRe.test(worker)) throw new Error("anchor not found: web-search array block");

const projection = `// Issue #386: every surface cue the web-search recogniser reasons about — the
// explicit command prefixes, the action/source/signal vocabulary, the topic
// connectives, the query noise, the follow-up instruction verbs and clause
// boundaries, and the research/enumeration vocabulary — is sourced from the
// language-independent meaning lexicon (data/seed/meanings-web-search*.lino,
// meanings-web-research.lino, meanings-web-followup.lino, embedded in
// MEANINGS_LINO above). The code references those meanings by their semantic
// *role* and by the *slot* each word form occupies (prefix / suffix / bare),
// never by raw words baked into a per-language list — adding a language or a
// synonym is a pure data edit. Mirrors WebSearchMarkers / markers() in
// src/solver_handlers/web_search_intent.rs.
const ROLE_WEB_SEARCH_EXPLICIT_PREFIX = "web_search_explicit_prefix";
const ROLE_WEB_SEARCH_ACTION = "web_search_action";
const ROLE_WEB_SEARCH_STRONG_ACTION = "web_search_strong_action";
const ROLE_WEB_SEARCH_SIGNAL = "web_search_signal";
const ROLE_WEB_SEARCH_TOPIC_MARKER = "web_search_topic_marker";
const ROLE_WEB_SEARCH_IMPERATIVE_LEAD = "web_search_imperative_lead";
const ROLE_WEB_SEARCH_QUERY_LEADING_NOISE = "web_search_query_leading_noise";
const ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE = "web_search_query_trailing_noise";
const ROLE_WEB_SEARCH_SOURCE_ONLY = "web_search_source_only";
const ROLE_FOLLOWUP_INSTRUCTION_VERB = "followup_instruction_verb";
const ROLE_CLAUSE_CONTINUATION_MARKER = "clause_continuation_marker";
const ROLE_RESEARCH_QUESTION_OPENER = "research_question_opener";
const ROLE_RESEARCH_SUPERLATIVE_MODIFIER = "research_superlative_modifier";
const ROLE_RESEARCH_EVIDENCE_DOMAIN = "research_evidence_domain";
const ROLE_RESEARCH_EVALUATION_DOMAIN = "research_evaluation_domain";
const ROLE_ENUMERATION_REQUEST_OPENER = "enumeration_request_opener";
const ROLE_ENUMERATION_CONSTRAINT = "enumeration_constraint";

// The literal lead-in (form.before, the text before the … slot) of every
// prefix-slot form of a role, in lexicon declaration order. A meaning's roles
// apply to all its forms, so keep only the slot we asked for. Mirrors
// prefix_literals.
function searchPrefixLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "prefix")
    .map((form) => form.before);
}
// The literal tail (form.after) of every suffix-slot form of a role. Mirrors
// suffix_literals.
function searchSuffixLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "suffix")
    .map((form) => form.after);
}
// The surface text of every bare-slot form of a role (drop any prefix/suffix
// surfaces the same meaning also owns). Mirrors bare_literals.
function searchBareLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "bare")
    .map((form) => form.text);
}
// The distinct surface words of a role, trimmed + lowercased for equality
// comparison against a cleaned query. Mirrors source_literals (words_for_role).
function searchSourceLiterals(role) {
  const seen = new Set();
  const out = [];
  for (const form of roleWordForms(role)) {
    const key = form.text.trim().toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      out.push(key);
    }
  }
  return out;
}

// Build (once) the marker projection from the meaning lexicon, then cache it —
// roleWordForms walks the whole lexicon, so memoize like the Rust OnceLock.
let WEB_SEARCH_MARKERS_CACHE = null;
function webSearchMarkers() {
  if (WEB_SEARCH_MARKERS_CACHE) return WEB_SEARCH_MARKERS_CACHE;
  WEB_SEARCH_MARKERS_CACHE = {
    explicitPrefixes: searchPrefixLiterals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
    actionMarkers: searchBareLiterals(ROLE_WEB_SEARCH_ACTION),
    strongActionMarkers: searchBareLiterals(ROLE_WEB_SEARCH_STRONG_ACTION),
    signalMarkers: searchBareLiterals(ROLE_WEB_SEARCH_SIGNAL),
    topicAfterMarkers: searchPrefixLiterals(ROLE_WEB_SEARCH_TOPIC_MARKER),
    topicBeforeMarkers: searchSuffixLiterals(ROLE_WEB_SEARCH_TOPIC_MARKER),
    imperativeLeadMarkers: searchPrefixLiterals(ROLE_WEB_SEARCH_IMPERATIVE_LEAD),
    leadingNoise: searchPrefixLiterals(ROLE_WEB_SEARCH_QUERY_LEADING_NOISE),
    trailingNoise: searchSuffixLiterals(ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE),
    sourceOnly: searchSourceLiterals(ROLE_WEB_SEARCH_SOURCE_ONLY),
    followupVerbs: searchBareLiterals(ROLE_FOLLOWUP_INSTRUCTION_VERB),
    continuationMarkers: searchBareLiterals(ROLE_CLAUSE_CONTINUATION_MARKER),
    researchQuestionPrefixes: searchPrefixLiterals(ROLE_RESEARCH_QUESTION_OPENER),
    researchModifiers: searchBareLiterals(ROLE_RESEARCH_SUPERLATIVE_MODIFIER),
    researchEvidenceDomains: searchBareLiterals(ROLE_RESEARCH_EVIDENCE_DOMAIN),
    researchEvaluationDomains: searchBareLiterals(ROLE_RESEARCH_EVALUATION_DOMAIN),
    enumerationPrefixes: searchPrefixLiterals(ROLE_ENUMERATION_REQUEST_OPENER),
    enumerationConstraintMarkers: searchBareLiterals(ROLE_ENUMERATION_CONSTRAINT),
  };
  return WEB_SEARCH_MARKERS_CACHE;
}

// A request to filter the user's OWN contributed facts ("facts I contributed",
// "my facts") is conversation search, not a web search. Mirrors
// is_personal_fact_filter_request in src/solver_handlers/web_search_intent.rs.
function isPersonalFactFilterRequest(normalized) {
  const text = String(normalized || "");
  return (
    text.includes("facts i have contributed") ||
    text.includes("facts ive contributed") ||
    text.includes("facts i contributed") ||
    text.includes("my facts")
  );
}

`;
worker = worker.replace(arraysRe, projection);

// --- 2: universal follow-up boundary truncation ------------------------------
// Replace the regex-list truncation with the structural algorithm: a follow-up
// clause is a followup_instruction_verb surface sitting immediately after a
// boundary (sentence punctuation or a clause_continuation_marker run).
const oldTruncate = `function truncateSearchInstructionTail(value) {
  let query = String(value || "").trim();
  for (const pattern of SEARCH_QUERY_TRAILING_INSTRUCTION_PATTERNS) {
    query = query.replace(pattern, "").trim();
  }
  return query;
}`;
const newTruncate = `// Sentence-ending punctuation that can introduce a follow-up instruction
// clause — ASCII plus the fullwidth/ideographic forms a CJK prompt uses.
// Mirrors is_sentence_boundary.
const SEARCH_SENTENCE_BOUNDARY = new Set([
  ".",
  "?",
  "!",
  ";",
  ":",
  "\\u3002",
  "\\uff1f",
  "\\uff01",
  "\\uff1b",
  "\\uff1a",
]);

// ASCII-only lowercase: folds A–Z and nothing else, so the result keeps the same
// length (in UTF-16 code units) as the input and computed offsets stay aligned.
// Mirrors Rust str::to_ascii_lowercase (a full toLowerCase could change length,
// e.g. 'İ' -> 'i̇', and misalign the cut offsets).
function asciiLowercase(value) {
  return String(value || "").replace(/[A-Z]/g, (character) =>
    String.fromCharCode(character.charCodeAt(0) + 32),
  );
}

// Is the single Unicode code point \`code\` a letter or number? Mirrors Rust
// char::is_alphanumeric closely enough for token-boundary detection.
function isSearchAlnum(code) {
  return /[\\p{L}\\p{N}]/u.test(String.fromCodePoint(code));
}

// Does \`index\` begin a token in \`text\` (the preceding code point is non-
// alphanumeric, or there is none)? Surrogate-pair aware. Mirrors is_token_start.
function isSearchTokenStart(text, index) {
  if (index <= 0) return true;
  const i = index - 1;
  let code = text.charCodeAt(i);
  if (code >= 0xdc00 && code <= 0xdfff && i > 0) {
    const high = text.charCodeAt(i - 1);
    if (high >= 0xd800 && high <= 0xdbff) {
      code = (high - 0xd800) * 0x400 + (code - 0xdc00) + 0x10000;
    }
  }
  return !isSearchAlnum(code);
}

// Does \`index\` end a token in \`text\` (the following code point is non-
// alphanumeric, or there is none)? Mirrors is_token_end.
function isSearchTokenEnd(text, index) {
  if (index >= text.length) return true;
  return !isSearchAlnum(text.codePointAt(index));
}

// Whether \`haystack\` ends with \`marker\` as a whole token. CJK markers match as
// bare substrings; space-delimited markers need a preceding whitespace (or for
// the whole string to be exactly the marker). Mirrors ends_with_token.
function searchEndsWithToken(haystack, marker) {
  if (containsCjk(marker)) return haystack.endsWith(marker);
  if (haystack === marker) return true;
  if (!haystack.endsWith(marker)) return false;
  const head = haystack.slice(0, haystack.length - marker.length);
  return /\\s$/u.test(head);
}

// If the text immediately before \`verbStart\` is a follow-up boundary, return the
// code-unit offset at which to cut (the start of the boundary run); otherwise
// null. Mirrors boundary_before.
function searchBoundaryBefore(text, verbStart, markers) {
  const head = text.slice(0, verbStart).trimEnd();
  if (head.length === 0) return null;
  if (SEARCH_SENTENCE_BOUNDARY.has(head[head.length - 1])) return head.length;
  // Walk back over a run of clause-continuation markers ("and", "then",
  // "and then"); the cut falls at the start of the run.
  let cursor = head;
  let matched = false;
  for (;;) {
    const trimmed = cursor.trimEnd();
    let rest = null;
    for (const marker of markers.continuationMarkers) {
      if (searchEndsWithToken(trimmed, marker)) {
        rest = trimmed.slice(0, trimmed.length - marker.length);
        break;
      }
    }
    if (rest === null) break;
    cursor = rest;
    matched = true;
  }
  return matched ? cursor.trimEnd().length : null;
}

// Drop a trailing follow-up instruction clause ("… and summarize who won",
// "… . Compare their patents") from a query. A universal boundary algorithm, not
// a list of memorised fragments: a follow-up clause is one of the lexicon's
// followup_instruction_verb surfaces sitting immediately after a *boundary* —
// sentence punctuation or a run of clause_continuation_marker words — and the
// query is cut at the start of the earliest such boundary. A bare verb with no
// boundary before it is part of the topic and left untouched. Mirrors
// truncate_search_instruction_tail in src/solver_handlers/web_search_intent.rs.
function truncateSearchInstructionTail(value) {
  const markers = webSearchMarkers();
  const text = String(value || "");
  // ASCII-lowercase keeps offsets identical to \`text\`; the non-ASCII verbs are
  // already lowercase in the lexicon and unaffected by the fold.
  const lower = asciiLowercase(text);
  let cut = text.length;
  for (const verb of markers.followupVerbs) {
    const cjk = containsCjk(verb);
    let from = 0;
    for (;;) {
      const start = lower.indexOf(verb, from);
      if (start === -1) break;
      const end = start + verb.length;
      from = end;
      // Space-delimited scripts require a whole-token match; CJK verbs have no
      // word boundaries and match as bare substrings.
      if (!cjk && (!isSearchTokenStart(lower, start) || !isSearchTokenEnd(lower, end))) {
        continue;
      }
      const boundary = searchBoundaryBefore(lower, start, markers);
      if (boundary !== null) cut = Math.min(cut, boundary);
    }
  }
  return text.slice(0, cut).trim();
}`;
replaceOnce(oldTruncate, newTruncate, "truncateSearchInstructionTail");

// --- 3: point every recogniser at the projected markers ----------------------
replaceOnce(
  `function extractSemanticWebSearchQuery(prompt, normalized) {
  const hasAction = containsAnySearchMarker(normalized, WEB_SEARCH_ACTION_MARKERS);
  if (!hasAction) return "";
  const hasStrongAction = containsAnySearchMarker(
    normalized,
    WEB_SEARCH_STRONG_ACTION_MARKERS,
  );
  if (!hasStrongAction && !containsAnySearchMarker(normalized, WEB_SEARCH_SIGNAL_MARKERS)) {
    return "";
  }
  for (const marker of SEARCH_QUERY_AFTER_MARKERS) {`,
  `function extractSemanticWebSearchQuery(prompt, normalized) {
  const markers = webSearchMarkers();
  const hasAction = containsAnySearchMarker(normalized, markers.actionMarkers);
  if (!hasAction) return "";
  const hasStrongAction = containsAnySearchMarker(
    normalized,
    markers.strongActionMarkers,
  );
  if (!hasStrongAction && !containsAnySearchMarker(normalized, markers.signalMarkers)) {
    return "";
  }
  for (const marker of markers.topicAfterMarkers) {`,
  "extractSemanticWebSearchQuery head",
);
replaceOnce(
  `  for (const marker of SEARCH_QUERY_BEFORE_MARKERS) {`,
  `  for (const marker of markers.topicBeforeMarkers) {`,
  "extractSemanticWebSearchQuery before-markers",
);
replaceOnce(
  `  for (const marker of SEARCH_ACTION_AFTER_MARKERS) {`,
  `  for (const marker of markers.imperativeLeadMarkers) {`,
  "extractSemanticWebSearchQuery imperative-leads",
);

replaceOnce(
  `function cleanSemanticSearchQuery(value) {
  let query = cleanSearchQuery(truncateSearchInstructionTail(value));
  while (true) {
    const before = query;
    for (const prefix of SEARCH_QUERY_LEADING_NOISE) {
      query = stripSearchNoisePrefix(query, prefix);
    }
    for (const suffix of SEARCH_QUERY_TRAILING_NOISE) {`,
  `function cleanSemanticSearchQuery(value) {
  const markers = webSearchMarkers();
  let query = cleanSearchQuery(truncateSearchInstructionTail(value));
  while (true) {
    const before = query;
    for (const prefix of markers.leadingNoise) {
      query = stripSearchNoisePrefix(query, prefix);
    }
    for (const suffix of markers.trailingNoise) {`,
  "cleanSemanticSearchQuery",
);

replaceOnce(
  `  if (SEARCH_QUERY_SOURCE_ONLY.includes(queryKey)) return "";`,
  `  if (webSearchMarkers().sourceOnly.includes(queryKey)) return "";`,
  "validSearchQuery source-only",
);

replaceOnce(
  `  for (const prefix of IMPLICIT_RESEARCH_QUESTION_PREFIXES) {
    if (text.startsWith(prefix)) {`,
  `  for (const prefix of webSearchMarkers().researchQuestionPrefixes) {
    if (text.startsWith(prefix)) {`,
  "stripImplicitResearchPrefix",
);

replaceOnce(
  `function extractImplicitResearchQuestion(normalized) {
  const text = String(normalized || "");
  if (!startsWithAny(text, IMPLICIT_RESEARCH_QUESTION_PREFIXES)) return "";
  const padded = \` \${text} \`;
  const hasModifier = IMPLICIT_RESEARCH_MODIFIERS.some((marker) =>
    padded.includes(marker),
  );
  const hasEvidenceDomain = IMPLICIT_RESEARCH_EVIDENCE_DOMAINS.some((marker) =>
    padded.includes(marker),
  );
  const hasEvaluationDomain = IMPLICIT_RESEARCH_EVALUATION_DOMAINS.some((marker) =>
    padded.includes(marker),
  );`,
  `function extractImplicitResearchQuestion(normalized) {
  const markers = webSearchMarkers();
  const text = String(normalized || "");
  if (!startsWithAny(text, markers.researchQuestionPrefixes)) return "";
  const padded = \` \${text} \`;
  const hasModifier = markers.researchModifiers.some((marker) =>
    padded.includes(marker),
  );
  const hasEvidenceDomain = markers.researchEvidenceDomains.some((marker) =>
    padded.includes(marker),
  );
  const hasEvaluationDomain = markers.researchEvaluationDomains.some((marker) =>
    padded.includes(marker),
  );`,
  "extractImplicitResearchQuestion",
);

replaceOnce(
  `  for (const prefix of ENUMERATION_RESEARCH_PREFIXES) {`,
  `  for (const prefix of webSearchMarkers().enumerationPrefixes) {`,
  "stripEnumerationResearchPrefix",
);

replaceOnce(
  `  return containsAnySearchMarker(
    normalized,
    ENUMERATION_RESEARCH_CONSTRAINT_MARKERS,
  );`,
  `  return containsAnySearchMarker(
    normalized,
    webSearchMarkers().enumerationConstraintMarkers,
  );`,
  "looksLikeEnumerationResearchQuery",
);

// --- 4: extractWebSearchRequest — projected prefixes + personal-fact guard ----
replaceOnce(
  `  if (
    normalized.startsWith("search conversations ") ||
    normalized.startsWith("search my conversations ") ||
    normalized.startsWith("search my chats ")
  ) {
    return "";
  }
  for (const prefix of WEB_SEARCH_EXPLICIT_PREFIXES) {`,
  `  if (
    normalized.startsWith("search conversations ") ||
    normalized.startsWith("search my conversations ") ||
    normalized.startsWith("search my chats ") ||
    isPersonalFactFilterRequest(normalized)
  ) {
    return "";
  }
  for (const prefix of webSearchMarkers().explicitPrefixes) {`,
  "extractWebSearchRequest guard + prefixes",
);

// --- safety: every legacy array identifier must be gone ----------------------
for (const dead of [
  "WEB_SEARCH_EXPLICIT_PREFIXES",
  "WEB_SEARCH_ACTION_MARKERS",
  "WEB_SEARCH_STRONG_ACTION_MARKERS",
  "WEB_SEARCH_SIGNAL_MARKERS",
  "SEARCH_QUERY_AFTER_MARKERS",
  "SEARCH_QUERY_BEFORE_MARKERS",
  "SEARCH_ACTION_AFTER_MARKERS",
  "SEARCH_QUERY_LEADING_NOISE",
  "SEARCH_QUERY_TRAILING_NOISE",
  "SEARCH_QUERY_SOURCE_ONLY",
  "SEARCH_QUERY_TRAILING_INSTRUCTION_PATTERNS",
  "IMPLICIT_RESEARCH_QUESTION_PREFIXES",
  "IMPLICIT_RESEARCH_MODIFIERS",
  "IMPLICIT_RESEARCH_EVIDENCE_DOMAINS",
  "IMPLICIT_RESEARCH_EVALUATION_DOMAINS",
  "ENUMERATION_RESEARCH_PREFIXES",
  "ENUMERATION_RESEARCH_CONSTRAINT_MARKERS",
]) {
  // Match the dead identifier only as a standalone JS token. Several new ROLE_*
  // constants legitimately contain an old array name as a substring — e.g.
  // ROLE_WEB_SEARCH_QUERY_LEADING_NOISE ⊃ SEARCH_QUERY_LEADING_NOISE — so a bare
  // includes() would false-positive on the very projection that replaced them.
  const standalone = new RegExp(`(?<![A-Za-z0-9_])${dead}(?![A-Za-z0-9_])`);
  if (standalone.test(worker)) throw new Error(`leftover reference to ${dead}`);
}

fs.writeFileSync(workerPath, worker);
console.log("routed web-search recogniser through webSearchMarkers()");
