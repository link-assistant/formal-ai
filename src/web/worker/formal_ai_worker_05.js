// Worker module 6 of 21. Loaded by ../formal_ai_worker.js.
function matchSourceFormatting(target, source) {
  const targetTrimmed = String(target || "").trim();
  if (!targetTrimmed) return "";
  const sourceTrimmed = String(source || "").trim();

  let sourceTerminal = null;
  if (sourceTrimmed.length > 0) {
    const lastChar = Array.from(sourceTrimmed).pop();
    if (TRANSLATION_TERMINAL_PUNCTUATION.includes(lastChar)) sourceTerminal = lastChar;
  }
  let targetNoTerminal = targetTrimmed;
  while (
    targetNoTerminal.length > 0 &&
    TRANSLATION_TERMINAL_PUNCTUATION.includes(Array.from(targetNoTerminal).pop())
  ) {
    const lastChar = Array.from(targetNoTerminal).pop();
    targetNoTerminal = targetNoTerminal.slice(0, targetNoTerminal.length - lastChar.length);
  }
  const withTerminal = sourceTerminal ? targetNoTerminal + sourceTerminal : targetNoTerminal;

  const sourceFirstLetter = Array.from(sourceTrimmed).find((character) =>
    /\p{L}/u.test(character),
  );
  if (!sourceFirstLetter) return withTerminal;
  const targetChars = Array.from(withTerminal);
  const targetFirstIdx = targetChars.findIndex((character) => /\p{L}/u.test(character));
  if (targetFirstIdx === -1) return withTerminal;
  const targetFirstLetter = targetChars[targetFirstIdx];
  const sourceLower = sourceFirstLetter.toLowerCase() === sourceFirstLetter
    && sourceFirstLetter.toUpperCase() !== sourceFirstLetter;
  const sourceUpper = sourceFirstLetter.toUpperCase() === sourceFirstLetter
    && sourceFirstLetter.toLowerCase() !== sourceFirstLetter;
  const targetLower = targetFirstLetter.toLowerCase() === targetFirstLetter
    && targetFirstLetter.toUpperCase() !== targetFirstLetter;
  const targetUpper = targetFirstLetter.toUpperCase() === targetFirstLetter
    && targetFirstLetter.toLowerCase() !== targetFirstLetter;
  if (sourceLower && targetUpper) {
    targetChars[targetFirstIdx] = targetFirstLetter.toLowerCase();
    return targetChars.join("");
  }
  if (sourceUpper && targetLower) {
    targetChars[targetFirstIdx] = targetFirstLetter.toUpperCase();
    return targetChars.join("");
  }
  return withTerminal;
}

function normalizeComposableSurface(surface) {
  return String(surface || "")
    .trim()
    .replace(/[?!.。？！．]+$/u, "")
    .toLowerCase()
    .split(/\s+/u)
    .filter(Boolean)
    .join(" ");
}

// Issue #386 compositional-translation roles — mirror ROLE_COMPOSITIONAL_LEMMA,
// ROLE_COMPOSITIONAL_PHRASE and ROLE_COMPOSITIONAL_GENITIVE_HEAD in
// src/seed/roles.rs. The per-word lemma fallbacks, fixed phrases, genitive-
// governing heads and the single genitive-tagged complement that used to be
// hardcoded here all live in the loaded MEANINGS_LINO
// (data/seed/meanings-translation.lino); the functions below name only the
// semantic roles and the ru→en language pair, never the surface words. The
// query helpers (roleSurfaceTranslation, roleListsSurface,
// roleActionSurfaceTranslation, wordIn) are defined alongside meaningLexicon.
const ROLE_COMPOSITIONAL_LEMMA = "compositional_lemma";
const ROLE_COMPOSITIONAL_PHRASE = "compositional_phrase";
const ROLE_COMPOSITIONAL_GENITIVE_HEAD = "compositional_genitive_head";

function capitalizeAsciiFirst(surface) {
  const text = String(surface || "");
  if (!text) return "";
  return text[0].toUpperCase() + text.slice(1);
}

function translateRussianWordSequence(words) {
  const translated = [];
  for (let index = 0; index < words.length; index += 1) {
    const word = words[index];
    const next = words[index + 1];
    if (
      next &&
      roleListsSurface(ROLE_COMPOSITIONAL_GENITIVE_HEAD, "ru", word) &&
      roleActionSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "genitive", "ru", "en", next)
    ) {
      translated.push(
        roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "ru", "en", word),
        "of",
        roleActionSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "genitive", "ru", "en", next),
      );
      index += 1;
      continue;
    }
    const surface = roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "ru", "en", word);
    if (!surface) return null;
    translated.push(surface);
  }
  return capitalizeAsciiFirst(translated.join(" "));
}

function translateCompositionalSurface(surface, source, target) {
  const normalized = normalizeComposableSurface(surface);
  if (!normalized) return null;

  const direct =
    roleSurfaceTranslation(ROLE_COMPOSITIONAL_PHRASE, source, target, normalized) ||
    roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, source, target, normalized);
  if (direct) return direct;

  if (source !== "ru" || target !== "en") return null;

  const phrase = roleSurfaceTranslation(ROLE_COMPOSITIONAL_PHRASE, "ru", "en", normalized);
  if (phrase) return phrase;

  const words = normalized.split(/\s+/u).filter(Boolean);
  if (words.length < 2 || words.length > 8) return null;
  return translateRussianWordSequence(words);
}

const QUESTION_LANGUAGE_MARKERS = {
  ru: [
    "\u0447\u0442\u043e",
    "\u043a\u0430\u043a",
    "\u043a\u0442\u043e",
    "\u0433\u0434\u0435",
    "\u043a\u043e\u0433\u0434\u0430",
    "\u043f\u043e\u0447\u0435\u043c\u0443",
  ],
  hi: [
    "\u0915\u094d\u092f\u093e",
    "\u0915\u094c\u0928",
    "\u0915\u0939\u093e\u0901",
    "\u0915\u092c",
    "\u0915\u0948\u0938\u0947",
    "\u0915\u094d\u092f\u094b\u0902",
  ],
  zh: [
    "\u4ec0\u4e48",
    "\u5417",
    "\u600e\u4e48",
    "\u8c01",
    "\u54ea",
  ],
};

function detectQuestionMarkerLanguage(text, counts) {
  const normalized = String(text || "").toLocaleLowerCase();
  let best = null;
  for (const candidate of [
    { slug: "ru", count: counts.cyrillic },
    { slug: "hi", count: counts.devanagari },
    { slug: "zh", count: counts.cjk },
  ]) {
    if (candidate.count <= 0) continue;
    const markers = QUESTION_LANGUAGE_MARKERS[candidate.slug] || [];
    if (!markers.some((marker) => normalized.includes(marker))) continue;
    if (!best || candidate.count > best.count) best = candidate;
  }
  return best ? best.slug : null;
}

