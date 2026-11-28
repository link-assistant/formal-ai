// Web-search query extraction, procedural how-to answers from wikiHow and the
// frame-policy checks.
// Loaded by ../formal_ai_worker.js.
// Keep implicit client research from replacing the solver's procedure and sources.
function claimsProceduralRequest(prompt, history) {
  const normalized = normalizePrompt(prompt);
  if (extractExplicitWebSearchQuery(prompt) || extractExplicitWebSearchQuery(normalized)) return false;
  if (practicalHelpGoal(prompt) || extractProceduralHowToTask(normalized)) return true;
  const prior = lastHistoryTurn(history, "user");
  if (practicalHelpFollowupGoal(normalized, prior)) return true;
  if (isProceduralElaborationRequest(normalized) && priorProceduralHowToDialogue(history)) return true;
  const parts = splitLeadingGreetingCompoundPrompt(prompt);
  return Boolean(parts && isGreetingPrompt(normalizePrompt(parts.greeting), parts.greeting)
    && extractProceduralHowToTask(normalizePrompt(parts.remainder)));
}

function truncateSearchInstructionTail(value) {
  const markers = webSearchMarkers();
  const text = String(value || "");
  // ASCII-lowercase keeps offsets identical to `text`; the non-ASCII verbs are
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
}

function cleanSemanticSearchQuery(value) {
  const markers = webSearchMarkers();
  let query = cleanSearchQuery(truncateSearchInstructionTail(value));
  while (true) {
    const before = query;
    for (const prefix of markers.leadingNoise) {
      query = stripSearchNoisePrefix(query, prefix);
    }
    for (const suffix of markers.trailingNoise) {
      query = stripSearchNoiseSuffix(query, suffix);
    }
    if (query === before) return query;
  }
}

function validSearchQuery(value) {
  const query = cleanSemanticSearchQuery(value);
  return validCleanSearchQuery(query);
}

function validNewsSearchQuery(value) {
  const query = cleanSearchQuery(truncateSearchInstructionTail(value));
  return validCleanSearchQuery(query);
}

function validCleanSearchQuery(query) {
  const queryKey = query.toLowerCase();
  if (webSearchMarkers().sourceOnly.includes(queryKey)) return "";
  return query && !normalizeUrlCandidate(query) ? query : "";
}

function rawSearchMarkerIndex(prompt, marker) {
  return String(prompt || "").toLowerCase().indexOf(marker);
}

function queryAfterRawMarker(prompt, marker) {
  const text = String(prompt || "").trim();
  const index = rawSearchMarkerIndex(text, marker);
  return index === -1 ? "" : validSearchQuery(text.slice(index + marker.length));
}

function queryBeforeRawMarker(prompt, marker) {
  const text = String(prompt || "").trim();
  const index = rawSearchMarkerIndex(text, marker);
  return index === -1 ? "" : validSearchQuery(text.slice(0, index));
}

function queryAfterNormalizedMarker(normalized, marker) {
  const index = String(normalized || "").indexOf(marker);
  return index === -1 ? "" : validSearchQuery(normalized.slice(index + marker.length));
}

function queryBeforeNormalizedMarker(normalized, marker) {
  const index = String(normalized || "").indexOf(marker);
  return index === -1 ? "" : validSearchQuery(normalized.slice(0, index));
}

