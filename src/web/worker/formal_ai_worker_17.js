// Worker module 18 of 21. Loaded by ../formal_ai_worker.js.
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
  const hasAction = containsAnySearchMarker(normalized, markers.actionMarkers);
  if (!hasAction) return "";
  const hasStrongAction = containsAnySearchMarker(
    normalized,
    markers.strongActionMarkers,
  );
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
  for (const marker of markers.imperativeLeadMarkers) {
    const query =
      queryAfterRawMarker(prompt, marker) ||
      queryAfterNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  return "";
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
// "записи о boeing", "关于波音的财务记录". Fires only when the prompt names a
// retrievable record subject (ROLE_WEB_SEARCH_RECORDS_SUBJECT) tied to a subject
// by a topic connective (ROLE_WEB_SEARCH_TOPIC_MARKER). Mirrors
// extract_records_information_request in
// src/solver_handlers/web_search_intent.rs.
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
    normalized.startsWith("search conversations ") ||
    normalized.startsWith("search my conversations ") ||
    normalized.startsWith("search my chats ") ||
    isPersonalFactFilterRequest(normalized)
  ) {
    return "";
  }
  const explicitQuery =
    extractExplicitWebSearchQuery(prompt) || extractExplicitWebSearchQuery(normalized);
  if (explicitQuery) {
    return { query: explicitQuery, kind: "explicit_prefix" };
  }
  const semanticQuery = extractSemanticWebSearchQuery(prompt, normalized);
  if (semanticQuery) {
    return { query: semanticQuery, kind: "semantic_action" };
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
  const researchQuery = extractImplicitResearchQuestion(normalized);
  return researchQuery
    ? { query: researchQuery, kind: "implicit_research_question" }
    : null;
}

