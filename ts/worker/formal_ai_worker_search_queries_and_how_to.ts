// Web-search query extraction, procedural how-to answers from wikiHow and the
// frame-policy checks.
// Loaded by ../formal_ai_worker.js.
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
  // Issue #481: telegraphic English prompts can omit the connector in
  for (const form of roleWordForms(ROLE_PROCEDURAL_REQUEST_ELIDED_LEAD)) {
    if (!clean.startsWith(form.before)) continue;
    const correction = correctCommonProceduralTypos(
      cleanProceduralFragment(clean.slice(form.before.length)),
    );
    const task = correction.task;
    const split = splitKnownProceduralActionObject(task);
    if (!split || !split.object) continue;
    return {
      task,
      action: split.action,
      object: split.object,
      corrections: correction.corrections,
    };
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

function capitalizeForWikiHow(word) {
  const text = String(word || "");
  if (!text) return "";
  return text.charAt(0).toUpperCase() + text.slice(1);
}

function wikiHowPageTitle(task) {
  return String(task || "")
    .split(/[^\p{L}\p{N}]+/u)
    .filter(Boolean)
    .map(capitalizeForWikiHow)
    .join("-");
}

function wikiHowParseApiUrl(pageTitle, encodePage = true) {
  const encodedPage = encodePage ? encodeURIComponent(pageTitle).replace(/%2D/gi, "-") : pageTitle;
  return `https://www.wikihow.com/api.php?action=parse&page=${encodedPage}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
}

function decodeBasicHtmlEntities(value) {
  return String(value || "")
    .replace(/&nbsp;|&#160;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#039;|&apos;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#(\d+);/g, (_match, code) => {
      const value = Number(code);
      if (!Number.isFinite(value) || value < 0 || value > 0x10ffff) return "";
      return String.fromCodePoint(value);
    });
}

function compactStepText(value) {
  const text = decodeBasicHtmlEntities(stripHtml(value))
    .replace(/\[[0-9]+\]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (text.length <= 180) return text;
  const sentence = text.match(/^(.{40,180}?[.!?])\s/u);
  if (sentence) return sentence[1].trim();
  return `${text.slice(0, 177).trim()}...`;
}

function extractWikiHowSteps(html) {
  const lines = String(html || "").split(/\n+/u);
  const steps = [];
  const seen = new Set();
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed.startsWith("<li>") || trimmed.startsWith("<li><b>")) {
      continue;
    }
    const text = compactStepText(trimmed);
    if (text.length < 40 || seen.has(text)) continue;
    seen.add(text);
    steps.push(text);
    if (steps.length >= 6) break;
  }
  return steps;
}

async function fetchWikiHowProcedure(pageTitle, evidence) {
  const url = wikiHowParseApiUrl(pageTitle);
  if (typeof fetch !== "function") {
    return { ok: false, url, error: "fetch_unavailable", steps: [] };
  }
  try {
    const response = await fetch(url, { method: "GET", mode: "cors" });
    evidence.push(`http_fetch:status:${response.status}`);
    if (!response.ok) {
      return { ok: false, url, error: `http_${response.status}`, steps: [] };
    }
    const data = await response.json();
    if (data && data.error) {
      return {
        ok: false,
        url,
        error: data.error.code || "wikihow_error",
        steps: [],
      };
    }
    const parse = data && data.parse ? data.parse : null;
    const html = parse && parse.text ? parse.text["*"] : "";
    const steps = extractWikiHowSteps(html);
    const title = compactStepText(parse && parse.displaytitle ? parse.displaytitle : pageTitle);
    const sourceUrl = `https://www.wikihow.com/${encodeURIComponent(pageTitle).replace(/%2D/gi, "-")}`;
    return {
      ok: steps.length > 0,
      url,
      title: title || pageTitle,
      sourceUrl,
      error: steps.length > 0 ? "" : "no_explicit_steps",
      steps,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    evidence.push(`http_fetch:error:${message.toLowerCase().includes("cors") ? "cors" : "network"}`);
    return { ok: false, url, error: message || "network", steps: [] };
  }
}

function appendUniqueEvidence(target, source) {
  const seen = new Set(target);
  for (const item of source || []) {
    if (!item || seen.has(item)) continue;
    seen.add(item);
    target.push(item);
  }
}

// The docs_method_explanation rule of data/seed/handler-rules.lino: its claim
function tryDocsMethodExplanation(prompt) {
  const hit = runHandlerRuleSet("docs_method_explanation", prompt, String(prompt || "").toLowerCase(), []);
  if (!hit) return null;
  const method = claimOperandDocumentedMethod(prompt)[0] || "";
  return { ...hit, formalizedObject: method };
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

// Native discovery plan over the actual parsed task and declared source route.
function proceduralDiscoveryPlan(task, language, pageTitle, searchQuery, fallbackQuery, providers) {
  const installGate = task.action === "install" ? howResponse("procedural_how_to_install_gate", language,
    { search_query: searchQuery, fallback_query: fallbackQuery }) : "";
  return howResponse("procedural_how_to_plan", language, {
    task: task.task, action: task.action, object: task.object, install_gate: installGate,
    candidate: pageTitle, api_url: wikiHowParseApiUrl(pageTitle, false), search_query: searchQuery, providers, k: webSearchRrfK(),
  });
}

// The native plan's typed request/stage records; capture diagnostics stay separate.
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

async function tryProceduralHowTo(prompt, language, preferences = {}) {
  const normalized = normalizePrompt(prompt);
  const task = extractProceduralHowToTask(normalized);
  if (!task) return null;

  const query = proceduralFallbackQuery(task.task);
  // Issue #991: the browser and the Rust solver run the same bounded
  const synthesized = await trySynthesizedHowToGuide(task, preferences);
  if (synthesized) return synthesized;
  const searchQuery = proceduralSearchQuery(task);
  const pageTitle = wikiHowPageTitle(task.task);
  const apiUrl = wikiHowParseApiUrl(pageTitle);
  const providerSummary = WEB_SEARCH_PROVIDERS.map((provider) => provider.id).join(", ");
  const isInstallProcedure = task.action === "install";
  // Honor the wikiHow opt-out: when disabled we skip the wikiHow API stage and
  const wikihowEnabled = externalServiceEnabled(preferences, "externalServiceWikihow");
  const evidence = [
    `procedural_how_to:request:${task.task}`,
    `procedural_how_to:action:${task.action}`,
    ...(task.object ? [`procedural_how_to:object:${task.object}`] : []),
    ...(isInstallProcedure
      ? [
          "procedural_how_to:stage:official_documentation",
          "procedural_how_to:source_gate:official_documentation_first",
          `web_search:request:${searchQuery}`,
          ...(searchQuery !== query ? [`web_search:request:${query}`] : []),
        ]
      : []),
    `procedural_how_to:stage:wikipedia`,
    `procedural_how_to:stage:wikidata`,
  ];
  if (!isInstallProcedure) {
    if (wikihowEnabled) {
      evidence.push(
        `procedural_how_to:stage:wikihow_api`,
        `procedural_how_to:wikihow_candidate:${pageTitle}`,
        `http_fetch:request:${apiUrl}`,
      );
    } else {
      evidence.push("procedural_how_to:service_disabled:wikihow");
    }
  }
  for (const correction of task.corrections || []) {
    evidence.push(`spelling_correction:${correction.from}->${correction.to}`);
  }

  const pathVariant = `${isInstallProcedure ? "install_" : ""}${wikihowEnabled ? "wikihow" : "no_wikihow"}`;
  const lines = [
    howResponse("procedural_how_to_heading", language, { task: task.task, action: task.action, object: task.object }),
    "",
    ...(isInstallProcedure
      ? [howResponse("procedural_how_to_live_install_gate", language, { search_query: searchQuery, fallback_query: query }), ""]
      : []),
    howResponse(`procedural_how_to_source_path_${pathVariant}`, language === "ru" ? "ru" : "en"),
    "",
  ];

  let confidence = 0.78;
  let diagnostics = null;
  let formalizedObject = "";
  let officialSearchUsable = false;
  let sourceContentObserved = false;

  if (isInstallProcedure) {
    evidence.push("procedural_how_to:stage:web_search");
    const officialSearch = await runWebSearchQuery(
      searchQuery,
      language,
      "official_documentation",
    );
    if (officialSearch) {
      appendUniqueEvidence(evidence, officialSearch.evidence);
      diagnostics = officialSearch.diagnostics || diagnostics;
      formalizedObject = officialSearch.formalizedObject || formalizedObject;
      officialSearchUsable = officialSearch.confidence >= 0.8;
      if (officialSearchUsable) {
        sourceContentObserved = true;
        confidence = Math.max(confidence, 0.82);
        lines.push(howResponse("procedural_how_to_official_search", "en", { query: searchQuery }));
        lines.push("");
        lines.push(officialSearch.content);
      }
    }
    if (!officialSearchUsable) {
      lines.push(howResponse("procedural_how_to_official_search_miss", "en", { query: searchQuery, fallback: query }));
      lines.push("");
    }
  }

  if (!officialSearchUsable) {
    if (isInstallProcedure) {
      if (wikihowEnabled) {
        evidence.push(
          `procedural_how_to:stage:wikihow_api`,
          `procedural_how_to:wikihow_candidate:${pageTitle}`,
          `http_fetch:request:${apiUrl}`,
        );
      } else {
        evidence.push("procedural_how_to:service_disabled:wikihow");
      }
    }
    const wikiHow = wikihowEnabled
      ? await fetchWikiHowProcedure(pageTitle, evidence)
      : { ok: false, error: "service_disabled" };

    if (wikiHow.ok) {
      sourceContentObserved = true;
      evidence.push(`procedural_how_to:wikihow_steps:${wikiHow.steps.length}`);
      evidence.push(`source:${wikiHow.sourceUrl}`);
      formalizedObject = `WH:${pageTitle}`;
      confidence = 0.86;
      lines.push(howResponse("procedural_how_to_wikihow_returned", "en", { title: wikiHow.title, candidate: pageTitle }));
      lines.push("");
      wikiHow.steps.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
      lines.push("");
      lines.push(`[Source](${wikiHow.sourceUrl})`);
    } else {
      if (wikihowEnabled) {
        evidence.push(`procedural_how_to:wikihow_miss:${wikiHow.error || "no_match"}`);
      }
      evidence.push("procedural_how_to:stage:web_search");
      const missNote = wikihowEnabled
        ? howResponse("procedural_how_to_wikihow_miss", "en", { candidate: pageTitle, error: wikiHow.error || "no_match" })
        : howResponse("procedural_how_to_wikihow_disabled", "en");
      const fallbackSearchQuery = isInstallProcedure ? query : searchQuery;
      const webSearch = await runWebSearchQuery(
        fallbackSearchQuery,
        language,
        isInstallProcedure ? "general_how_to_fallback" : "",
      );
      if (webSearch) {
        sourceContentObserved = Array.isArray(webSearch.diagnostics?.fused) && webSearch.diagnostics.fused.length > 0;
        appendUniqueEvidence(evidence, webSearch.evidence);
        diagnostics = webSearch.diagnostics || diagnostics;
        formalizedObject = webSearch.formalizedObject || formalizedObject;
        lines.push(missNote);
        lines.push("");
        lines.push(howResponse("procedural_how_to_fallback_search", "en", { query: fallbackSearchQuery }));
        lines.push("");
        lines.push(webSearch.content);
      } else {
        evidence.push(`web_search:request:${fallbackSearchQuery}`);
        for (const provider of WEB_SEARCH_PROVIDERS) {
          evidence.push(`web_search:provider:${provider.id}`);
        }
        evidence.push(`web_search:combined:rrf:k=${webSearchRrfK()}`);
        lines.push(missNote);
        lines.push("");
        lines.push(howResponse("procedural_how_to_fallback_search_plan", "en", { query: fallbackSearchQuery, providers: providerSummary, k: webSearchRrfK() }));
      }
    }
  }
  if (!evidence.includes("procedural_how_to:stage:web_search")) {
    evidence.push("procedural_how_to:stage:web_search");
    evidence.push(`web_search:request:${searchQuery}`);
    if (isInstallProcedure && searchQuery !== query) {
      evidence.push(`web_search:request:${query}`);
    }
    for (const provider of WEB_SEARCH_PROVIDERS) {
      evidence.push(`web_search:provider:${provider.id}`);
    }
    evidence.push(`web_search:combined:rrf:k=${webSearchRrfK()}`);
  }
  evidence.push("procedural_how_to:stage:recursive_fetch_check");
  evidence.push("procedural_how_to:source_gate:explicit_steps_only");

  const servicesEnabled = Object.entries(preferences).every(([key, value]) =>
    !key.startsWith("externalService") || value !== false);
  const discoveryOnly = !sourceContentObserved && servicesEnabled;
  return {
    intent: "procedural_how_to",
    content: discoveryOnly ? proceduralDiscoveryPlan(task, language, pageTitle, searchQuery, query, providerSummary)
      : lines.join("\n"),
    solverEvents: proceduralDiscoveryEvents(task, pageTitle, apiUrl, searchQuery, query, wikihowEnabled, !officialSearchUsable && wikihowEnabled),
    confidence,
    evidence,
    diagnostics,
    query,
    wikihowCandidate: pageTitle,
    formalizedObject,
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
  const task = extractProceduralHowToTask(normalizePrompt(user));
  return task ? { user, task } : null;
}

// Issue #444: a bare follow-up such as "Can you give me specific instructions?"
async function tryProceduralHowToFollowup(prompt, language, history = [], preferences = {}) {
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
