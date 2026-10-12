// Seed hydration with its bootstrap fallbacks, the localized response lookup,
// response-language detection, prompt normalization and concept record
// matching.
// Loaded by ../formal_ai_worker.js.
let wasm;
let mode = "wasm worker";

// Offline responses are shipped seed data generated from the canonical Links
// Notation corpus, then replaced by the live seed at initialization.
let MULTILINGUAL_ANSWERS = JSON.parse(JSON.stringify(self.FORMAL_AI_BOOTSTRAP_RESPONSES || {}));
let CONCEPTS = [];
let CONCEPT_CONTEXTS = [];
let FACTS = [];
let PROJECTS = [];
// Filled from data/seed/brainstorm-seeds.lino and data/seed/personas.lino when
// the seed loads (issue #918: the bootstrap copies of their lists, items and
// templates are gone, so the seed is the only place those words live).
let BRAINSTORM_SEEDS = { triggers: [], categories: [], defaultCount: 0, countCardinal: "" };
let PERSONA_SEEDS = {
  triggers: [],
  defaultPersona: "",
  bodyTemplate: "",
  fallbackBody: "",
  personas: [],
  topics: [],
};
let COREFERENCE_SEEDS = { pronouns: [], antecedents: [] };
let TOOLS = [];
let SEED_RAW = {};
let NUMERIC_LIST_OPERATIONS_LINO = "";
let CODING_IDIOMS_LINO = "";
let TERMINAL_COMMANDS_LINO = "";
let SHELL_INTENTS_LINO = "";
let PROGRAM_PLAN_RULES_LINO = "";
let OPERATION_VOCABULARY_LINO = "";
let MARKET_PRICE_REFERENCES_LINO = "";
let ENTITY_NAMES_LINO = "";
let cachedKnownEntityNames = null;
let cachedTimezonePlaces = null;
let MEANINGS_LINO = "";
let AGENT_INFO = {};
// Bootstrap copy of `data/seed/language-detection.lino`, replaced verbatim by
// the seed registry at init() time. Issue #706: every field a language needs
// lives here as data, so registering a new language never edits worker code.
let LANGUAGE_RULES = [
  { language: "ru", script: "Cyrillic", start: 0x0400, end: 0x04ff },
  { language: "hi", script: "Devanagari", start: 0x0900, end: 0x097f },
  { language: "zh", script: "Han", start: 0x4e00, end: 0x9fff },
  { language: "en", script: "Latin", start: 0x0041, end: 0x007a, fallback: true, alphabeticOnly: true },
];
let PROMPT_PATTERNS = [];

function seedFileBaseName(path) {
  return String(path || "").replace(/\\/g, "/").split("/").pop() || "";
}

function seedRawText(raw, fileName) {
  const entries = Object.entries(raw || {});
  for (const [path, text] of entries) {
    if (seedFileBaseName(path) === fileName && text) return String(text);
  }
  return "";
}

function seedRawTexts(raw, predicate) {
  return Object.entries(raw || {})
    .filter(([path, text]) => text && predicate(seedFileBaseName(path)))
    .sort(([left], [right]) => seedFileBaseName(left).localeCompare(seedFileBaseName(right)))
    .map(([, text]) => String(text));
}