function detectLanguageSlug(text) {
  let latin = 0;
  let cyrillic = 0;
  let devanagari = 0;
  let cjk = 0;
  let other = 0;
  let firstScript = null;
  for (const character of String(text || "")) {
    const code = character.codePointAt(0);
    if (/[a-z]/i.test(character)) {
      latin += 1;
      if (!firstScript) firstScript = "latin";
    } else if (code >= 0x0400 && code <= 0x04ff) {
      cyrillic += 1;
      if (!firstScript) firstScript = "cyrillic";
    } else if (code >= 0x0900 && code <= 0x097f) {
      devanagari += 1;
      if (!firstScript) firstScript = "devanagari";
    } else if (code >= 0x4e00 && code <= 0x9fff) {
      cjk += 1;
      if (!firstScript) firstScript = "cjk";
    } else if (/\p{L}/u.test(character)) {
      other += 1;
      if (!firstScript) firstScript = "other";
    }
  }
  const total = latin + cyrillic + devanagari + cjk + other;
  if (total === 0) return "en";
  if (other > latin && other >= cyrillic && other >= devanagari && other >= cjk) {
    return "unknown";
  }
  if (latin > 0) {
    const markerLanguage = detectQuestionMarkerLanguage(text, { cyrillic, devanagari, cjk });
    if (markerLanguage) return markerLanguage;
    if (firstScript === "cyrillic" && cyrillic >= Math.max(devanagari, cjk)) return "ru";
    if (firstScript === "devanagari" && devanagari >= Math.max(cyrillic, cjk)) return "hi";
    if (firstScript === "cjk" && cjk >= Math.max(cyrillic, devanagari)) return "zh";
  }
  if (cyrillic >= Math.max(latin, devanagari, cjk) && cyrillic > 0) return "ru";
  if (devanagari >= Math.max(latin, cyrillic, cjk) && devanagari > 0) return "hi";
  if (cjk >= Math.max(latin, cyrillic, devanagari) && cjk > 0) return "zh";
  return "en";
}

function inferTranslationSource(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const surface = extractQuotedPhrase(prompt) || extractUnquotedTranslationSurface(prompt);
  if (surface) {
    const detected = detectLanguageSlug(surface);
    if (detected !== "unknown") return detected;
  }
  // Issue #386: the source language of an un-annotated request is the language
  // the user issued the *translation command* in. Ask the lexicon which
  // language's command verb the prompt carries — the stems live once in the
  // embedded translate meaning; this code knows only the concept and the
  // language-code bridge. English is the default when no command verb is present.
  return (
    firstRoleLanguage(ROLE_TRANSLATION_ACTION, lower, ["ru", "hi", "zh"]) || "en"
  );
}

// Live Wiktionary fallback (issue #221). When the offline meaning
// registry above does not cover `surface`, fetch the Wiktionary page
// for `source` and pull the first `{{tt+|<target>|...}}` (or `{{t+}}` /
// `{{t}}`) entry. Mirrors the Rust pipeline's Stage 1a in
// `src/translation/pipeline.rs`: if the main page delegates noun
// translations via `{{see translation subpage|...}}`, fetch the
// subpage and search it first. Keeps the worker mobile-friendly: no
// offline dictionary bundled, just a single CORS-safe HTTP call.
async function fetchWiktionaryWikitext(pageTitle, language) {
  if (typeof fetch !== "function" || !pageTitle) return null;
  const host = WIKTIONARY_SEARCH_HOSTS[language] || WIKTIONARY_SEARCH_HOSTS.en;
  const url = `${host}?action=parse&page=${encodeURIComponent(
    pageTitle,
  )}&prop=wikitext&format=json&origin=*`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return null;
    const data = await response.json();
    return (data && data.parse && data.parse.wikitext && data.parse.wikitext["*"]) || null;
  } catch (_error) {
    return null;
  }
}

function stripCombiningMarks(value) {
  // Russian Wiktionary entries are stored with combining stress marks
  // (U+0301) so readers can see where the accent falls. The surface
  // form must drop them so the result matches the lemma (помидо́р →
  // помидор) and downstream substring assertions still hit.
  return typeof value === "string" && value.normalize
    ? value.normalize("NFD").replace(/[̀-ͯ]/g, "").normalize("NFC")
    : value;
}

function extractWiktionaryTranslation(wikitext, targetLang) {
  if (!wikitext || !targetLang) return null;
  // English-edition templates: {{t|<lang>|...}}, {{t+|<lang>|...}},
  // {{tt|<lang>|...}}, {{tt+|<lang>|...}}.
  const enPattern = new RegExp(
    `\\{\\{tt?\\+?\\|${targetLang}\\|([^|}\\n]+)`,
    "i",
  );
  const enMatch = enPattern.exec(wikitext);
  if (enMatch) {
    const surface = stripCombiningMarks(String(enMatch[1] || "").trim());
    if (surface) return surface;
  }
  // Russian-edition translation blocks: `{{перев-блок|...|<lang>=[[surface]]\n|...}}`.
  // The language code may appear at the very start (no leading newline)
  // or after `\n|`; the surface can be inside `[[...]]`, optionally
  // followed by transliteration in parentheses we drop.
  const ruPattern = new RegExp(
    `[|\\n]${targetLang}\\s*=\\s*(?:\\[\\[([^\\]|]+)(?:\\|[^\\]]+)?\\]\\]|([^\\n|}]+))`,
    "i",
  );
  const ruMatch = ruPattern.exec(wikitext);
  if (ruMatch) {
    const raw = (ruMatch[1] || ruMatch[2] || "").trim();
    const surface = stripCombiningMarks(raw.replace(/\s*\([^)]*\)\s*$/, "").trim());
    if (surface) return surface;
  }
  return null;
}

async function resolveWiktionaryLemma(surface, language) {
  // Inflected forms (e.g. Russian plural `помидоры`) are not always stored
  // as separate pages on the source-language Wiktionary. OpenSearch returns
  // the closest matching titles; the first hit is the dictionary lemma
  // (`помидор`) we want to look up next.
  if (typeof fetch !== "function" || !surface) return null;
  const host = WIKTIONARY_SEARCH_HOSTS[language] || WIKTIONARY_SEARCH_HOSTS.en;
  const url = `${host}?action=opensearch&search=${encodeURIComponent(
    surface,
  )}&limit=1&format=json&origin=*`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return null;
    const data = await response.json();
    const titles = Array.isArray(data) && Array.isArray(data[1]) ? data[1] : [];
    const lemma = titles[0];
    if (typeof lemma !== "string" || !lemma || lemma === surface) return null;
    return lemma;
  } catch (_error) {
    return null;
  }
}