function extractSemanticWebSearchQuery(prompt, normalized) {
  const markers = webSearchMarkers();
  const imperativeCandidate = imperativeLeadCandidate(
    normalized,
    markers.imperativeLeadMarkers,
    markers,
  );
  const hasImperativeLead = Boolean(imperativeCandidate);
  const hasAction =
    hasImperativeLead || containsAnySearchMarker(normalized, markers.actionMarkers);
  if (!hasAction) return "";
  const hasStrongAction =
    Boolean(
      imperativeLeadCandidate(
        normalized,
        markers.strongImperativeLeadMarkers,
        markers,
      ),
    ) ||
    containsAnySearchMarker(normalized, markers.strongActionMarkers);
  if (!hasStrongAction && !containsAnySearchMarker(normalized, markers.signalMarkers)) {
    return "";
  }
  for (const marker of markers.topicAfterMarkers) {
    const query =
      queryAfterRawMarker(prompt, marker) ||
      queryAfterNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  for (const marker of markers.topicBeforeMarkers) {
    const query =
      queryBeforeRawMarker(prompt, marker) ||
      queryBeforeNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  return validSearchQuery(imperativeCandidate);
}

// A typed search action may open the prompt, follow a seeded question opener,
// or follow a named external source. Arbitrary mid-sentence verbs remain prose.
function imperativeLeadCandidate(normalized, leads, markers) {
  const text = String(normalized || "");
  for (const lead of leads) {
    if (text.startsWith(lead)) return text.slice(lead.length);
    const index = text.indexOf(lead);
    if (index === -1) continue;
    const introducer = text.slice(0, index);
    const questionLed = startsWithAny(text, markers.researchQuestionPrefixes);
    const sourceLed = containsAnySearchMarker(introducer, markers.sourceMarkers);
    if (questionLed || sourceLed) return text.slice(index + lead.length);
  }
  return "";
}

function questionIsInterrogative(prompt, normalized) {
  return (
    startsWithAny(normalized, webSearchMarkers().researchQuestionPrefixes) ||
    /[?？]\s*$/u.test(String(prompt || ""))
  );
}

function extractTopicSubject(normalized) {
  const markers = webSearchMarkers();
  for (const marker of markers.topicAfterMarkers) {
    const query = queryAfterNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  for (const marker of markers.topicBeforeMarkers) {
    const query = queryBeforeNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  return "";
}

// Semantic frame shared with the Rust handler: interrogative + named external
function extractSourceGroundedQuestion(prompt, normalized) {
  const markers = webSearchMarkers();
  if (
    !questionIsInterrogative(prompt, normalized) ||
    !containsAnySearchMarker(normalized, markers.sourceMediumMarkers)
  ) {
    return "";
  }
  return extractTopicSubject(normalized);
}

// Semantic frame shared with the Rust handler: named external source + recency
// + topic connective.
function extractCurrentSourceInformationRequest(normalized) {
  const markers = webSearchMarkers();
  if (
    !containsAnySearchMarker(normalized, markers.sourceMediumMarkers) ||
    !containsAnySearchMarker(normalized, markers.newsRecencyMarkers) ||
    !containsAnySearchMarker(normalized, markers.informationMarkers)
  ) {
    return "";
  }
  return extractTopicSubject(normalized);
}

function extractExplicitWebSearchQuery(prompt) {
  const markers = webSearchMarkers();
  for (const prefix of markers.explicitPrefixes) {
    const query = stripSearchPrefix(prompt, prefix);
    if (query) return query;
  }
  for (const { before, after } of markers.explicitCircumfixes) {
    const query = stripSearchCircumfix(prompt, before, after);
    if (query) return query;
  }
  for (const suffix of markers.explicitSuffixes) {
    const query = stripSearchSuffix(prompt, suffix);
    if (query) return query;
  }
  return "";
}

function extractLatestNewsSearchRequest(normalized) {
  const markers = webSearchMarkers();
  const text = String(normalized || "");
  if (
    !containsAnySearchMarker(text, markers.newsSubjectMarkers) ||
    !containsAnySearchMarker(text, markers.newsRecencyMarkers)
  ) {
    return "";
  }
  return validNewsSearchQuery(text);
}

// A verbless "records about a subject" request — "financial records for boeing",
function extractRecordsInformationRequest(normalized) {
  const markers = webSearchMarkers();
  const text = String(normalized || "");
  if (!containsAnySearchMarker(text, markers.recordsSubjectMarkers)) {
    return "";
  }
  const hasTopicMarker = markers.topicAfterMarkers
    .concat(markers.topicBeforeMarkers)
    .some((marker) => containsSearchMarker(text, marker));
  if (!hasTopicMarker) {
    return "";
  }
  return validNewsSearchQuery(text);
}

// A question asking which public events are currently active, such as
function extractCurrentPublicEventQuestion(normalized) {
  const markers = webSearchMarkers();
  const text = String(normalized || "");
  if (!startsWithAny(text, markers.researchQuestionPrefixes)) return "";
  if (
    !containsAnySearchMarker(text, markers.publicEventSubjectMarkers) ||
    !containsAnySearchMarker(text, markers.newsRecencyMarkers)
  ) {
    return "";
  }
  return validSearchQuery(stripImplicitResearchPrefix(text));
}

function conceptLookupResolves(prompt) {
  const query = extractConceptQuery(prompt);
  return !!(query && lookupConceptQuery(query));
}

function termInformationPromptIsLocalContext(normalized) {
  const text = String(normalized || "");
  return (
    lexiconMentionsRole(ROLE_SELF_INTRODUCTION_REQUEST, text) ||
    lexiconMentionsRole(ROLE_CAPABILITY_QUERY, text) ||
    lexiconMentionsRole(ROLE_CAPABILITY_QUERY_MORE, text)
  );
}

function termInformationQueryIsLocalContext(query) {
  const text = cleanSearchQuery(query).toLowerCase();
  return (
    lexiconMentionsRole(ROLE_NON_REFERENTIAL_SUBJECT, text) ||
    lexiconMentionsRole(ROLE_ASSISTANT_SELF_REFERENCE, text)
  );
}

function extractTermInformationRequest(prompt, normalized) {
  if (conceptLookupResolves(prompt) || termInformationPromptIsLocalContext(normalized)) {
    return "";
  }
  const text = String(normalized || "");
  const markers = webSearchMarkers();
  // Word order belongs to the language, not to the intent: prefix openers
  const candidates = [
    ...markers.termInformationPrefixes.map((p) => (text.startsWith(p) ? text.slice(p.length) : "")),
    ...markers.termInformationSuffixes.map((s) =>
      s && text.endsWith(s) ? text.slice(0, text.length - s.length) : ""),
    ...markers.termInformationCircumfixes.map(({ before, after }) =>
      after && text.startsWith(before) && text.endsWith(after)
        ? text.slice(before.length, text.length - after.length)
        : ""),
  ];
  for (const candidate of candidates.filter(Boolean)) {
    if (termInformationQueryIsLocalContext(candidate)) return "";
    const query = validSearchQuery(candidate);
    if (query) return query;
  }
  return "";
}

function stripImplicitResearchPrefix(value) {
  const text = String(value || "");
  for (const prefix of webSearchMarkers().researchQuestionPrefixes) {
    if (text.startsWith(prefix)) {
      return text.slice(prefix.length);
    }
  }
  return text;
}

function extractImplicitResearchQuestion(normalized) {
  const markers = webSearchMarkers();
  const text = String(normalized || "");
  if (!startsWithAny(text, markers.researchQuestionPrefixes)) return "";
  const padded = ` ${text} `;
  const hasModifier = markers.researchModifiers.some((marker) =>
    padded.includes(marker),
  );
  const hasEvidenceDomain = markers.researchEvidenceDomains.some((marker) =>
    padded.includes(marker),
  );
  const hasEvaluationDomain = markers.researchEvaluationDomains.some((marker) =>
    padded.includes(marker),
  );
  if (!hasModifier && !(hasEvidenceDomain && hasEvaluationDomain)) return "";
  return validSearchQuery(stripImplicitResearchPrefix(text));
}

function stripEnumerationResearchPrefix(value) {
  const text = String(value || "").trim();
  const lower = text.toLowerCase();
  for (const prefix of webSearchMarkers().enumerationPrefixes) {
    if (lower.startsWith(prefix)) {
      return cleanSearchQuery(text.slice(prefix.length));
    }
  }
  return "";
}

function looksLikeEnumerationResearchQuery(query) {
  const normalized = normalizePrompt(query);
  if (normalized.split(/\s+/u).filter(Boolean).length < 3) return false;
  return containsAnySearchMarker(
    normalized,
    webSearchMarkers().enumerationConstraintMarkers,
  );
}

function extractEnumerationResearchRequest(prompt, normalized) {
  const rawQuery = stripEnumerationResearchPrefix(prompt);
  if (rawQuery && looksLikeEnumerationResearchQuery(rawQuery)) {
    return validSearchQuery(rawQuery);
  }
  const normalizedQuery = stripEnumerationResearchPrefix(normalized);
  return normalizedQuery && looksLikeEnumerationResearchQuery(normalizedQuery)
    ? validSearchQuery(normalizedQuery)
    : "";
}

function extractWebSearchRequest(prompt, normalized) {
  if (
    (["local_path_scope_desktop", "local_path_scope_home", "local_path_scope_current"].some((role) => lexiconMentionsRole(role, normalized)) &&
      ["local_path_search_action", "local_path_list_action", "local_path_contents_request", "local_path_type_request", "local_path_route_question"].some((role) => lexiconMentionsRole(role, normalized) || roleWordForms(role).some((form) => surfacePresent(normalized, normalizePrompt(form.before || form.after))))) ||
    normalized.startsWith("search conversations ") ||
    normalized.startsWith("search my conversations ") ||
    normalized.startsWith("search my chats ") ||
    isPersonalFactFilterRequest(normalized)
  ) return "";
  const explicitQuery =
    extractExplicitWebSearchQuery(prompt) || extractExplicitWebSearchQuery(normalized);
  if (explicitQuery) {
    return { query: explicitQuery, kind: "explicit_prefix" };
  }
  const semanticQuery = extractSemanticWebSearchQuery(prompt, normalized);
  if (semanticQuery) {
    return { query: semanticQuery, kind: "semantic_action" };
  }
  if (handlerRuleSetClaims("docs_method_explanation", prompt, normalized)) return "";
  const sourceQuestion = extractSourceGroundedQuestion(prompt, normalized);
  if (sourceQuestion) {
    return { query: sourceQuestion, kind: "implicit_research_question" };
  }
  const currentSourceQuery = extractCurrentSourceInformationRequest(normalized);
  if (currentSourceQuery) {
    return { query: currentSourceQuery, kind: "implicit_research_question" };
  }
  const latestNewsQuery = extractLatestNewsSearchRequest(normalized);
  if (latestNewsQuery) {
    return { query: latestNewsQuery, kind: "latest_news" };
  }
  const recordsQuery = extractRecordsInformationRequest(normalized);
  if (recordsQuery) {
    return { query: recordsQuery, kind: "records_information_request" };
  }
  const enumerationQuery = extractEnumerationResearchRequest(prompt, normalized);
  if (enumerationQuery) {
    return { query: enumerationQuery, kind: "enumeration_research_request" };
  }
  const currentEventQuery = extractCurrentPublicEventQuestion(normalized);
  if (currentEventQuery) {
    return { query: currentEventQuery, kind: "implicit_research_question" };
  }
  const termInformationQuery = extractTermInformationRequest(prompt, normalized);
  if (termInformationQuery) {
    return { query: termInformationQuery, kind: "implicit_research_question" };
  }
  const researchQuery = extractImplicitResearchQuestion(normalized);
  return researchQuery
    ? { query: researchQuery, kind: "implicit_research_question" }
    : null;
}

function extractWebSearchQuery(prompt, normalized) {
  const request = extractWebSearchRequest(prompt, normalized);
  return request ? request.query : "";
}

// Mirrors is_unresolved_bare_term_prompt in src/solver_unknown_reasoning.rs.
function extractUnresolvedBareTermSearchQuery(prompt) {
  const trimmed = cleanBareTermSearchFocus(prompt);
  if (!trimmed) return "";
  const normalized = normalizePrompt(trimmed);
  if (normalized.split(/\s+/u).filter(Boolean).length !== 1) return "";
  const characters = Array.from(trimmed);
  const hasLetter = characters.some((character) => /\p{L}/u.test(character));
  const enoughSurface =
    characters.length >= 2 || characters.some((character) => character.charCodeAt(0) > 0x7f);
  return hasLetter && enoughSurface ? trimmed : "";
}

function cleanBareTermSearchFocus(value) {
  return String(value || "")
    .trim()
    .replace(/^[`"'\u201c\u201d\u2018\u2019\u00ab\u00bb]+/u, "")
    .replace(/[`"'\u201c\u201d\u2018\u2019\u00ab\u00bb]+$/u, "")
    .replace(/[?!.。,;:]+$/u, "")
    .trim();
}

function cleanProceduralFragment(value) {
  let clean = String(value || "")
    .trim()
    .replace(/^[`"' ]+/u, "")
    .replace(/[`"' ]+$/u, "")
    .replace(/[?!.,;:]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
  // The trailing "step by step" / politeness modifiers are the slot-marked
  for (const form of roleWordForms(ROLE_PROCEDURAL_TASK_MODIFIER)) {
    if (clean.endsWith(form.after)) {
      clean = clean.slice(0, clean.length - form.after.length).trim();
      break;
    }
  }
  return clean;
}

function correctCommonProceduralTypos(task) {
  // The misspelling -> correction pairs are the common_typo meaning's bare
  const typos = roleWordForms(ROLE_COMMON_TYPO);
  const corrections = [];
  const corrected = String(task || "")
    .split(/\s+/u)
    .filter(Boolean)
    .map((token) => {
      for (const form of typos) {
        if (token === form.text) {
          if (!corrections.some((correction) => correction.from === form.text)) {
            corrections.push({ from: form.text, to: form.action });
          }
          return form.action;
        }
      }
      return token;
    })
    .join(" ");
  return { task: corrected, corrections };
}

function splitProceduralActionObject(task) {
  const text = String(task || "").trim();
  if (!text) return null;
  const firstSpace = text.search(/\s/u);
  const action = firstSpace === -1 ? text : text.slice(0, firstSpace);
  const object = firstSpace === -1 ? "" : text.slice(firstSpace + 1).trim();
  return action ? { action, object } : null;
}

function splitKnownProceduralActionObject(task) {
  const forms = roleWordForms(ROLE_PROCEDURAL_ACTION_VERB)
    .slice()
    .sort((left, right) => right.text.length - left.text.length);
  const text = String(task || "").trim();
  for (const form of forms) {
    const actionSurface = String(form.text || "").trim();
    if (!actionSurface || !text.startsWith(actionSurface)) continue;
    const rest = text.slice(actionSurface.length);
    if (
      rest &&
      !/^\s/u.test(rest) &&
      !containsCjk(actionSurface)
    ) {
      continue;
    }
    return {
      action: form.action || actionSurface,
      object: cleanProceduralFragment(rest),
    };
  }
  return null;
}

function extractElidedProceduralHowToTask(clean) {
  // An elided lead ("как …") only guesses a procedure from an
  // infinitive-shaped word, and an opinion lead shares that word: "как ты
  // думаешь, есть ли свобода воли" asks for an opinion, and "есть" merely
  // ends like an infinitive. The seeded opinion lead outranks the guess.
  if (handlerRuleSetClaims("opinion_question", clean, clean.toLowerCase())) return null;
  // Issue #481: telegraphic English prompts can omit the connector in
  for (const form of roleWordForms(ROLE_PROCEDURAL_REQUEST_ELIDED_LEAD)) {
    if (!clean.startsWith(form.before)) continue;
    const correction = correctCommonProceduralTypos(
      cleanProceduralFragment(clean.slice(form.before.length)),
    );
    const task = correction.task;
    const split = splitKnownProceduralActionObject(task);
    if (split?.object) return { task, ...split, corrections: correction.corrections };
    const language = detectLanguageFromRules(clean);
    const endings = (handlerRulesPolicy("procedural_how_to", `infinitive-endings-${language}`) || "").split(/\s+/u).filter(Boolean);
    const words = task.split(/[^\p{L}\p{M}]+/u).filter(Boolean);
    const verb = words.find((word) => endings.some((ending) => word.length > ending.length + 1 && word.endsWith(ending)));
    if (verb) return { task, action: verb, object: words.filter((word) => word !== verb).join(" "), corrections: correction.corrections };
  }
  return null;
}

function extractProceduralHowToTask(normalized) {
  // The prefixes are the slot-marked surface forms of the procedural_request
  const clean = cleanProceduralFragment(normalized);
  for (const form of roleWordForms(ROLE_PROCEDURAL_REQUEST)) {
    if (!clean.startsWith(form.before)) continue;
    const correction = correctCommonProceduralTypos(
      cleanProceduralFragment(clean.slice(form.before.length)),
    );
    const task = correction.task;
    if (!task) return null;
    const actionOverride = form.action || null;
    if (actionOverride) {
      return {
        task,
        action: actionOverride,
        object: task,
        corrections: correction.corrections,
      };
    }
    const split = splitProceduralActionObject(task);
    if (!split) return null;
    return {
      task,
      action: split.action,
      object: split.object,
      corrections: correction.corrections,
    };
  }
  return extractElidedProceduralHowToTask(clean);
}

// Issue #444: external *trusted* services are opt-out. A preference value of
function externalServiceEnabled(preferences, key) {
  return !(preferences && preferences[key] === false);
}

// Issue #918: the search-query frames and every line of the procedural plan
function howFillOnce(template, values) {
  return String(template || "").replace(/\{([^{}]*)\}/gu, (whole, name) =>
    Object.prototype.hasOwnProperty.call(values, name) ? String(values[name]) : whole);
}

function howPolicy(handler, key, values) {
  return howFillOnce(handlerRulesPolicy(handler, key), values);
}

function howResponse(intent, language, values = {}) {
  return howFillOnce(answerFor(intent, language), values);
}

function proceduralFallbackQuery(task) {
  return howPolicy("procedural_how_to", "fallback-query", { task });
}

function proceduralSearchQuery(task) {
  const fallbackQuery = proceduralFallbackQuery(task.task);
  if (!task || task.action !== "install") return fallbackQuery;
  const target = String(task.object || task.task || "").trim();
  return howPolicy("procedural_how_to", "install-query", { target: target || task.task }).trim();
}

// Typed request and source-routing records shared with the native solver.
function proceduralDiscoveryEvents(task, pageTitle, apiUrl, searchQuery, fallbackQuery, wikihowEnabled, wikihowAttempted) {
  const request = task.action === "do" && task.object ? task.object : task.task;
  const pairs = [["procedural_how_to:request", request], ["procedural_how_to:action", task.action]];
  if (task.object) pairs.push(["procedural_how_to:object", task.object]);
  if (task.action === "install") pairs.push(["procedural_how_to:stage", "official_documentation"],
    ["procedural_how_to:source_gate", "official_documentation_first"]);
  pairs.push(["procedural_how_to:stage", "wikipedia"], ["procedural_how_to:stage", "wikidata"]);
  if (wikihowAttempted) pairs.push(["procedural_how_to:stage", "wikihow_api"],
    ["procedural_how_to:wikihow_candidate", pageTitle], ["http_fetch:request", apiUrl]);
  if (!wikihowEnabled) pairs.push(["procedural_how_to:service_disabled", "wikihow"]);
  pairs.push(["procedural_how_to:stage", "web_search"], ["web_search:request", searchQuery]);
  if (task.action === "install" && searchQuery !== fallbackQuery) pairs.push(["web_search:request", fallbackQuery]);
  for (const provider of WEB_SEARCH_PROVIDERS) pairs.push(["web_search:provider_planned", provider.id]);
  pairs.push(["web_search:fusion_planned", `rrf:k=${webSearchRrfK()}`],
    ["procedural_how_to:stage", "recursive_fetch_check"], ["procedural_how_to:source_gate", "explicit_steps_only"]);
  return pairs.map(([kind, payload]) => ({ kind, payload: String(payload) }));
}

// Recover wikiHow procedures through the public Wayback availability API and
// the existing web-capture adapter when the browser cannot read the HTML.
const HOW_TO_CAPTURE_TIMEOUT_MS = 3500;
const HOW_TO_CAPTURE_MAXIMUM_BYTES = 2 * 1024 * 1024;

function howToWikiHowSlug(task) {
  return String(task || "").toLowerCase().split(/[^\p{L}\p{N}\p{M}]+/u).filter(Boolean).join("-");
}

async function howToWithinCaptureDeadline(read) {
  const controller = new AbortController();
  let timer;
  try {
    const deadline = new Promise((_, reject) => {
      timer = setTimeout(() => {
        reject(new Error("capture_deadline"));
        controller.abort();
      }, HOW_TO_CAPTURE_TIMEOUT_MS);
    });
    return await Promise.race([read(controller.signal), deadline]);
  } finally {
    clearTimeout(timer);
  }
}

async function howToReadBoundedResponse(response, signal) {
  signal?.throwIfAborted();
  if (!response.body?.getReader) {
    const text = await response.text();
    signal?.throwIfAborted();
    if (new TextEncoder().encode(text).length > HOW_TO_CAPTURE_MAXIMUM_BYTES) {
      throw new Error("capture_too_large");
    }
    return text;
  }
  const reader = response.body.getReader();
  const cancel = () => { Promise.resolve(reader.cancel(signal?.reason)).catch(() => {}); };
  signal?.addEventListener("abort", cancel, { once: true });
  const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
  let bytes = 0;
  let emptyChunks = 0;
  let text = "";
  try {
    for (let reads = 0; reads < 8192; reads++) {
      const { done, value } = await reader.read();
      signal?.throwIfAborted();
      if (done) return text + decoder.decode();
      emptyChunks = value.byteLength === 0 ? emptyChunks + 1 : 0;
      bytes += value.byteLength;
      if (bytes > HOW_TO_CAPTURE_MAXIMUM_BYTES || emptyChunks > 16) {
        cancel();
        throw new Error(emptyChunks > 16 ? "capture_stalled" : "capture_too_large");
      }
      text += decoder.decode(value, { stream: true });
    }
    cancel();
    throw new Error("capture_chunk_limit");
  } finally {
    signal?.removeEventListener("abort", cancel);
    reader.releaseLock?.();
  }
}

async function howToCaptureThroughComponent(url, notes) {
  const component = self.FormalAIWebSearchComponent;
  if (!component?.fetchUrl || component.package === "unavailable") return null;
  try {
    const text = await howToWithinCaptureDeadline(async (signal) => {
      const response = await component.fetchUrl(url, notes, HOW_TO_CAPTURE_TIMEOUT_MS);
      if (!response?.ok) throw new Error(`http_${response?.status || 0}`);
      return howToReadBoundedResponse(response, signal);
    });
    return { url, text, sha256: await sourceWalkSha256Hex(text), fetchedAt: String(sourceWalkNowSeconds()), cached: false };
  } catch (error) {
    notes.push(`capture_failed url=${url} reason=${error.message}`);
    return null;
  }
}

async function tryProceduralHowTo(prompt, language, preferences = {}) {
  const practical = await tryPracticalHelp(prompt,language,preferences);
  if(practical)return practical;
  const normalized = normalizePrompt(prompt);
  const task = extractProceduralHowToTask(normalized);
  if (!task) return null;

  const sourceLanguage = detectLanguageFromRules(prompt);
  const research = await researchProceduralHowTo(task, preferences, sourceLanguage, language);
  if (research.answer) return research.answer;
  const query = proceduralFallbackQuery(task.task);
  const searchQuery = proceduralSearchQuery(task);
  const wiki = howToSourceRegistry().find((record) => record.id === "wikihow");
  const apiUrl = wiki ? (sourceWalkEntryUrl(wiki, task.task, sourceLanguage) || sourceWalkEntryUrl(wiki, task.task)) : "";
  const pageTitle = apiUrl ? new URL(apiUrl).searchParams.get("page") : "";
  const isInstall = task.action === "install";
  const wikihowEnabled = externalServiceEnabled(preferences, "externalServiceWikihow");
  const wikiOutcome = research.guide.outcomes.find((outcome) => outcome.sourceId === "wikihow");
  const wikiAttempted = Boolean(wikiOutcome?.pages || (wikiOutcome
    && !["disabled", "unreachable_cached", "offline_cache_miss", "unbound_template"].includes(wikiOutcome.status)));
  const solverEvents = proceduralDiscoveryEvents(task, pageTitle, apiUrl, searchQuery, query, wikihowEnabled, wikiAttempted);
  const evidence = [
    ...solverEvents.map(({ kind, payload }) => `${kind}:${payload}`),
    ...research.evidence.filter(item => !item.startsWith("how_to:insufficient_evidence ")),
    ...(task.corrections || []).map(({ from, to }) => `spelling_correction:${from}->${to}`),
  ];

  const fallbackQuery = isInstall ? query : searchQuery;
  const official = isInstall
    ? await runWebSearchQuery(searchQuery, sourceLanguage, "official_documentation", preferences) : null;
  const indexed = await runWebSearchQuery(fallbackQuery, sourceLanguage, isInstall ? "general_how_to_fallback" : "", preferences);
  for (const result of [official, indexed]) {
    if (result) evidence.push(...result.evidence.filter((item) => !evidence.includes(item)));
  }
  const discovered = [official, indexed].flatMap(result => result?.diagnostics?.fused || []);
  const candidates = indexedProcedureCandidates(searchQuery, sourceLanguage, discovered);
  const before = research.guide.outcomes.length;
  const accepted = await indexedProcedureGuide(research.guide, candidates, sourceLanguage, preferences);
  solverEvents.push(...howToGuideEvents(research.guide));
  for (const outcome of research.guide.outcomes.slice(before)) {
    evidence.push(`how_to:source ${howToOutcomeTracePayload(outcome)}`);
  }
  if (accepted) {
    evidence.push(`source:${research.guide.steps[0].sourceUrl}`);
    for (const step of research.guide.steps) evidence.push(`how_to:step ${howToStepProvenance(step)}`);
    return { intent: "procedural_how_to", content: howToGuideMarkdown(research.guide, language, false),
      solverEvents, confidence: 0.8, evidence, guide: research.guide,
      diagnostics: indexed?.diagnostics || official?.diagnostics || null,
      query, wikihowCandidate: pageTitle, formalizedObject: `HOWTO:${task.task}` };
  }
  const content = howResponse("procedural_how_to_unverified", language, { task: task.task });
  evidence.push(...research.evidence.filter(item => item.startsWith("how_to:insufficient_evidence ")));
  return {
    intent: "procedural_how_to", content, solverEvents, confidence: 0.78, evidence,
    diagnostics: indexed?.diagnostics || official?.diagnostics || null,
    query, wikihowCandidate: pageTitle, formalizedObject: `HOWTO:${task.task}`,
  };
}

function splitLeadingGreetingCompoundPrompt(prompt) {
  const source = String(prompt || "").trim();
  const match = source.match(/^([\s\S]+?)(?:[,;；，、\n。！？]|[.!?]\s+)([\s\S]+)$/u);
  if (!match) return null;
  const greeting = String(match[1] || "").trim();
  const remainder = stripLeadingCompoundCoordinator(match[2] || "");
  if (!greeting || !remainder) return null;
  return { greeting, remainder };
}

function stripLeadingCompoundCoordinator(text) {
  const trimmed = String(text || "").trim();
  const lowered = trimmed.toLowerCase();
  for (const coordinator of ["and", "then"]) {
    if (lowered === coordinator) return "";
    const prefix = `${coordinator} `;
    if (lowered.startsWith(prefix)) {
      return trimmed.slice(prefix.length).trimStart();
    }
  }
  return trimmed;
}

async function tryGreetingProceduralCompound(prompt, language, preferences = {}) {
  const parts = splitLeadingGreetingCompoundPrompt(prompt);
  if (!parts) return null;
  if (!isGreetingPrompt(normalizePrompt(parts.greeting), parts.greeting)) return null;

  const procedureLanguage = detectLanguage(parts.remainder);
  const procedure = await tryProceduralHowTo(parts.remainder, procedureLanguage, preferences);
  if (!procedure) return null;

  const greetingLanguage = detectLanguage(parts.greeting) || language;
  const temperature = numericPreference(preferences.temperature, 0.7, 0, 1);
  const randomize = preferences.greetingVariations !== false && temperature > 0;
  const greetingEvidence = [
    "rule:greeting",
    `language:${greetingLanguage}`,
    `variation:${randomize ? "random" : "canonical"}`,
    `temperature:${temperature.toFixed(2)}`,
  ];
  const evidence = [
    "composition:compound_response",
    `sub_impulse:${parts.greeting}`,
    `sub_impulse:${parts.remainder}`,
    "sub_intent:greeting",
    `sub_intent:${procedure.intent}`,
    ...greetingEvidence,
    ...(procedure.evidence || []),
  ];

  const greeting = answerFor("greeting", greetingLanguage, { randomize });
  const childEvents = [];
  const formalization = crateModule("crate/intent_formalization.mjs");
  formalization.recordIntentFormalization(childEvents,
    formalization.formalizeIntentRecord(parts.remainder, procedureLanguage));
  const registry = crateModule("crate/method_registry.mjs");
  const method = registry.methodForRoute(registry.recordMethodRegistry(childEvents), procedure.intent);
  if (method) childEvents.push({ kind: "method", payload: method.name });
  return {
    intent: "compound_response",
    content: `${greeting}\n\n${procedure.content}`,
    solverEvents: [
      { kind: "sub_impulse", payload: parts.greeting }, { kind: "sub_impulse", payload: parts.remainder },
      { kind: "sub_result", payload: `independent=true intent=greeting answer=${greeting}` },
      ...childEvents, ...(procedure.solverEvents || []),
      { kind: "sub_result", payload: `independent=true intent=${procedure.intent} answer=${procedure.content}` },
      { kind: "composition:compound_response", payload: "parts=2" },
    ],
    confidence: Math.min(0.9, procedure.confidence || 0.78),
    evidence,
    diagnostics: procedure.diagnostics || null,
    query: procedure.query || "",
    wikihowCandidate: procedure.wikihowCandidate || "",
    formalizedObject: procedure.formalizedObject || "",
    procedureLanguage,
    procedurePrompt: parts.remainder,
    trace: [
      `sub_impulse:${parts.greeting}`,
      `sub_impulse:${parts.remainder}`,
      "composition:compound_response",
    ],
  };
}

// Recognise a request for the concrete steps of an active procedure by
function isProceduralElaborationRequest(normalized) {
  return lexiconMentionsRole(ROLE_PROCEDURAL_ELABORATION, normalized);
}

// The prior exchange must have been a how-to procedure: the previous user turn
function priorProceduralHowToDialogue(history) {
  const assistant = lastHistoryTurn(history, "assistant");
  if (!assistant) return null;
  const user = lastHistoryTurn(history, "user");
  if (!user) return null;
  const goal=practicalHelpGoal(user);
  const task = extractProceduralHowToTask(normalizePrompt(user)) || (goal?{task:goal}:null);
  return task ? { user, task } : null;
}

// Issue #444: a bare follow-up such as "Can you give me specific instructions?"
async function tryProceduralHowToFollowup(prompt, language, history = [], preferences = {}) {
  const practical = await tryPracticalHelpFollowup(prompt,language,history,preferences);
  if(practical)return practical;
  const canonical = normalizePrompt(prompt);
  if (!isProceduralElaborationRequest(canonical)) return null;
  const dialogue = priorProceduralHowToDialogue(history);
  if (!dialogue) return null;
  const procedure = await tryProceduralHowTo(dialogue.user, language, preferences);
  if (!procedure) return null;
  // Front-load the follow-up evidence so the rebind is visible in the trace,
  // matching the log.append order on the Rust side.
  procedure.solverEvents = [
    { kind: "procedural_how_to:followup", payload: canonical },
    { kind: "procedural_how_to:followup_task", payload: dialogue.task.task },
    ...(procedure.solverEvents || []),
  ];
  procedure.evidence = [
    `procedural_how_to:followup:${canonical}`,
    `procedural_how_to:followup_task:${dialogue.task.task}`,
    ...procedure.evidence,
  ];
  return procedure;
}

function stripHtml(value) {
  return String(value || "")
    .replace(/<[^>]*>/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function wikipediaPageUrl(language, key) {
  const lang = language && WIKIPEDIA_SEARCH_HOSTS[language] ? language : "en";
  const slug = encodeURIComponent(String(key || "")).replace(/%2F/gi, "/");
  return `https://${lang}.wikipedia.org/wiki/${slug}`;
}

async function searchWikipediaPages(query, language, limit) {
  if (typeof fetch !== "function") return null;
  const apiHeaders = {
    accept: "application/json",
    "api-user-agent":
      "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
  };
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const base = WIKIPEDIA_SEARCH_HOSTS[lang] || WIKIPEDIA_SEARCH_HOSTS.en;
    const url = `${base}?q=${encodeURIComponent(query)}&limit=${limit || 5}`;
    try {
      const response = await fetch(url, { headers: apiHeaders });
      if (!response || !response.ok) continue;
      const data = await response.json();
      if (!data || !Array.isArray(data.pages) || data.pages.length === 0) {
        continue;
      }
      return {
        language: lang,
        pages: data.pages.slice(0, limit || 5).map((page) => ({
          title: String(page.title || page.key || "Untitled"),
          url: wikipediaPageUrl(lang, page.key || page.title || ""),
          excerpt: stripHtml(page.excerpt || page.description || ""),
        })),
      };
    } catch (_error) {
      // Try the next language host.
    }
  }
  return null;
}

const FRAME_POLICY_CHECK_ENDPOINT = "https://api.microlink.io/";

function framePolicyCheckUrl(url) {
  const params = new URLSearchParams({ url });
  return `${FRAME_POLICY_CHECK_ENDPOINT}?${params.toString()}`;
}

function currentEmbedderOrigin() {
  try {
    const origin = self && self.location && self.location.origin;
    return origin && origin !== "null" ? origin : "";
  } catch (_error) {
    return "";
  }
}

function isPrivateOrLocalHostname(hostname) {
  const host = String(hostname || "").toLowerCase();
  if (
    !host ||
    host === "localhost" ||
    host.endsWith(".localhost") ||
    host.endsWith(".local")
  ) {
    return true;
  }
  if (host === "::1" || host === "[::1]") {
    return true;
  }
  const parts = host.split(".");
  if (parts.length !== 4 || parts.some((part) => !/^\d+$/.test(part))) {
    return false;
  }
  const octets = parts.map((part) => Number(part));
  if (octets.some((part) => part < 0 || part > 255)) return false;
  const [first, second] = octets;
  return (
    first === 10 ||
    first === 127 ||
    (first === 172 && second >= 16 && second <= 31) ||
    (first === 192 && second === 168) ||
    (first === 169 && second === 254)
  );
}

function isPublicHttpUrl(url) {
  try {
    const parsed = new URL(url);
    return (
      (parsed.protocol === "http:" || parsed.protocol === "https:") &&
      !isPrivateOrLocalHostname(parsed.hostname)
    );
  } catch (_error) {
    return false;
  }
}

function normalizeFramePolicyHeaders(headers) {
  const normalized = {};
  for (const [key, value] of Object.entries(headers || {})) {
    const name = String(key || "").toLowerCase();
    if (name !== "x-frame-options" && name !== "content-security-policy") {
      continue;
    }
    normalized[name] = Array.isArray(value)
      ? value.map((item) => String(item || "")).join(", ")
      : String(value || "");
  }
  return normalized;
}

function frameAncestorsSourceSets(csp) {
  const sourceSets = [];
  for (const policy of String(csp || "").split(",")) {
    for (const directive of policy.split(";")) {
      const trimmed = directive.trim();
      if (!/^frame-ancestors(?:\s|$)/i.test(trimmed)) continue;
      const sources = trimmed
        .replace(/^frame-ancestors/i, "")
        .trim()
        .split(/\s+/)
        .filter(Boolean);
      sourceSets.push(sources);
    }
  }
  return sourceSets;
}

function sourceExpressionMatches(source, targetUrl, embedderUrl) {
  const token = String(source || "").trim().toLowerCase();
  if (!token || token === "'none'") return false;
  if (token === "*") return true;
  if (token === "'self'") return embedderUrl.origin === targetUrl.origin;
  if (/^[a-z][a-z0-9+.-]*:$/.test(token)) {
    return embedderUrl.protocol === token;
  }

  let candidate = token;
  if (!candidate.includes("://")) {
    candidate = `${targetUrl.protocol}//${candidate}`;
  }
  let parsed;
  try {
    parsed = new URL(candidate);
  } catch (_error) {
    return false;
  }
  if (parsed.protocol !== embedderUrl.protocol) return false;
  if (parsed.port && parsed.port !== "*" && parsed.port !== embedderUrl.port) {
    return false;
  }
  const host = parsed.hostname.toLowerCase();
  const embedderHost = embedderUrl.hostname.toLowerCase();
  if (host.startsWith("*.")) {
    const suffix = host.slice(2);
    return embedderHost.endsWith(`.${suffix}`);
  }
  return embedderHost === host;
}