function extractWebSearchQuery(prompt, normalized) {
  const request = extractWebSearchRequest(prompt, normalized);
  return request ? request.query : "";
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
  // surface forms of the procedural_task_modifier meaning
  // (data/seed/meanings-how.lino, loaded into MEANINGS_LINO): every form
  // is a suffix whose text after the … marker (form.after) is the tail to
  // strip, scanned in declaration order so the longer Russian "напиши по шагам"
  // still precedes its "по шагам" tail. No per-language modifier list lives
  // here — only the concept. Mirrors clean_procedural_fragment in
  // src/solver_handler_how.rs (issue #386).
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
  // surface forms (data/seed/meanings-how.lino, loaded into MEANINGS_LINO
  // above): each form's text is the misspelled token and its action field names
  // the correct spelling. No per-language typo table lives here — only the
  // concept. Mirrors correct_common_procedural_typos in
  // src/solver_handler_how.rs (issue #386).
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
  // "how to order X" and arrive as "how order X". The lead and the approved
  // action surfaces are both seed roles; the worker names only those concepts.
  // This keeps weak "how …" / "как …" / "कैसे …" / "如何…" prefixes from
  // claiming arbitrary "how <word>" prompts while still supporting all seeded
  // languages. Mirrors extract_elided_procedural_how_to_task in
  // src/solver_handler_how.rs.
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
  // meaning (data/seed/meanings-how.lino, loaded into MEANINGS_LINO): every
  // form is a prefix whose literal before the … marker (form.before) is the
  // matchable prefix, scanned in declaration order so "how to do " still
  // precedes "how to ". A form may name the canonical operation in its action
  // field (do / perform / implement / create / write); an empty action means
  // the operation is taken from the task's first word. No per-language prefix
  // list lives here — only the concept. Mirrors extract_procedural_how_to_task
  // in src/solver_handler_how.rs (issue #386).
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

function wikiHowParseApiUrl(pageTitle) {
  const encodedPage = encodeURIComponent(pageTitle).replace(/%2D/gi, "-");
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

const PANDAS_DATAFRAME_JOIN_DOCS_URL =
  "https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html";

function hasNormalizedWord(normalized, word) {
  return String(normalized || "")
    .split(/\s+/)
    .some((token) => token === word);
}

// True when the prompt opens with an imperative to search the web, so it should
// be answered by the web-search handler rather than the narrow docs handler.
// Mirrors is_explicit_web_search in src/solver_handler_docs.rs by meaning. The
// search imperative is read from the web_search_imperative_lead role — only its
// prefix forms are clause-initial leads, so the literal before each ellipsis
// ("search ", "look up ", "найди ", "搜索", …) is matched against the start of
// the prompt. The medium is read from the web_medium role; its surfaces are
// space-wrapped, so containsSearchMarker matches them with the whole-token
// padding convention (which also catches a medium word at the very end).
function isExplicitWebSearchPrompt(normalized) {
  const text = String(normalized || "");
  const requestsSearch = roleWordForms(ROLE_WEB_SEARCH_IMPERATIVE_LEAD)
    .filter((form) => form.slot === "prefix")
    .some((form) => text.startsWith(form.before));
  if (!requestsSearch) return false;
  return roleWordForms(ROLE_WEB_MEDIUM).some((form) =>
    containsSearchMarker(text, form.text),
  );
}

// True when the prompt is phrased as a request to have something explained.
// Mirrors is_explanation_request in src/solver_handler_docs.rs: every
// interrogative and imperative lead-in lives in the explanation_request_lead
// role, so no question word is hardcoded here. Each surface is matched by its
// slot — a prefix form ("how …", "explain …", "как …", "解释…") by the literal
// before the ellipsis against the start of the prompt, a bare form (" how ",
// "कैसे काम", "如何工作", …) as a raw substring anywhere (the space-wrapped bare
// forms thus match only on whole-word boundaries).
function isExplanationRequest(normalized) {
  const text = String(normalized || "");
  return roleWordForms(ROLE_EXPLANATION_REQUEST_LEAD).some((form) =>
    form.slot === "prefix"
      ? text.startsWith(form.before)
      : text.includes(form.text),
  );
}

// True when the prompt asks how the pandas DataFrame.join method works. Mirrors
// is_pandas_dataframe_join_prompt in src/solver_handler_docs.rs. The prompt must
// address pandas, read as an explanation request, and not be an explicit web
// search. The join is then recognised through two kinds of evidence: code-
// resident API identifiers (DataFrame.join, df.join, the join+dataframe pairing)
// — written the same in every language, so they legitimately live here as the
// bridge from a multilingual question to one documented API — and the
// translatable noun "method", matched through the code_method_noun role rather
// than the four per-language words it used to hardcode, paired with the join
// identifier.
function isPandasDataFrameJoinPrompt(prompt, normalized) {
  const lower = String(prompt || "").toLowerCase();
  const text = String(normalized || "").trim();
  if (isExplicitWebSearchPrompt(text)) return false;
  if (!hasNormalizedWord(text, "pandas")) return false;
  if (!isExplanationRequest(text)) return false;
  return (
    lower.includes("dataframe.join") ||
    lower.includes("df.join") ||
    (hasNormalizedWord(text, "join") && hasNormalizedWord(text, "dataframe")) ||
    (hasNormalizedWord(text, "join") &&
      lexiconMentionsRole(ROLE_CODE_METHOD_NOUN, text))
  );
}

function docsMethodContent(language) {
  if (language === "ru") {
    return [
      "pandas `DataFrame.join` добавляет столбцы из `other` DataFrame или именованной Series к вызывающему DataFrame и возвращает новый DataFrame.",
      "В рамках этого метода: по умолчанию это left join по индексу вызывающего DataFrame. Если задан `on`, pandas сопоставляет этот столбец или уровень индекса с индексом объекта `other`. Параметр `how` управляет объединением ключей (`left`, `right`, `outer`, `inner`, `cross`, `left_anti` или `right_anti`). `lsuffix` и `rsuffix` нужны при совпадающих именах столбцов, `sort` сортирует ключи join, а `validate` проверяет связи one-to-one, one-to-many, many-to-one или many-to-many. Для join столбец-к-столбцу документация pandas указывает на `DataFrame.merge`.",
      `Источник: [pandas.DataFrame.join](${PANDAS_DATAFRAME_JOIN_DOCS_URL}) (официальная документация pandas).`,
    ].join("\n\n");
  }

  if (language === "hi") {
    return [
      "pandas `DataFrame.join` कॉल करने वाले DataFrame में `other` DataFrame या named Series के columns जोड़ता है और नया DataFrame लौटाता है.",
      "इस method के दायरे में: default रूप से यह caller के index पर left join करता है. `on` देने पर pandas caller के उस column या index level को `other` object के index से मिलाता है. `how` parameter keys को मिलाने का तरीका चुनता है (`left`, `right`, `outer`, `inner`, `cross`, `left_anti`, या `right_anti`). Column नाम टकराने पर `lsuffix` और `rsuffix`, join keys को sort करने के लिए `sort`, और one-to-one, one-to-many, many-to-one, या many-to-many संबंध जांचने के लिए `validate` इस्तेमाल करें. Column-on-column joins के लिए pandas docs `DataFrame.merge` की ओर भेजते हैं.",
      `Source: [pandas.DataFrame.join](${PANDAS_DATAFRAME_JOIN_DOCS_URL}) (official pandas docs).`,
    ].join("\n\n");
  }

  if (language === "zh") {
    return [
      "pandas `DataFrame.join` 会把 `other` DataFrame 或具名 Series 的列加入调用方，并返回新的 DataFrame。",
      "只看这个方法：默认情况下，它使用调用方的 index 执行 left join。设置 `on` 时，pandas 会把调用方的列或索引层级与 `other` 对象的 index 匹配。`how` 参数控制键的组合方式（`left`、`right`、`outer`、`inner`、`cross`、`left_anti` 或 `right_anti`）。列名冲突时使用 `lsuffix` 和 `rsuffix`，用 `sort` 排序 join keys，用 `validate` 检查 one-to-one、one-to-many、many-to-one 或 many-to-many 关系。对于列到列的 join，pandas 文档指向 `DataFrame.merge`。",
      `Source: [pandas.DataFrame.join](${PANDAS_DATAFRAME_JOIN_DOCS_URL}) (official pandas docs).`,
    ].join("\n\n");
  }

  return [
    "pandas `DataFrame.join` joins columns from the `other` DataFrame or named Series into the caller and returns a new DataFrame.",
    "Scoped to this method: by default, it performs a left join using the caller's index. If `on` is set, pandas matches that caller column or index level against the `other` object's index. The `how` parameter controls key handling (`left`, `right`, `outer`, `inner`, `cross`, `left_anti`, or `right_anti`). Use `lsuffix` and `rsuffix` when column names overlap, `sort` to order join keys, and `validate` to check one-to-one, one-to-many, many-to-one, or many-to-many relationships. For column-on-column joins, the pandas docs point to `DataFrame.merge`.",
    `Source: [pandas.DataFrame.join](${PANDAS_DATAFRAME_JOIN_DOCS_URL}) (official pandas docs).`,
  ].join("\n\n");
}

function tryDocsMethodExplanation(prompt, language) {
  const normalized = normalizePrompt(prompt);
  if (!isPandasDataFrameJoinPrompt(prompt, normalized)) return null;

  return {
    intent: "docs_method_explanation",
    content: docsMethodContent(language),
    confidence: 0.92,
    evidence: [
      "docs_method:project:pandas",
      "docs_method:method:pandas.DataFrame.join",
      "docs_method:source_kind:official-docs",
      `source:${PANDAS_DATAFRAME_JOIN_DOCS_URL}`,
      `language:${language}`,
    ],
    formalizedObject: "pandas.DataFrame.join",
  };
}

// Issue #444: external *trusted* services are opt-out. A preference value of
// exactly `false` disables the service; a missing/undefined value keeps it
// enabled, so the assistant's default behavior is unchanged unless the user opts
// out in settings. The `key` arguments mirror the `settings_key` recorded in
// data/seed/sources-registry.lino and the EXTERNAL_TRUSTED_SERVICES catalog in
// src/web/app.js, keeping the registry the single source of truth.
function externalServiceEnabled(preferences, key) {
  return !(preferences && preferences[key] === false);
}

function proceduralSearchQuery(task) {
  const fallbackQuery = `how to ${task.task}`;
  if (!task || task.action !== "install") return fallbackQuery;
  const target = String(task.object || task.task || "").trim();
  return `${target || task.task} install official documentation`.trim();
}

async function tryProceduralHowTo(prompt, language, preferences = {}) {
  const normalized = normalizePrompt(prompt);
  const task = extractProceduralHowToTask(normalized);
  if (!task) return null;

  const query = `how to ${task.task}`;
  const searchQuery = proceduralSearchQuery(task);
  const pageTitle = wikiHowPageTitle(task.task);
  const apiUrl = wikiHowParseApiUrl(pageTitle);
  const providerSummary = WEB_SEARCH_PROVIDERS.map((provider) => provider.id).join(", ");
  const isInstallProcedure = task.action === "install";
  // Honor the wikiHow opt-out: when disabled we skip the wikiHow API stage and
  // its live fetch entirely, emit a service_disabled marker, and route straight
  // to the web-search fallback.
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

  const sourcePath = isInstallProcedure
    ? wikihowEnabled
      ? "Source path: Wikipedia -> Wikidata -> official documentation web search -> wikiHow API fallback -> community web search fallback -> recursive fetch check."
      : "Source path: Wikipedia -> Wikidata -> official documentation web search -> community web search fallback -> recursive fetch check (wikiHow disabled in settings)."
    : wikihowEnabled
      ? "Source path: Wikipedia -> Wikidata -> wikiHow API -> web search fallback -> recursive fetch check."
      : "Source path: Wikipedia -> Wikidata -> web search fallback -> recursive fetch check (wikiHow disabled in settings).";
  const russianSourcePath = isInstallProcedure
    ? wikihowEnabled
      ? "Путь источников: Wikipedia -> Wikidata -> official documentation web search -> wikiHow API fallback -> community web search fallback -> recursive fetch check."
      : "Путь источников: Wikipedia -> Wikidata -> official documentation web search -> community web search fallback -> recursive fetch check (wikiHow отключен в настройках)."
    : wikihowEnabled
      ? "Путь источников: Wikipedia -> Wikidata -> wikiHow API -> web search fallback -> recursive fetch check."
      : "Путь источников: Wikipedia -> Wikidata -> web search fallback -> recursive fetch check (wikiHow отключен в настройках).";
  const installGate = `For install tasks, the first source gate prefers the product's official documentation or official repository install page before community how-to sources. It starts with \`${searchQuery}\` and keeps \`${query}\` as fallback.`;
  let lines;
  if (language === "ru") {
    lines = [
      `План поиска процедуры для \`${task.task}\` (действие \`${task.action}\`, объект \`${task.object}\`).`,
      "",
      ...(isInstallProcedure
        ? [
            `Для задач установки первый source gate ищет официальную документацию продукта или официальную страницу установки в репозитории, а уже потом переходит к общим how-to источникам. Он начинает с \`${searchQuery}\` и держит \`${query}\` как fallback.`,
            "",
          ]
        : []),
      russianSourcePath,
      "",
    ];
  } else if (language === "hi") {
    lines = [
      `\`${task.task}\` के लिए procedural discovery plan (action \`${task.action}\`, object \`${task.object}\`).`,
      "",
      ...(isInstallProcedure
        ? [
            `इंस्टॉल वाले कामों में पहला source gate उत्पाद की आधिकारिक documentation या official repository install page को प्राथमिकता देता है; उसके बाद ही community how-to sources देखे जाते हैं. Solver पहले \`${searchQuery}\` चलाता है और \`${query}\` को fallback रखता है.`,
            "",
          ]
        : []),
      sourcePath,
      "",
    ];
  } else if (language === "zh") {
    lines = [
      `\`${task.task}\` 的过程发现计划（action \`${task.action}\`, object \`${task.object}\`）。`,
      "",
      ...(isInstallProcedure
        ? [
            `对于安装类任务，第一个 source gate 优先查找产品官方 documentation 或官方仓库的安装页面，然后才使用社区 how-to 来源。Solver 先运行 \`${searchQuery}\`，并把 \`${query}\` 保留为 fallback。`,
            "",
          ]
        : []),
      sourcePath,
      "",
    ];
  } else {
    lines = [
      `Procedural discovery plan for \`${task.task}\` (action \`${task.action}\`, object \`${task.object}\`).`,
      "",
      ...(isInstallProcedure ? [installGate, ""] : []),
      sourcePath,
      "",
    ];
  }

  let confidence = 0.78;
  let diagnostics = null;
  let formalizedObject = "";
  let officialSearchUsable = false;

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
        confidence = Math.max(confidence, 0.82);
        lines.push(`Official-documentation web search for \`${searchQuery}\`:`);
        lines.push("");
        lines.push(officialSearch.content);
      }
    }
    if (!officialSearchUsable) {
      lines.push(
        `Official-documentation web search for \`${searchQuery}\` did not return ranked guidance; preserving \`${query}\` as the general how-to fallback.`,
      );
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
      evidence.push(`procedural_how_to:wikihow_steps:${wikiHow.steps.length}`);
      evidence.push(`source:${wikiHow.sourceUrl}`);
      formalizedObject = `WH:${pageTitle}`;
      confidence = 0.86;
      lines.push(`wikiHow API returned \`${wikiHow.title}\` for candidate \`${pageTitle}\`.`);
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
        ? `wikiHow candidate \`${pageTitle}\` did not return explicit steps (${wikiHow.error || "no_match"}).`
        : "wikiHow is disabled in settings.";
      const fallbackSearchQuery = isInstallProcedure ? query : searchQuery;
      const webSearch = await runWebSearchQuery(
        fallbackSearchQuery,
        language,
        isInstallProcedure ? "general_how_to_fallback" : "",
      );
      if (webSearch) {
        appendUniqueEvidence(evidence, webSearch.evidence);
        diagnostics = webSearch.diagnostics || diagnostics;
        formalizedObject = webSearch.formalizedObject || formalizedObject;
        lines.push(missNote);
        lines.push("");
        lines.push(`Fallback web search for \`${fallbackSearchQuery}\`:`);
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
        lines.push(
          `Fallback web search for \`${fallbackSearchQuery}\` should use ${providerSummary} and reciprocal rank fusion (k = ${webSearchRrfK()}).`,
        );
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

  return {
    intent: "procedural_how_to",
    content: lines.join("\n"),
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
  const match = source.match(/^([\s\S]+?)(?:[,;；，、\n]|[.!?。！？]\s+)([\s\S]+)$/u);
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

  return {
    intent: "compound_response",
    content: `${answerFor("greeting", greetingLanguage, { randomize })}\n\n${procedure.content}`,
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
// *meaning*, not a hardcoded per-language phrase table (issue #386 convention).
// Each surface of the procedural_elaboration meaning lives in
// data/seed/meanings-how.lino (loaded into MEANINGS_LINO); this code knows
// only the concept. Mirrors is_procedural_elaboration_request in
// src/solver_handler_how.rs.
function isProceduralElaborationRequest(normalized) {
  return lexiconMentionsRole(ROLE_PROCEDURAL_ELABORATION, normalized);
}

// The prior exchange must have been a how-to procedure: the previous user turn
// re-parses as a procedural request and the assistant answered it. Mirrors the
// last_assistant_turn + last_user_turn re-parse gate in
// try_procedural_how_to_followup (src/solver_handler_how.rs).
function priorProceduralHowToDialogue(history) {
  const assistant = lastHistoryTurn(history, "assistant");
  if (!assistant) return null;
  const user = lastHistoryTurn(history, "user");
  if (!user) return null;
  const task = extractProceduralHowToTask(normalizePrompt(user));
  return task ? { user, task } : null;
}

// Issue #444: a bare follow-up such as "Can you give me specific instructions?"
// carries no "how to" lead-in of its own and would otherwise dead-end at the
// unknown opener. When the current prompt evidences the procedural_elaboration
// meaning and the prior turn was an answered how-to request, re-run the original
// discovery so the elaboration rebinds to the recovered task. Mirrors
// try_procedural_how_to_followup in src/solver_handler_how.rs (mirror parity).
async function tryProceduralHowToFollowup(prompt, language, history = [], preferences = {}) {
  const canonical = normalizePrompt(prompt);
  if (!isProceduralElaborationRequest(canonical)) return null;
  const dialogue = priorProceduralHowToDialogue(history);
  if (!dialogue) return null;
  const procedure = await tryProceduralHowTo(dialogue.user, language, preferences);
  if (!procedure) return null;
  // Front-load the follow-up evidence so the rebind is visible in the trace,
  // matching the log.append order on the Rust side.
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