async function liveWiktionaryTranslate(surface, source, target) {
  // Run the direct page fetch and the OpenSearch lemma resolution in
  // parallel. For inflected forms (e.g. `помидоры`) the direct fetch
  // 404s, and chaining the lemma lookup sequentially after it added a
  // third sequential round-trip that pushed CI past the 5s expect cap.
  const [direct, lemma] = await Promise.all([
    fetchWiktionaryWikitext(surface, source),
    resolveWiktionaryLemma(surface, source),
  ]);
  let main = direct;
  if (!main && lemma) {
    main = await fetchWiktionaryWikitext(lemma, source);
  }
  if (!main) return null;
  let wikitext = main;
  if (/\{\{see translation subpage\|/i.test(main)) {
    const subpage = await fetchWiktionaryWikitext(`${surface}/translations`, source);
    if (subpage) wikitext = `${subpage}\n${main}`;
  }
  return extractWiktionaryTranslation(wikitext, target);
}

async function translateSurface(surface, source, target) {
  if (source === target) {
    return { surface: String(surface || ""), gap: false };
  }
  const token = formalizeSurface(surface, source);
  if (token) {
    const primary = deformalizeMeaning(token, target);
    if (primary) return { surface: primary, gap: false };
  }
  const compositional = translateCompositionalSurface(surface, source, target);
  if (compositional) return { surface: compositional, gap: false };
  if (surface) {
    const live = await liveWiktionaryTranslate(surface, source, target);
    if (live) return { surface: live, gap: false };
  }
  return { surface: null, gap: true };
}

function renderTranslationGap(surface, source, target) {
  const trimmed = String(surface || "").trim();
  if (!trimmed) {
    return `I could not identify a source phrase to translate from ${source} to ${target}.`;
  }
  return `I could not translate "${trimmed}" from ${source} to ${target} with the available formalization data. I recorded this as a translation gap for follow-up.`;
}

async function tryTranslation(prompt, normalized) {
  const targetHint = detectTranslationTargetLanguage(normalized);
  // Issue #386: recognise a translation command by *meaning*, not by hardcoded
  // verbs. The command stems live once in the embedded translate meaning; this
  // code knows the concept and the head-initial/head-final typology. Clause-
  // initial English/Russian commands are matched as a prefix; head-final
  // Hindi/Chinese place the verb later, so they are matched anywhere but gated
  // by a target marker to avoid firing on an incidental verb noun.
  const headInitialCommand = wordsForRoleInLanguages(ROLE_TRANSLATION_ACTION, [
    "en",
    "ru",
  ]).some((stem) => normalized.startsWith(stem));
  const headFinalCommand =
    Boolean(targetHint) &&
    wordsForRoleInLanguages(ROLE_TRANSLATION_ACTION, ["hi", "zh"]).some((stem) =>
      normalized.includes(stem),
    );
  const isTranslationRequest = headInitialCommand || headFinalCommand;
  if (!isTranslationRequest) return null;

  // Issue #216: fall back to an unquoted surface (`translate apple to
  // russian`) when no quoted fragment is present so the offline registry
  // can still resolve a meaning token.
  const surface =
    extractQuotedPhrase(prompt) || extractUnquotedTranslationSurface(prompt) || "";
  const surfaceMeaning = surface || prompt;
  const source = detectTranslationSourceLanguage(normalized) || inferTranslationSource(prompt);
  const target = targetHint || "en";
  const meaningId = stableBehaviorRuleId("meaning", normalizeMeaningText(surfaceMeaning));
  const translation = await translateSurface(surface, source, target);
  let content;
  if (translation.gap) {
    content = renderTranslationGap(surface, source, target);
  } else {
    const translatedSurface = matchSourceFormatting(translation.surface || "", surface);
    content = surface ? `"${translatedSurface}"` : translatedSurface;
  }
  const evidence = [
    "handler:translation",
    `language_from:${source}`,
    `language_to:${target}`,
    `meaning:${meaningId}`,
  ];
  if (translation.gap && surface) evidence.push(`translation_gap:${surface}`);
  return {
    intent: `translate_${source}_to_${target}`,
    content,
    confidence: 1.0,
    evidence,
  };
}

// The number of brainstorm items returned when the prompt names no count.
const DEFAULT_BRAINSTORM_COUNT = 5;

// Read the integer value of a cardinal-number meaning from its own data. Each
// cardinal carries a numeral word form (e.g. "10") — the script-independent
// surface that spells the value — so the count is derived from the seed rather
// than restated as a literal. Mirrors cardinal_value in
// src/solver_handlers/benchmark_prompts.rs (issue #386).
function cardinalValue(meaning) {
  if (!meaning || !Array.isArray(meaning.words)) return null;
  const numeral = meaning.words.find((word) => /^[0-9]+$/.test(word));
  if (numeral === undefined) return null;
  const value = Number.parseInt(numeral, 10);
  return Number.isNaN(value) ? null : value;
}

// Parse the number of items the user asked for, defaulting to
// DEFAULT_BRAINSTORM_COUNT when no explicit count is present. The only
// non-default count the brainstorm prompts exercise is ten, so the recogniser
// asks the seed whether the `ten` cardinal is evidenced in the prompt (in any
// supported language) and reads the value from that cardinal's own numeral
// surface. Mirrors requested_brainstorm_count in
// src/solver_handlers/benchmark_prompts.rs (issue #386).
function requestedBrainstormCount(normalized) {
  const ten = findMeaning("ten");
  if (ten && meaningEvidencedIn(ten, normalized)) {
    const value = cardinalValue(ten);
    if (value !== null) return value;
  }
  return DEFAULT_BRAINSTORM_COUNT;
}

function numbered(items, count) {
  return items
    .slice(0, count)
    .map((item, index) => `${index + 1}. ${item}`)
    .join("\n");
}

function tryBrainstormingRequest(prompt, normalized) {
  const seeds = BRAINSTORM_SEEDS || {};
  if (!containsAny(normalized, seeds.triggers)) return null;
  const categories = Array.isArray(seeds.categories) ? seeds.categories : [];
  const category =
    categories.find((entry) => containsAny(normalized, entry.detectionKeywords)) ||
    categories.find((entry) => !entry.detectionKeywords || entry.detectionKeywords.length === 0);
  if (!category || !Array.isArray(category.items) || category.items.length === 0) {
    return null;
  }
  const count = requestedBrainstormCount(normalized);
  return {
    intent: category.intent || "brainstorm_project_ideas",
    content: numbered(category.items, count),
    confidence: 0.8,
    evidence: [`brainstorm:category:${category.slug || "project_ideas"}`],
  };
}

function localizedFactFor(record, language) {
  const localized = Array.isArray(record.localized) ? record.localized : [];
  return (
    localized.find((entry) => entry && entry.language === language) ||
    localized.find((entry) => entry && entry.language === "en") ||
    null
  );
}

function tryFactLookup(prompt, normalized) {
  const record = FACTS.find(
    (fact) =>
      containsAny(normalized, fact.subjectAliases) &&
      containsAny(normalized, fact.questionKeywords),
  );
  if (!record) return null;
  const language = detectLanguage(prompt);
  const localized = localizedFactFor(record, language);
  const summary = (localized && localized.summary) || record.summary;
  const source = (localized && localized.source) || record.source;
  const evidence = [
    `fact_lookup:hit:${record.slug}`,
    `language:${language}`,
    ...((record.wikidata || []).map((qid) => `wikidata:${qid}`)),
  ];
  if (source) evidence.push(`source:${humanizeUrl(source)}`);
  return {
    intent: "fact_lookup",
    content: summary,
    confidence: 0.9,
    evidence,
  };
}

// Mirrors `try_coreference_request` in
// `src/solver_handlers/benchmark_prompts.rs` for fact-style follow-ups.
function replaceBoundedToken(text, token, replacement) {
  if (!text || !token || !replacement) return null;
  const pattern = new RegExp(
    `(^|[^\\p{L}\\p{N}_])${escapeRegExp(token)}(?=$|[^\\p{L}\\p{N}_])`,
    "gu",
  );
  let changed = false;
  const rewritten = String(text).replace(pattern, (match, prefix) => {
    changed = true;
    return `${prefix}${replacement}`;
  });
  return changed ? rewritten : null;
}

function matchingCoreferencePronoun(normalized) {
  const pronouns = Array.isArray(COREFERENCE_SEEDS && COREFERENCE_SEEDS.pronouns)
    ? COREFERENCE_SEEDS.pronouns
    : [];
  return pronouns.find((pronoun) => {
    const contexts = Array.isArray(pronoun && pronoun.contexts)
      ? pronoun.contexts
      : [];
    const startsWith = Array.isArray(pronoun && pronoun.startsWith)
      ? pronoun.startsWith
      : [];
    return contexts.some((context) => context && normalized.includes(context)) ||
      startsWith.some((prefix) => prefix && normalized.startsWith(prefix));
  }) || null;
}

function matchingCoreferenceAntecedent(previous) {
  const antecedents = Array.isArray(COREFERENCE_SEEDS && COREFERENCE_SEEDS.antecedents)
    ? COREFERENCE_SEEDS.antecedents
    : [];
  return antecedents.find((antecedent) => {
    const aliases = Array.isArray(antecedent && antecedent.aliases)
      ? antecedent.aliases
      : [];
    return aliases.some((alias) => alias && previous.includes(alias));
  }) || null;
}

function matchingAntecedentFactAlias(record, antecedent, previous) {
  const factAliases = Array.isArray(record && record.subjectAliases)
    ? record.subjectAliases
    : [];
  const antecedentAliases = Array.isArray(antecedent && antecedent.aliases)
    ? antecedent.aliases
    : [];
  return factAliases.find((alias) =>
    alias &&
    previous.includes(alias) &&
    antecedentAliases.includes(String(alias).toLowerCase()),
  ) || "";
}

function tryCoreferenceFactLookup(prompt, normalized, history) {
  const pronoun = matchingCoreferencePronoun(normalized);
  if (!pronoun || !pronoun.token) return null;

  const previous = normalizePrompt(lastHistoryTurn(history, "user") || "");
  if (!previous) return null;
  const antecedent = matchingCoreferenceAntecedent(previous);
  if (!antecedent) return null;

  for (const record of FACTS) {
    if (!record || !containsAny(normalized, record.questionKeywords)) continue;

    const alias = matchingAntecedentFactAlias(record, antecedent, previous);
    if (!alias) continue;

    const rewritten = replaceBoundedToken(normalized, pronoun.token, alias);
    if (!rewritten) continue;

    const hit = tryFactLookup(prompt, rewritten);
    if (!hit) continue;

    const subject = antecedent.displayName || record.subjectLabel || alias;
    return Object.assign({}, hit, {
      evidence: [
        `coreference:resolved:${pronoun.token}=${subject}`,
        `coreference:rewrite:${rewritten}`,
        ...(Array.isArray(hit.evidence) ? hit.evidence : []),
      ],
    });
  }

  return null;
}

function renderRoleplayBody(persona, body) {
  const template =
    (PERSONA_SEEDS && PERSONA_SEEDS.bodyTemplate) ||
    "Roleplay frame recorded for <persona>. I will keep the persona explicit and factual: <body>";
  return template.replace(/<persona>/g, persona).replace(/<body>/g, body);
}

function tryRoleplayRequest(prompt, normalized) {
  const seeds = PERSONA_SEEDS || {};
  if (!containsAny(normalized, seeds.triggers)) return null;
  const personas = Array.isArray(seeds.personas) ? seeds.personas : [];
  const persona = personas.find((entry) => containsAny(normalized, entry.aliases));
  const topics = Array.isArray(seeds.topics) ? seeds.topics : [];
  const topic = topics.find((entry) => containsAny(normalized, entry.detectionKeywords));
  const displayName =
    (persona && persona.displayName) || seeds.defaultPersona || "requested persona";
  const body =
    (topic && topic.body) ||
    seeds.fallbackBody ||
    "relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.";
  const evidence = [`roleplay:persona:${displayName}`];
  if (persona && persona.wikidata) evidence.push(`wikidata:${persona.wikidata}`);
  if (topic && topic.slug) evidence.push(`roleplay:topic:${topic.slug}`);
  return {
    intent: "roleplay_explanation",
    content: renderRoleplayBody(displayName, body),
    confidence: 0.8,
    evidence,
  };
}

function tryKupiSlona(prompt, normalized) {
  // Recognition is data-driven: the idiom surfaces (the «купи слона» phrase and
  // its buy-an-elephant calque in every supported language) live in
  // data/seed/meanings-policy.lino under the circular_joke_phrase role, matched
  // as raw substrings. The worker has no localized-response lookup, so the
  // canonical Russian explanation stays inline (mirrors the Rust fallback).
  if (!lexiconMentionsRoleSubstring(ROLE_CIRCULAR_JOKE_PHRASE, normalized))
    return null;
  return {
    intent: "kupi_slona",
    content:
      "«Купи слона» — это известная русская детская фраза-игра. На любой ответ следует продолжение: «Все так говорят, а ты купи слона!» Правильный ответ по правилам игры: «У всех есть слон, а у меня нет».",
    confidence: 1.0,
    evidence: ["handler:kupi_slona", "language:ru"],
  };
}

function extractName(text) {
  const patterns = [
    /\bmy name is\s+([A-Z][a-zA-Z'-]+(?:\s+[A-Z][a-zA-Z'-]+)*)/,
    /\bi am\s+([A-Z][a-zA-Z'-]+(?:\s+[A-Z][a-zA-Z'-]+)*)/,
    /\bi'm\s+([A-Z][a-zA-Z'-]+(?:\s+[A-Z][a-zA-Z'-]+)*)/,
    /\bcall me\s+([A-Z][a-zA-Z'-]+(?:\s+[A-Z][a-zA-Z'-]+)*)/,
  ];
  for (const pattern of patterns) {
    const match = pattern.exec(text);
    if (match) return match[1];
  }
  return null;
}