function hydrateLinoSeedText(raw) {
  NUMERIC_LIST_OPERATIONS_LINO = seedRawText(raw, "numeric-list-operations.lino");
  CODING_IDIOMS_LINO = seedRawText(raw, "coding-idioms.lino");
  TERMINAL_COMMANDS_LINO = seedRawText(raw, "terminal-commands.lino");
  SHELL_INTENTS_LINO = seedRawText(raw, "shell-intents.lino");
  PROGRAM_PLAN_RULES_LINO = seedRawText(raw, "program-plan-rules.lino");
  OPERATION_VOCABULARY_LINO = seedRawText(raw, "operation-vocabulary.lino");
  MARKET_PRICE_REFERENCES_LINO = seedRawText(raw, "market-price-references.lino");
  ENTITY_NAMES_LINO = seedRawText(raw, "entity-names.lino");
  const unknownOpeners = seedRawText(raw, "unknown-openers.lino");
  if (unknownOpeners) {
    UNKNOWN_OPENERS_LINO = unknownOpeners;
    cachedUnknownOpenerRegistry = null;
  }
  cachedKnownEntityNames = null;
  cachedTimezonePlaces = null;
  MEANINGS_LINO = seedRawTexts(
    raw,
    (fileName) => fileName === "meanings.lino" || /^meanings-[a-z0-9-]+\.lino$/.test(fileName),
  ).join("\n");

  cachedNumericListOntology = null;
  cachedCodingIdioms = null;
  cachedTerminalCommandVocabulary = null;
  cachedShellIntentVocabulary = null;
  cachedOperationVocabulary = null;
  cachedProgramPlanRules = null;
  cachedMeaningLexicon = null;
  cachedMarketPriceReferences = null;
  if (typeof installSeedProgramTasks === "function") installSeedProgramTasks(raw); // R1021-6, formal_ai_worker_program_requests.js
  if (typeof installDocumentedLanguageCommands === "function") installDocumentedLanguageCommands(raw); // R1165-6, formal_ai_worker_documented_commands.js
}
// Intent routing rules loaded from `seed/intent-routing.lino` at init time.
// `intents` mirror `seed::IntentRoute` from the Rust crate, so the browser
// and the Rust solver behave identically when classifying prompts. There is
// no bootstrap copy of the routes and their phrases (R1188-U1): until init()
// hydrates them the list is empty, as the unknown-opener pools are.
let INTENT_ROUTING = {
  intents: [],
  articlePrefixes: ["the ", "a ", "an "],
  tracePrefixes: ["answer_", "trace_"],
};

function fallbackEntry(intent) {
  const seeded = (self.FORMAL_AI_BOOTSTRAP_RESPONSES || {})[intent];
  if (seeded && seeded.en) return normalizeEntry(seeded.en, intent);
  // R1188-U1: an intent with no seeded text answers with the seeded unknown
  // response; before the seed is hydrated it degrades to the intent slug (a
  // meaning, never a bootstrap copy of seed prose), as cached_response does in
  // rust/src/engine_responses.rs.
  const unknown = intent === "unknown" ? null : (MULTILINGUAL_ANSWERS.unknown || {}).en;
  if (unknown) return normalizeEntry(unknown, "unknown");
  return { text: intent, variants: [intent] };
}

function normalizeEntry(value, intent) {
  if (value && typeof value === "object" && typeof value.text === "string") {
    const variants =
      Array.isArray(value.variants) && value.variants.length > 0
        ? value.variants
        : [value.text];
    const acknowledgements = Array.isArray(value.acknowledgements)
      ? value.acknowledgements.filter(Boolean)
      : [];
    const followUps = Array.isArray(value.followUps)
      ? value.followUps.filter(Boolean)
      : [];
    return {
      text: value.text,
      variants: variants,
      acknowledgements: acknowledgements,
      followUps: followUps,
    };
  }
  if (typeof value === "string") {
    return {
      text: value,
      variants: [value],
      acknowledgements: [],
      followUps: [],
    };
  }
  return fallbackEntry(intent);
}

function responseEntryFor(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  const raw = table[language] || table.en || fallbackEntry(intent);
  return normalizeEntry(raw, intent);
}

function answerFor(intent, language, options) {
  const opts = options || {};
  const entry = responseEntryFor(intent, language);
  if (opts.randomize && Array.isArray(entry.variants) && entry.variants.length > 1) {
    const idx = Math.floor(Math.random() * entry.variants.length);
    return entry.variants[idx] || entry.text;
  }
  return entry.text;
}

