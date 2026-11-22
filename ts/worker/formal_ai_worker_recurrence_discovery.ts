function recurrenceNamesSubject(normalized, candidate) {
  return [candidate.label, candidate.match_label].some(label => {
    const name = recurrenceFold(label);
    return name.length >= 3 && recurrenceFold(normalized) === name;
  });
}

function recurrenceSearchPhrases(prompt, normalized) {
  const roles = [ROLE_PROGRAM_SYNTHESIS_SUBJECT, ROLE_PROGRAM_SYNTHESIS_ACTION,
    ROLE_PROGRAM_SYNTHESIS_DOMAIN, "coding_request_verb", "coding_request_object",
    "program_language_alias", "request_function_word", "statement_function_word"];
  const words = normalized.split(/\s+/u).filter(word =>
    /\p{L}/u.test(word) && !roles.some(role => lexiconMentionsRole(role, word)));
  const candidates = words.slice(0, 24);
  const spans = [1, 2, 3].flatMap(size => candidates
    .slice(0, candidates.length - size + 1).map((_, index) => candidates.slice(index, index + size).join(" ")));
  const declared = extractPythonFunctionName(prompt).replaceAll("_", " ");
  return [...new Set([declared, ...spans].filter(phrase =>
    phrase.length >= 3 && phrase.length <= 80))].slice(0, 12);
}

function recurrenceRegistryOrigin(record, language) {
  try {
    const template = record?.api?.replace("{language}", language);
    return template ? new URL(template).origin : null;
  } catch {
    return null;
  }
}

async function recurrenceEncyclopediaLabel(phrase, language, preferences) {
  const wikipedia = sourceWalkRegistry().find(item => item.id === "wikipedia");
  if (!wikipedia?.apiLanguages.includes(language) || !sourceWalkServiceAllowed(preferences, wikipedia)) return null;
  const origin = recurrenceRegistryOrigin(wikipedia, language);
  if (!origin) return null;
  const api = origin + "/w/api.php?action=query&format=json&formatversion=2&origin=*";
  const search = await recurrenceCapture(api + "&list=search&srlimit=1&srnamespace=0&srsearch="
    + sourceWalkPercentEncode(phrase), preferences);
  const best = search?.value?.query?.search?.[0];
  if (!best || typeof best.title !== "string" || best.title.length > 160) return null;
  const highlights = typeof best.snippet === "string"
    ? Array.from(best.snippet.matchAll(/<span class="searchmatch">([^<>]+)<\/span>/gu), match => match[1])
    : [];
  if (![best.title, ...highlights].some(term => recurrenceFold(term) === recurrenceFold(phrase))) return null;
  const translated = await recurrenceCapture(api + "&prop=langlinks&lllang=en&titles="
    + sourceWalkPercentEncode(best.title), preferences);
  const page = translated?.value?.query?.pages?.[0];
  const target = page?.langlinks?.find(link => link.lang === "en")?.title;
  return page?.title === best.title && typeof target === "string" && target.length <= 80
    ? { label: target, bridge: [search.capture, translated.capture] } : null;
}

async function recurrenceSearch(api, query, language, preferences) {
  return recurrenceCapture(api.replace("{query}", sourceWalkPercentEncode(query))
    .replace("{language}", sourceWalkPercentEncode(language)) + "&origin=*", preferences);
}

async function recurrenceSearchMatches(api, phrase, language, preferences) {
  const direct = await recurrenceSearch(api, phrase, language, preferences);
  const candidates = (result, expected, bridge) => {
    const items = result?.value?.query?.wikilambdasearch_functions;
    return Array.isArray(items) ? items.slice(0, 5).filter(candidate =>
      recurrenceSourceReference(candidate.page_title) && typeof candidate.label === "string"
      && typeof candidate.match_rate === "number" && candidate.match_rate >= 0.65
      && recurrenceNamesSubject(expected, candidate)).map(candidate => ({
      ...candidate, bridge, search: result.capture,
    })) : [];
  };
  const matches = candidates(direct, phrase, []);
  if (matches.length || language === "en") return matches;
  const translated = await recurrenceEncyclopediaLabel(phrase, language, preferences);
  if (!translated) return [];
  const english = await recurrenceSearch(api, translated.label, "en", preferences);
  return candidates(english, translated.label, translated.bridge);
}

function isSourceRecurrenceRequest(prompt, normalized) {
  return looksLikePythonFunctionSynthesis(prompt, normalized) && browserSynthesisLanguage(normalized) === "python";
}

function sourceRecurrenceGapForCatalog(prompt, normalized, language, local) {
  if (!local?.evidence?.includes("program_parameter:operand_unbound") || !isSourceRecurrenceRequest(prompt, normalized)) return null;
  return {
    ...local,
    intent: "write_program_skill_gap",
    content: answerFor("compose_no_verified_capture", language),
    evidence: [...local.evidence, "synthesis:publisher_recurrence_unverified", "response:write_program:skill_gap"],
  };
}

async function discoverSourceRecurrence(prompt, normalized, preferences) {
  if (!isSourceRecurrenceRequest(prompt, normalized)) return null;
  const record = sourceWalkRegistry().find(item => item.id === "wikifunctions");
  if (!record?.api || !record.fetchApi || !sourceWalkServiceAllowed(preferences, record)) return null;
  const phrases = recurrenceSearchPhrases(prompt, normalized);
  const language = detectLanguageFromRules(prompt);
  const matches = new Map();
  const results = await runWithConcurrencyLimit(phrases.map(phrase => () =>
    recurrenceSearchMatches(record.api, phrase, language, preferences)), 4);
  for (const candidate of results.flat()) {
    if ((matches.get(candidate.page_title)?.match_rate ?? 0) < candidate.match_rate) {
      matches.set(candidate.page_title, candidate);
    }
  }
  const ranked = Array.from(matches.values()).sort((left, right) => right.match_rate - left.match_rate);
  for (const candidate of ranked.slice(0, 4)) {
    const recurrence = await recurrenceReadAbstract(record.fetchApi, candidate, preferences);
    if (!recurrence) continue;
    const result = trySourceRecurrenceSynthesis(prompt, recurrence);
    if (result) return result;
  }
  return null;
}