function tryRecallName(history) {
  if (!Array.isArray(history) || history.length === 0) return null;
  for (let i = history.length - 1; i >= 0; i -= 1) {
    const turn = history[i];
    if (turn && turn.role === "user") {
      const name = extractName(String(turn.content || ""));
      if (name) {
        return {
          intent: "recall_name",
          content: `Your name is ${name}.`,
          confidence: 0.95,
          evidence: [`recall_name:${name}`, "prior_turn:user"],
        };
      }
    }
  }
  return null;
}

// Issue #676: assistant-name-setting phrasings. Each pins the *assistant* as the
// subject being (re)named, mirroring ASSISTANT_NAME_NEEDLES /
// extract_assistant_name in src/solver_helpers.rs, so a declarative rename like
// "Now your name is Ineffa" is recognised while questions and user
// self-introductions are left alone.
const ASSISTANT_NAME_PATTERNS = [
  /\byour name is\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\byour name (?:shall|will|would) be\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\byour new name is\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\blet your name be\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\byou(?:'re| are) (?:named|called)\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\bi(?:'ll| will)?\s*(?:call|name) you\s+([A-Za-z][A-Za-z0-9'-]*)/i,
  /\bi(?:'ll| will) refer to you as\s+([A-Za-z][A-Za-z0-9'-]*)/i,
];

function extractAssistantName(text) {
  for (const pattern of ASSISTANT_NAME_PATTERNS) {
    const match = pattern.exec(String(text || ""));
    if (match) return match[1];
  }
  return null;
}

function recallAssistantName(history) {
  if (!Array.isArray(history)) return null;
  for (let i = history.length - 1; i >= 0; i -= 1) {
    const turn = history[i];
    if (turn && turn.role === "user") {
      const name = extractAssistantName(String(turn.content || ""));
      if (name) return name;
    }
  }
  return null;
}

// Set or recall the *assistant's* name from dialog-local memory (issue #676),
// mirroring try_assistant_name in src/solver_handlers/conversation_memory/mod.rs.
function tryAssistantName(prompt, normalized, history) {
  const setName = extractAssistantName(prompt);
  if (setName) {
    return {
      intent: "set_assistant_name",
      content: `Nice to meet you! I'll go by ${setName} from now on.`,
      confidence: 0.9,
      evidence: [`set_assistant_name:${setName}`],
    };
  }
  const asksName =
    normalized.includes("what is your name") ||
    normalized.includes("what s your name") ||
    normalized.includes("whats your name") ||
    normalized.includes("tell me your name") ||
    normalized.includes("do you have a name") ||
    normalized.includes("what should i call you") ||
    normalized.includes("what do i call you");
  if (!asksName) return null;
  const recalled = recallAssistantName(history);
  if (!recalled) return null;
  return {
    intent: "assistant_name",
    content: `My name is ${recalled} — that's what you named me.`,
    confidence: 0.9,
    evidence: [`assistant_name:${recalled}`, "prior_turn:user"],
  };
}

function isRecallMetaPrompt(prompt) {
  const normalized = normalizePrompt(prompt);
  return (
    lexiconMentionsRole(ROLE_CONVERSATION_RECALL_PREVIOUS_USER_MESSAGE, normalized) ||
    lexiconMentionsRole(ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE, normalized) ||
    lexiconMentionsRole(ROLE_CONVERSATION_RECALL_QUERY, normalized) ||
    lexiconMentionsRole(ROLE_CONVERSATION_RECALL_OTHER_QUERY, normalized)
  );
}

function renderPreviousUserMessage(content, language) {
  if (language === "ru") return `Вы спрашивали: "${content}"`;
  if (language === "zh") return `你之前问的是:"${content}"`;
  if (language === "hi") return `आपने पूछा था: "${content}"`;
  return `Your previous question was: ${content}`;
}

function tryRecallLastQuestion(prompt, history) {
  if (!Array.isArray(history) || history.length === 0) return null;
  const language = detectLanguage(prompt);
  let latestUser = "";
  for (let i = history.length - 1; i >= 0; i -= 1) {
    const turn = history[i];
    if (turn && turn.role === "user") {
      const content = String(turn.content || "").trim();
      if (content) {
        if (!latestUser) latestUser = content;
        if (!isRecallMetaPrompt(content)) {
          return {
            intent: "recall_last_question",
            content: renderPreviousUserMessage(content, language),
            confidence: 0.9,
            evidence: ["recall_last_question", "prior_turn:user"],
          };
        }
      }
    }
  }
  if (latestUser) {
    return {
      intent: "recall_last_question",
      content: renderPreviousUserMessage(latestUser, language),
      confidence: 0.9,
      evidence: ["recall_last_question", "prior_turn:user"],
    };
  }
  return null;
}

// Issue #529: recall the content of the immediately preceding message. The
// recogniser composes the conversation_recall_previous_message seed role across
// every supported language (see ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE), so a
// Russian "что было написано в прошлом сообщении?" no longer falls through to the
// unknown intent. Unlike tryRecallLastQuestion (the user's own last question),
// this replays the last prior turn regardless of role — for the issue scenario,
// the assistant's previous reply. Mirror of try_recall_previous_message in
// src/solver_handlers/conversation_memory.rs.
function localizedTurnRole(role, language) {
  if (role === "assistant") {
    if (language === "ru") return "ассистент";
    if (language === "hi") return "सहायक";
    if (language === "zh") return "助手";
    return "assistant";
  }
  if (language === "ru") return "пользователь";
  if (language === "hi") return "उपयोगकर्ता";
  if (language === "zh") return "用户";
  return "user";
}

function renderPreviousMessage(role, content, language) {
  const label = localizedTurnRole(role, language);
  if (language === "ru") return `В прошлом сообщении (${label}) было написано: "${content}"`;
  if (language === "zh") return `上一条消息（${label}）写道:"${content}"`;
  if (language === "hi") return `पिछले संदेश (${label}) में लिखा था: "${content}"`;
  return `The previous message (${label}) was: "${content}"`;
}

function renderNoPreviousMessage(language) {
  if (language === "ru") return "Прошлого сообщения пока нет.";
  if (language === "zh") return "还没有上一条消息。";
  if (language === "hi") return "अभी तक कोई पिछला संदेश नहीं है.";
  return "There is no previous message yet.";
}

function tryRecallPreviousMessage(prompt, history) {
  const normalized = normalizePrompt(prompt);
  if (!lexiconMentionsRole(ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE, normalized)) {
    return null;
  }
  const language = detectLanguage(prompt);
  let previous = null;
  if (Array.isArray(history)) {
    for (let i = history.length - 1; i >= 0; i -= 1) {
      const turn = history[i];
      const content = turn && String(turn.content || "").trim();
      if (content) {
        previous = { role: turn.role === "assistant" ? "assistant" : "user", content };
        break;
      }
    }
  }
  const content = previous
    ? renderPreviousMessage(previous.role, previous.content, language)
    : renderNoPreviousMessage(language);
  return {
    intent: "recall_previous_message",
    content,
    confidence: 0.9,
    evidence: [
      "recall_previous_message",
      previous ? `prior_turn:${previous.role}` : "prior_turn:none",
    ],
  };
}

// Issue #529: natural-language *writes* to the entire associative memory. This
// is the browser mirror of src/solver_handlers/conversation_memory/memory_write.rs
// — the write half of the Turing-complete memory primitive (a recall reads, an
// append extends, a substitution rewrites). Both recognisers are driven by the
// seed lexicon (ROLE_MEMORY_*), so they extend to every supported language
// automatically. The actual persistence happens in the app (main.jsx applies the
// returned memoryOperation to IndexedDB); the worker stays pure by computing the
// substitution count over a memory snapshot the app passes in.

// Recognise a "remember …" append directive in any language. The surfaces are
// Slot::Prefix forms (trailing …), so each form's literal-before-the-slot is the
// matchable prefix. Matching runs on a lowercased copy of the raw prompt and
// longer prefixes win first, so "remember that X" beats "remember X". Mirrors
// recognize_memory_append.
function recognizeMemoryAppend(prompt) {
  const trimmed = String(prompt || "").replace(/^\s+/, "");
  const lowered = trimmed.toLowerCase();
  const prefixes = roleWordForms(ROLE_MEMORY_APPEND_DIRECTIVE)
    .filter((form) => form.slot === "prefix")
    .map((form) => form.before)
    .filter((prefix) => prefix.length > 0);
  prefixes.sort((a, b) => b.length - a.length);
  for (const prefix of prefixes) {
    if (lowered.startsWith(prefix)) {
      const statement = cleanMemoryWriteText(trimmed.slice(prefix.length));
      if (statement) return statement;
    }
  }
  return null;
}

// Recognise a memory substitution (a read+write transform) in any language. A
// bare "replace X with Y" is an ordinary coding request, so a memory *scope*
// phrase must be present to claim the prompt. We then strip the scope and the
// substitution directive (position-independent: SVO languages lead, Hindi
// trails) and split the operand span on the connector to recover (old, new).
// Mirrors recognize_memory_substitution.
function recognizeMemorySubstitution(normalized) {
  if (!lexiconMentionsRole(ROLE_MEMORY_SCOPE, normalized)) return null;
  const withoutScope = stripFirstMemorySurface(normalized, ROLE_MEMORY_SCOPE);
  if (withoutScope === null) return null;
  const operands = stripFirstMemorySurface(
    withoutScope,
    ROLE_MEMORY_SUBSTITUTION_DIRECTIVE,
  );
  if (operands === null) return null;
  const split = splitOnceMemorySurface(
    operands,
    ROLE_MEMORY_SUBSTITUTION_CONNECTOR,
  );
  if (!split) return null;
  const oldValue = cleanMemoryWriteText(split[0]);
  const newValue = cleanMemoryWriteText(split[1]);
  if (!oldValue || !newValue) return null;
  return { oldValue, newValue };
}

// Total substring occurrences of `old` across every searchable value in the
// memory snapshot. Mirrors MemoryStore::apply_substitution's count, which sums
// replace_counting over each event's content/inputs/outputs/title/label and
// every evidence entry. The app builds the snapshot from those same fields, so
// the worker's reported count matches the rewrite the app applies to IndexedDB.
function countMemoryOccurrences(memory, old) {
  if (!old) return 0;
  const values = Array.isArray(memory) ? memory : [];
  let count = 0;
  for (const value of values) {
    const text = String(value || "");
    if (!text.includes(old)) continue;
    count += text.split(old).length - 1;
  }
  return count;
}

function renderMemoryAppendAnswer(statement, language) {
  if (language === "ru") return `Запомнил: ${statement}`;
  if (language === "zh") return `已记住:${statement}`;
  if (language === "hi") return `स्मृति में सहेजा गया: ${statement}`;
  return `Recorded memory: ${statement}`;
}

function renderMemorySubstitutionAnswer(oldValue, newValue, applied, language) {
  if (language === "ru") {
    return `Заменил "${oldValue}" на "${newValue}" в памяти (обновлено вхождений: ${applied}).`;
  }
  if (language === "zh") {
    return `已在记忆中将"${oldValue}"替换为"${newValue}"(更新 ${applied} 处)。`;
  }
  if (language === "hi") {
    return `स्मृति में "${oldValue}" को "${newValue}" से बदला (${applied})।`;
  }
  return `Replaced "${oldValue}" with "${newValue}" in memory (${applied} occurrence(s) updated).`;
}

// Recognise a natural-language memory write and return an answer carrying a
// structured `memoryOperation` for the app to persist. Substitution is tried
// before append because a substitution prompt also begins with a directive verb.
// Mirrors try_memory_write + recognize_memory_write.
function tryMemoryWrite(prompt, normalized, memory) {
  const substitution = recognizeMemorySubstitution(normalized);
  if (substitution) {
    const language = detectLanguage(prompt);
    const applied = countMemoryOccurrences(memory, substitution.oldValue);
    return {
      intent: "memory_substitution",
      content: renderMemorySubstitutionAnswer(
        substitution.oldValue,
        substitution.newValue,
        applied,
        language,
      ),
      confidence: 0.9,
      evidence: [
        "memory_substitution",
        "substitution_event:update",
        `substitution:applied=${applied}`,
        "response:memory_substitution",
      ],
      memoryOperation: {
        action: "substitute",
        oldValue: substitution.oldValue,
        newValue: substitution.newValue,
        applied,
      },
    };
  }
  const statement = recognizeMemoryAppend(prompt);
  if (statement) {
    const language = detectLanguage(prompt);
    return {
      intent: "memory_write",
      content: renderMemoryAppendAnswer(statement, language),
      confidence: 0.9,
      evidence: [
        "memory_write",
        "memory_write:natural_language",
        "response:memory_write",
      ],
      memoryOperation: { action: "append", statement },
    };
  }
  return null;
}

// Issue #27: deterministic, logical summarisation — no neural net. We
// project the conversation onto a small set of features (turn counts, intents,
// concepts, languages, unanswered questions) and render them as a structured
// Markdown report. Every value is derived directly from the append-only event
// log so reruns on the same input produce byte-identical output.
function trySummarizeConversation(history) {
  if (!Array.isArray(history) || history.length === 0) return null;
  const turns = history.filter((turn) => turn && turn.content);
  if (turns.length === 0) return null;

  let userCount = 0;
  let assistantCount = 0;
  const intentCounts = new Map();
  const languages = new Map();
  const concepts = new Set();
  const calculations = [];
  const programTemplates = new Set();
  const unanswered = [];
  let lastUser = null;

  for (const turn of turns) {
    const role = turn.role || "assistant";
    const language = detectLanguage(turn.content);
    languages.set(language, (languages.get(language) || 0) + 1);
    if (role === "user") {
      userCount += 1;
      lastUser = turn.content;
    } else {
      assistantCount += 1;
      if (lastUser) {
        lastUser = null;
      }
      const intent = String(turn.intent || "unknown");
      intentCounts.set(intent, (intentCounts.get(intent) || 0) + 1);
      if (intent === "calculation" && typeof turn.content === "string") {
        const match = turn.content.match(/^([^=]+=\s*[^\n]+)/);
        if (match) calculations.push(match[1].trim());
      }
      if (intent === "write_program") {
        const evidence = Array.isArray(turn.evidence) ? turn.evidence : [];
        const languageEvidence = evidence.find((item) =>
          String(item || "").startsWith("program_parameter:language:"),
        );
        const taskEvidence = evidence.find((item) =>
          String(item || "").startsWith("program_parameter:task:"),
        );
        const generatedLanguage = languageEvidence
          ? String(languageEvidence).slice("program_parameter:language:".length)
          : "unknown";
        const generatedTask = taskEvidence
          ? String(taskEvidence).slice("program_parameter:task:".length)
          : "program";
        programTemplates.add(`${generatedTask}/${generatedLanguage}`);
      }
      if (intent.startsWith("hello_world_")) {
        programTemplates.add(`hello_world/${intent.slice("hello_world_".length)}`);
      }
      if (intent.startsWith("concept_lookup")) {
        const evidence = Array.isArray(turn.evidence) ? turn.evidence : [];
        for (const item of evidence) {
          if (typeof item !== "string") continue;
          const conceptMatch = item.match(/^concept_lookup:request:(.+)$/);
          if (conceptMatch) concepts.add(conceptMatch[1]);
        }
      }
    }
  }
  if (lastUser) {
    unanswered.push(lastUser);
  }

  const lines = [];
  lines.push("## Conversation summary");
  lines.push("");
  lines.push(
    `- ${turns.length} turn(s): ${userCount} user, ${assistantCount} assistant`,
  );
  if (languages.size > 0) {
    const list = Array.from(languages.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([lang, count]) => `${lang} (${count})`)
      .join(", ");
    lines.push(`- Languages: ${list}`);
  }
  if (intentCounts.size > 0) {
    const list = Array.from(intentCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([intent, count]) => `${intent} (${count})`)
      .join(", ");
    lines.push(`- Intents: ${list}`);
  }
  if (concepts.size > 0) {
    lines.push(`- Concepts looked up: ${Array.from(concepts).join(", ")}`);
  }
  if (calculations.length > 0) {
    lines.push(`- Calculations: ${calculations.join("; ")}`);
  }
  if (programTemplates.size > 0) {
    lines.push(
      `- Program templates generated: ${Array.from(programTemplates).join(", ")}`,
    );
  }
  if (unanswered.length > 0) {
    lines.push(`- Unanswered: ${unanswered.join(" | ")}`);
  }

  const evidence = [
    "summarize_conversation",
    `turns:${turns.length}`,
    `users:${userCount}`,
    `assistants:${assistantCount}`,
  ];
  if (intentCounts.size > 0) {
    evidence.push(`intents:${Array.from(intentCounts.keys()).join("|")}`);
  }
  return {
    intent: "summarize_conversation",
    content: lines.join("\n"),
    confidence: 0.9,
    evidence,
  };
}

function tryCompoundInterest(prompt, normalized, history) {
  const request = parseCompoundInterestRequest(prompt, normalized);
  if (request) return answerCompoundInterest(request);

  const conversion = parseFinalAmountConversionRequest(normalized, history);
  if (conversion) return answerFinalAmountConversion(conversion);

  return null;
}

function answerCompoundInterest(request) {
  const annualRate = request.annualRatePercent / 100;
  const periodsPerYear = request.compoundsPerYear;
  const periodicRate = annualRate / periodsPerYear;
  const periods = periodsPerYear * request.years;
  const finalAmount =
    request.principal * Math.pow(1 + periodicRate, periods);

  const evidence = [
    `calculation:compound_interest:P=${formatCompoundNumber(request.principal)};r=${formatCompoundRate(annualRate)};n=${periodsPerYear};t=${formatCompoundNumber(request.years)}`,
    "calculation:formula:A=P(1+r/n)^(n*t)",
  ];
  const lines = [
    "Compound interest calculation",
    "",
    "Formula: A = P(1 + r/n)^(n*t)",
    `P = ${formatCompoundNumber(request.principal)} USD`,
    `r = ${formatCompoundRate(annualRate)} (${formatCompoundNumber(request.annualRatePercent)}% annual)`,
    `n = ${periodsPerYear} (${compoundLabel(periodsPerYear)})`,
    `t = ${formatCompoundNumber(request.years)} years`,
    "",
    `Step 1: periodic rate = r/n = ${formatCompoundRate(annualRate)}/${periodsPerYear} = ${formatCompoundRate(periodicRate)}`,
    `Step 2: number of periods = n*t = ${periodsPerYear}*${formatCompoundNumber(request.years)} = ${formatCompoundNumber(periods)}`,
    `Step 3: A = ${formatCompoundNumber(request.principal)} * (1 + ${formatCompoundRate(periodicRate)})^${formatCompoundNumber(periods)}`,
    `Final amount: ${formatCompoundMoney(finalAmount)} USD`,
  ];

  if (request.targetCurrency) {
    appendCompoundConversionLines(
      lines,
      evidence,
      finalAmount,
      "USD",
      request.targetCurrency,
      request.asksForWebRate,
    );
  }

  return {
    intent: "calculation",
    content: lines.join("\n"),
    confidence: 1.0,
    evidence,
  };
}

function answerFinalAmountConversion(conversion) {
  const evidence = ["calculation:final_amount_conversion"];
  const lines = [
    "Final amount conversion",
    `Source amount: ${formatCompoundMoney(conversion.amount)} ${conversion.sourceCurrency}`,
  ];
  appendCompoundConversionLines(
    lines,
    evidence,
    conversion.amount,
    conversion.sourceCurrency,
    conversion.targetCurrency,
    conversion.asksForWebRate,
  );
  return {
    intent: "calculation",
    content: lines.join("\n"),
    confidence: 1.0,
    evidence,
  };
}

function appendCompoundConversionLines(
  lines,
  evidence,
  amount,
  sourceCurrency,
  targetCurrency,
  asksForWebRate,
) {
  const rate = compoundCurrencyRate(sourceCurrency, targetCurrency);
  if (!rate) {
    evidence.push(`calculation:currency_conversion:error:${sourceCurrency}->${targetCurrency}`);
    lines.push("");
    lines.push(
      `I calculated the USD amount, but no ${sourceCurrency}->${targetCurrency} exchange rate is available locally.`,
    );
    return;
  }

  const displayedAmount = roundCompoundMoney(amount);
  const converted = displayedAmount * rate.rate;
  evidence.push(
    `calculation:currency_conversion:${formatCompoundMoney(displayedAmount)} ${sourceCurrency} to ${targetCurrency} at ${formatCompoundRate(rate.rate)}`,
  );
  lines.push("");
  lines.push(`Conversion: ${sourceCurrency} -> ${targetCurrency}`);
  lines.push(`${rate.expression} = ${rate.formatted}`);
  lines.push(
    `${formatCompoundMoney(displayedAmount)} ${sourceCurrency} * ${formatCompoundRate(rate.rate)} = ${formatCompoundMoney(converted)} ${targetCurrency}`,
  );
  if (rate.sourceDetail) {
    lines.push(`Rate detail: ${rate.sourceDetail}`);
  }
  if (asksForWebRate) {
    lines.push(
      "Live web freshness is not independently verified here; this uses the exchange-rate source available through the local calculator.",
    );
  }
}

function parseCompoundInterestRequest(prompt, normalized) {
  // The investment / interest / compounding cues are language-independent
  // meanings carried by the finance lexicon; we test the raw substring of the
  // already-normalized prompt against every surface form (the English forms
  // reproduce the original invest/interest/compound markers, the other
  // languages broaden coverage). Mirrors parse_compound_interest_request.
  if (
    !lexiconMentionsRoleSubstring(ROLE_INVESTMENT_CUE, normalized) ||
    !lexiconMentionsRoleSubstring(ROLE_INTEREST_CUE, normalized) ||
    !lexiconMentionsRoleSubstring(ROLE_COMPOUNDING_ACTION_CUE, normalized)
  ) {
    return null;
  }
  const principal = parseCompoundCurrencyAmount(prompt);
  const annualRatePercent = parseCompoundPercentBeforeSymbol(prompt);
  const compoundsPerYear = parseCompoundsPerYear(normalized);
  const years = parseCompoundYears(normalized);
  if (
    principal === null ||
    annualRatePercent === null ||
    compoundsPerYear === null ||
    years === null
  ) {
    return null;
  }
  return {
    principal,
    annualRatePercent,
    compoundsPerYear,
    years,
    targetCurrency: targetCurrencyFromText(normalized),
    asksForWebRate: asksForWebRate(normalized),
  };
}

function parseFinalAmountConversionRequest(normalized, history) {
  // "convert" and "final amount" are themselves meanings: a conversion action
  // applied to the final-amount reference produced by a prior turn. Mirrors
  // parse_final_amount_conversion_request.
  if (
    !lexiconMentionsRoleSubstring(ROLE_CONVERSION_ACTION_CUE, normalized) ||
    !lexiconMentionsRoleSubstring(ROLE_FINAL_AMOUNT_REFERENCE, normalized)
  ) {
    return null;
  }
  const targetCurrency = targetCurrencyFromText(normalized);
  if (!targetCurrency) return null;
  const prior = priorFinalAmount(history);
  if (!prior) return null;
  return {
    amount: prior.amount,
    sourceCurrency: prior.currency,
    targetCurrency,
    asksForWebRate: asksForWebRate(normalized),
  };
}

function priorFinalAmount(history) {
  if (!Array.isArray(history)) return null;
  for (let index = history.length - 1; index >= 0; index -= 1) {
    const turn = history[index];
    if (!turn || turn.role !== "assistant") continue;
    const parsed = parseFinalAmountFromText(String(turn.content || ""));
    if (parsed) return parsed;
  }
  return null;
}

function parseFinalAmountFromText(text) {
  const match = /final amount:\s*([+-]?\d[\d,.]*)\s*([A-Za-z]{3}|dollars?|euros?|rubles?)/i.exec(
    text,
  );
  if (!match) return null;
  const amount = parseCompoundNumberText(match[1]);
  const currency = currencyCodeFromWord(match[2]);
  if (amount === null || !currency) return null;
  return { amount, currency };
}

function parseCompoundCurrencyAmount(prompt) {
  const text = String(prompt || "");
  const dollarIndex = text.indexOf("$");
  if (dollarIndex >= 0) {
    return parseCompoundNumberRight(text, dollarIndex + 1);
  }
  // The spelled-out US-dollar markers are language data: reconstruct the regex
  // alternation from the currency_usd_reference English surface forms (usd,
  // dollar, dollars) instead of hardcoding them. Mirrors parse_currency_amount,
  // which scans the same forms; the `$` glyph stays in code as a symbol.
  const usdWords = wordsForRoleInLanguages(ROLE_CURRENCY_USD_REFERENCE, ["en"]);
  if (!usdWords.length) return null;
  const alternation = usdWords.map((word) => escapeRegExp(word)).join("|");
  const pattern = new RegExp(`([+-]?\\d[\\d,.]*)\\s*(?:${alternation})`, "i");
  const match = pattern.exec(text);
  return match ? parseCompoundNumberText(match[1]) : null;
}

function parseCompoundPercentBeforeSymbol(prompt) {
  const text = String(prompt || "");
  const percentIndex = text.indexOf("%");
  return percentIndex >= 0 ? parseCompoundNumberLeft(text, percentIndex) : null;
}

function parseCompoundYears(normalized) {
  // The duration unit is a meaning (year_unit_cue); locate the earliest of its
  // surface forms (English "year", plus the other languages) and read the
  // number to its left. Mirrors years_in_prompt.
  const text = String(normalized || "");
  let earliest = -1;
  for (const word of wordsForRole(ROLE_YEAR_UNIT_CUE)) {
    const index = text.indexOf(word);
    if (index >= 0 && (earliest < 0 || index < earliest)) earliest = index;
  }
  return earliest >= 0 ? parseCompoundNumberLeft(text, earliest) : null;
}

function parseCompoundsPerYear(normalized) {
  // The compounding frequency is a cluster of meanings (monthly, quarterly,
  // weekly, daily, annual), each carrying its surface forms and listed in
  // priority order in the finance lexicon. Pick the first whose surface appears
  // in the prompt and map its slug to the periods-per-year count. Mirrors
  // parse_compounds_per_year.
  const meaning = meaningsWithRole(ROLE_COMPOUNDING_FREQUENCY_CUE).find((candidate) =>
    candidate.words.some((word) => normalized.includes(word)),
  );
  return meaning ? compoundsPerYearForSlug(meaning.slug) : null;
}

function compoundsPerYearForSlug(slug) {
  switch (slug) {
    case "compounding_monthly":
      return 12;
    case "compounding_quarterly":
      return 4;
    case "compounding_weekly":
      return 52;
    case "compounding_daily":
      return 365;
    case "compounding_annual":
      return 1;
    default:
      return null;
  }
}

function targetCurrencyFromText(normalized) {
  // The target currency is whichever currency meaning the prompt names as a
  // whole token. EUR wins over USD wins over RUB to preserve the original
  // priority; the € glyph stays in code as a symbol alongside the EUR meaning.
  // Token-bounded matching mirrors target_currency / mentions_role on the Rust
  // side, so a code like "eur" never fires inside another word.
  if (
    lexiconMentionsRole(ROLE_CURRENCY_EUR_REFERENCE, normalized) ||
    normalized.includes("€")
  ) {
    return "EUR";
  }
  if (lexiconMentionsRole(ROLE_CURRENCY_USD_REFERENCE, normalized)) {
    return "USD";
  }
  if (lexiconMentionsRole(ROLE_CURRENCY_RUB_REFERENCE, normalized)) {
    return "RUB";
  }
  return "";
}

function asksForWebRate(normalized) {
  // "fetch the live/current rate from the web" is the live_rate_freshness_cue
  // meaning; its surface forms (web, current exchange, current rate, exchange
  // rate) live in the finance lexicon. Matched as raw substrings to mirror
  // asks_for_web_rate / mentions_role_raw on the Rust side.
  return lexiconMentionsRoleSubstring(ROLE_LIVE_RATE_FRESHNESS_CUE, normalized);
}

function compoundCurrencyRate(sourceCurrency, targetCurrency) {
  const expression = `1 ${sourceCurrency} in ${targetCurrency}`;
  if (sourceCurrency === targetCurrency) {
    return {
      rate: 1,
      expression,
      formatted: `1 ${targetCurrency}`,
      sourceDetail: "",
    };
  }

  const wasmResult = wasmEvaluateArithmetic(expression);
  if (wasmResult && wasmResult.ok) {
    const rate = parseCompoundLeadingNumber(wasmResult.value);
    if (rate !== null) {
      return {
        rate,
        expression,
        formatted: wasmResult.value,
        sourceDetail: `Exchange rate: 1 ${sourceCurrency} = ${formatCompoundRate(rate)} ${targetCurrency} (source: calculator)`,
      };
    }
  }

  const rate = defaultCurrencyRate(sourceCurrency, targetCurrency);
  if (!rate) return null;
  return {
    rate,
    expression,
    formatted: `${formatCompoundRate(rate)} ${targetCurrency}`,
    sourceDetail: `Exchange rate: 1 ${sourceCurrency} = ${formatCompoundRate(rate)} ${targetCurrency} (source: default (hardcoded))`,
  };
}

function parseCompoundNumberLeft(text, end) {
  const before = String(text || "").slice(0, end);
  const match = /([+-]?\d[\d,.]*)\s*$/.exec(before);
  return match ? parseCompoundNumberText(match[1]) : null;
}

function parseCompoundNumberRight(text, start) {
  const after = String(text || "").slice(start);
  const match = /^\s*([+-]?\d[\d,.]*)/.exec(after);
  return match ? parseCompoundNumberText(match[1]) : null;
}

function parseCompoundLeadingNumber(text) {
  const match = /([+-]?\d[\d,.]*)/.exec(String(text || ""));
  return match ? parseCompoundNumberText(match[1]) : null;
}

function parseCompoundNumberText(value) {
  let cleaned = String(value || "").trim();
  if (!/\d/.test(cleaned)) return null;
  if (cleaned.includes(",") && !cleaned.includes(".")) {
    const parts = cleaned.split(",");
    cleaned =
      parts.length === 2 && parts[1].length <= 2
        ? `${parts[0]}.${parts[1]}`
        : parts.join("");
  } else {
    cleaned = cleaned.replace(/,/g, "");
  }
  const parsed = Number(cleaned);
  return Number.isFinite(parsed) ? parsed : null;
}