function normalizeAssistantNamePreference(value) {
  return String(value || "")
    .replace(/[\r\n\t]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^[`"']+|[`"']+$/g, "")
    .trim()
    .slice(0, 64);
}

function assistantNameAnswer(language, preferences) {
  const name = normalizeAssistantNamePreference(
    preferences && preferences.assistantName,
  );
  if (!name) return answerFor("assistant_name", language);
  if (language === "ru") {
    return `Меня зовут ${name}. Я formal AI.`;
  }
  if (language === "hi") {
    return `मेरा नाम ${name} है। मैं formal AI हूँ।`;
  }
  if (language === "zh") {
    return `我的名字是 ${name}。我是 formal AI。`;
  }
  return `My name is ${name}. I'm formal AI.`;
}

// Mirrors `rust/src/web_engine_core.rs::parse_opener_registry`. Issue #706 moved the
// pools out of this file into `data/seed/unknown-openers.lino`, so registering a
// language's openers is a data edit shared by the Rust core, the WASM worker and
// this worker. There is no bootstrap copy: until `init()` hydrates the text the
// pools are empty and the unknown answer is the seed answer alone, so an opener
// is only ever added once its data has actually been loaded. The first opener of
// a pool equals the one embedded in the seed text, so the varied answer stays a
// strict superset of the seed; the same prompt always picks the same opener
// (FNV-1a hash, mirrored from `stableBehaviorRuleId`).
let UNKNOWN_OPENERS_LINO = "";
let cachedUnknownOpenerRegistry = null;

function linoChildValues(node, name) {
  return node.children.filter((child) => child.name === name).map((child) => child.value);
}

function unknownOpenerRegistry() {
  if (cachedUnknownOpenerRegistry) return cachedUnknownOpenerRegistry;
  const root = parseLinoTree(String(UNKNOWN_OPENERS_LINO || "")).children[0] || { children: [] };
  cachedUnknownOpenerRegistry = {
    fallbackLanguage: linoChildValues(root, "fallback_language")[0] || "en",
    sentenceSeparators: linoChildValues(root, "sentence_separator"),
    pools: root.children.filter((child) => child.name === "pool")
      .map((pool) => ({
        language: linoChildValues(pool, "language")[0] || "",
        openers: linoChildValues(pool, "opener"),
      }))
      .filter((pool) => pool.openers.length > 0),
  };
  return cachedUnknownOpenerRegistry;
}

function unknownOpenersFor(language) {
  const { pools, fallbackLanguage } = unknownOpenerRegistry();
  const match = pools.find((pool) => pool.language === language) ||
    pools.find((pool) => pool.language === fallbackLanguage);
  return match ? match.openers : [];
}

function selectUnknownOpener(prompt, language) {
  const fromWasm = wasmSelectUnknownOpener(prompt, language);
  if (fromWasm) return fromWasm;
  const pool = unknownOpenersFor(language);
  if (pool.length === 0) return "";
  const trimmed = String(prompt || "").trim();
  if (trimmed === "") return pool[0];
  const id = stableBehaviorRuleId("unknown_opener", trimmed);
  const hex = id.split("_").pop() || "0";
  let value;
  try {
    value = BigInt(`0x${hex}`);
  } catch (_err) {
    value = 0n;
  }
  const index = Number(value % BigInt(pool.length));
  return pool[index] || pool[0];
}

function stripLeadingUnknownOpener(text, language) {
  const trimmed = String(text || "").trimStart();
  const openers = unknownOpenersFor(language);
  for (const known of openers) {
    if (trimmed.startsWith(known)) {
      return trimmed.slice(known.length).trimStart();
    }
  }
  for (const separator of unknownOpenerRegistry().sentenceSeparators) {
    const idx = trimmed.indexOf(separator);
    if (idx >= 0) {
      return trimmed.slice(idx + separator.length).trimStart();
    }
  }
  return trimmed;
}

function unknownAnswerWithVariation(prompt, language) {
  const seedText = answerFor("unknown", language);
  const opener = selectUnknownOpener(prompt, language);
  if (!opener) return seedText;
  const body = stripLeadingUnknownOpener(seedText, language);
  if (!body) return opener;
  return `${opener} ${body}`;
}

function numericPreference(value, fallback, min, max) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function pickVariant(values, randomize) {
  if (!Array.isArray(values) || values.length === 0) return "";
  if (!randomize || values.length === 1) return values[0];
  return values[Math.floor(Math.random() * values.length)] || values[0];
}

function includeFollowUpQuestion(probability, randomize) {
  if (probability <= 0) return false;
  if (probability >= 1) return true;
  if (!randomize) return probability >= 0.5;
  return Math.random() < probability;
}

function courtesyResponseFor(language, preferences) {
  const prefs = preferences || {};
  const entry = responseEntryFor("courtesy_response", language);
  const temperature = numericPreference(prefs.temperature, 0.7, 0, 1);
  const followUpProbability = numericPreference(
    prefs.followUpProbability,
    0.75,
    0,
    1,
  );
  const randomize = temperature > 0;
  const acknowledgements =
    entry.acknowledgements.length > 0 ? entry.acknowledgements : [entry.text];
  const followUps = entry.followUps;
  const acknowledgement = pickVariant(acknowledgements, randomize);
  const includeFollowUp =
    followUps.length > 0 &&
    includeFollowUpQuestion(followUpProbability, randomize);
  return {
    content: includeFollowUp
      ? `${acknowledgement} ${pickVariant(followUps, randomize)}`
      : acknowledgement,
    temperature: temperature,
    randomize: randomize,
    followUpProbability: followUpProbability,
    followUpIncluded: includeFollowUp,
  };
}

function definitionFusionByDefault(preferences) {
  const value = preferences && preferences.definitionFusion;
  if (value === true) return true;
  if (value === false) return false;
  const normalized = String(value || "").trim().toLowerCase();
  return ["auto", "on", "true", "1", "merge", "fusion"].includes(normalized);
}

// Language detection and prompt normalization are owned by the Rust core
// (`rust/src/web_engine_core.rs`) and exposed to the worker through the WASM
// exports `engine_detect_language` and `engine_normalize_prompt`. The JS
// branches below are pre-WASM fallbacks used during init() and on browsers
// that could not instantiate the worker — they must stay byte-for-byte
// compatible with the Rust path so the offline trace and the live answer
// agree (PR #134 feedback 4489651616).
// Issue #556: during a response-language follow-up the solver replays the
// previous request with a language forced onto every localizable handler. The
// handlers derive their output language from detectLanguage(prompt), so the
// forced language is applied here — a single seam that every handler already
// reads, mirroring how SolverConfig.forced_response_language flows through the
// Rust dispatch. solve() sets this via setForcedResponseLanguage() and always
// restores the previous value, so nested replays remain balanced.
let FORCED_RESPONSE_LANGUAGE = null;

function setForcedResponseLanguage(language) {
  const previous = FORCED_RESPONSE_LANGUAGE;
  FORCED_RESPONSE_LANGUAGE = isKnownResponseLanguage(language) ? language : null;
  return previous;
}

function detectLanguage(prompt) {
  if (FORCED_RESPONSE_LANGUAGE) return FORCED_RESPONSE_LANGUAGE;
  const text = String(prompt || "");
  const fromWasm = wasmDetectLanguage(text);
  return fromWasm !== null ? fromWasm : crateModule("crate/language.mjs").detect(text);
}

// Registry-driven fallback detection, reached only when the Rust→WASM worker is
// unavailable (a `file://` demo). `detect_with` in `src/language.rs` stays the
// authority and owns the full tie-breaking ladder; issue #658 (R380) keeps this
// mirror thin, so it is the reduced form: the dominant script other than the
// fallback's decides, and a language sharing the fallback script (Spanish on
// Latin) votes through its markers only when no rival script is present. Issue
// #706: every field read here comes from `data/seed/language-detection.lino`,
// so registering a language never edits this file.
function detectLanguageFromRules(text) {
  const back = LANGUAGE_RULES.find((rule) => rule.fallback) || {};
  const counts = new Map();
  let other = 0;
  for (const character of text) {
    const code = character.codePointAt(0);
    const rule = LANGUAGE_RULES.find((candidate) => code >= candidate.start &&
      code <= candidate.end && (!candidate.alphabeticOnly || /\p{L}/u.test(character)));
    if (rule) counts.set(rule.script, (counts.get(rule.script) || 0) + 1);
    else if (/\p{L}/u.test(character)) other += 1;
  }
  const at = (rule) => counts.get(rule && rule.script) || 0;
  const rival = LANGUAGE_RULES
    .filter((rule) => !rule.fallback && rule.script !== back.script && at(rule) > 0)
    .sort((left, right) => at(right) - at(left))[0];
  if (other > at(back) && (!rival || other >= at(rival))) return "unknown";
  if (rival) return rival.language;
  const normalized = text.toLowerCase();
  const byMarker = LANGUAGE_RULES.find((rule) => !rule.fallback && at(rule) > 0 &&
    (rule.markers || []).some((marker) => normalized.includes(String(marker).toLowerCase())));
  return byMarker ? byMarker.language : back.language || "en";
}

// Issue #324: the user can choose which language drives responses. The default
// ("last_message") answers in the detected language of the current message
// (fixing the Russian-prompt/English-answer bug). "preferred" pins responses to
// an explicitly selected language and "ui" follows the UI-language preference.
// Both fall back to the detected language when their source is "auto"/unset so
// the deterministic default behavior is never lost.
const RESPONSE_LANGUAGE_MODES = ["last_message", "preferred", "ui"];

// Issue #706: the set of answerable languages is the detection registry, not
// a hardcoded list — registering a language in
// `data/seed/language-detection.lino` makes it selectable here too.
function isKnownResponseLanguage(slug) {
  if (!slug) return false;
  return LANGUAGE_RULES.some((rule) => rule.language === slug);
}

function responseLanguageFor(detected, preferences, userContext) {
  const prefs = preferences || {};
  const mode = RESPONSE_LANGUAGE_MODES.includes(prefs.responseLanguage)
    ? prefs.responseLanguage
    : "last_message";
  if (mode === "preferred" && isKnownResponseLanguage(prefs.preferredLanguage)) {
    return prefs.preferredLanguage;
  }
  if (mode === "ui") {
    if (isKnownResponseLanguage(prefs.uiLanguage)) return prefs.uiLanguage;
    // "auto" UI language follows the browser; fall back to the detected
    // message language when no concrete browser language is supplied.
    // `browserLanguages` may arrive as an array or a comma-joined string
    // (see `collectUserContext` in app.js).
    const raw = userContext ? userContext.browserLanguages : null;
    const browser = Array.isArray(raw)
      ? raw
      : typeof raw === "string"
        ? raw.split(",")
        : [];
    for (const tag of browser) {
      const slug = String(tag || "").slice(0, 2).toLowerCase();
      if (isKnownResponseLanguage(slug)) return slug;
    }
  }
  return detected;
}

// CONCEPTS is populated from `seed/concepts.lino` at init() time.

function normalizePrompt(prompt) {
  const text = String(prompt || "");
  const fromWasm = wasmNormalizePrompt(text);
  if (fromWasm !== null) return expandSeededContractions(fromWasm);
  // Keep letters, numbers and every Unicode mark (category M): Devanagari
  // matras, the nukta and the virama are marks, so a bare \p{L}\p{N} filter
  // would strip them and corrupt Hindi words (issue #312). Mark-awareness via
  // \p{M} mirrors the Rust `normalize_prompt`, which keeps `is_alphanumeric()`
  // characters plus its script-combining-mark ranges. Crucially it does NOT
  // keep the whole U+0900–U+097F block: Indic punctuation such as the danda
  // "।" (U+0964) is category Po, so both sides collapse it to a space. The
  // boundary-aware role matcher (issue #386) depends on that parity — a
  // retained danda would defeat the whole-token match for phrases like
  // "अपना परिचय दो।".
  return expandSeededContractions(text.toLowerCase().replace(/[^\p{L}\p{N}\p{M}]+/gu, " ").trim());
}

// Issue #1175 p020: normalization turns an apostrophe into a space, so "you're" reaches every matcher as the
// token pair `you re` and never equals a surface the seed writes out ("you are"). The `contraction` /
// `expansion` pairs of data/seed/languages.lino rewrite such a pair in place. Mirrors `expand_contractions`
// in rust/src/engine.rs; a pair is cached only once the ledger has loaded.
let cachedSeededContractions = null;
function expandSeededContractions(normalized) {
  if (!cachedSeededContractions) {
    const ledger = seedRawText(SEED_RAW, "languages.lino");
    if (!ledger) return normalized;
    cachedSeededContractions = [];
    let contracted = null;
    for (const [, key, value] of ledger.matchAll(/^\s*(contraction|expansion) "([^"]*)"\s*$/gmu)) {
      if (key === "contraction") contracted = value.split(" ").filter(Boolean);
      else if (contracted && contracted.length) cachedSeededContractions.push({ contracted, expansion: value });
      if (key === "expansion") contracted = null;
    }
  }
  const tokens = String(normalized || "").split(" ");
  const out = [];
  for (let index = 0; index < tokens.length;) {
    const pair = cachedSeededContractions.find((p) => p.contracted.every((token, at) => tokens[index + at] === token));
    out.push(pair ? pair.expansion : tokens[index]);
    index += pair ? pair.contracted.length : 1;
  }
  return out.join(" ");
}

function normalizeConceptTerm(value) {
  let lower = String(value || "").toLowerCase();
  for (const prefix of conceptQueryCues("article")) {
    if (lower.startsWith(prefix)) {
      lower = lower.slice(prefix.length);
      break;
    }
  }
  return lower.trim().replace(/[?.!,;:]+$/g, "").trim();
}

function recordMatchesTerm(record, normalized) {
  return (
    normalizeConceptTerm(record.term) === normalized ||
    normalizeConceptTerm(record.slug) === normalized ||
    (Array.isArray(record.aliases) &&
      record.aliases.some(
        (alias) => normalizeConceptTerm(alias) === normalized,
      ))
  );
}

function recordMatchesQueryTerm(record, normalized, contextNormalized) {
  if (recordMatchesTerm(record, normalized)) return true;
  if (!contextNormalized) return false;
  return recordMatchesTerm(record, `${normalized} ${contextNormalized}`);
}

function contextRecordMatches(contextRecord, contextNormalized) {
  if (!contextRecord) return false;
  if (
    Array.isArray(contextRecord.aliases) &&
    contextRecord.aliases.some(
      (alias) => normalizeConceptTerm(alias) === contextNormalized,
    )
  ) {
    return true;
  }
  return (
    Array.isArray(contextRecord.labels) &&
    contextRecord.labels.some(
      (label) => normalizeConceptTerm(label.text) === contextNormalized,
    )
  );
}

function resolveContextRecord(contextNormalized) {
  if (!contextNormalized) return null;
  for (const record of CONCEPT_CONTEXTS) {
    if (contextRecordMatches(record, contextNormalized)) return record;
  }
  return null;
}

function recordHasContext(record, contextNormalized) {
  if (
    Array.isArray(record.contexts) &&
    record.contexts.some(
      (candidate) => normalizeConceptTerm(candidate) === contextNormalized,
    )
  ) {
    return true;
  }
  // Registry fallback: resolve the user-supplied context through the
  // concept-contexts registry and see whether the resolved record's slug is
  // referenced by the concept's `contextLinks` list. Matches the Rust
  // ranker (src/concepts.rs::record_has_context).
  const contextRecord = resolveContextRecord(contextNormalized);
  if (contextRecord && Array.isArray(record.contextLinks)) {
    return record.contextLinks.some(
      (slug) => String(slug).trim() === contextRecord.slug,
    );
  }
  return false;
}

function localizedConceptFor(record, language) {
  if (!record || !Array.isArray(record.localized)) return null;
  return (
    record.localized.find((loc) => loc && loc.language === language) ||
    record.localized.find((loc) => loc && loc.language === "en") ||
    null
  );
}

function contextLabelFor(contextRecord, language) {
  if (!contextRecord || !Array.isArray(contextRecord.labels)) {
    return null;
  }
  const exact = contextRecord.labels.find(
    (label) => label && label.language === language,
  );
  if (exact && exact.text) return exact.text;
  const english = contextRecord.labels.find(
    (label) => label && label.language === "en",
  );
  if (english && english.text) return english.text;
  return contextRecord.slug || null;
}

function rankConceptForPair(termRaw, contextRaw) {
  const normalized = normalizeConceptTerm(termRaw);
  if (!normalized) return null;
  const contextNormalized = contextRaw ? normalizeConceptTerm(contextRaw) : "";

  const termMatches = CONCEPTS.filter((record) =>
    recordMatchesQueryTerm(record, normalized, contextNormalized),
  );
  if (termMatches.length === 0) return null;

  if (contextNormalized) {
    const ctxHit = termMatches.find((record) =>
      recordHasContext(record, contextNormalized),
    );
    if (ctxHit) {
      return {
        record: ctxHit,
        contextMatch: true,
        context: contextNormalized,
      };
    }
  }

  // No context match: prefer records with no contexts declared.
  termMatches.sort((a, b) => {
    const ac = (Array.isArray(a.contexts) && a.contexts.length > 0) ? 1 : 0;
    const bc = (Array.isArray(b.contexts) && b.contexts.length > 0) ? 1 : 0;
    return ac - bc;
  });
  return {
    record: termMatches[0],
    contextMatch: false,
    context: contextNormalized || null,
  };
}

function lookupConceptQuery(query) {
  if (!query) return null;
  const direct = rankConceptForPair(query.term, query.context);
  if (query.context) {
    const reversed = rankConceptForPair(query.context, query.term);
    if (reversed && (!direct || (!direct.contextMatch && reversed.contextMatch))) {
      return reversed;
    }
  }
  return direct || null;
}

function lookupConcept(term) {
  const hit = lookupConceptQuery({ term: term, context: null });
  return hit ? hit.record : null;
}

// The concept-lookup patterns of data/seed/prompt-patterns.lino, longest-first
// so "what is a " beats "what is " when both match. There is no bootstrap
// copy: the seed is the only place these words live (issue #918).
function conceptPatternsByKind(kind) {
  const matches = PROMPT_PATTERNS.filter(
    (p) => p && p.intent === "concept_lookup" && p.kind === kind && p.text,
  ).map((p) => p.text);
  // Sort longest-first so more specific patterns win.
  matches.sort((a, b) => b.length - a.length);
  return matches;
}

let cachedConceptResponseLanguageMarkers = null;

function meaningDefinedLanguageCode(meaning) {
  const ledger = seedRawText(SEED_RAW, "languages.lino");
  for (const slug of meaning && Array.isArray(meaning.definedBy)
    ? meaning.definedBy
    : []) {
    const name = slug.replace(/^language_/, "");
    const match = new RegExp(`^  language ([a-z-]+)\\r?\\n    name ${name}$`, "im").exec(ledger);
    if (match) return match[1];
  }
  return null;
}

function conceptResponseLanguageMarkers() {
  if (cachedConceptResponseLanguageMarkers) {
    return cachedConceptResponseLanguageMarkers;
  }
  const markers = [];
  for (const meaning of meaningsWithRole(ROLE_RESPONSE_LANGUAGE_MARKER)) {
    const language = meaningDefinedLanguageCode(meaning);
    if (!language) continue;
    for (const word of meaning.words || []) {
      const marker = String(word || "").toLowerCase();
      if (marker) markers.push({ marker, language });
    }
  }
  markers.sort((a, b) => b.marker.length - a.marker.length);
  cachedConceptResponseLanguageMarkers = markers;
  return markers;
}

function stripTrailingResponseLanguageMarker(original, lower) {
  const sourceOriginal = String(original || "").trim();
  const sourceLower = String(
    lower || sourceOriginal.toLowerCase(),
  ).trim();
  for (const { marker, language } of conceptResponseLanguageMarkers()) {
    if (!sourceLower.endsWith(marker)) continue;
    const start = sourceLower.length - marker.length;
    if (start === 0) continue;
    const before = sourceLower.slice(0, start).slice(-1);
    if (!isResponseLanguageMarkerBoundary(before, marker)) continue;
    const stemOriginal = sourceOriginal
      .slice(0, start)
      .trim()
      .replace(/[,，;；:：、]+$/u, "")
      .trim();
    const stemLower = sourceLower
      .slice(0, start)
      .trim()
      .replace(/[,，;；:：、]+$/u, "")
      .trim();
    if (!stemLower) continue;
    return {
      original: stemOriginal || stemLower,
      lower: stemLower,
      language,
    };
  }
  return { original: sourceOriginal, lower: sourceLower, language: null };
}

function isResponseLanguageMarkerBoundary(before, marker) {
  return (
    /\s/u.test(before) ||
    /^[,，;；:：、]$/u.test(before) ||
    containsCjk(marker)
  );
}

function splitTermAndContext(bodyOriginal, bodyLower) {
  const delimiters = conceptPatternsByKind("context_delimiter");
  for (const delimiter of delimiters) {
    const idx = bodyLower.indexOf(delimiter);
    if (idx >= 0) {
      const term = bodyLower.slice(0, idx).trim();
      const context = bodyLower.slice(idx + delimiter.length).trim();
      const termOriginal = bodyOriginal.slice(0, idx).trim();
      const contextOriginal = bodyOriginal
        .slice(idx + delimiter.length)
        .trim();
      if (term && context) {
        return {
          term: term,
          context: context,
          termOriginal: termOriginal || term,
          contextOriginal: contextOriginal || context,
        };
      }
    }
  }
  return {
    term: bodyLower,
    context: null,
    termOriginal: bodyOriginal || bodyLower,
    contextOriginal: null,
  };
}

/**
 * The concept-query vocabulary for `role`, in seed order: the `concept_lookup`
 * cue records of data/seed/code-task-cues.lino (issue #918). Mirrors
 * query_cues in rust/src/concepts.rs.
 * @param {string} role
 * @returns {string[]}
 */
function conceptQueryCues(role) {
  return codeTaskCuePhrases("concept_lookup", role);
}

function stripLeadingRequest(input) {
  const lower = input.toLowerCase();
  const prefixes = conceptQueryCues("request_prefix");
  const questionStarts = conceptQueryCues("question_start");
  for (const prefix of prefixes) {
    if (!lower.startsWith(prefix)) continue;
    const rest = input.slice(prefix.length).trimStart();
    const restLower = rest.toLowerCase();
    if (
      questionStarts.some((questionStart) =>
        restLower.startsWith(questionStart),
      )
    ) {
      return rest;
    }
  }
  return input;
}
