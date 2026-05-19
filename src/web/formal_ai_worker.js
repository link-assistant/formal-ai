// Universal solver implementation for the demo worker.
//
// Every reasoning path here mirrors the Rust `FormalAiEngine` in
// `src/solver.rs` so the website, CLI, Telegram bot, library, and HTTP server
// all produce the same answers for the same prompts. The answer the user
// sees is always a projection of an append-only event log — there is no
// hardcoded prompt→answer table.
//
// All multilingual phrases, concept summaries, and the tool registry are
// loaded from `seed/*.lino` files at startup via `seed_loader.js`. Editing a
// `.lino` file is enough to retune the agent — no JavaScript change required.

function currentAssetVersion() {
  try {
    const search = self.location && self.location.search;
    const match = search && /[?&]v=([^&]+)/.exec(search);
    return match ? decodeURIComponent(match[1].replace(/\+/g, " ")) : "";
  } catch (_error) {
    return "";
  }
}

function withAssetVersion(url) {
  const version = currentAssetVersion();
  if (!version) return url;
  return `${url}${url.includes("?") ? "&" : "?"}v=${encodeURIComponent(
    version,
  )}`;
}

try {
  importScripts(withAssetVersion("seed_loader.js"));
} catch (_error) {
  // Seed loader is optional: tests that mock the worker may exclude it.
}

let wasm;
let mode = "wasm worker";

// Hard-coded fallbacks. These are only used if `seed/*.lino` fails to load,
// e.g. when the worker runs from a `file://` URL. The shipped GitHub Pages
// build always fetches the seed successfully.
const FALLBACK_IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

const FALLBACK_GREETING_ANSWER = "Hi, how may I help you?";

const FALLBACK_COURTESY_RESPONSE_ANSWER =
  "Glad to hear it. What would you like to do next?";

const FALLBACK_UNKNOWN_ANSWER =
  "I cannot answer that from local Links Notation rules yet. Please add a fact or add a rule in Links Notation, then run the request again.";

const FALLBACK_CLARIFICATION_ANSWER =
  "I'm sorry for the confusion. I am formal-ai, a deterministic symbolic AI. I can answer greetings, identity questions, concept lookups (what is X?), arithmetic, and Hello World programs. If you'd like to ask about something specific, try one of those or add a fact in Links Notation.";

// Mutable runtime tables — populated from seed at init(). Each entry is
// `{ text, variants }` so the worker can return either the canonical phrase
// (for deterministic tests and tool calls) or a random variant (for greeting
// randomisation introduced in issue #27). Non-greeting intents currently ship
// a single phrase, so `variants` is `[text]` and randomisation is a no-op.
let MULTILINGUAL_ANSWERS = {
  greeting: {
    en: { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] },
  },
  farewell: {
    en: { text: "Goodbye! Feel free to return any time.", variants: ["Goodbye! Feel free to return any time."] },
  },
  courtesy_response: {
    en: {
      text: FALLBACK_COURTESY_RESPONSE_ANSWER,
      variants: [FALLBACK_COURTESY_RESPONSE_ANSWER],
    },
  },
  identity: {
    en: { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] },
  },
  clarification: {
    en: {
      text: FALLBACK_CLARIFICATION_ANSWER,
      variants: [FALLBACK_CLARIFICATION_ANSWER],
    },
  },
  unknown: {
    en: { text: FALLBACK_UNKNOWN_ANSWER, variants: [FALLBACK_UNKNOWN_ANSWER] },
  },
};
let CONCEPTS = [];
let CONCEPT_CONTEXTS = [];
let FACTS = [];
let BRAINSTORM_SEEDS = {
  triggers: [
    "brainstorm",
    "give me five ideas",
    "give me 5 ideas",
    "give me ten ideas",
    "give me 10 ideas",
    "suggest five",
    "suggest 5",
    "suggest ten",
    "suggest 10",
  ],
  categories: [
    {
      slug: "project_ideas",
      intent: "brainstorm_project_ideas",
      detectionKeywords: [],
      items: [
        "A local Links Notation notebook with searchable traces.",
        "A deterministic code-review checklist generator.",
        "A multilingual prompt-variation test corpus.",
        "A CLI that converts issue requirements into traceable tests.",
        "A source-cache inspector for reproducible agent runs.",
        "A changelog-fragment consistency checker.",
        "A prompt-matrix generator for four-language smoke tests.",
        "A Wikidata anchor verifier for local seed records.",
        "A trace viewer that groups events by solver phase.",
        "A small offline issue-to-test planning tool.",
      ],
    },
  ],
};
let PERSONA_SEEDS = {
  triggers: ["pretend you are", "act as", "roleplay", "explain like you are"],
  defaultPersona: "requested persona",
  bodyTemplate:
    "Roleplay frame recorded for <persona>. I will keep the persona explicit and factual: <body>",
  fallbackBody:
    "relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
  personas: [
    { displayName: "Albert Einstein", aliases: ["einstein"], wikidata: "Q937" },
    { displayName: "Ada Lovelace", aliases: ["ada lovelace"], wikidata: "Q7259" },
    { displayName: "teacher", aliases: ["teacher"], wikidata: "" },
  ],
  topics: [
    {
      slug: "algorithm",
      detectionKeywords: ["algorithm", "algorithms"],
      body:
        "an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result.",
    },
    {
      slug: "time_dilation",
      detectionKeywords: ["time dilation"],
      body:
        "time dilation means clocks can measure different elapsed times when observers move differently or sit in different gravitational fields.",
    },
  ],
};
let TOOLS = [];
let SEED_RAW = {};
let AGENT_INFO = {};
let LANGUAGE_RULES = [
  { language: "ru", start: 0x0400, end: 0x04ff },
  { language: "hi", start: 0x0900, end: 0x097f },
  { language: "zh", start: 0x4e00, end: 0x9fff },
];
let PROMPT_PATTERNS = [];
// Intent routing rules loaded from `seed/intent-routing.lino` at init time.
// `intents` mirror `seed::IntentRoute` from the Rust crate, so the browser
// and the Rust solver behave identically when classifying prompts. The
// fallback below mirrors the contents of `data/seed/intent-routing.lino`
// so the worker remains functional even when the `.lino` fetch fails (for
// example when the demo is opened from `file://`).
let INTENT_ROUTING = {
  intents: [
    {
      id: "intent_greeting",
      slug: "greeting",
      responseLink: "response:greeting",
      keywords: ["hi", "hello", "hey", "привет", "здравствуйте", "नमस्ते", "你好", "您好"],
      phrases: [],
      tokens: ["greet"],
      combos: [],
    },
    {
      id: "intent_farewell",
      slug: "farewell",
      responseLink: "response:farewell",
      keywords: ["bye", "goodbye", "пока", "ciao", "再见", "अलविदा"],
      phrases: ["до свидания", "досвидания"],
      tokens: [],
      combos: [],
    },
    {
      id: "intent_courtesy_response",
      slug: "courtesy_response",
      responseLink: "response:courtesy_response",
      keywords: ["thanks", "спасибо", "благодарю", "धन्यवाद", "शुक्रिया", "谢谢"],
      phrases: [
        "thank you",
        "i am fine thank you",
        "i am fine thanks",
        "i m fine thank you",
        "i m fine thanks",
        "i am good thank you",
        "i am good thanks",
        "i m good thank you",
        "i m good thanks",
        "fine thank you",
        "fine thanks",
        "good thank you",
        "good thanks",
        "doing well thank you",
        "doing well thanks",
        "у меня все хорошо спасибо",
        "у меня всё хорошо спасибо",
        "все хорошо спасибо",
        "всё хорошо спасибо",
        "хорошо спасибо",
        "нормально спасибо",
        "मैं ठीक हूँ धन्यवाद",
        "ठीक हूँ धन्यवाद",
        "मैं अच्छा हूँ धन्यवाद",
        "我很好谢谢",
        "我很好 谢谢",
        "好的谢谢",
        "好的 谢谢",
      ],
      tokens: [],
      combos: [],
    },
    {
      id: "intent_identity",
      slug: "identity",
      responseLink: "response:identity",
      keywords: [],
      phrases: [
        "who are you",
        "what are you",
        "who is formal ai",
        "what is formal ai",
        "who is formalai",
        "what is formalai",
        "tell me about yourself",
        "introduce yourself",
        "кто ты",
        "что ты",
        "तुम कौन हो",
        "你是谁",
        "你是誰",
      ],
      tokens: [],
      combos: [
        ["who", "you"],
        ["what", "you"],
        ["tell", "yourself"],
        ["introduce", "yourself"],
        ["кто", "ты"],
        ["что", "ты"],
        ["who", "formal", "ai"],
        ["what", "formal", "ai"],
      ],
    },
  ],
  articlePrefixes: ["the ", "a ", "an "],
  tracePrefixes: ["answer_", "trace_"],
};

function fallbackEntry(intent) {
  if (intent === "greeting") {
    return { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] };
  }
  if (intent === "courtesy_response") {
    return {
      text: FALLBACK_COURTESY_RESPONSE_ANSWER,
      variants: [FALLBACK_COURTESY_RESPONSE_ANSWER],
    };
  }
  if (intent === "identity") {
    return { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] };
  }
  if (intent === "clarification") {
    return {
      text: FALLBACK_CLARIFICATION_ANSWER,
      variants: [FALLBACK_CLARIFICATION_ANSWER],
    };
  }
  return { text: FALLBACK_UNKNOWN_ANSWER, variants: [FALLBACK_UNKNOWN_ANSWER] };
}

function normalizeEntry(value, intent) {
  if (value && typeof value === "object" && typeof value.text === "string") {
    const variants =
      Array.isArray(value.variants) && value.variants.length > 0
        ? value.variants
        : [value.text];
    return { text: value.text, variants: variants };
  }
  if (typeof value === "string") {
    return { text: value, variants: [value] };
  }
  return fallbackEntry(intent);
}

function answerFor(intent, language, options) {
  const opts = options || {};
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  const raw = table[language] || table.en || fallbackEntry(intent);
  const entry = normalizeEntry(raw, intent);
  if (opts.randomize && Array.isArray(entry.variants) && entry.variants.length > 1) {
    const idx = Math.floor(Math.random() * entry.variants.length);
    return entry.variants[idx] || entry.text;
  }
  return entry.text;
}

function numericPreference(value, fallback, min, max) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function definitionFusionByDefault(preferences) {
  const value = preferences && preferences.definitionFusion;
  if (value === true) return true;
  if (value === false) return false;
  const normalized = String(value || "").trim().toLowerCase();
  return ["auto", "on", "true", "1", "merge", "fusion"].includes(normalized);
}

// Language detection and prompt normalization are owned by the Rust core
// (`src/web_engine_core.rs`) and exposed to the worker through the WASM
// exports `engine_detect_language` and `engine_normalize_prompt`. The JS
// branches below are pre-WASM fallbacks used during init() and on browsers
// that could not instantiate the worker — they must stay byte-for-byte
// compatible with the Rust path so the offline trace and the live answer
// agree (PR #134 feedback 4489651616).
function detectLanguage(prompt) {
  const text = String(prompt || "");
  const fromWasm = wasmDetectLanguage(text);
  if (fromWasm !== null) {
    if (fromWasm === "unknown") {
      return AGENT_INFO.default_language || "en";
    }
    return fromWasm;
  }
  for (const ch of text) {
    const code = ch.codePointAt(0);
    for (const rule of LANGUAGE_RULES) {
      if (
        rule.language !== "en" &&
        code >= rule.start &&
        code <= rule.end
      ) {
        return rule.language;
      }
    }
  }
  if (/[a-zA-Z]/.test(text)) return "en";
  return AGENT_INFO.default_language || "en";
}

// CONCEPTS is populated from `seed/concepts.lino` at init() time.

function normalizePrompt(prompt) {
  const text = String(prompt || "");
  const fromWasm = wasmNormalizePrompt(text);
  if (fromWasm !== null) return fromWasm;
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

function normalizeConceptTerm(value) {
  let lower = String(value || "").toLowerCase();
  for (const prefix of ["the ", "a ", "an "]) {
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
    recordMatchesTerm(record, normalized),
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
  if (direct) return direct;
  if (query.context) {
    const reversed = rankConceptForPair(query.context, query.term);
    if (reversed) return reversed;
  }
  return null;
}

function lookupConcept(term) {
  const hit = lookupConceptQuery({ term: term, context: null });
  return hit ? hit.record : null;
}

// Default concept-lookup patterns when seed/prompt-patterns.lino is missing.
// Sorted longest-first so "what is a " beats "what is " when both match.
const DEFAULT_CONCEPT_SUFFIXES = [
  " क्या होता है",
  " क्या है",
  " कौन हैं",
  " कौन है",
  "是甚麼",
  "是什么",
  "是誰",
  "是谁",
];
const DEFAULT_CONCEPT_PREFIXES = [
  "what is a ",
  "what is an ",
  "what is the ",
  "what is ",
  "what's a ",
  "what's an ",
  "what's the ",
  "what's ",
  "what does ",
  "tell me about ",
  "tell me what ",
  "define ",
  "explain ",
  "describe ",
  "who is ",
  "who was ",
  "что такое ",
  "что это ",
  "кто такой ",
  "кто такая ",
  "кто это ",
  "расскажи о ",
  "расскажи про ",
  "назови ",
  "опиши ",
  "объясни ",
  "什么是",
  "甚麼是",
  "请解释",
  "请说说",
  "介绍一下",
];

function conceptPatternsByKind(kind) {
  const matches = PROMPT_PATTERNS.filter(
    (p) => p && p.intent === "concept_lookup" && p.kind === kind && p.text,
  ).map((p) => p.text);
  // Sort longest-first so more specific patterns win.
  matches.sort((a, b) => b.length - a.length);
  if (matches.length > 0) return matches;
  if (kind === "suffix") return DEFAULT_CONCEPT_SUFFIXES;
  if (kind === "prefix") return DEFAULT_CONCEPT_PREFIXES;
  return [];
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

function stripLeadingRequest(input) {
  const lower = input.toLowerCase();
  const prefixes = [
    "please tell me,",
    "please tell me",
    "tell me,",
    "tell me",
  ];
  const questionStarts = ["who ", "what ", "what's ", "who's "];
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

function extractInvertedWhoIs(input, lower) {
  if (!lower.startsWith("who ") || !lower.endsWith(" is")) return null;
  const body = input.slice("who ".length, input.length - " is".length).trim();
  if (!body) return null;
  const normalized = body.toLowerCase();
  if (["is", "was", "are"].includes(normalized)) return null;
  return body;
}

function extractConceptQuery(prompt) {
  let trimmedRaw = String(prompt || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!trimmedRaw) return null;
  trimmedRaw = stripLeadingRequest(trimmedRaw);

  const suffixes = conceptPatternsByKind("suffix");
  for (const suffix of suffixes) {
    if (trimmedRaw.endsWith(suffix)) {
      return finalizeConceptBody(
        trimmedRaw.slice(0, -suffix.length).trim(),
      );
    }
  }

  const lower = trimmedRaw.toLowerCase();
  const invertedWhoBody = extractInvertedWhoIs(trimmedRaw, lower);
  if (invertedWhoBody) return finalizeConceptBody(invertedWhoBody);

  const prefixes = conceptPatternsByKind("prefix");
  let body = null;
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      body = trimmedRaw.slice(prefix.length);
      break;
    }
  }
  if (!body) return null;
  return finalizeConceptBody(body);
}

function extractConceptTerm(prompt) {
  const query = extractConceptQuery(prompt);
  return query ? query.term : null;
}

// Issue #21: render a percent-encoded URL in its readable IRI form for
// display, while leaving the original encoded form available as the href.
// `decodeURI` keeps reserved URI delimiters (`; / ? : @ & = + $ , #`) intact,
// so query strings are preserved; malformed escapes fall back to the original
// string.
function humanizeUrl(url) {
  if (typeof url !== "string" || url.length === 0) return url;
  if (!url.includes("%")) return url;
  try {
    return decodeURI(url);
  } catch (_error) {
    return url;
  }
}

// Render a source URL as a Markdown link [human](encoded) when humanization
// changes anything, or the bare URL otherwise.
function renderSourceLink(source) {
  const human = humanizeUrl(source);
  return human === source ? source : `[${human}](${source})`;
}

function finalizeConceptBody(body) {
  let originalBase = String(body || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!originalBase) return null;
  let original = originalBase;
  let lower = original.toLowerCase();
  // Strip trailing "mean"/"stand for" markers shared across English idioms.
  // The lowercased view drives matching while the original-case view is kept
  // so downstream Wikipedia URL lookups preserve Cyrillic capitalization
  // (see docs/case-studies/issue-27/README.md).
  for (const suffix of [" mean", " stand for"]) {
    if (lower.endsWith(suffix)) {
      original = original.slice(0, -suffix.length).trim();
      lower = lower.slice(0, -suffix.length).trim();
      break;
    }
  }
  if (!lower) return null;
  return splitTermAndContext(original, lower);
}

function tokenizeArithmetic(input) {
  const tokens = [];
  let i = 0;
  while (i < input.length) {
    const ch = input[i];
    if (ch === " " || ch === "\t" || ch === "_" || ch === ",") {
      i += 1;
      continue;
    }
    if (ch === "+") {
      tokens.push({ kind: "+" });
      i += 1;
    } else if (ch === "-" || ch === "−") {
      tokens.push({ kind: "-" });
      i += 1;
    } else if (ch === "*" || ch === "×" || ch === "·") {
      tokens.push({ kind: "*" });
      i += 1;
    } else if (ch === "/" || ch === "÷") {
      tokens.push({ kind: "/" });
      i += 1;
    } else if (ch === "%") {
      tokens.push({ kind: "%" });
      i += 1;
    } else if (ch === "(") {
      tokens.push({ kind: "(" });
      i += 1;
    } else if (ch === ")") {
      tokens.push({ kind: ")" });
      i += 1;
    } else if ((ch >= "0" && ch <= "9") || ch === ".") {
      let j = i;
      while (
        j < input.length &&
        ((input[j] >= "0" && input[j] <= "9") || input[j] === ".")
      ) {
        j += 1;
      }
      const slice = input.slice(i, j);
      const value = Number(slice);
      if (Number.isNaN(value)) {
        throw new Error("unparseable");
      }
      tokens.push({ kind: "num", value });
      i = j;
    } else {
      throw new Error("unparseable");
    }
  }
  return tokens;
}

const ARITHMETIC_WORD_TOKENS = new Map([
  ["zero", "0"],
  ["one", "1"],
  ["two", "2"],
  ["three", "3"],
  ["four", "4"],
  ["five", "5"],
  ["six", "6"],
  ["seven", "7"],
  ["eight", "8"],
  ["nine", "9"],
  ["ten", "10"],
  ["ноль", "0"],
  ["нуль", "0"],
  ["один", "1"],
  ["одна", "1"],
  ["одно", "1"],
  ["два", "2"],
  ["две", "2"],
  ["три", "3"],
  ["четыре", "4"],
  ["пять", "5"],
  ["шесть", "6"],
  ["семь", "7"],
  ["восемь", "8"],
  ["девять", "9"],
  ["десять", "10"],
  ["plus", "+"],
  ["плюс", "+"],
  ["minus", "-"],
  ["минус", "-"],
  ["times", "*"],
  ["умножить", "*"],
  ["умножь", "*"],
  ["modulo", "%"],
  ["mod", "%"],
]);

const ARITHMETIC_WORD_OPERATORS = [
  " plus ",
  " minus ",
  " times ",
  " multiplied by ",
  " divided by ",
  " modulo ",
  " mod ",
  " плюс ",
  " минус ",
  " умножить ",
  " умножь ",
  " умножить на ",
  " разделить на ",
  " делить на ",
];

const ARITHMETIC_NUMBER_WORDS = [
  " zero ",
  " one ",
  " two ",
  " three ",
  " four ",
  " five ",
  " six ",
  " seven ",
  " eight ",
  " nine ",
  " ten ",
  " ноль ",
  " нуль ",
  " один ",
  " одна ",
  " одно ",
  " два ",
  " две ",
  " три ",
  " четыре ",
  " пять ",
  " шесть ",
  " семь ",
  " восемь ",
  " девять ",
  " десять ",
];

function normalizeArithmeticWords(expression) {
  const lower = String(expression).toLowerCase();
  const normalizedPhrases = lower
    .replace(/\s+multiplied by\s+/g, " * ")
    .replace(/\s+divided by\s+/g, " / ")
    .replace(/\s+умножить на\s+/g, " * ")
    .replace(/\s+разделить на\s+/g, " / ")
    .replace(/\s+делить на\s+/g, " / ");
  return normalizedPhrases
    .split(/\s+/)
    .filter(Boolean)
    .map((token) => ARITHMETIC_WORD_TOKENS.get(token) || token)
    .join(" ");
}

function evaluateArithmetic(expression) {
  const normalized = normalizeArithmeticWords(expression);
  const tokens = tokenizeArithmetic(normalized);
  if (tokens.length === 0) {
    throw new Error("empty");
  }
  let cursor = 0;
  const peek = () => tokens[cursor];
  const advance = () => tokens[cursor++];
  function parsePrimary() {
    const tok = advance();
    if (!tok) throw new Error("unparseable");
    if (tok.kind === "num") return tok.value;
    if (tok.kind === "(") {
      const inner = parseAdditive();
      const close = advance();
      if (!close || close.kind !== ")") throw new Error("unbalanced");
      return inner;
    }
    throw new Error("unparseable");
  }
  function parseUnary() {
    const tok = peek();
    if (tok && tok.kind === "-") {
      advance();
      return -parseUnary();
    }
    if (tok && tok.kind === "+") {
      advance();
      return parseUnary();
    }
    return parsePrimary();
  }
  function parseMultiplicative() {
    let left = parseUnary();
    while (true) {
      const tok = peek();
      if (!tok || (tok.kind !== "*" && tok.kind !== "/" && tok.kind !== "%")) {
        break;
      }
      const op = tok.kind;
      advance();
      const right = parseUnary();
      if (op === "*") {
        left = left * right;
      } else if (right === 0) {
        throw new Error("division by zero");
      } else if (op === "/") {
        left = left / right;
      } else {
        left = left % right;
      }
      if (!Number.isFinite(left)) throw new Error("overflow");
    }
    return left;
  }
  function parseAdditive() {
    let left = parseMultiplicative();
    while (true) {
      const tok = peek();
      if (!tok || (tok.kind !== "+" && tok.kind !== "-")) break;
      const isPlus = tok.kind === "+";
      advance();
      const right = parseMultiplicative();
      left = isPlus ? left + right : left - right;
      if (!Number.isFinite(left)) throw new Error("overflow");
    }
    return left;
  }
  const value = parseAdditive();
  if (cursor !== tokens.length) {
    throw new Error("unparseable");
  }
  return value;
}

function formatArithmeticResult(value) {
  if (!Number.isFinite(value)) return "non-finite";
  if (Math.abs(value % 1) === 0 && Math.abs(value) < 1e15) {
    return value.toFixed(0);
  }
  const rendered = value.toFixed(10);
  const trimmed = rendered.replace(/0+$/, "").replace(/\.$/, "");
  return trimmed === "" || trimmed === "-" ? "0" : trimmed;
}

function parseLinearExpression(input) {
  let position = 0;
  let variable = null;

  function peek() {
    return input[position] || "";
  }

  function skipWhitespace() {
    while (/\s/.test(peek())) position += 1;
  }

  function consume(expected) {
    if (peek() === expected) {
      position += 1;
      return true;
    }
    return false;
  }

  function constant(value) {
    return { coefficient: 0, constant: value };
  }

  function variableValue() {
    return { coefficient: 1, constant: 0 };
  }

  function hasVariable(value) {
    return Math.abs(value.coefficient) > Number.EPSILON;
  }

  function add(left, right) {
    return {
      coefficient: left.coefficient + right.coefficient,
      constant: left.constant + right.constant,
    };
  }

  function subtract(left, right) {
    return {
      coefficient: left.coefficient - right.coefficient,
      constant: left.constant - right.constant,
    };
  }

  function multiply(left, right) {
    if (hasVariable(left) && hasVariable(right)) {
      throw new Error("non-linear equation");
    }
    if (hasVariable(left)) {
      return {
        coefficient: left.coefficient * right.constant,
        constant: left.constant * right.constant,
      };
    }
    if (hasVariable(right)) {
      return {
        coefficient: right.coefficient * left.constant,
        constant: right.constant * left.constant,
      };
    }
    return constant(left.constant * right.constant);
  }

  function divide(left, right) {
    if (hasVariable(right)) throw new Error("variable denominator");
    if (Math.abs(right.constant) <= Number.EPSILON) throw new Error("division by zero");
    return {
      coefficient: left.coefficient / right.constant,
      constant: left.constant / right.constant,
    };
  }

  function parseExpression() {
    let value = parseTerm();
    while (true) {
      skipWhitespace();
      if (consume("+")) {
        value = add(value, parseTerm());
      } else if (consume("-") || consume("−")) {
        value = subtract(value, parseTerm());
      } else {
        return value;
      }
    }
  }

  function parseTerm() {
    let value = parseFactor();
    while (true) {
      skipWhitespace();
      if (consume("*") || consume("×") || consume("·")) {
        value = multiply(value, parseFactor());
      } else if (consume("/") || consume("÷")) {
        value = divide(value, parseFactor());
      } else {
        return value;
      }
    }
  }

  function parseFactor() {
    skipWhitespace();
    if (consume("+")) return parseFactor();
    if (consume("-") || consume("−")) {
      const value = parseFactor();
      return { coefficient: -value.coefficient, constant: -value.constant };
    }
    if (consume("(")) {
      const value = parseExpression();
      skipWhitespace();
      if (!consume(")")) throw new Error("unbalanced parentheses");
      return value;
    }
    if (/[0-9.]/.test(peek())) return parseNumber();
    if (/\p{L}/u.test(peek())) return parseVariable();
    throw new Error("expression could not be parsed");
  }

  function parseNumber() {
    const start = position;
    let hasDigit = false;
    let hasDot = false;
    while (/[0-9.]/.test(peek())) {
      if (peek() === ".") {
        if (hasDot) break;
        hasDot = true;
      } else {
        hasDigit = true;
      }
      position += 1;
    }
    if (!hasDigit) throw new Error("expression could not be parsed");
    const value = Number(input.slice(start, position));
    if (!Number.isFinite(value)) throw new Error("expression could not be parsed");
    return constant(value);
  }

  function parseVariable() {
    const start = position;
    while (/[\p{L}_]/u.test(peek())) position += 1;
    const name = input.slice(start, position);
    if (!name) throw new Error("expression could not be parsed");
    if (variable && variable !== name) throw new Error("multiple variables");
    variable = name;
    return variableValue();
  }

  const value = parseExpression();
  skipWhitespace();
  if (position !== input.length) throw new Error("expression could not be parsed");
  return { value, variable };
}

function solveLinearEquation(expression) {
  const parts = String(expression).split("=");
  if (parts.length !== 2) throw new Error("expression could not be parsed");
  const left = parseLinearExpression(parts[0]);
  const right = parseLinearExpression(parts[1]);
  const variable = left.variable || right.variable;
  if (!variable || (left.variable && right.variable && left.variable !== right.variable)) {
    throw new Error("expression could not be parsed");
  }
  const coefficient = left.value.coefficient - right.value.coefficient;
  if (Math.abs(coefficient) <= Number.EPSILON) {
    throw new Error("expression could not be parsed");
  }
  const value = (right.value.constant - left.value.constant) / coefficient;
  return `${variable} = ${formatArithmeticResult(value)}`;
}

function hasArithmeticWordOperator(expression) {
  const lower = ` ${String(expression).toLowerCase()} `;
  return ARITHMETIC_WORD_OPERATORS.some((operator) => lower.includes(operator));
}

function hasSpelledArithmetic(expression) {
  const lower = ` ${String(expression).toLowerCase()} `;
  const hasNumberWord = ARITHMETIC_NUMBER_WORDS.some((number) =>
    lower.includes(number),
  );
  return hasNumberWord && hasArithmeticWordOperator(expression);
}

function extractArithmeticExpression(prompt) {
  const trimmed = String(prompt || "").trim();
  if (!trimmed) return null;
  const prefixes = [
    "please calculate ",
    "please compute ",
    "can you calculate ",
    "can you compute ",
    "could you calculate ",
    "could you compute ",
    "what is ",
    "what's ",
    "what does ",
    "calculate ",
    "compute ",
    "evaluate ",
    "how much is ",
    "solve ",
    "сколько будет ",
    "посчитай ",
    "посчитайте ",
    "вычисли ",
    "вычислите ",
    "рассчитай ",
    "рассчитайте ",
    "请计算",
    "请算一下",
    "计算一下",
    "算一下",
    "计算",
    "कृपया गणना करें ",
    "गणना करें ",
  ];
  let working = trimmed;
  let changed = true;
  while (changed) {
    changed = false;
    const lower = working.toLowerCase();
    for (const prefix of prefixes) {
      if (lower.startsWith(prefix)) {
        working = working.slice(prefix.length).trimStart();
        changed = true;
        break;
      }
    }
  }
  working = working.replace(/[?.!]+$/g, "").trim();
  const suffixes = [
    /\s+equals?$/i,
    /\s+=$/g,
    /\s+please$/i,
    /\s+for me$/i,
    /\s+пожалуйста$/i,
    /\s*是多少$/,
    /\s*等于多少$/,
    /\s*等于几$/,
    /\s*कितना है$/,
    /\s*क्या है$/,
    /\s*की गणना करें$/,
  ];
  changed = true;
  while (changed) {
    changed = false;
    for (const suffix of suffixes) {
      const next = working.replace(suffix, "").trim();
      if (next !== working) {
        working = next;
        changed = true;
        break;
      }
    }
  }
  if (!working) return null;
  const workingLower = working.toLowerCase();
  const hasLetter = /\p{L}/u.test(working);
  const hasSymbolic = /[+*/%^=×·÷−$€¥₹₽]/.test(working) || (!hasLetter && /-/.test(working));
  const hasWordOperator = hasArithmeticWordOperator(working);
  const hasSpelled = hasSpelledArithmetic(working);
  const hasWord =
    hasWordOperator ||
    [
      " sqrt",
      " usd ",
      " eur ",
      " rub ",
      " dollar",
      " euro",
      " kg ",
      " kb ",
      " mb ",
      " ms ",
      " seconds",
      " days",
      " months",
      " gram",
      " tons",
      "руб",
      "доллар",
      "евро",
      "тонн",
      "кг",
      "феврал",
      "январ",
      "месяц",
      "换成",
      "美元",
      "欧元",
      "公斤",
      "二月",
      "一月",
      "个月",
      "天",
      "ग्राम",
      "किलोग्राम",
      "डॉलर",
      "यूरो",
      "फरवरी",
      "जनवरी",
      "महीने",
      "दिन",
    ].some((signal) => ` ${workingLower} `.includes(signal));
  const hasDigit = /[0-9]/.test(working);
  if (!hasDigit && !hasSpelled) return null;
  if (!hasSymbolic && !hasWord && hasLetter) return null;
  const allowed = /^[0-9+\-*/%().=\s_×·÷−,a-zA-Z]+$/;
  if (!allowed.test(working) && !hasWordOperator) return null;
  return working;
}

function extractFencedBlock(text, languages) {
  const fence = "```";
  let cursor = 0;
  while (true) {
    const open = text.indexOf(fence, cursor);
    if (open === -1) return null;
    const infoStart = open + fence.length;
    const newlineRel = text.indexOf("\n", infoStart);
    const infoEnd = newlineRel === -1 ? text.length : newlineRel;
    const info = text.slice(infoStart, infoEnd).trim().toLowerCase();
    const bodyStart = Math.min(infoEnd + 1, text.length);
    const closeRel = text.indexOf(fence, bodyStart);
    if (closeRel === -1) return null;
    const body = text.slice(bodyStart, closeRel).replace(/\n+$/, "");
    if (info === "" || languages.some((lang) => info === lang)) {
      return body;
    }
    cursor = closeRel + fence.length;
  }
}

function extractJavaScriptProgram(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const asksToRun =
    lower.includes("run this javascript") ||
    lower.includes("run this js") ||
    lower.includes("execute this javascript") ||
    lower.includes("execute this js") ||
    lower.includes("run the following javascript") ||
    lower.includes("run the following js") ||
    lower.includes("evaluate this javascript") ||
    lower.includes("evaluate this js");
  if (!asksToRun) return null;
  const fenced = extractFencedBlock(prompt, ["javascript", "js"]);
  if (fenced !== null) return fenced;
  const backticks = prompt.match(/`([^`]+)`/);
  if (backticks) return backticks[1];
  const quoted = prompt.match(/"([^"]+)"/);
  return quoted ? quoted[1] : null;
}

// Look up an intent route by id (e.g. "intent_greeting"). Returns `null`
// when the routing table is empty (no `.lino` seed) so callers can decide
// whether to fall back to legacy hardcoded matching.
function findIntentRoute(id) {
  if (!INTENT_ROUTING || !Array.isArray(INTENT_ROUTING.intents)) return null;
  for (const route of INTENT_ROUTING.intents) {
    if (route && route.id === id) return route;
  }
  return null;
}

function tokensOf(normalized) {
  return normalized ? normalized.split(/\s+/).filter(Boolean) : [];
}

function tokenContains(normalized, expected) {
  return tokensOf(normalized).includes(String(expected || ""));
}

// Match a normalized prompt against an intent route using the same
// semantics as `src/engine.rs::matches_intent_route`:
//   - `keywords` / `phrases`: exact whole-prompt match
//   - `tokens`: any whitespace-separated token equals the value
//   - `combos`: every combo entry must appear as a token
function matchesIntentRoute(normalized, rawPrompt, id) {
  const route = findIntentRoute(id);
  if (!route) return false;
  const raw = String(rawPrompt || "")
    .toLowerCase()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (route.keywords && route.keywords.some((kw) => kw === normalized || kw === raw)) {
    return true;
  }
  if (route.phrases && route.phrases.some((ph) => ph === normalized || ph === raw)) {
    return true;
  }
  if (route.tokens && route.tokens.some((tok) => tokenContains(normalized, tok))) {
    return true;
  }
  if (
    route.combos &&
    route.combos.some(
      (combo) =>
        Array.isArray(combo) &&
        combo.length > 0 &&
        combo.every((tok) => tokenContains(normalized, tok)),
    )
  ) {
    return true;
  }
  return false;
}

function isIdentityPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_identity");
}

function isGreetingPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_greeting");
}

function isFarewellPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_farewell");
}

function isCourtesyResponsePrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_courtesy_response");
}

function isPunctuationOnlyPrompt(prompt) {
  const trimmed = String(prompt || "").trim();
  return /^[.!?…。？！]+$/.test(trimmed);
}

function containsAny(normalized, values) {
  if (!normalized || !Array.isArray(values)) return false;
  return values.some((value) => value && normalized.includes(String(value).toLowerCase()));
}

function tryCapabilities(prompt, normalized) {
  const language = detectLanguage(prompt);
  const isCapabilities =
    language === "ru"
      ? normalized.includes("что ты умеешь") ||
        normalized.includes("чем ты можешь") ||
        normalized.includes("что ты можешь") ||
        normalized.includes("что умеет") ||
        normalized.includes("что можешь") ||
        normalized.includes("твои возможности") ||
        normalized.includes("что за дичь") ||
        normalized.includes("что это такое") ||
        normalized.includes("что происходит") ||
        normalized.includes("что ты делаешь")
      : language === "zh"
        ? normalized.includes("你能做什么") ||
          normalized.includes("你会做什么") ||
          normalized.includes("你有什么功能") ||
          normalized.includes("你能干什么")
        : language === "hi"
          ? normalized.includes("आप क्या कर सकते") ||
            normalized.includes("तुम क्या कर सकते") ||
            normalized.includes("क्या क्या कर सकते")
          : normalized.includes("what can you do") ||
            normalized.includes("what you can do") ||
            normalized.includes("what are your capabilities") ||
            normalized.includes("what are you capable of") ||
            normalized.includes("what do you do") ||
            normalized.includes("show me what you can do") ||
            normalized.includes("what features do you have") ||
            normalized.includes("how can you help") ||
            normalized.includes("what are your features");
  if (!isCapabilities) return null;
  const content =
    language === "ru"
      ? "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\n- **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n- **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n- **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n- **Перевод**: перевожу фразы между языками.\n- **Память**: помню контекст разговора в рамках сессии.\n\nЯ работаю на основе локальных символьных правил, без нейросетевого инференса."
      : language === "zh"
        ? "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\n- **问候**：回应「你好」等问候语。\n- **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n- **概念查找**：解释术语，例如「什么是维基百科？」\n- **算术**：计算表达式，例如「2 + 2 等于多少？」\n- **翻译**：在语言之间翻译短语。\n- **记忆**：在会话中记住上下文。\n\n我基于本地符号规则运行，不进行神经网络推理。"
        : language === "hi"
          ? "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\n- **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n- **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n- **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n- **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n- **अनुवाद**: भाषाओं के बीच अनुवाद।\n- **स्मृति**: सत्र में संदर्भ याद रखना।\n\nमैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।"
          : "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\n- **Greetings**: respond to «Hi», «Hello», and similar.\n- **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n- **Concept lookup**: explain terms — try «What is Wikipedia?»\n- **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n- **Translation**: translate phrases between languages.\n- **Memory**: recall context within the current session.\n\nI run on local symbolic rules, without any neural network inference.";
  return {
    intent: "capabilities",
    content,
    confidence: 1.0,
    evidence: ["handler:capabilities", `language:${language}`],
  };
}

function requestedBrainstormCount(normalized) {
  const tenHints = [
    " 10 ",
    "10.",
    "10 ",
    " 10",
    "ten ",
    "десять",
    "10 идей",
    "10 имён",
    "दस ",
    "十个",
    "10 个",
  ];
  return tenHints.some((hint) => normalized.includes(hint)) ? 10 : 5;
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
  const count = requestedBrainstormCount(` ${normalized} `);
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
  if (!normalized.includes("купи слона")) return null;
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

function tryRecallLastQuestion(history) {
  if (!Array.isArray(history) || history.length === 0) return null;
  for (let i = history.length - 1; i >= 0; i -= 1) {
    const turn = history[i];
    if (turn && turn.role === "user") {
      const content = String(turn.content || "").trim();
      if (content) {
        return {
          intent: "recall_last_question",
          content: `Your previous question was: ${content}`,
          confidence: 0.9,
          evidence: ["recall_last_question", "prior_turn:user"],
        };
      }
    }
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
  const helloWorlds = new Set();
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
      if (intent.startsWith("hello_world_")) {
        helloWorlds.add(intent.slice("hello_world_".length));
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
  if (helloWorlds.size > 0) {
    lines.push(
      `- Hello-world programs generated for: ${Array.from(helloWorlds).join(", ")}`,
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

function tryArithmetic(prompt) {
  const expression = extractArithmeticExpression(prompt);
  if (!expression) return null;
  try {
    const isEquation = expression.includes("=");
    let formatted;
    let backend = "js";
    if (isEquation) {
      formatted = solveLinearEquation(expression);
    } else {
      const wasmResult = wasmEvaluateArithmetic(expression);
      if (wasmResult && wasmResult.ok) {
        formatted = wasmResult.value;
        backend = "wasm";
      } else if (wasmResult && wasmResult.error) {
        throw new Error(wasmResult.error);
      } else {
        formatted = formatArithmeticResult(evaluateArithmetic(expression));
      }
    }
    const content = isEquation
      ? `${expression.trim()} => ${formatted}`
      : `${expression.trim()} = ${formatted}`;
    return {
      intent: "calculation",
      content: content,
      confidence: 1.0,
      evidence: [
        `calculation:${content}`,
        `calculation_backend:${backend}`,
      ],
    };
  } catch (error) {
    const message = String(error && error.message ? error.message : error);
    return {
      intent: "calculation_error",
      content: `I could not evaluate \`${expression.trim()}\`: ${message}.`,
      confidence: 0.4,
      evidence: [`calculation_error:${message}`],
    };
  }
}

function renderConceptInContext(language, context, record) {
  const contextNormalized = normalizeConceptTerm(context);
  const contextRecord = resolveContextRecord(contextNormalized);
  const contextLabel =
    (contextRecord && contextLabelFor(contextRecord, language)) || context;
  const sameAsLabel =
    String(contextLabel).trim().toLowerCase() ===
    String(context).trim().toLowerCase();
  const intentVariant = sameAsLabel
    ? "concept_lookup_in_context_no_alias"
    : "concept_lookup_in_context";
  const variantTable = MULTILINGUAL_ANSWERS[intentVariant] || {};
  const baseTable = MULTILINGUAL_ANSWERS.concept_lookup_in_context || {};
  const templateEntry =
    variantTable[language] ||
    variantTable.en ||
    baseTable[language] ||
    baseTable.en ||
    null;
  const template = templateEntry
    ? (typeof templateEntry === "string" ? templateEntry : templateEntry.text)
    : "In the context of {context} ({context_label}), {term} ({category}) means: {summary}\n\nSource: {source} ({source_kind}).";
  const localized = localizedConceptFor(record, language);
  const term = (localized && localized.term) || record.term;
  const summary = (localized && localized.summary) || record.summary;
  const source = (localized && localized.source) || record.source;
  const sourceKind =
    (localized && localized.sourceKind) || record.sourceKind;
  const sourceMarkup = renderSourceLink(source);
  return template
    .replace(/\{context_label\}/g, contextLabel)
    .replace(/\{context\}/g, context)
    .replace(/\{term\}/g, term)
    .replace(/\{category\}/g, record.category)
    .replace(/\{summary\}/g, summary)
    .replace(/\{source\}/g, sourceMarkup)
    .replace(/\{source_kind\}/g, sourceKind);
}

function renderConceptPlain(language, record) {
  const localized = localizedConceptFor(record, language);
  const term = (localized && localized.term) || record.term;
  const summary = (localized && localized.summary) || record.summary;
  const source = (localized && localized.source) || record.source;
  const sourceKind =
    (localized && localized.sourceKind) || record.sourceKind;
  const sourceMarkup = renderSourceLink(source);
  return `${term} (${record.category}): ${summary}\n\nSource: ${sourceMarkup} (${sourceKind}).`;
}

function tryConceptLookup(prompt) {
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const evidence = [`concept_lookup:request:${query.term}`];
  if (query.context) {
    evidence.push(`concept_lookup:context:${query.context}`);
  }
  const lookup = lookupConceptQuery(query);
  if (!lookup) {
    // Surface the miss in evidence so the demo's trace panel can show why
    // the handler declined the prompt. Returning null lets later handlers
    // (Wikipedia lookup, fallback) still get a chance.
    return null;
  }
  const record = lookup.record;
  const language = detectLanguage(prompt);
  const localized = localizedConceptFor(record, language);
  const effectiveSource = (localized && localized.source) || record.source;
  // Issue #21: emit the percent-decoded IRI form for the trace panel.
  const humanSource = humanizeUrl(effectiveSource);
  evidence.push(`concept_lookup:hit:${record.slug}`);
  evidence.push(`source:${humanSource}`);
  if (record.wikidata) {
    evidence.push(`wikidata:${record.wikidata}`);
  }
  if (lookup.contextMatch && lookup.context) {
    evidence.push(`concept_lookup:context-match:${lookup.context}`);
    const body = renderConceptInContext(language, lookup.context, record);
    return {
      intent: "concept_lookup_in_context",
      content: body,
      confidence: 0.9,
      evidence,
    };
  }
  if (lookup.context) {
    evidence.push(`concept_lookup:context-mismatch:${lookup.context}`);
  }
  const body = renderConceptPlain(language, record);
  return {
    intent: "concept_lookup",
    content: body,
    confidence: 0.9,
    evidence,
  };
}

function extractDefinitionMergeTerm(prompt, allowPlainConcept) {
  const text = String(prompt || "");
  const normalized = normalizePrompt(text);
  const asksMerge =
    normalized.includes("merge") ||
    normalized.includes("merged") ||
    normalized.includes("combine") ||
    normalized.includes("combined") ||
    normalized.includes("fuse") ||
    normalized.includes("fusion");
  const asksDefinition =
    normalized.includes("definition") ||
    normalized.includes("definitions") ||
    normalized.includes("translation") ||
    normalized.includes("translations") ||
    normalized.includes("translated") ||
    normalized.includes("wikipedia");
  if (!asksMerge || !asksDefinition) {
    if (allowPlainConcept) {
      const query = extractConceptQuery(text);
      if (query && !query.context) return query.term;
    }
    return null;
  }

  const lower = text.toLowerCase();
  const markers = [
    "translated definitions for ",
    "translated definitions of ",
    "wikipedia definitions for ",
    "wikipedia definitions of ",
    "definitions for ",
    "definitions of ",
    "definition for ",
    "definition of ",
    "translations for ",
    "translations of ",
    "translation for ",
    "translation of ",
  ];
  for (const marker of markers) {
    const index = lower.indexOf(marker);
    if (index < 0) continue;
    const candidate = trimDefinitionMergeTail(text.slice(index + marker.length));
    if (candidate) return candidate.toLowerCase();
  }
  const query = extractConceptQuery(text);
  return query ? query.term : null;
}

function trimDefinitionMergeTail(value) {
  const text = String(value || "");
  const lower = text.toLowerCase();
  let end = text.length;
  for (const delimiter of [" from ", " using ", " with ", " by ", " into ", " across "]) {
    const index = lower.indexOf(delimiter);
    if (index >= 0) end = Math.min(end, index);
  }
  return text
    .slice(0, end)
    .trim()
    .replace(/^['"`“”«»]+|['"`“”«»]+$/g, "")
    .replace(/[?。.!,;:]+$/g, "")
    .trim();
}

function inferredSourceLanguage(source) {
  const value = String(source || "");
  if (value.includes("://ru.wikipedia.org/")) return "ru";
  if (value.includes("://hi.wikipedia.org/")) return "hi";
  if (value.includes("://zh.wikipedia.org/")) return "zh";
  return "en";
}

function normalizeDefinitionFact(value) {
  return String(value || "")
    .toLocaleLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "");
}

function pushDefinitionFragment(fragments, language, summary, source, sourceKind) {
  const cleanSummary = String(summary || "").trim();
  if (!cleanSummary) return;
  const duplicate = fragments.some(
    (fragment) =>
      fragment.language === language &&
      normalizeDefinitionFact(fragment.summary) === normalizeDefinitionFact(cleanSummary),
  );
  if (duplicate) return;
  fragments.push({
    language: String(language || "en"),
    summary: cleanSummary,
    source: String(source || "").trim(),
    sourceKind: String(sourceKind || "").trim(),
  });
}

function definitionFragments(record) {
  const fragments = [];
  pushDefinitionFragment(
    fragments,
    inferredSourceLanguage(record && record.source),
    record && record.summary,
    record && record.source,
    record && record.sourceKind,
  );
  for (const localized of Array.isArray(record && record.localized) ? record.localized : []) {
    pushDefinitionFragment(
      fragments,
      localized && localized.language,
      localized && localized.summary,
      localized && localized.source,
      localized && localized.sourceKind,
    );
  }
  return fragments;
}

function sourceLanguages(fragments) {
  const languages = [];
  for (const fragment of fragments) {
    if (!languages.includes(fragment.language)) languages.push(fragment.language);
  }
  return languages;
}

function sourceUrls(fragments) {
  const sources = [];
  for (const fragment of fragments) {
    if (!fragment.source || sources.includes(fragment.source)) continue;
    sources.push(fragment.source);
  }
  return sources;
}

function splitDefinitionSentences(summary) {
  const sentences = [];
  let current = "";
  for (const character of String(summary || "")) {
    current += character;
    if ([".", "!", "?", "।", "。"].includes(character)) {
      const sentence = current.trim();
      if (sentence) sentences.push(sentence);
      current = "";
    }
  }
  const tail = current.trim();
  if (tail) sentences.push(tail);
  return sentences;
}

function mergedDefinitionFacts(fragments) {
  const facts = [];
  const seen = new Set();
  for (const fragment of fragments) {
    for (const sentence of splitDefinitionSentences(fragment.summary)) {
      const key = normalizeDefinitionFact(sentence);
      if (!key || seen.has(key)) continue;
      seen.add(key);
      facts.push({ language: fragment.language, text: sentence });
    }
  }
  return facts;
}

function uniqueSourceFragments(fragments) {
  const unique = [];
  const seen = new Set();
  for (const fragment of fragments) {
    if (!fragment.source) continue;
    const key = `${fragment.language}\n${fragment.source}`;
    if (seen.has(key)) continue;
    seen.add(key);
    unique.push(fragment);
  }
  return unique;
}

function renderDefinitionMerge(record, fragments, facts) {
  const english = localizedConceptFor(record, "en");
  const displayTerm = (english && english.term) || record.term;
  const anchor = record.wikidata ? ` [${record.wikidata}]` : "";
  const lines = [
    `Merged definition of ${displayTerm}${anchor}`,
    `Source languages: ${sourceLanguages(fragments).join(", ")}`,
    "",
    "Facts:",
  ];
  for (const fact of facts) {
    lines.push(`- [${fact.language}] ${fact.text}`);
  }
  lines.push("Sources:");
  for (const fragment of uniqueSourceFragments(fragments)) {
    lines.push(
      `- [${fragment.language}] ${renderSourceLink(fragment.source)} (${fragment.sourceKind})`,
    );
  }
  return lines.join("\n");
}

function tryDefinitionMerge(prompt, options) {
  const opts = options || {};
  const term = extractDefinitionMergeTerm(prompt, Boolean(opts.allowPlainConcept));
  if (!term) return null;
  const evidence = [`definition_merge:request:${term}`];
  if (opts.allowPlainConcept) evidence.push("definition_merge:mode:auto");
  const lookup = lookupConceptQuery({ term, context: null });
  if (!lookup) return null;
  const record = lookup.record;
  const fragments = definitionFragments(record);
  if (fragments.length === 0) return null;
  evidence.push(`definition_merge:hit:${record.slug}`);
  if (record.wikidata) evidence.push(`wikidata:${record.wikidata}`);
  for (const language of sourceLanguages(fragments)) {
    evidence.push(`definition_merge:language:${language}`);
  }
  for (const source of sourceUrls(fragments)) {
    evidence.push(`source:${humanizeUrl(source)}`);
  }
  const facts = mergedDefinitionFacts(fragments);
  evidence.push(`definition_merge:facts:${facts.length}`);
  return {
    intent: "definition_merge",
    content: renderDefinitionMerge(record, fragments, facts),
    confidence: 0.9,
    evidence,
  };
}

// Known person name corrections for typo suggestions. Each entry maps a
// canonical name to a list of common misspellings (all lowercase).
const KNOWN_PERSON_VARIANTS = [
  { canonical: "Elon Musk", variants: ["elon musk", "elon mask", "elon muск"] },
  { canonical: "Donald Trump", variants: ["donald trump", "donald tramp", "donald tromp"] },
  { canonical: "Joe Biden", variants: ["joe biden", "joe bidan", "joe bidon"] },
  { canonical: "Barack Obama", variants: ["barack obama", "barak obama", "barrack obama"] },
  { canonical: "Vladimir Putin", variants: ["vladimir putin", "vladimir puting", "vladmir putin"] },
  { canonical: "Albert Einstein", variants: ["albert einstein", "albert einstien", "albert enstien"] },
  { canonical: "Isaac Newton", variants: ["isaac newton", "isaak newton", "issac newton"] },
  { canonical: "Nikola Tesla", variants: ["nikola tesla", "nicolas tesla", "nikolai tesla"] },
];

function editDistance(a, b) {
  const m = a.length, n = b.length;
  const dp = Array.from({ length: m + 1 }, (_, i) =>
    Array.from({ length: n + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0))
  );
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = a[i - 1] === b[j - 1]
        ? dp[i - 1][j - 1]
        : 1 + Math.min(dp[i - 1][j - 1], dp[i - 1][j], dp[i][j - 1]);
    }
  }
  return dp[m][n];
}

function suggestNameCorrection(term) {
  const lower = term.toLowerCase();
  for (const { canonical, variants } of KNOWN_PERSON_VARIANTS) {
    if (variants.includes(lower)) return canonical;
  }
  for (const { canonical, variants } of KNOWN_PERSON_VARIANTS) {
    const canonicalLower = canonical.toLowerCase();
    if (
      variants.some((v) => editDistance(lower, v) === 1) ||
      editDistance(lower, canonicalLower) === 1
    ) {
      return canonical;
    }
  }
  return null;
}

function isWhoIsPrompt(normalized) {
  return (
    normalized.startsWith("who is ") ||
    normalized.startsWith("who was ") ||
    normalized.startsWith("who are ") ||
    normalized.startsWith("кто такой ") ||
    normalized.startsWith("кто такая ") ||
    normalized.startsWith("кто это ") ||
    normalized.startsWith("кто ") ||
    normalized.endsWith(" कौन है") ||
    normalized.endsWith(" कौन हैं") ||
    normalized.endsWith("是谁") ||
    normalized.endsWith("是誰")
  );
}

function tryWhoIsQuestion(prompt) {
  const normalized = prompt.toLowerCase().trim();
  if (!isWhoIsPrompt(normalized)) return null;
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const term = query.term;
  const suggestion = suggestNameCorrection(term);
  const content = suggestion
    ? `I don't have a Links Notation fact for "${term}" yet. Did you mean "${suggestion}"? Add a fact or rule in Links Notation and run the request again.`
    : `I don't have a Links Notation fact for "${term}" yet. Add a fact or rule in Links Notation and run the request again.`;
  return {
    intent: "who_is_question",
    content,
    confidence: 0.5,
    evidence: [`concept_lookup:miss:${term}`, "response:who_is_question"],
  };
}

// Wikipedia REST summary endpoint per language. Browser-friendly: CORS is
// enabled by Wikimedia for these summary endpoints, so the worker can fetch
// without a proxy from GitHub Pages.
const WIKIPEDIA_HOSTS = {
  en: "https://en.wikipedia.org/api/rest_v1/page/summary",
  ru: "https://ru.wikipedia.org/api/rest_v1/page/summary",
  hi: "https://hi.wikipedia.org/api/rest_v1/page/summary",
  zh: "https://zh.wikipedia.org/api/rest_v1/page/summary",
};

// Wikipedia full-text page search endpoint per language (CORS-enabled). Returns
// ranked page results matching a free-text query — more effective than the
// title-only search for context-aware disambiguation because the ranker scores
// body content, not just the title.
const WIKIPEDIA_SEARCH_HOSTS = {
  en: "https://en.wikipedia.org/w/rest.php/v1/search/page",
  ru: "https://ru.wikipedia.org/w/rest.php/v1/search/page",
  hi: "https://hi.wikipedia.org/w/rest.php/v1/search/page",
  zh: "https://zh.wikipedia.org/w/rest.php/v1/search/page",
};

function wikipediaHostsFor(language) {
  // Try the detected language first, then fall back to English so a Russian
  // query for an English-only article still returns a definition.
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  return ordered.map((lang) => ({
    language: lang,
    url: WIKIPEDIA_HOSTS[lang] || WIKIPEDIA_HOSTS.en,
  }));
}

function capitalizeWords(term) {
  return term
    .split(/(\s+)/)
    .map((part) =>
      /\S/.test(part) ? part.charAt(0).toUpperCase() + part.slice(1) : part,
    )
    .join("");
}

function wikipediaTermVariants(term) {
  const seen = new Set();
  const variants = [];
  const push = (value) => {
    if (!value) return;
    const slug = String(value)
      .trim()
      .replace(/\s+/g, "_")
      .replace(/_+/g, "_");
    if (!slug || seen.has(slug)) return;
    seen.add(slug);
    variants.push(slug);
  };
  const trimmed = String(term || "").trim();
  push(trimmed);
  push(capitalizeWords(trimmed));
  push(capitalizeWords(trimmed.toLowerCase()));
  push(trimmed.toLowerCase());
  // Biography titles on Wikipedia (notably ru.wikipedia.org) use the
  // "Surname, Given names" form: querying "Илон Маск" 404s, but "Маск, Илон"
  // resolves. For two-word terms try the swap in both original and
  // capitalized casing so other language hosts can match too.
  const words = trimmed.split(/\s+/).filter(Boolean);
  if (words.length === 2) {
    const swapped = `${words[1]}, ${words[0]}`;
    push(swapped);
    push(capitalizeWords(swapped.toLowerCase()));
  }
  return variants;
}

// Resolve a context-qualified term to a Wikipedia page slug via full-text page
// search. Tries multiple query formulations (uppercase term, mixed case) on the
// detected language host then on English, returning the first match found.
// This helps disambiguate short acronyms like "KISS" or "DRY" when the user
// provides a programming/domain context.
async function searchWikipediaSlug(term, context, language) {
  if (typeof fetch !== "function") return null;
  const apiHeaders = {
    accept: "application/json",
    "api-user-agent":
      "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
  };
  const upper = term.toUpperCase();
  // Build candidate queries in preference order: uppercase term with context is
  // most discriminating; plain term with context is the fallback.
  const queries = [];
  if (upper !== term) queries.push(`${upper} ${context}`.trim());
  queries.push(`${term} ${context}`.trim());
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const base = WIKIPEDIA_SEARCH_HOSTS[lang] || WIKIPEDIA_SEARCH_HOSTS.en;
    for (const query of queries) {
      const url = `${base}?q=${encodeURIComponent(query)}&limit=5`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (!response || !response.ok) continue;
        const data = await response.json();
        if (!data || !Array.isArray(data.pages) || data.pages.length === 0)
          continue;
        // Return the key of the top result; callers will fetch the full summary.
        const page = data.pages[0];
        return {
          slug: page.key,
          title: page.title || page.key,
          language: lang,
          query,
        };
      } catch (_error) {
        // Ignore and try next query / language host.
      }
    }
  }
  return null;
}

async function fetchWikipediaSummary(term, language, context) {
  if (typeof fetch !== "function") return null;
  const apiHeaders = {
    accept: "application/json",
    "api-user-agent":
      "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
  };

  // When context is provided, first try a title-search to find the most
  // relevant article slug (e.g. "Kiss" + "рамках програмирования" → "KISS
  // principle"). This prevents ambiguous short terms from matching the wrong
  // article (e.g. the rock band instead of the software design principle).
  if (context) {
    const found = await searchWikipediaSlug(term, context, language);
    if (found) {
      const summaryBase =
        WIKIPEDIA_HOSTS[found.language] || WIKIPEDIA_HOSTS.en;
      const url = `${summaryBase}/${encodeURIComponent(found.slug)}`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (response && response.ok) {
          const data = await response.json();
          if (
            data &&
            typeof data === "object" &&
            data.type !== "disambiguation"
          ) {
            const extract = String(data.extract || "").trim();
            if (extract) {
              const title = String(data.title || term);
              const pageUrl =
                (data.content_urls &&
                  data.content_urls.desktop &&
                  data.content_urls.desktop.page) ||
                url;
              return {
                title,
                extract,
                url: pageUrl,
                language: found.language,
                matchKind: "context_search",
                matchedSlug: found.slug,
                matchedTitle: found.title || title,
                searchQuery: found.query || "",
              };
            }
          }
        }
      } catch (_error) {
        // Fall through to bare-term lookup below.
      }
    }
  }

  // Bare-term fallback: try direct slug variants without context.
  const hosts = wikipediaHostsFor(language);
  const variants = wikipediaTermVariants(term);
  for (const host of hosts) {
    for (const slug of variants) {
      const url = `${host.url}/${encodeURIComponent(slug)}`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (!response || !response.ok) continue;
        const data = await response.json();
        if (!data || typeof data !== "object") continue;
        if (data.type === "disambiguation") continue;
        const extract = String(data.extract || "").trim();
        if (!extract) continue;
        const title = String(data.title || term);
        const pageUrl =
          (data.content_urls &&
            data.content_urls.desktop &&
            data.content_urls.desktop.page) ||
          url;
        return {
          title,
          extract,
          url: pageUrl,
          language: host.language,
          matchKind: "direct",
          matchedSlug: slug,
        };
      } catch (_error) {
        // Swallow network/parse errors and continue to the next variant.
      }
    }
  }
  // All direct slug variants were disambiguation pages or not found. Use the
  // full-text search endpoint to find the top-ranked article for the term
  // (e.g. "tesla" → "Tesla, Inc." instead of the disambiguation page).
  const found = await searchWikipediaSlug(term, "", language);
  if (found) {
    const summaryBase = WIKIPEDIA_HOSTS[found.language] || WIKIPEDIA_HOSTS.en;
    const url = `${summaryBase}/${encodeURIComponent(found.slug)}`;
    try {
      const response = await fetch(url, { headers: apiHeaders });
      if (response && response.ok) {
        const data = await response.json();
        if (
          data &&
          typeof data === "object" &&
          data.type !== "disambiguation"
        ) {
          const extract = String(data.extract || "").trim();
          if (extract) {
            const title = String(data.title || term);
            const pageUrl =
              (data.content_urls &&
                data.content_urls.desktop &&
                data.content_urls.desktop.page) ||
              url;
            return {
              title,
              extract,
              url: pageUrl,
              language: found.language,
              matchKind: "search",
              matchedSlug: found.slug,
              matchedTitle: found.title || title,
              searchQuery: found.query || "",
            };
          }
        }
      }
    } catch (_error) {
      // Search-based fallback failed; return null below.
    }
  }
  return null;
}

function isClosestWikipediaMatch(summary) {
  return summary && summary.matchKind === "search";
}

function closestMatchNote(summary, language) {
  const title = summary && summary.title ? summary.title : "the top result";
  if (language === "ru") {
    return `Ближайшее совпадение по поиску Wikipedia: «${title}». Если это не то, уточните запрос.`;
  }
  if (language === "zh") {
    return `Wikipedia 搜索的最接近匹配是“${title}”。如果这不是你的意思，请进一步说明。`;
  }
  if (language === "hi") {
    return `Wikipedia खोज में सबसे नज़दीकी मिलान "${title}" है। अगर आपका मतलब यह नहीं था, तो कृपया स्पष्ट करें।`;
  }
  return `Closest match from Wikipedia search: "${title}". If that is not what you meant, clarify the prompt.`;
}

function wikipediaClarificationMessage(summary, language) {
  const title = summary && summary.title ? summary.title : "the top result";
  if (language === "ru") {
    return `Похоже, вы имели в виду «${title}». Уточните, отвечать по этой статье Wikipedia?`;
  }
  if (language === "zh") {
    return `你是指“${title}”吗？请确认后我再根据这篇 Wikipedia 文章回答。`;
  }
  if (language === "hi") {
    return `क्या आपका मतलब "${title}" था? Wikipedia के इस लेख से उत्तर देने से पहले कृपया स्पष्ट करें।`;
  }
  return `Did you mean "${title}"? Please clarify before I answer from that Wikipedia article.`;
}

// ---------------------------------------------------------------------------
// Wikidata-backed fact reasoning pipeline (issue #127).
//
// Rather than matching against hardcoded summaries in `data/seed/facts.lino`,
// fact questions ("what is the capital of X?", "столица X", "X की राजधानी",
// "X的首都") are parsed into a structured query
// `{ relation, subjectTerm, language, forceFresh }`. The query is then
// resolved against:
//
//   1. An in-memory cache (1-week TTL) keyed by `relation:subject:language`.
//      The cache is pre-warmed from the seed `FACTS` entries so the test
//      matrix stays deterministic offline.
//   2. Wikidata `wbsearchentities` to resolve the subject term to a Q-ID.
//   3. Wikidata `wbgetentities` to fetch the property claim (P36 = capital,
//      P1082 = population, P38 = currency, P37 = official language, P30 =
//      continent, P2046 = area, P35 = head of state, P6 = head of government).
//   4. Wikidata `wbgetentities` again to resolve the target Q-ID to a label
//      in the user's prevailing language (and to a Wikipedia sitelink).
//
// Every step is recorded as a `fact_query:*` event so the reasoning trace
// shows the structured query, the cache decision, the Wikidata round-trips,
// and the final resolved answer. A user can force a fresh resolution by
// adding markers like "fresh", "no cache", "не из кэша", "без кеша",
// "ताज़ा", or "刷新" to the prompt.
// ---------------------------------------------------------------------------

const WIKIDATA_API = "https://www.wikidata.org/w/api.php";

const FACT_RELATIONS = [
  {
    relation: "capital",
    property: "P36",
    valueType: "entity",
  },
  {
    relation: "population",
    property: "P1082",
    valueType: "quantity",
  },
  {
    relation: "currency",
    property: "P38",
    valueType: "entity",
  },
  {
    relation: "official_language",
    property: "P37",
    valueType: "entity",
  },
  {
    relation: "continent",
    property: "P30",
    valueType: "entity",
  },
  {
    relation: "area",
    property: "P2046",
    valueType: "quantity",
  },
  {
    relation: "head_of_state",
    property: "P35",
    valueType: "entity",
  },
  {
    relation: "head_of_government",
    property: "P6",
    valueType: "entity",
  },
];

function relationConfig(relation) {
  return FACT_RELATIONS.find((entry) => entry.relation === relation) || null;
}

// Markers that flag the user wants a fresh (uncached) result. Detected in all
// four supported languages plus a couple of common English phrasings.
const FORCE_FRESH_MARKERS = [
  "fresh",
  "no cache",
  "no-cache",
  "without cache",
  "skip cache",
  "ignore cache",
  "refresh",
  "не из кэша",
  "не из кеша",
  "без кэша",
  "без кеша",
  "обнови",
  "свежий ответ",
  "свежие данные",
  "ताज़ा",
  "ताज़े",
  "बिना कैश",
  "नया जवाब",
  "刷新",
  "新鲜",
  "不要缓存",
  "不用缓存",
];

function shouldForceFresh(normalized, prompt) {
  const lowerPrompt = String(prompt || "").toLowerCase();
  return FORCE_FRESH_MARKERS.some(
    (marker) => normalized.includes(marker) || lowerPrompt.includes(marker),
  );
}

// Multilingual relation patterns. Each entry has a list of triggers that, when
// present in the normalized prompt, identify the relation. Subject extraction
// uses the `extract` regexes which capture the subject term verbatim from the
// original (un-normalized) prompt — that preserves Cyrillic/Devanagari/CJK
// scripts that the normalizer otherwise strips.
const FACT_QUESTION_PATTERNS = [
  {
    relation: "capital",
    // English
    extract: [
      /\bcapital\s+(?:city\s+)?of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+capital\b/i,
      /\bwhich\s+city\s+is\s+(?:the\s+)?capital\s+of\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwhich\s+city\s+is\s+([^?.!,;:]+?)['’]s\s+capital\b/i,
      // Russian: "столица России", "какова столица России",
      // "столицей какой страны является Москва" — only the first form is
      // resolved; the inverted form falls through to other handlers.
      /столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какова\s+столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какая\s+столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      // Hindi: "X की राजधानी क्या है"
      /([^?.!,;:]+?)\s+की\s+राजधानी(?:\s+क्या\s+है)?(?:[?.!,;:]|$)/i,
      // Chinese: "X的首都" / "X的首都是什么"
      /([^?。.!!,,;:、]+?)的首都(?:是什么|是哪里|是哪个城市)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "population",
    extract: [
      /\bpopulation\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bhow\s+many\s+people\s+(?:live|are\s+there)\s+in\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+population\b/i,
      /население\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какое\s+население\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+(?:जनसंख्या|आबादी)(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的人口(?:是多少|有多少)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "currency",
    extract: [
      /\bcurrency\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+currency\b/i,
      /валюта\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какая\s+валюта\s+в\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+मुद्रा(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的(?:货币|貨幣)(?:是什么|是哪种)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "official_language",
    extract: [
      /\bofficial\s+language\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwhat\s+language\s+(?:do\s+they\s+speak|is\s+spoken)\s+in\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /государственный\s+язык\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /официальный\s+язык\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+(?:राजभाषा|आधिकारिक\s+भाषा)(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的(?:官方语言|官方語言)(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "continent",
    extract: [
      /\bcontinent\s+(?:is\s+)?([^?.!,;:]+?)\s+(?:on|in)\b/i,
      /\bwhich\s+continent\s+is\s+([^?.!,;:]+?)\s+(?:on|in)\b/i,
      /на\s+каком\s+континенте\s+(?:находится|расположена|расположен)\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+किस\s+महाद्वीप\s+में\s+है(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)在哪个(?:大洲|洲)(?:[?。.!!,,;:、]|$)/i,
    ],
  },
];

// Words/phrases that should be stripped from a captured subject before we
// hand it off to Wikidata. These are not part of the entity name — they leak
// from question prefixes the regex didn't consume (e.g. "the country called
// France" → "France"). Order matters: longer prefixes first.
const SUBJECT_TRIM_PREFIXES = [
  "the country called ",
  "the country ",
  "country ",
  "the city of ",
  "the city ",
  "city of ",
  "country called ",
  "republic of ",
  "kingdom of ",
  "is ",
  "in ",
  "of the ",
  "of ",
  "страна ",
  "страны ",
  "стране ",
  "страну ",
];

function trimSubjectTerm(raw) {
  let value = String(raw || "")
    .replace(/[«»"'`“”„‟‹›]+/g, "")
    .replace(/\s+/g, " ")
    .trim();
  let changed = true;
  while (changed) {
    changed = false;
    const lower = value.toLowerCase();
    for (const prefix of SUBJECT_TRIM_PREFIXES) {
      if (lower.startsWith(prefix)) {
        value = value.slice(prefix.length).trim();
        changed = true;
        break;
      }
    }
  }
  return value;
}

function parseFactQuestion(prompt, normalized) {
  const text = String(prompt || "");
  if (!text.trim()) return null;
  for (const pattern of FACT_QUESTION_PATTERNS) {
    for (const regex of pattern.extract) {
      const match = regex.exec(text);
      if (!match) continue;
      const subjectTerm = trimSubjectTerm(match[1]);
      if (!subjectTerm) continue;
      // Reject single-letter or pure-punctuation captures so we don't fire
      // on noise like "x." or "?".
      if (subjectTerm.length < 2 && !/[Ѐ-鿿]/.test(subjectTerm)) {
        continue;
      }
      return {
        relation: pattern.relation,
        subjectTerm,
        language: detectLanguage(prompt),
        forceFresh: shouldForceFresh(normalized, prompt),
      };
    }
  }
  return null;
}

// In-memory cache. Keyed by `relation:subject_normalized:language`. The TTL
// matches the user-requested 1 week. Pre-warmed from FACTS at init() so the
// offline test matrix sees the same starting cache the Rust solver does.
const FACT_QUERY_CACHE = new Map();
const FACT_QUERY_TTL_MS = 7 * 24 * 60 * 60 * 1000;

function factCacheKey(relation, subjectTerm, language) {
  return [
    String(relation || "").toLowerCase(),
    String(subjectTerm || "")
      .toLowerCase()
      .replace(/\s+/g, " ")
      .trim(),
    String(language || "en").toLowerCase(),
  ].join(":");
}

function factCacheGet(relation, subjectTerm, language) {
  const key = factCacheKey(relation, subjectTerm, language);
  const entry = FACT_QUERY_CACHE.get(key);
  if (!entry) return null;
  if (
    entry.expiresAt &&
    typeof entry.expiresAt === "number" &&
    entry.expiresAt < Date.now()
  ) {
    FACT_QUERY_CACHE.delete(key);
    return null;
  }
  return entry;
}

function factCachePut(relation, subjectTerm, language, value) {
  const key = factCacheKey(relation, subjectTerm, language);
  const ttl = typeof value.ttlMs === "number" ? value.ttlMs : FACT_QUERY_TTL_MS;
  const entry = Object.assign({}, value, {
    expiresAt: Date.now() + ttl,
  });
  FACT_QUERY_CACHE.set(key, entry);
  return entry;
}

// Pre-warm the runtime cache from the seed `facts.lino`. Each seed record can
// optionally declare `relation`, `subjectQid`, `valueQid`, plus per-language
// `subjectLabel`/`valueLabel`/`valueText` overrides — those are the structured
// cache anchors. The legacy fields (`summary`, `subjectAliases`,
// `questionKeywords`) remain in place for the `tryFactLookup` substring path.
function warmFactCacheFromSeed() {
  if (!Array.isArray(FACTS)) return;
  const languages = ["en", "ru", "hi", "zh"];
  for (const record of FACTS) {
    if (!record || !record.relation || !record.subjectAliases) continue;
    const localizedMap = new Map();
    if (Array.isArray(record.localized)) {
      for (const loc of record.localized) {
        if (loc && loc.language) localizedMap.set(loc.language, loc);
      }
    }
    for (const lang of languages) {
      const loc = localizedMap.get(lang) || localizedMap.get("en") || {};
      const summary =
        (loc && loc.summary) || record.summary || "";
      const source = (loc && loc.source) || record.source || "";
      const sourceKind =
        (loc && loc.sourceKind) || record.sourceKind || "wikipedia";
      const valueLabel = (loc && loc.valueLabel) || record.valueLabel || "";
      const subjectLabel =
        (loc && loc.subjectLabel) || record.subjectLabel || "";
      // The aliases for the subject language drive cache key lookup. For each
      // alias (already lowercased by seed_loader.js), pre-seed a cache entry.
      const aliases = Array.isArray(record.subjectAliases)
        ? record.subjectAliases
        : [];
      for (const alias of aliases) {
        if (!alias) continue;
        factCachePut(record.relation, alias, lang, {
          relation: record.relation,
          subjectTerm: alias,
          subjectLabel: subjectLabel || alias,
          subjectQid: record.subjectQid || "",
          valueLabel,
          valueQid: record.valueQid || "",
          summary,
          source,
          sourceKind,
          language: lang,
          fromSeed: true,
          ttlMs: FACT_QUERY_TTL_MS,
        });
      }
    }
  }
}

async function wikidataSearchEntity(term, language) {
  if (typeof fetch !== "function") return null;
  // Wikidata supports per-language search; English fallback ensures broad
  // recall even for non-Latin scripts.
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const url = `${WIKIDATA_API}?action=wbsearchentities&format=json&origin=*&type=item&limit=5&language=${encodeURIComponent(
      lang,
    )}&search=${encodeURIComponent(term)}`;
    try {
      const response = await fetch(url, {
        headers: {
          accept: "application/json",
          "api-user-agent":
            "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
        },
      });
      if (!response || !response.ok) continue;
      const data = await response.json();
      if (data && Array.isArray(data.search) && data.search.length > 0) {
        const hit = data.search[0];
        return {
          qid: hit.id,
          label: hit.label || term,
          description: hit.description || "",
          language: lang,
        };
      }
    } catch (_error) {
      // Try the next language.
    }
  }
  return null;
}

async function wikidataFetchEntityClaim(qid, property, language) {
  if (typeof fetch !== "function") return null;
  const url = `${WIKIDATA_API}?action=wbgetentities&format=json&origin=*&ids=${encodeURIComponent(
    qid,
  )}&props=claims%7Clabels%7Csitelinks&languages=${encodeURIComponent(
    language,
  )}%7Cen`;
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
    if (!data || !data.entities) return null;
    const entity = data.entities[qid];
    if (!entity) return null;
    const claims = (entity.claims || {})[property] || [];
    const subjectLabel =
      (entity.labels && (entity.labels[language] || entity.labels.en) || {})
        .value || "";
    const sitelinks = entity.sitelinks || {};
    return { claims, subjectLabel, sitelinks };
  } catch (_error) {
    return null;
  }
}

async function wikidataResolveLabel(qid, language) {
  if (typeof fetch !== "function") return null;
  const url = `${WIKIDATA_API}?action=wbgetentities&format=json&origin=*&ids=${encodeURIComponent(
    qid,
  )}&props=labels%7Csitelinks&languages=${encodeURIComponent(language)}%7Cen`;
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
    if (!data || !data.entities) return null;
    const entity = data.entities[qid];
    if (!entity) return null;
    const label =
      (entity.labels && (entity.labels[language] || entity.labels.en) || {})
        .value || "";
    const sitelinks = entity.sitelinks || {};
    return { label, sitelinks };
  } catch (_error) {
    return null;
  }
}

function wikipediaSitelinkUrl(sitelinks, language) {
  if (!sitelinks || typeof sitelinks !== "object") return "";
  const key = `${language}wiki`;
  const fallback = "enwiki";
  const entry = sitelinks[key] || sitelinks[fallback];
  if (!entry) return "";
  if (entry.url) return entry.url;
  if (entry.title) {
    const lang = sitelinks[key] ? language : "en";
    return `https://${lang}.wikipedia.org/wiki/${encodeURIComponent(
      String(entry.title).replace(/\s+/g, "_"),
    ).replace(/%2F/gi, "/")}`;
  }
  return "";
}

// Localized templates for rendering the final answer. The seed value is
// inserted via `{value}`; the subject is inserted via `{subject}`.
const FACT_RESPONSE_TEMPLATES = {
  capital: {
    en: "The capital of {subject} is {value}.",
    ru: "Столица {subject} — {value}.",
    hi: "{subject} की राजधानी {value} है।",
    zh: "{subject}的首都是{value}。",
  },
  population: {
    en: "The population of {subject} is approximately {value}.",
    ru: "Население {subject} составляет примерно {value}.",
    hi: "{subject} की जनसंख्या लगभग {value} है।",
    zh: "{subject}的人口约为 {value}。",
  },
  currency: {
    en: "The currency of {subject} is the {value}.",
    ru: "Валюта {subject} — {value}.",
    hi: "{subject} की मुद्रा {value} है।",
    zh: "{subject}的货币是{value}。",
  },
  official_language: {
    en: "The official language of {subject} is {value}.",
    ru: "Государственный язык {subject} — {value}.",
    hi: "{subject} की राजभाषा {value} है।",
    zh: "{subject}的官方语言是{value}。",
  },
  continent: {
    en: "{subject} is located on the continent of {value}.",
    ru: "{subject} расположена на континенте {value}.",
    hi: "{subject} {value} महाद्वीप पर स्थित है।",
    zh: "{subject}位于{value}。",
  },
  area: {
    en: "The area of {subject} is approximately {value}.",
    ru: "Площадь {subject} составляет примерно {value}.",
    hi: "{subject} का क्षेत्रफल लगभग {value} है।",
    zh: "{subject}的面积约为 {value}。",
  },
  head_of_state: {
    en: "The head of state of {subject} is {value}.",
    ru: "Глава государства {subject} — {value}.",
    hi: "{subject} के राष्ट्राध्यक्ष {value} हैं।",
    zh: "{subject}的国家元首是{value}。",
  },
  head_of_government: {
    en: "The head of government of {subject} is {value}.",
    ru: "Глава правительства {subject} — {value}.",
    hi: "{subject} के सरकार के प्रमुख {value} हैं।",
    zh: "{subject}的政府首脑是{value}。",
  },
};

function renderFactSummary(relation, subjectLabel, valueLabel, language) {
  const templates =
    FACT_RESPONSE_TEMPLATES[relation] || FACT_RESPONSE_TEMPLATES.capital;
  const template = templates[language] || templates.en;
  return template
    .replace("{subject}", subjectLabel || "")
    .replace("{value}", valueLabel || "");
}

function factQueryEvidence(record, language) {
  const evidence = [
    `fact_query:relation:${record.relation}`,
    `fact_query:subject:${record.subjectLabel || record.subjectTerm}`,
    `language:${language}`,
  ];
  if (record.subjectQid) evidence.push(`wikidata:${record.subjectQid}`);
  if (record.valueQid) evidence.push(`wikidata:${record.valueQid}`);
  if (record.source) evidence.push(`source:${humanizeUrl(record.source)}`);
  if (record.fromSeed) evidence.push("fact_query:cache:seed");
  else if (record.fromCache) evidence.push("fact_query:cache:hit");
  else evidence.push("fact_query:cache:miss");
  return evidence;
}

async function resolveFactQueryViaWikidata(query, log) {
  // Stage 1: subject resolution via wbsearchentities.
  if (log) log.push(`fact_query:wbsearchentities:request:${query.subjectTerm}`);
  const subject = await wikidataSearchEntity(query.subjectTerm, query.language);
  if (!subject) {
    if (log) log.push("fact_query:wbsearchentities:miss");
    return null;
  }
  if (log) log.push(`fact_query:wbsearchentities:resolved:${subject.qid}`);

  const config = relationConfig(query.relation);
  if (!config) return null;

  // Stage 2: claim fetch via wbgetentities.
  if (log) log.push(`fact_query:wbgetentities:request:${config.property}`);
  const claimData = await wikidataFetchEntityClaim(
    subject.qid,
    config.property,
    query.language,
  );
  if (!claimData || !claimData.claims || claimData.claims.length === 0) {
    if (log) log.push("fact_query:wbgetentities:no_claim");
    return null;
  }
  const claim = claimData.claims[0];
  const mainsnak = claim && claim.mainsnak;
  if (!mainsnak || !mainsnak.datavalue) {
    if (log) log.push("fact_query:wbgetentities:no_datavalue");
    return null;
  }

  // Stage 3: value resolution.
  let valueLabel = "";
  let valueQid = "";
  if (config.valueType === "entity") {
    const value = mainsnak.datavalue.value || {};
    valueQid = value.id || "";
    if (!valueQid) {
      if (log) log.push("fact_query:wbgetentities:value_not_entity");
      return null;
    }
    if (log) log.push(`fact_query:label_resolve:request:${valueQid}`);
    const labelResult = await wikidataResolveLabel(valueQid, query.language);
    if (!labelResult || !labelResult.label) {
      if (log) log.push("fact_query:label_resolve:miss");
      return null;
    }
    valueLabel = labelResult.label;
    if (log) log.push(`fact_query:label_resolve:${valueLabel}`);
    // Capture the Wikipedia sitelink for the value entity as the canonical
    // evidence source — that's the human-readable artifact users can verify.
    const url =
      wikipediaSitelinkUrl(labelResult.sitelinks, query.language) ||
      wikipediaSitelinkUrl(claimData.sitelinks, query.language);
    return {
      relation: query.relation,
      subjectTerm: query.subjectTerm,
      subjectLabel: claimData.subjectLabel || subject.label,
      subjectQid: subject.qid,
      valueLabel,
      valueQid,
      summary: renderFactSummary(
        query.relation,
        claimData.subjectLabel || subject.label,
        valueLabel,
        query.language,
      ),
      source: url,
      sourceKind: "wikidata",
      language: query.language,
      fromCache: false,
      fromSeed: false,
    };
  }

  // Quantity values (population, area) are not Q-IDs.
  const value = mainsnak.datavalue.value || {};
  const rawAmount = String(value.amount || "").replace(/^\+/, "");
  if (!rawAmount) {
    if (log) log.push("fact_query:wbgetentities:value_empty");
    return null;
  }
  valueLabel = rawAmount;
  if (log) log.push(`fact_query:quantity:${valueLabel}`);
  const url = wikipediaSitelinkUrl(claimData.sitelinks, query.language);
  return {
    relation: query.relation,
    subjectTerm: query.subjectTerm,
    subjectLabel: claimData.subjectLabel || subject.label,
    subjectQid: subject.qid,
    valueLabel,
    valueQid: "",
    summary: renderFactSummary(
      query.relation,
      claimData.subjectLabel || subject.label,
      valueLabel,
      query.language,
    ),
    source: url,
    sourceKind: "wikidata",
    language: query.language,
    fromCache: false,
    fromSeed: false,
  };
}

async function tryFactQuery(prompt, normalized, preferences) {
  const query = parseFactQuestion(prompt, normalized);
  if (!query) return null;

  // Trace events: every step of the reasoning pipeline is recorded so the
  // browser memory log shows the structured query, the cache decision, and
  // any Wikidata calls.
  const trace = [];
  trace.push(`fact_query:request:${prompt}`);
  trace.push(`fact_query:relation:${query.relation}`);
  trace.push(`fact_query:subject:${query.subjectTerm}`);
  trace.push(`fact_query:language:${query.language}`);
  if (query.forceFresh) trace.push("fact_query:force_fresh");

  // Stage 1: cache check (skipped when the user asked for fresh data).
  if (!query.forceFresh) {
    trace.push("fact_query:cache:check");
    const cached = factCacheGet(
      query.relation,
      query.subjectTerm,
      query.language,
    );
    if (cached) {
      trace.push(`fact_query:cache:hit:${cached.fromSeed ? "seed" : "runtime"}`);
      const evidence = factQueryEvidence(
        Object.assign({}, cached, { fromCache: true }),
        query.language,
      );
      return {
        intent: "fact_query",
        content: cached.summary,
        confidence: 0.92,
        evidence,
        trace,
      };
    }
    trace.push("fact_query:cache:miss");
  } else {
    trace.push("fact_query:cache:bypass");
  }

  // Stage 2: Wikidata resolution.
  const resolved = await resolveFactQueryViaWikidata(query, trace);
  if (!resolved) {
    trace.push("fact_query:wikidata:no_match");
    return null;
  }

  // Stage 3: cache the resolution.
  factCachePut(query.relation, query.subjectTerm, query.language, resolved);
  trace.push(`fact_query:cache:store:${factCacheKey(
    query.relation,
    query.subjectTerm,
    query.language,
  )}`);

  trace.push(`fact_query:response:${resolved.summary}`);
  return {
    intent: "fact_query",
    content: resolved.summary,
    confidence: 0.92,
    evidence: factQueryEvidence(resolved, query.language),
    trace,
  };
}

async function tryWikipediaLookup(prompt, language, preferences) {
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  // Avoid hitting the network for terms that already resolved in CONCEPTS;
  // that path is handled by `tryConceptLookup`. We try the full
  // `(term, context)` query first so that "what is iir in ml" doesn't waste
  // a network call when a context-aware record exists.
  if (lookupConceptQuery(query)) return null;
  // Pass the original-case term to Wikipedia: non-Latin scripts (e.g. Cyrillic
  // for "Илон Маск") require correct capitalization in the REST URL because
  // ru.wikipedia.org does not redirect the all-lowercase slug.
  const wikiTerm = query.termOriginal || query.term;
  const wikiContext = query.contextOriginal || query.context;
  const summary = await fetchWikipediaSummary(wikiTerm, language, wikiContext);
  if (!summary) return null;
  const isClosestMatch = isClosestWikipediaMatch(summary);
  const guessProbability = numericPreference(
    preferences && preferences.guessProbability,
    0.8,
    0,
    1,
  );
  const humanUrl = humanizeUrl(summary.url);
  const evidence = [
    `wikipedia_lookup:${summary.title}`,
    `source:${humanUrl}`,
    `language:${summary.language}`,
  ];
  if (wikiContext) evidence.push(`wikipedia_lookup:context:${wikiContext}`);
  if (isClosestMatch) {
    evidence.push(`wikipedia_lookup:closest_match:${summary.title}`);
  }
  if (isClosestMatch && guessProbability < 0.5) {
    evidence.push("ambiguity:ask");
    return {
      intent: "clarification",
      content: wikipediaClarificationMessage(summary, language),
      confidence: 0.65,
      evidence,
    };
  }
  const bodyLines = [
    `${summary.title}: ${summary.extract}\n\n` +
      `Source: [${humanUrl}](${summary.url}) (wikipedia).`,
  ];
  if (isClosestMatch) {
    bodyLines.push(closestMatchNote(summary, language));
    evidence.push("ambiguity:guess");
  }
  return {
    intent: "wikipedia_lookup",
    content: bodyLines.join("\n\n"),
    confidence: 0.85,
    evidence,
  };
}

const SOFTWARE_ACTION_WORDS = [
  "write",
  "build",
  "create",
  "implement",
  "make",
  "develop",
  "generate",
  "design",
  "scaffold",
];

const SOFTWARE_ARTIFACTS = [
  ["browser extension", "browser extension"],
  ["command line tool", "command-line tool"],
  ["github action", "action"],
  ["mobile app", "mobile app"],
  ["cli tool", "command-line tool"],
  ["web app", "web app"],
  ["application", "application"],
  ["extension", "extension"],
  ["dashboard", "dashboard"],
  ["scraper", "scraper"],
  ["library", "library"],
  ["website", "website"],
  ["plugin", "plugin"],
  ["add on", "extension"],
  ["addon", "extension"],
  ["service", "service"],
  ["bot", "bot"],
  ["app", "app"],
  ["api", "API"],
  ["sdk", "SDK"],
  ["tool", "tool"],
  ["mod", "mod"],
];

const SOFTWARE_FEATURE_MARKERS = [
  "add",
  "admin",
  "audit",
  "backup",
  "calendar",
  "chart",
  "check",
  "conflict",
  "cooldown",
  "csv",
  "customer",
  "damage",
  "date",
  "email",
  "expense",
  "export",
  "file",
  "filter",
  "history",
  "hp",
  "import",
  "invoice",
  "log",
  "maintenance",
  "notification",
  "payment",
  "progress",
  "protection",
  "record",
  "reminder",
  "rename",
  "report",
  "resistance",
  "retry",
  "schedule",
  "scrape",
  "stack",
  "status",
  "sync",
  "track",
  "tracking",
  "upload",
  "validate",
];

const GAME_TRACKER_TYPESCRIPT = `type Cooldown = {
  name: string;
  remainingRounds: number;
};

type UnitState = {
  id: string;
  name: string;
  hp: number;
  maxHp: number;
  protection: number;
  resistance: number;
  cooldowns: Cooldown[];
};

type DamageResult = {
  damageTaken: number;
  prevented: number;
  unit: UnitState;
};

export function mitigateDamage(unit: UnitState, rawDamage: number): DamageResult {
  const prevented = Math.max(0, unit.protection) + Math.max(0, unit.resistance);
  const damageTaken = Math.max(0, rawDamage - prevented);
  return {
    damageTaken,
    prevented,
    unit: { ...unit, hp: Math.max(0, unit.hp - damageTaken) },
  };
}

export function setStacks(
  unit: UnitState,
  protection: number,
  resistance: number,
): UnitState {
  return {
    ...unit,
    protection: Math.max(0, protection),
    resistance: Math.max(0, resistance),
  };
}

export function tickCooldowns(unit: UnitState): UnitState {
  return {
    ...unit,
    cooldowns: unit.cooldowns
      .map((cooldown) => ({
        ...cooldown,
        remainingRounds: Math.max(0, cooldown.remainingRounds - 1),
      }))
      .filter((cooldown) => cooldown.remainingRounds > 0),
  };
}`;

const GENERIC_PROJECT_TYPESCRIPT = `type ProjectRecord = {
  id: string;
  title: string;
  status: "open" | "done";
  notes: string[];
};

type ProjectCommand =
  | { type: "add"; id: string; title: string }
  | { type: "note"; id: string; note: string }
  | { type: "complete"; id: string };

export function applyCommand(
  records: ProjectRecord[],
  command: ProjectCommand,
): ProjectRecord[] {
  switch (command.type) {
    case "add":
      return [
        ...records,
        { id: command.id, title: command.title, status: "open", notes: [] },
      ];
    case "note":
      return records.map((record) =>
        record.id === command.id
          ? { ...record, notes: [...record.notes, command.note] }
          : record,
      );
    case "complete":
      return records.map((record) =>
        record.id === command.id ? { ...record, status: "done" } : record,
      );
  }
}`;

const GENERIC_PROJECT_JAVASCRIPT = `export function applyCommand(records, command) {
  switch (command.type) {
    case "add":
      return [...records, { id: command.id, title: command.title, status: "open", notes: [] }];
    case "note":
      return records.map((record) =>
        record.id === command.id
          ? { ...record, notes: [...record.notes, command.note] }
          : record,
      );
    case "complete":
      return records.map((record) =>
        record.id === command.id ? { ...record, status: "done" } : record,
      );
    default:
      throw new Error("Unknown command: " + command.type);
  }
}`;

const GENERIC_PROJECT_PYTHON = `from dataclasses import dataclass, field


@dataclass(frozen=True)
class ProjectRecord:
    id: str
    title: str
    status: str = "open"
    notes: tuple[str, ...] = field(default_factory=tuple)


def apply_command(records: tuple[ProjectRecord, ...], command: dict) -> tuple[ProjectRecord, ...]:
    if command["type"] == "add":
        return (*records, ProjectRecord(id=command["id"], title=command["title"]))
    if command["type"] == "note":
        return tuple(
            ProjectRecord(r.id, r.title, r.status, (*r.notes, command["note"]))
            if r.id == command["id"] else r
            for r in records
        )
    if command["type"] == "complete":
        return tuple(
            ProjectRecord(r.id, r.title, "done", r.notes)
            if r.id == command["id"] else r
            for r in records
        )
    raise ValueError(f"Unknown command: {command['type']}")
`;

const GENERIC_PROJECT_RUST = `#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: String,
    pub title: String,
    pub status: ProjectStatus,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Open,
    Done,
}

pub enum ProjectCommand {
    Add { id: String, title: String },
    Note { id: String, note: String },
    Complete { id: String },
}

pub fn apply_command(mut records: Vec<ProjectRecord>, command: ProjectCommand) -> Vec<ProjectRecord> {
    match command {
        ProjectCommand::Add { id, title } => records.push(ProjectRecord {
            id,
            title,
            status: ProjectStatus::Open,
            notes: Vec::new(),
        }),
        ProjectCommand::Note { id, note } => {
            for record in &mut records {
                if record.id == id {
                    record.notes.push(note.clone());
                }
            }
        }
        ProjectCommand::Complete { id } => {
            for record in &mut records {
                if record.id == id {
                    record.status = ProjectStatus::Done;
                }
            }
        }
    }
    records
}`;

function containsAny(value, needles) {
  return needles.some((needle) => value.includes(needle));
}

function containsToken(normalized, token) {
  return String(normalized || "").split(/\s+/).includes(token);
}

function containsAnyToken(normalized, tokens) {
  return String(normalized || "")
    .split(/\s+/)
    .some((token) => tokens.includes(token));
}

function detectSoftwareAction(normalized) {
  return SOFTWARE_ACTION_WORDS.find((word) => containsToken(normalized, word)) || null;
}

function detectSoftwareArtifact(normalized) {
  const match = SOFTWARE_ARTIFACTS.find(([needle]) => {
    if (needle.includes(" ")) return normalized.includes(needle);
    return containsToken(normalized, needle);
  });
  return match ? { surface: match[0], label: match[1] } : null;
}

function extractSoftwareTarget(prompt, artifact) {
  const markers = [
    `${artifact.surface} for `,
    `${artifact.surface} to `,
    `${artifact.label} for `,
    `${artifact.label} to `,
    " for ",
    " to ",
  ];
  for (const marker of markers) {
    const target = extractAfterMarker(prompt, marker);
    if (target) return target;
  }
  return "the requested environment";
}

function extractAfterMarker(prompt, marker) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  const lowerMarker = marker.toLowerCase();
  const start = lower.indexOf(lowerMarker);
  if (start < 0) return null;
  const tail = source.slice(start + lowerMarker.length);
  const stopMatch = /[?.,;\n]/.exec(tail);
  const stop = stopMatch ? stopMatch.index : tail.length;
  const raw = tail
    .slice(0, stop)
    .split(" with ")[0]
    .split(" that ")[0]
    .split(" and ")[0]
    .trim();
  if (!raw) return null;
  return capitalizeShortTarget(raw);
}

function capitalizeShortTarget(raw) {
  const compact = String(raw || "").trim().split(/\s+/).slice(0, 5).join(" ");
  if (!compact) return compact;
  if (/[A-ZА-Я]/.test(compact)) return compact;
  return compact.charAt(0).toUpperCase() + compact.slice(1);
}

function sentenceCase(raw) {
  const trimmed = String(raw || "").trim().replace(/^[-* ]+|[-* ]+$/g, "");
  if (!trimmed) return "";
  return trimmed.charAt(0).toUpperCase() + trimmed.slice(1);
}

function extractSoftwareFeatures(prompt) {
  const features = [];
  const segments = String(prompt || "").split(/[.;\n]/);
  for (const segment of segments) {
    for (const clause of segment.split(",")) {
      const cleaned = clause.trim();
      if (!cleaned) continue;
      const lower = cleaned.toLowerCase();
      if (!containsAny(lower, SOFTWARE_FEATURE_MARKERS)) continue;
      const feature = sentenceCase(cleaned);
      if (feature && !features.includes(feature)) features.push(feature);
    }
  }
  if (features.length === 0) {
    features.push("Capture state, user commands, persistence, validation, and tests.");
  }
  return features;
}

function isGameUnitTracker(normalized) {
  const domain =
    normalized.includes("dnd") ||
    normalized.includes("d d") ||
    normalized.includes("wargame") ||
    normalized.includes("tabletop") ||
    normalized.includes("unit") ||
    normalized.includes("token") ||
    normalized.includes("owlbear");
  const mechanics =
    normalized.includes("hp") ||
    normalized.includes("damage") ||
    normalized.includes("protection") ||
    normalized.includes("resistance") ||
    normalized.includes("cooldown");
  return domain && mechanics;
}

function classifySoftwareRequirement(requirement, gameTracker) {
  const lower = String(requirement || "").toLowerCase();
  if (gameTracker || containsAny(lower, ["track", "hp", "status", "damage", "cooldown"])) {
    return "state_tracking";
  }
  if (containsAny(lower, ["import", "export", "csv", "backup", "report", "calendar"])) {
    return "data_exchange";
  }
  if (containsAny(lower, ["reminder", "notification", "schedule", "weekly"])) {
    return "automation";
  }
  if (containsAny(lower, ["validate", "check", "conflict", "audit"])) {
    return "validation";
  }
  if (containsAny(lower, ["api", "discord", "telegram", "github", "browser"])) {
    return "integration";
  }
  if (containsAny(lower, ["dashboard", "chart", "filter", "progress"])) {
    return "user_interface";
  }
  return "project_behavior";
}

function softwareSubtaskTitle(category, requirement) {
  switch (category) {
    case "state_tracking":
      return `Model state fields and pure transitions for ${requirement}`;
    case "data_exchange":
      return `Define parsers, serializers, and backup flow for ${requirement}`;
    case "automation":
      return `Schedule deterministic jobs and delivery checks for ${requirement}`;
    case "validation":
      return `Encode validation rules and failure messages for ${requirement}`;
    case "integration":
      return `Isolate host API boundaries and mocks for ${requirement}`;
    case "user_interface":
      return `Design focused views and state updates for ${requirement}`;
    default:
      return `Implement and test the smallest behavior for ${requirement}`;
  }
}

function deriveSoftwareSubtasks(requirements, gameTracker) {
  return requirements.map((requirement, index) => {
    const category = classifySoftwareRequirement(requirement, gameTracker);
    return {
      requirementId: `R${index + 1}`,
      category,
      title: softwareSubtaskTitle(category, requirement),
    };
  });
}

function detectSoftwareDeliveryMode(normalized) {
  if (containsAny(normalized, ["manual instruction", "instructions", "no code"])) {
    return "manual_instructions";
  }
  if (containsAny(normalized, ["execute", "run command", "run it", "webvm"])) {
    return "immediate_execution";
  }
  if (
    containsAny(normalized, ["bash", "shell"]) ||
    containsAnyToken(normalized, ["script", "scripts", "commands"])
  ) {
    return "script_generation";
  }
  return "code_generation";
}

function detectSoftwareImplementationLanguage(normalized) {
  if (containsAny(normalized, ["python", "django", "fastapi"])) return "python";
  if (containsAny(normalized, ["rust", "cargo"])) return "rust";
  if (containsAny(normalized, ["javascript", "node.js", "node "])) return "javascript";
  return "typescript";
}

function softwareApprovalGates(normalized, deliveryMode) {
  const gates = ["task_formalization", "implementation_plan"];
  if (normalized.includes("requirement")) gates.push("requirements");
  if (containsAny(normalized, ["each step", "step by step"])) gates.push("each_step");
  if (deliveryMode === "code_generation") {
    gates.push("generated_code");
  } else if (deliveryMode === "manual_instructions") {
    gates.push("manual_instructions");
  } else {
    gates.push("generated_script");
    gates.push("bash_command");
  }
  if (containsAny(normalized, ["shell", "bash", "command", "docker", "webvm"])) {
    gates.push("bash_command");
  }
  return [...new Set(gates)].sort();
}

function softwareImplementationCode(meaning) {
  if (meaning.gameTracker) {
    return {
      label: "TypeScript",
      fence: "typescript",
      body: GAME_TRACKER_TYPESCRIPT,
    };
  }
  if (meaning.implementationLanguage === "python") {
    return { label: "Python", fence: "python", body: GENERIC_PROJECT_PYTHON };
  }
  if (meaning.implementationLanguage === "rust") {
    return { label: "Rust", fence: "rust", body: GENERIC_PROJECT_RUST };
  }
  if (meaning.implementationLanguage === "javascript") {
    return { label: "JavaScript", fence: "javascript", body: GENERIC_PROJECT_JAVASCRIPT };
  }
  return { label: "TypeScript", fence: "typescript", body: GENERIC_PROJECT_TYPESCRIPT };
}

function softwareDomainLabel(meaning) {
  return meaning.gameTracker ? "tabletop_game_unit_tracker" : "software_project";
}

function softwareApprovalLabel(approved) {
  return approved ? "approved" : "proposed";
}

function linoString(value) {
  return `"${String(value || "")
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")}"`;
}

function softwareMeaningLino(meaning, approved) {
  const lines = ["software_project_request"];
  lines.push(`  action ${linoString(meaning.action)}`);
  lines.push(`  artifact ${linoString(meaning.artifact)}`);
  lines.push(`  artifact_surface ${linoString(meaning.artifactSurface)}`);
  lines.push(`  target ${linoString(meaning.target)}`);
  lines.push(`  domain ${linoString(softwareDomainLabel(meaning))}`);
  lines.push(`  delivery_mode ${meaning.deliveryMode}`);
  lines.push(`  implementation_language ${linoString(meaning.implementationLanguage)}`);
  lines.push(`  approval_state ${softwareApprovalLabel(approved)}`);
  lines.push("  approval_required true");
  for (const gate of meaning.approvalGates) {
    lines.push(`  approval_gate ${linoString(gate)}`);
  }
  for (const requirement of meaning.requirements) {
    lines.push(`  requirement ${linoString(requirement)}`);
    lines.push(
      `  requirement_category ${linoString(
        classifySoftwareRequirement(requirement, meaning.gameTracker),
      )}`,
    );
  }
  for (const subtask of meaning.subtasks) {
    lines.push(
      `  subtask ${linoString(
        `${subtask.requirementId} [${subtask.category}] ${subtask.title}`,
      )}`,
    );
  }
  if (meaning.gameTracker) {
    lines.push('  state_model "unit_state"');
    lines.push('  command "apply_damage"');
    lines.push('  command "set_stacks"');
    lines.push('  command "tick_cooldowns"');
    lines.push('  validation "damage_mitigation_floor_at_zero"');
    lines.push('  validation "cooldowns_decrement_without_negative_rounds"');
  } else {
    lines.push('  state_model "project_records"');
    lines.push('  command "create_record"');
    lines.push('  command "update_record"');
    lines.push('  command "export_state"');
    lines.push('  validation "pure_state_transitions_before_host_api"');
  }
  return lines.join("\n") + "\n";
}

function softwareMeaningKey(meaning) {
  return [
    `action=${meaning.action}`,
    `artifact=${meaning.artifact}`,
    `target=${meaning.target}`,
    `game_tracker=${meaning.gameTracker}`,
    `delivery_mode=${meaning.deliveryMode}`,
    `implementation_language=${meaning.implementationLanguage}`,
    ...meaning.requirements.map((requirement) => `requirement=${requirement}`),
    ...meaning.subtasks.map((subtask) => `subtask=${subtask.category}:${subtask.title}`),
  ].join(";");
}

function stableSoftwareMeaningId(meaning) {
  let hash = 0xcbf29ce484222325n;
  const source = softwareMeaningKey(meaning);
  for (let index = 0; index < source.length; index += 1) {
    hash ^= BigInt(source.charCodeAt(index));
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `software_project_request_${hash.toString(16)}`;
}

function formalizeSoftwareProjectRequest(prompt) {
  const normalized = normalizePrompt(prompt);
  if (normalized.includes("hello") && normalized.includes("world")) return null;
  const action = detectSoftwareAction(normalized);
  const artifact = detectSoftwareArtifact(normalized);
  if (!action || !artifact) return null;
  const requirements = extractSoftwareFeatures(prompt);
  const gameTracker = isGameUnitTracker(normalized);
  const deliveryMode = detectSoftwareDeliveryMode(normalized);
  return {
    action,
    artifactSurface: artifact.surface,
    artifact: artifact.label,
    target: extractSoftwareTarget(prompt, artifact),
    requirements,
    subtasks: deriveSoftwareSubtasks(requirements, gameTracker),
    deliveryMode,
    implementationLanguage: detectSoftwareImplementationLanguage(normalized),
    approvalGates: softwareApprovalGates(normalized, deliveryMode),
    gameTracker,
  };
}

function softwareReasoningSteps(meaning) {
  const steps = [
    `Classify the impulse as a request to ${meaning.action} a ${meaning.artifact} instead of a fact lookup.`,
    `Bind the target environment to ${meaning.target} and keep the first response reviewable.`,
    `Extract ${meaning.requirements.length} requirement(s) into the meaning record before planning.`,
    `Decompose the requirement graph into ${meaning.subtasks.length} implementation subtask(s) with category labels.`,
    `Select delivery mode ${meaning.deliveryMode} and approval gates: ${meaning.approvalGates.join(", ")}.`,
  ];
  if (meaning.gameTracker) {
    steps.push(
      "Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.",
    );
  }
  steps.push("Ask for approval before producing code, scripts, manual instructions, or execution steps.");
  return steps;
}

function softwarePlanSteps(meaning) {
  const steps = [
    "Review the formalized task, requirement graph, approval gates, and delivery mode with the user.",
  ];
  if (meaning.gameTracker) {
    steps.push(
      `Confirm the ${meaning.target} storage and selected-token API boundaries.`,
      "Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.",
      "Write pure transition functions for damage mitigation, stack edits, and round ticks.",
      "Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.",
      "Wire the tested core into the extension panel and host persistence.",
    );
    return steps;
  }
  steps.push(
    `Confirm the host API and data boundaries for ${meaning.target}.`,
    "Define the smallest serializable state records for the requirements.",
  );
  for (const subtask of meaning.subtasks) {
    steps.push(`Implement ${subtask.category}: ${subtask.title}.`);
  }
  steps.push(
    `Generate a ${meaning.implementationLanguage} starter core plus language-appropriate repository initialization and checks.`,
  );
  steps.push("Keep shell, Docker, or WebVM commands behind the configured approval gates.");
  return steps;
}

function softwareEvidence(meaning, approved) {
  const evidence = [
    "formalization:text_to_links_notation",
    `meaning:${stableSoftwareMeaningId(meaning)}`,
    `software_project:action:${meaning.action}`,
    `software_project:artifact:${meaning.artifact}`,
    `software_project:target:${meaning.target}`,
    `software_project:domain:${softwareDomainLabel(meaning)}`,
    `software_project:delivery_mode:${meaning.deliveryMode}`,
    `software_project:implementation_language:${meaning.implementationLanguage}`,
    `approval_state:${softwareApprovalLabel(approved)}`,
    `software_project:strategy:${meaning.gameTracker ? "game_unit_tracker" : "bounded_project_plan"}`,
  ];
  for (const gate of meaning.approvalGates) {
    evidence.push(`approval_gate:${gate}`);
  }
  for (const requirement of meaning.requirements) {
    evidence.push(`requirement:${requirement}`);
    evidence.push(`requirement_category:${classifySoftwareRequirement(requirement, meaning.gameTracker)}`);
  }
  for (const subtask of meaning.subtasks) {
    evidence.push(`software_project:subtask:${subtask.requirementId}:${subtask.category}:${subtask.title}`);
  }
  return evidence;
}

function renderSoftwareProjectPlan(meaning) {
  const lines = [];
  lines.push(
    `Implementation plan pending approval for a ${meaning.artifact} targeting ${meaning.target}.`,
  );
  lines.push("");
  lines.push("Formalized meaning:");
  lines.push("```lino");
  lines.push(softwareMeaningLino(meaning, false).trimEnd());
  lines.push("```");
  lines.push("");
  lines.push("Reasoning steps:");
  softwareReasoningSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  lines.push("Requirement model:");
  meaning.requirements.forEach((requirement, index) => {
    const category = classifySoftwareRequirement(requirement, meaning.gameTracker);
    lines.push(`${index + 1}. [${category}] ${requirement}`);
  });
  lines.push("");
  lines.push("Subtasks:");
  meaning.subtasks.forEach((subtask, index) => {
    lines.push(`${index + 1}. ${subtask.requirementId} -> ${subtask.title}`);
  });
  lines.push("");
  lines.push("Approval gates:");
  meaning.approvalGates.forEach((gate) => {
    lines.push(`- ${gate}`);
  });
  lines.push("");
  lines.push("Proposed plan:");
  softwarePlanSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  lines.push(
    "Reply `approve plan` to generate the starter implementation, or describe what to change.",
  );
  return lines.join("\n");
}

function renderSoftwareProjectImplementation(meaning) {
  const lines = [];
  lines.push(
    `Approved implementation starter for a ${meaning.artifact} targeting ${meaning.target}.`,
  );
  lines.push("");
  lines.push("Formalized meaning:");
  lines.push("```lino");
  lines.push(softwareMeaningLino(meaning, true).trimEnd());
  lines.push("```");
  lines.push("");
  lines.push("Implementation steps:");
  softwarePlanSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  const code = softwareImplementationCode(meaning);
  lines.push(`Starter ${code.label} core:`);
  lines.push("");
  lines.push(`\`\`\`${code.fence}`);
  lines.push(code.body);
  lines.push("```");
  lines.push("");
  lines.push("Generated code checks:");
  lines.push(`1. Initialize a ${code.label} project in an isolated workspace.`);
  lines.push("2. Run the language-native syntax/type check before host integration.");
  return lines.join("\n");
}

function isSoftwareApprovalPrompt(normalized) {
  const compact = String(normalized || "").replace(/[.!?,]/g, "").trim();
  return [
    "approve",
    "approved",
    "approve plan",
    "yes",
    "yes proceed",
    "proceed",
    "go ahead",
    "looks good",
    "do it",
    "start implementation",
    "generate code",
    "convert to code",
  ].includes(compact);
}

function lastHistoryTurn(history, role) {
  if (!Array.isArray(history)) return null;
  for (let index = history.length - 1; index >= 0; index -= 1) {
    const turn = history[index];
    if (turn && turn.role === role && turn.content) return String(turn.content);
  }
  return null;
}

function priorSoftwareProjectMeaning(history) {
  const assistant = lastHistoryTurn(history, "assistant");
  if (
    !assistant ||
    !assistant.includes("software_project_request") ||
    !assistant.includes("approve plan")
  ) {
    return null;
  }
  const user = lastHistoryTurn(history, "user");
  return user ? formalizeSoftwareProjectRequest(user) : null;
}

function trySoftwareProjectRequest(prompt, history = []) {
  const normalized = normalizePrompt(prompt);
  if (isSoftwareApprovalPrompt(normalized)) {
    const prior = priorSoftwareProjectMeaning(history);
    if (prior) {
      return {
        intent: "software_project_implementation",
        content: renderSoftwareProjectImplementation(prior),
        confidence: 0.82,
        evidence: softwareEvidence(prior, true),
      };
    }
  }

  const meaning = formalizeSoftwareProjectRequest(prompt);
  if (!meaning) return null;

  return {
    intent: "software_project_plan",
    content: renderSoftwareProjectPlan(meaning),
    confidence: 0.78,
    evidence: softwareEvidence(meaning, false),
  };
}

function tryJavaScriptExecution(prompt) {
  const program = extractJavaScriptProgram(prompt);
  if (program === null) return null;
  const logs = [];
  const captureConsole = {
    log: (...args) =>
      logs.push(
        args
          .map((value) =>
            typeof value === "string" ? value : JSON.stringify(value),
          )
          .join(" "),
      ),
  };
  let result;
  let error = null;
  try {
    const runner = new Function(
      "console",
      `"use strict"; return (function(){ ${program}\n })();`,
    );
    result = runner(captureConsole);
  } catch (err) {
    error = err;
  }
  const lines = [];
  lines.push("Execution status: ran in the demo's Web Worker sandbox.");
  lines.push("Source:");
  lines.push("```javascript");
  lines.push(program);
  lines.push("```");
  if (error) {
    lines.push("");
    lines.push(`Error: ${error.message || String(error)}`);
  } else {
    if (logs.length > 0) {
      lines.push("");
      lines.push("Output:");
      lines.push("```text");
      lines.push(logs.join("\n"));
      lines.push("```");
    }
    if (result !== undefined) {
      lines.push("");
      lines.push(`Returned: \`${String(result)}\``);
    }
    if (logs.length === 0 && result === undefined) {
      lines.push("");
      lines.push("Program completed without output or return value.");
    }
  }
  lines.push("");
  lines.push(
    "Note: the browser worker has no DOM or network access, so side effects are limited.",
  );
  return {
    intent: error ? "javascript_execution_error" : "javascript_execution",
    content: lines.join("\n"),
    confidence: error ? 0.5 : 0.95,
    evidence: [
      `execution_status:javascript:${error ? "error" : "ran"}`,
      "language:javascript",
    ],
  };
}

function helloWorldLanguage(prompt) {
  const tokens = normalizePrompt(prompt).split(/\s+/);
  if (!(tokens.includes("hello") && tokens.includes("world"))) return null;
  if (tokens.includes("rust") || tokens.includes("rs")) return "rust";
  if (tokens.includes("python") || tokens.includes("py")) return "python";
  if (tokens.includes("typescript") || tokens.includes("ts"))
    return "typescript";
  if (
    tokens.includes("javascript") ||
    tokens.includes("js") ||
    tokens.includes("node")
  )
    return "javascript";
  if (tokens.includes("go") || tokens.includes("golang")) return "go";
  if (tokens.includes("c")) return "c";
  return null;
}

function tryHelloWorld(prompt) {
  const language = helloWorldLanguage(prompt);
  if (!language) return null;
  const seeds = {
    rust: {
      fence: "rust",
      code: 'fn main() {\n    println!("Hello, world!");\n}',
    },
    python: {
      fence: "python",
      code: 'print("Hello, world!")',
    },
    javascript: {
      fence: "javascript",
      code: 'console.log("Hello, world!");',
    },
    typescript: {
      fence: "typescript",
      code: 'console.log("Hello, world!");',
    },
    go: {
      fence: "go",
      code:
        'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, world!")\n}',
    },
    c: {
      fence: "c",
      code:
        '#include <stdio.h>\n\nint main(void) {\n    puts("Hello, world!");\n    return 0;\n}',
    },
  };
  const { fence, code } = seeds[language];
  const lines = [];
  lines.push(`Here is a minimal ${language} hello world program:`);
  lines.push("");
  lines.push("```" + fence);
  lines.push(code);
  lines.push("```");
  lines.push("");
  if (language === "javascript") {
    const logs = [];
    try {
      const runner = new Function(
        "console",
        `"use strict"; ${code}`,
      );
      runner({ log: (...args) => logs.push(args.join(" ")) });
      lines.push("Execution status: ran in the demo's Web Worker sandbox.");
      lines.push("Output:");
      lines.push("```text");
      lines.push(logs.join("\n") || "(no output)");
      lines.push("```");
    } catch (error) {
      lines.push(
        `Execution status: failed in sandbox — ${error.message || String(error)}.`,
      );
    }
  } else {
    lines.push(
      `Execution status: not run — the browser sandbox cannot invoke a ${language} toolchain. Copy the snippet into a ${language} environment to verify.`,
    );
  }
  return {
    intent: `hello_world_${language}`,
    content: lines.join("\n"),
    confidence: 0.9,
    evidence: [
      `hello_world:${language}`,
      `execution_status:${language}:${language === "javascript" ? "ran" : "unavailable"}`,
    ],
  };
}

function tryHistorical(prompt, history) {
  const normalized = normalizePrompt(prompt);
  // Issue #27: summarize triggers can be in non-Latin scripts that normalize
  // to an empty string, so test before bailing.
  if (isSummarizePrompt(prompt, normalized)) {
    return trySummarizeConversation(history);
  }
  if (!normalized) return null;
  if (normalized === "what is my name" || normalized === "what s my name") {
    const hit = tryRecallName(history);
    if (hit) return hit;
  }
  if (
    normalized === "what was my previous question" ||
    normalized === "what was the previous question" ||
    normalized === "what was my last question"
  ) {
    return tryRecallLastQuestion(history);
  }
  return null;
}

// Issue #27: trigger the summarize skill on a wide range of natural phrasings
// (English/Russian/Hindi/Chinese), not just two literal sentences. We match on
// the raw prompt for non-Latin scripts because normalizePrompt strips them.
function isSummarizePrompt(prompt, normalized) {
  const raw = String(prompt || "").trim().toLowerCase();
  if (
    normalized === "summarize" ||
    normalized === "summarise" ||
    normalized === "summarize chat" ||
    normalized === "summarise chat" ||
    normalized === "summarize so far" ||
    normalized === "summarise so far" ||
    normalized === "summary"
  ) {
    return true;
  }
  if (
    normalized.startsWith("summarize the conversation") ||
    normalized.startsWith("summarise the conversation") ||
    normalized.startsWith("summarize this conversation") ||
    normalized.startsWith("summarise this conversation") ||
    normalized.startsWith("summarize our conversation") ||
    normalized.startsWith("summarise our conversation") ||
    normalized.startsWith("summarize the chat") ||
    normalized.startsWith("summarise the chat") ||
    normalized.startsWith("summarize this chat") ||
    normalized.startsWith("summarise this chat") ||
    normalized.startsWith("give me a summary") ||
    normalized.startsWith("can you summarize") ||
    normalized.startsWith("can you summarise") ||
    normalized.startsWith("please summarize") ||
    normalized.startsWith("please summarise")
  ) {
    return true;
  }
  // Russian: суммируй / резюмируй / подведи итог / краткое резюме
  if (
    /^(суммируй|резюмируй|подведи\s+итог|кратк(ое|ий)\s+резюме|сделай\s+резюме|резюме\s+(беседы|разговора|чата))/.test(
      raw,
    )
  ) {
    return true;
  }
  // Hindi: सारांश / सारांश दो / संक्षेप
  if (/^(सारांश|संक्षेप|सार\s+दो|सारांश\s+दो)/.test(raw)) {
    return true;
  }
  // Chinese (simplified + traditional): 总结 / 總結 / 概括
  if (/^(总结|總結|概括|摘要)/.test(raw)) {
    return true;
  }
  return false;
}

function isFetchPrompt(normalized) {
  return normalized.startsWith("fetch ") && normalized.length > 6;
}

function trimUrlToken(token) {
  return String(token || "")
    .replace(/^[<>()\[\]{}"'`«»]+/u, "")
    .replace(/[<>()\[\]{}"'`«»]+$/u, "")
    .replace(/[.,!?;:…]+$/u, "");
}

function looksLikeHostname(value) {
  const host = String(value || "").trim();
  if (!host.includes(".") || host.startsWith(".") || host.endsWith(".")) {
    return false;
  }
  const labels = host.split(".");
  if (labels.some((label) => !label)) return false;
  const tld = labels[labels.length - 1] || "";
  if (tld.length < 2) return false;
  return labels.every(
    (label) =>
      /^[a-z0-9-]+$/i.test(label) &&
      !label.startsWith("-") &&
      !label.endsWith("-"),
  );
}

function normalizeUrlCandidate(candidate) {
  const text = String(candidate || "").trim();
  if (!text || /\s/.test(text) || text.includes("@")) return null;
  const lower = text.toLowerCase();
  const url =
    lower.startsWith("http://") || lower.startsWith("https://")
      ? text
      : lower.startsWith("www.") || looksLikeHostname(text)
        ? `https://${text}`
        : "";
  if (!url) return null;
  let parsed;
  try {
    parsed = new URL(url);
  } catch (_error) {
    return null;
  }
  if (!looksLikeHostname(parsed.hostname)) return null;
  return parsed.href.replace(/\/$/, "");
}

function firstUrlCandidate(prompt) {
  const tokens = String(prompt || "").split(/\s+/);
  for (const token of tokens) {
    const trimmed = trimUrlToken(token);
    const url = normalizeUrlCandidate(trimmed);
    if (url) return { raw: trimmed, url };
  }
  return null;
}

const HTTP_FETCH_PREFIXES = [
  "fetch ",
  "fetch url ",
  "fetch the url ",
  "http fetch ",
  "request ",
  "make request to ",
  "send request to ",
  "сделай запрос ",
  "сделай http запрос ",
  "выполни запрос ",
  "выполни http запрос ",
  "запроси ",
  "получи ",
  "http запрос к ",
  "http запрос на ",
];

const HTTP_FETCH_MARKERS = [
  "make a request to",
  "make an http request to",
  "send a request to",
  "send an http request to",
  "http request to",
  "http get to",
  "fetch the url",
  "fetch this url",
  "fetch the page",
  "сделай запрос к",
  "сделай запрос на",
  "сделай http запрос к",
  "сделай http запрос на",
  "выполни запрос к",
  "выполни запрос на",
  "выполни http запрос к",
  "выполни http запрос на",
  "запрос к",
  "запрос на",
  "http запрос к",
  "http запрос на",
];

const URL_NAVIGATE_PREFIXES = [
  "navigate to ",
  "navigate ",
  "go to ",
  "goto ",
  "visit ",
  "browse to ",
  "browse ",
  "show ",
  "show me ",
  "display ",
  "load ",
  "open ",
  "open url ",
  "open the url ",
  "open site ",
  "open website ",
  "open page ",
  "open the page ",
  "open the website ",
  "take me to ",
  "preview ",
  "view ",
  "see ",
  "get ",
  "перейди ",
  "перейди на ",
  "переходи на ",
  "переходи ",
  "перейдите на ",
  "открой ",
  "открой сайт ",
  "открой страницу ",
  "открой ссылку ",
  "открой урл ",
  "покажи ",
  "покажи сайт ",
  "покажи страницу ",
  "покажи мне ",
  "загрузи ",
  "загрузи страницу ",
  "посети ",
  "зайди на ",
  "зайди ",
  "просмотри ",
  "отобрази ",
];

const URL_NAVIGATE_MARKERS = [
  "navigate to",
  "go to",
  "goto",
  "browse to",
  "take me to",
  "open the page",
  "open the site",
  "open the website",
  "open the url",
  "open url",
  "перейди на",
  "переходи на",
  "перейдите на",
  "открой сайт",
  "открой страницу",
  "открой ссылку",
  "открой урл",
  "покажи сайт",
  "покажи страницу",
  "зайди на",
];

function startsWithAny(haystack, prefixes) {
  return prefixes.some((prefix) => haystack.startsWith(prefix));
}

function includesAny(haystack, markers) {
  return markers.some((marker) => haystack.includes(marker));
}

function isHttpFetchPrompt(prompt, normalized) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  if (isFetchPrompt(normalized)) return true;
  if (
    startsWithAny(normalized, HTTP_FETCH_PREFIXES) ||
    startsWithAny(raw, HTTP_FETCH_PREFIXES)
  ) {
    return true;
  }
  return (
    includesAny(normalized, HTTP_FETCH_MARKERS) ||
    includesAny(raw, HTTP_FETCH_MARKERS)
  );
}

function isUrlNavigatePrompt(prompt, normalized, rawCandidate) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  if (raw.startsWith(String(rawCandidate || "").toLowerCase())) {
    return true;
  }
  if (
    startsWithAny(normalized, URL_NAVIGATE_PREFIXES) ||
    startsWithAny(raw, URL_NAVIGATE_PREFIXES)
  ) {
    return true;
  }
  return (
    includesAny(normalized, URL_NAVIGATE_MARKERS) ||
    includesAny(raw, URL_NAVIGATE_MARKERS)
  );
}

function extractHttpFetchUrl(prompt, normalized) {
  const candidate = firstUrlCandidate(prompt);
  if (!candidate || !isHttpFetchPrompt(prompt, normalized)) {
    return null;
  }
  return candidate.url;
}

function extractUrlNavigateUrl(prompt, normalized) {
  const candidate = firstUrlCandidate(prompt);
  if (!candidate || !isUrlNavigatePrompt(prompt, normalized, candidate.raw)) {
    return null;
  }
  return candidate.url;
}

function cleanSearchQuery(value) {
  return String(value || "")
    .trim()
    .replace(/^[<>()\[\]{}"'`«»]+/u, "")
    .replace(/[<>()\[\]{}"'`«»]+$/u, "")
    .replace(/[.,!?;:…]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
}

function stripSearchPrefix(prompt, prefix) {
  const text = String(prompt || "").trim();
  if (text.toLowerCase().startsWith(prefix)) {
    return cleanSearchQuery(text.slice(prefix.length));
  }
  return "";
}

function extractWebSearchQuery(prompt, normalized) {
  if (
    normalized.startsWith("search conversations ") ||
    normalized.startsWith("search my conversations ") ||
    normalized.startsWith("search my chats ")
  ) {
    return "";
  }
  const prefixes = [
    "search the web for ",
    "search web for ",
    "search the internet for ",
    "search internet for ",
    "search online for ",
    "web search for ",
    "find on the internet ",
    "find online ",
    "look up online ",
    "найди в интернете ",
    "поищи в интернете ",
    "поиск в интернете ",
    "найди онлайн ",
    "поищи онлайн ",
    "найди в сети ",
    "поищи в сети ",
  ];
  for (const prefix of prefixes) {
    const rawQuery = stripSearchPrefix(prompt, prefix);
    const normalizedQuery = normalized.startsWith(prefix)
      ? cleanSearchQuery(normalized.slice(prefix.length))
      : "";
    const query = rawQuery || normalizedQuery;
    if (query && !normalizeUrlCandidate(query)) {
      return query;
    }
  }
  return "";
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

async function tryFetch(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractHttpFetchUrl(prompt, normalized);
  if (!url) return null;

  const evidence = [`http_fetch:request:${url}`];

  if (typeof fetch !== "function") {
    return {
      intent: "http_fetch",
      content: `HTTP fetch is not available in this environment.\n\nURL: [${url}](${url})`,
      confidence: 0.5,
      evidence,
      iframeUrl: url,
    };
  }

  try {
    const response = await fetch(url, { method: "GET", mode: "cors" });
    const status = response.status;
    const contentType = response.headers.get("content-type") || "";
    let body = "";
    if (contentType.includes("text/") || contentType.includes("application/json")) {
      const text = await response.text();
      body = text.length > 2000 ? `${text.slice(0, 2000)}\n\n*(truncated — ${text.length} bytes total)*` : text;
    }
    evidence.push(`http_fetch:status:${status}`);
    const lines = [
      `Fetched \`${url}\` — status **${status}**.`,
      "",
    ];
    if (body) {
      lines.push("Response body:");
      lines.push("```");
      lines.push(body);
      lines.push("```");
    } else {
      lines.push(`Content-Type: \`${contentType || "unknown"}\` — binary or empty body, not shown.`);
      lines.push("");
      lines.push(`You can view this URL directly: [${url}](${url})`);
    }
    return {
      intent: "http_fetch",
      content: lines.join("\n"),
      confidence: 0.95,
      evidence,
      iframeUrl: null,
    };
  } catch (err) {
    // CORS block or network failure — fall back to iframe.
    const isCors =
      err instanceof TypeError &&
      (err.message.toLowerCase().includes("cors") ||
        err.message.toLowerCase().includes("network") ||
        err.message.toLowerCase().includes("failed to fetch"));
    evidence.push(`http_fetch:error:${isCors ? "cors" : "network"}`);
    const lines = [
      `Could not fetch \`${url}\` directly${isCors ? " (CORS restriction)" : " (network error)"}.`,
      "",
      "The page is shown in the embedded frame below. Use the open-in-new-tab control if the site blocks embedding, or the full-screen control to view it at viewport size.",
    ];
    return {
      intent: "http_fetch",
      content: lines.join("\n"),
      confidence: 0.7,
      evidence,
      iframeUrl: url,
    };
  }
}

async function tryUrlNavigate(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractUrlNavigateUrl(prompt, normalized);
  if (!url) return null;

  const evidence = [`url_navigate:request:${url}`, `url_preview:iframe:${url}`];
  const lines = [
    `URL requested for \`${url}\`.`,
    "",
    `Open this link: [${url}](${url}).`,
    "",
    [
      "The page is shown in the embedded frame below when the site allows framing.",
      "Use the open-in-new-tab control if the site blocks embedding,",
      "or the full-screen control to view it at viewport size.",
    ].join(" "),
  ];

  return {
    intent: "url_navigate",
    content: lines.join("\n"),
    confidence: 0.95,
    evidence,
    iframeUrl: url,
  };
}

// Reciprocal Rank Fusion constant — Cormack et al. 2009 use k = 60 and we
// match that so combined ranks stay comparable across the CLI, server, and
// browser surfaces (issue #133).
//
// The authoritative value lives in `web_search_core::WEB_SEARCH_RRF_K` and is
// fetched from the WASM worker once it boots; the JS constants below are
// pre-WASM fallbacks used during init() and on browsers where the worker
// could not instantiate. The Rust→WASM port is the source of truth (R194).
const WEB_SEARCH_RRF_K_FALLBACK = 60;
const WEB_SEARCH_CONCURRENCY_FALLBACK = 5;
const WEB_SEARCH_PROVIDER_LIMIT_FALLBACK = 10;

const WEB_SEARCH_TEXT_ENCODER = new TextEncoder();
const WEB_SEARCH_TEXT_DECODER = new TextDecoder();

function webSearchRrfK() {
  if (wasm && typeof wasm.web_search_rrf_k === "function") {
    return wasm.web_search_rrf_k() >>> 0;
  }
  return WEB_SEARCH_RRF_K_FALLBACK;
}

function webSearchConcurrency() {
  if (wasm && typeof wasm.web_search_concurrency_per_category === "function") {
    return wasm.web_search_concurrency_per_category() >>> 0;
  }
  return WEB_SEARCH_CONCURRENCY_FALLBACK;
}

function webSearchProviderLimit() {
  if (wasm && typeof wasm.web_search_provider_limit === "function") {
    return wasm.web_search_provider_limit() >>> 0;
  }
  return WEB_SEARCH_PROVIDER_LIMIT_FALLBACK;
}

function wasmWriteInput(text) {
  if (!wasm || typeof wasm.input_ptr !== "function") return -1;
  const bytes = WEB_SEARCH_TEXT_ENCODER.encode(text);
  const capacity =
    typeof wasm.input_capacity === "function" ? wasm.input_capacity() : bytes.length;
  if (bytes.length > capacity) return -1;
  const view = new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), bytes.length);
  view.set(bytes);
  return bytes.length;
}

function wasmReadOutput(length) {
  if (!wasm || typeof wasm.output_ptr !== "function" || length <= 0) return "";
  const view = new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), length);
  return WEB_SEARCH_TEXT_DECODER.decode(view);
}

// Engine-core bridges (R194 follow-up). Each function returns a value when
// the WASM core is available, or `null` so the caller can fall back to the
// pure-JS branch. Keeping a JS fallback covers offline mode and old browsers
// where `WebAssembly.instantiate` is unavailable, but the canonical answer
// always comes from Rust when the worker booted successfully.
function wasmNormalizePrompt(text) {
  if (!wasm || typeof wasm.engine_normalize_prompt !== "function") return null;
  const length = wasmWriteInput(String(text || ""));
  if (length < 0) return null;
  const written = wasm.engine_normalize_prompt(length) >>> 0;
  return wasmReadOutput(written);
}

function wasmDetectLanguage(text) {
  if (!wasm || typeof wasm.engine_detect_language !== "function") return null;
  const length = wasmWriteInput(String(text || ""));
  if (length < 0) return null;
  const written = wasm.engine_detect_language(length) >>> 0;
  const slug = wasmReadOutput(written);
  return slug || null;
}

// Returns `{ ok: true, value }` on success, `{ ok: false, error }` on parse
// or runtime failure (division by zero, overflow). `null` means the WASM core
// is unavailable — the caller should fall back to the JS parser.
function wasmEvaluateArithmetic(expression) {
  if (!wasm || typeof wasm.engine_evaluate_arithmetic !== "function") return null;
  const length = wasmWriteInput(String(expression || ""));
  if (length < 0) return null;
  const written = wasm.engine_evaluate_arithmetic(length) >>> 0;
  if (written === 0) return null;
  const text = wasmReadOutput(written);
  if (text.startsWith("ERR:")) {
    return { ok: false, error: text.slice(4) };
  }
  return { ok: true, value: text };
}

// Delegates to `web_search_request_evidence` when the WASM core is loaded;
// otherwise returns null so the caller can fall back to the JS list. The
// Rust side owns the canonical evidence shape (issue #133 R194).
function wasmWebSearchRequestEvidence(query, language) {
  if (!wasm || typeof wasm.web_search_request_evidence !== "function") return null;
  const payload = `${String(query || "")}\n${String(language || "")}`;
  const length = wasmWriteInput(payload);
  if (length < 0) return null;
  const written = wasm.web_search_request_evidence(length) >>> 0;
  if (written === 0) return null;
  const text = wasmReadOutput(written);
  return text ? text.split("\n") : null;
}

// Delegates to `web_search_fuse`. Returns the fused entries array or null when
// WASM is unavailable / the payload exceeds the static INPUT buffer.
function wasmReciprocalRankFusion(perProviderResults) {
  if (!wasm || typeof wasm.web_search_fuse !== "function") return null;
  const rows = [];
  for (const provider of perProviderResults) {
    const id = String(provider.id || "");
    const list = Array.isArray(provider.results) ? provider.results : [];
    list.forEach((item, index) => {
      if (!item || !item.url) return;
      const rank = index + 1;
      const title = String(item.title || item.url).replace(/[\t\n]/g, " ");
      const excerpt = String(item.excerpt || "").replace(/[\t\n]/g, " ");
      const url = String(item.url).replace(/[\t\n]/g, " ");
      rows.push(`${id}\t${rank}\t${url}\t${title}\t${excerpt}`);
    });
  }
  if (rows.length === 0) return [];
  const length = wasmWriteInput(rows.join("\n"));
  if (length < 0) return null;
  const written = wasm.web_search_fuse(length) >>> 0;
  if (written === 0) return [];
  const text = wasmReadOutput(written);
  if (!text) return [];
  return parseFusedOutput(text);
}

// Parse the `serialize_rrf_output` format: one entry per line, fields
// separated by tabs, providers serialised as `id#rank` joined by `;`. The
// shape matches `web_search_core::serialize_rrf_output`.
function parseFusedOutput(text) {
  return text
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const fields = line.split("\t");
      const url = fields[0] || "";
      const title = fields[1] || url;
      const excerpt = fields[2] || "";
      const score = Number.parseFloat(fields[3] || "0") || 0;
      const providerSpecs = (fields[4] || "")
        .split("+")
        .filter((part) => part.length > 0)
        .map((part) => {
          const hash = part.lastIndexOf("#");
          if (hash < 0) return { id: part, rank: 0 };
          return {
            id: part.slice(0, hash),
            rank: Number.parseInt(part.slice(hash + 1), 10) || 0,
          };
        });
      return { url, title, excerpt, score, providers: providerSpecs };
    });
}

// Session-scoped CORS disable list. When a provider fetch throws a CORS or
// network error we record the timestamp so the planner skips it for the rest
// of the session and records the decision in memory.
const WEB_SEARCH_DISABLED = new Map();

function webSearchDisable(providerId, reason) {
  if (!WEB_SEARCH_DISABLED.has(providerId)) {
    WEB_SEARCH_DISABLED.set(providerId, { reason, at: Date.now() });
  }
}

function webSearchIsDisabled(providerId) {
  return WEB_SEARCH_DISABLED.has(providerId);
}

async function fetchProviderJson(providerId, url, options) {
  if (typeof fetch !== "function") {
    webSearchDisable(providerId, "no_fetch");
    return { ok: false, error: "fetch unavailable", finalUrl: url };
  }
  try {
    const response = await fetch(url, options || { mode: "cors" });
    if (!response || !response.ok) {
      return {
        ok: false,
        status: response ? response.status : 0,
        statusText: response ? response.statusText : "",
        finalUrl: url,
      };
    }
    const data = await response.json();
    return { ok: true, status: response.status, data, finalUrl: url };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const isCors =
      message.toLowerCase().includes("cors") ||
      message.toLowerCase().includes("network") ||
      message.toLowerCase().includes("failed to fetch");
    webSearchDisable(providerId, isCors ? "cors" : "network");
    return { ok: false, error: message, finalUrl: url, cors: isCors };
  }
}

async function searchDuckDuckGo(query, limit) {
  // DuckDuckGo Instant Answer — CORS-readable, no key. Returns the abstract
  // and a flat list of related-topic links. We treat the abstract link plus
  // the related topics as the ranked result list (issue #133).
  const url =
    "https://api.duckduckgo.com/?q=" +
    encodeURIComponent(query) +
    "&format=json&no_redirect=1&no_html=1";
  const outcome = await fetchProviderJson("duckduckgo", url);
  if (!outcome.ok || !outcome.data) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const data = outcome.data;
  const results = [];
  if (data.AbstractURL && data.AbstractText) {
    results.push({
      title: data.Heading || query,
      url: data.AbstractURL,
      excerpt: stripHtml(data.AbstractText),
    });
  }
  const topics = Array.isArray(data.RelatedTopics) ? data.RelatedTopics : [];
  for (const topic of topics) {
    if (!topic) continue;
    if (topic.FirstURL && topic.Text) {
      results.push({
        title: topic.Text.split(" - ")[0] || topic.Text,
        url: topic.FirstURL,
        excerpt: stripHtml(topic.Text),
      });
    } else if (Array.isArray(topic.Topics)) {
      for (const inner of topic.Topics) {
        if (inner && inner.FirstURL && inner.Text) {
          results.push({
            title: inner.Text.split(" - ")[0] || inner.Text,
            url: inner.FirstURL,
            excerpt: stripHtml(inner.Text),
          });
        }
      }
    }
    if (results.length >= limit) break;
  }
  return { ok: true, results: results.slice(0, limit), finalUrl: outcome.finalUrl };
}

async function searchWikipediaWebProvider(query, language, limit) {
  // Reuse the existing helper but adapt the shape to {title, url, excerpt}.
  const result = await searchWikipediaPages(query, language, limit);
  if (!result || !Array.isArray(result.pages)) {
    return { ok: false, results: [], finalUrl: "", language: language || "en" };
  }
  return {
    ok: true,
    results: result.pages.slice(0, limit),
    language: result.language,
    finalUrl: `https://${result.language}.wikipedia.org/w/rest.php/v1/search/page?q=${encodeURIComponent(query)}`,
  };
}

async function searchWikidataEntities(query, language, limit) {
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  const url =
    "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=" +
    encodeURIComponent(query) +
    "&language=" +
    encodeURIComponent(lang) +
    "&format=json&origin=*&limit=" +
    encodeURIComponent(limit);
  const outcome = await fetchProviderJson("wikidata", url);
  if (!outcome.ok || !outcome.data || !Array.isArray(outcome.data.search)) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const results = outcome.data.search.slice(0, limit).map((entry) => ({
    title: entry.label || entry.id || query,
    url: entry.concepturi || `https://www.wikidata.org/wiki/${entry.id}`,
    excerpt: stripHtml(entry.description || ""),
  }));
  return { ok: true, results, finalUrl: outcome.finalUrl };
}

const WEB_SEARCH_PROVIDERS = [
  { id: "duckduckgo", label: "DuckDuckGo Instant Answer", run: searchDuckDuckGo },
  {
    id: "wikipedia",
    label: "Wikipedia REST",
    run: (query, language, limit) =>
      searchWikipediaWebProvider(query, language, limit),
  },
  {
    id: "wikidata",
    label: "Wikidata entities",
    run: (query, language, limit) =>
      searchWikidataEntities(query, language, limit),
  },
];

async function runWithConcurrencyLimit(tasks, limit) {
  // Simple p-limit style runner so we never exceed the browser's per-origin
  // socket budget. Tasks are async functions returning a value; results are
  // returned in the original order.
  const cap = Math.max(1, Math.min(limit, tasks.length));
  const results = new Array(tasks.length);
  let cursor = 0;
  async function worker() {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= tasks.length) return;
      results[index] = await tasks[index]();
    }
  }
  await Promise.all(Array.from({ length: cap }, () => worker()));
  return results;
}

function reciprocalRankFusion(perProviderResults, k) {
  // R194: the Rust/WASM core owns the fusion logic so the offline trace and
  // the browser worker agree to the last byte. We try WASM first and only
  // fall back to the JS implementation when the worker booted in
  // `js fallback` mode (e.g. WASM disabled in the browser).
  const wasmFused = wasmReciprocalRankFusion(perProviderResults);
  if (wasmFused !== null) {
    return wasmFused;
  }
  // Cormack, Clarke, Buettcher 2009: score(d) = Σ 1 / (k + rank_i(d)).
  const fused = new Map();
  for (const provider of perProviderResults) {
    const list = Array.isArray(provider.results) ? provider.results : [];
    list.forEach((item, index) => {
      if (!item || !item.url) return;
      const rank = index + 1;
      const score = 1 / (k + rank);
      const existing = fused.get(item.url);
      if (existing) {
        existing.score += score;
        existing.providers.push({ id: provider.id, rank });
        if (!existing.title && item.title) existing.title = item.title;
        if (!existing.excerpt && item.excerpt) existing.excerpt = item.excerpt;
      } else {
        fused.set(item.url, {
          url: item.url,
          title: item.title || item.url,
          excerpt: item.excerpt || "",
          score,
          providers: [{ id: provider.id, rank }],
        });
      }
    });
  }
  return Array.from(fused.values()).sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return b.providers.length - a.providers.length;
  });
}

async function tryWebSearch(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const query = extractWebSearchQuery(prompt, normalized);
  if (!query) return null;

  const rrfK = webSearchRrfK();
  const concurrency = webSearchConcurrency();
  const providerLimit = webSearchProviderLimit();

  // R194: the Rust core (`web_search_core::build_request_evidence`) is the
  // source of truth for the `web_search:*` evidence prefix. We prepend its
  // output and fall back to the inline list when the WASM worker booted in
  // `js fallback` mode.
  const evidence = [];
  const wasmEvidence = wasmWebSearchRequestEvidence(query, language || "");
  if (Array.isArray(wasmEvidence) && wasmEvidence.length > 0) {
    for (const line of wasmEvidence) {
      if (line) evidence.push(line);
    }
  } else {
    evidence.push(`web_search:request:${query}`);
    for (const provider of WEB_SEARCH_PROVIDERS) {
      evidence.push(`web_search:provider:${provider.id}`);
    }
    evidence.push(`web_search:combined:rrf:k=${rrfK}`);
  }

  // Session-disabled providers are session state, not part of the canonical
  // plan, so we annotate them on top of the WASM-derived prefix.
  const active = WEB_SEARCH_PROVIDERS.filter(
    (provider) => !webSearchIsDisabled(provider.id),
  );
  for (const provider of WEB_SEARCH_PROVIDERS) {
    if (webSearchIsDisabled(provider.id)) {
      evidence.push(`web_search:disabled:${provider.id}`);
    }
  }

  if (active.length === 0) {
    return {
      intent: "web_search",
      content: `All CORS-readable search providers are disabled for this session. Tried: ${WEB_SEARCH_PROVIDERS.map((p) => p.id).join(", ")}.`,
      confidence: 0.3,
      evidence,
    };
  }

  const tasks = active.map((provider) => async () => {
    const startedAt = Date.now();
    const outcome = await provider.run(query, language, providerLimit);
    return Object.assign({ id: provider.id, label: provider.label, elapsedMs: Date.now() - startedAt }, outcome);
  });
  const perProvider = await runWithConcurrencyLimit(tasks, concurrency);

  for (const provider of perProvider) {
    if (!provider.ok) {
      evidence.push(`web_search:provider:${provider.id}:error:${provider.error || "no_results"}`);
      continue;
    }
    evidence.push(`web_search:provider:${provider.id}:count:${provider.results.length}`);
    if (provider.language) {
      evidence.push(`web_search:provider:${provider.id}:language:${provider.language}`);
    }
    provider.results.forEach((item, index) => {
      evidence.push(`web_search:rank:${provider.id}:${index + 1}:${item.url}`);
    });
  }

  const fused = reciprocalRankFusion(perProvider, rrfK);
  const top = fused.slice(0, providerLimit);
  top.forEach((entry, index) => {
    evidence.push(`web_search:fused:${index + 1}:${entry.providers.map((p) => p.id).join("+")}:${entry.url}`);
  });

  if (top.length === 0) {
    return {
      intent: "web_search",
      content: `No CORS-enabled web search results were returned for \`${query}\`.\n\nProviders tried: ${active.map((p) => p.label).join(", ")}.`,
      confidence: 0.35,
      evidence,
    };
  }

  const lines = [
    `Search results for \`${query}\` — top ${top.length} after reciprocal rank fusion (k = ${rrfK}).`,
    "",
    `Providers (default first): ${active.map((p) => p.id).join(", ")}.`,
    "",
  ];
  top.forEach((entry, index) => {
    const sources = entry.providers
      .map((p) => `${p.id}#${p.rank}`)
      .join(", ");
    const excerpt = entry.excerpt ? ` - ${entry.excerpt}` : "";
    lines.push(`${index + 1}. [${entry.title}](${entry.url}) — _via ${sources}_${excerpt}`);
  });

  return {
    intent: "web_search",
    content: lines.join("\n"),
    confidence: 0.85,
    evidence,
  };
}

function cleanContextValue(value) {
  return String(value || "").replace(/\s+/g, " ").trim();
}

function evidenceFromUserContext(userContext) {
  if (!userContext || typeof userContext !== "object") return [];
  const evidence = [];
  const fields = [
    ["uiLanguage", "ui_language"],
    ["browserLanguage", "browser_language"],
    ["colorScheme", "color_scheme"],
    ["timeZone", "time_zone"],
    ["locationInference", "location_inference"],
  ];
  for (const [key, label] of fields) {
    const value = cleanContextValue(userContext[key]);
    if (value) evidence.push(`user_context:${label}:${value}`);
  }
  return evidence;
}

function attachUserContext(answer, userContext) {
  if (!answer || typeof answer !== "object") return answer;
  const evidence = evidenceFromUserContext(userContext);
  if (evidence.length === 0) return answer;
  const steps = Array.isArray(answer.steps) ? answer.steps.slice() : [];
  const detail = evidence
    .map((item) => item.replace(/^user_context:/, ""))
    .join(", ");
  steps.push({ step: "user_context", detail });
  return Object.assign({}, answer, {
    evidence: [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      ...evidence,
    ],
    steps,
  });
}

async function solve(prompt, history, prefs) {
  const preferences = prefs || {};
  const autoDefinitionFusion = definitionFusionByDefault(preferences);
  const steps = [];
  const toolCalls = [];
  const events = [`impulse:${prompt}`];
  steps.push({ step: "impulse", detail: prompt });
  const normalized = normalizePrompt(prompt);
  events.push(`formalization:${normalized || "(empty)"}`);
  steps.push({ step: "formalize", detail: normalized || "(empty)" });
  const language = detectLanguage(prompt);
  events.push(`language:${language}`);
  steps.push({ step: "detect_language", detail: language });

  if (isPunctuationOnlyPrompt(prompt)) {
    events.push("handler:clarification");
    events.push(`clarification:punctuation_only:${String(prompt).trim()}`);
    steps.push({ step: "dispatch_handler", detail: "tryPunctuationOnlyPrompt" });
    const trimmed = String(prompt).trim();
    return finalize(events, steps, toolCalls, {
      intent: "clarification",
      content: `I received only punctuation (\`${trimmed}\`). What would you like me to do next?`,
      confidence: 0.8,
      evidence: [
        "handler:clarification",
        "clarification:punctuation_only",
        `language:${language}`,
      ],
    });
  }

  const capabilities = tryCapabilities(prompt, normalized);
  if (capabilities) {
    events.push(`handler:${capabilities.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryCapabilities" });
    return finalize(events, steps, toolCalls, capabilities);
  }

  if (isGreetingPrompt(normalized, prompt)) {
    events.push("rule:greeting");
    steps.push({ step: "match_rule", detail: "greeting" });
    const temperature = numericPreference(preferences.temperature, 0.7, 0, 1);
    const randomize = preferences.greetingVariations !== false && temperature > 0;
    return finalize(events, steps, toolCalls, {
      intent: "greeting",
      content: answerFor("greeting", language, { randomize: randomize }),
      confidence: 1.0,
      evidence: [
        "rule:greeting",
        `language:${language}`,
        `variation:${randomize ? "random" : "canonical"}`,
        `temperature:${temperature.toFixed(2)}`,
      ],
    });
  }
  if (isFarewellPrompt(normalized, prompt)) {
    events.push("rule:farewell");
    steps.push({ step: "match_rule", detail: "farewell" });
    return finalize(events, steps, toolCalls, {
      intent: "farewell",
      content: answerFor("farewell", language),
      confidence: 1.0,
      evidence: ["rule:farewell", `language:${language}`],
    });
  }
  if (isCourtesyResponsePrompt(normalized, prompt)) {
    events.push("rule:courtesy_response");
    steps.push({ step: "match_rule", detail: "courtesy_response" });
    return finalize(events, steps, toolCalls, {
      intent: "courtesy_response",
      content: answerFor("courtesy_response", language),
      confidence: 1.0,
      evidence: ["rule:courtesy_response", `language:${language}`],
    });
  }
  if (isIdentityPrompt(normalized, prompt)) {
    events.push("rule:identity");
    steps.push({ step: "match_rule", detail: "identity" });
    return finalize(events, steps, toolCalls, {
      intent: "identity",
      content: answerFor("identity", language),
      confidence: 1.0,
      evidence: ["rule:identity", `language:${language}`],
    });
  }

  const syncHandlers = [
    { name: "tryHistorical", run: () => tryHistorical(prompt, history) },
    { name: "tryBrainstormingRequest", run: () => tryBrainstormingRequest(prompt, normalized) },
    { name: "tryRoleplayRequest", run: () => tryRoleplayRequest(prompt, normalized) },
    { name: "tryKupiSlona", run: () => tryKupiSlona(prompt, normalized) },
    { name: "tryArithmetic", run: () => tryArithmetic(prompt) },
    { name: "tryJavaScriptExecution", run: () => tryJavaScriptExecution(prompt) },
    {
      name: "tryDefinitionMerge",
      run: () => tryDefinitionMerge(prompt, { allowPlainConcept: autoDefinitionFusion }),
    },
    { name: "tryConceptLookup", run: () => tryConceptLookup(prompt) },
    { name: "tryHelloWorld", run: () => tryHelloWorld(prompt) },
    { name: "trySoftwareProjectRequest", run: () => trySoftwareProjectRequest(prompt, history) },
  ];
  for (const handler of syncHandlers) {
    const hit = handler.run();
    if (hit) {
      events.push(`handler:${hit.intent}`);
      steps.push({ step: "dispatch_handler", detail: handler.name });
      if (hit.intent === "javascript_execution" || hit.intent === "javascript_execution_error") {
        toolCalls.push({
          tool: "eval_js",
          inputs: { prompt },
          outputs: { intent: hit.intent, confidence: hit.confidence },
        });
      }
      if (
        hit.intent === "concept_lookup" ||
        hit.intent === "concept_lookup_in_context" ||
        hit.intent === "definition_merge"
      ) {
        toolCalls.push({
          tool: "concept_lookup",
          inputs: { prompt },
          outputs: { intent: hit.intent, confidence: hit.confidence },
        });
      }
      return finalize(events, steps, toolCalls, hit);
    }
  }

  // Real-time fact reasoning: parse structured (relation, subject) queries, hit
  // the 1-week TTL cache, fall back to Wikidata/Wikipedia for any country or
  // entity. Cache warmed from `data/seed/facts.lino` so the test matrix and
  // offline browsers still answer instantly. The legacy substring-based
  // `tryFactLookup` remains as a fallback for non-relation seed facts
  // (e.g. who painted the Mona Lisa) until those are migrated to relations.
  steps.push({ step: "invoke_tool", detail: "fact_query" });
  const factQuery = await tryFactQuery(prompt, normalized, preferences);
  if (factQuery) {
    events.push(`handler:${factQuery.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFactQuery" });
    if (Array.isArray(factQuery.trace)) {
      for (const event of factQuery.trace) events.push(event);
    }
    toolCalls.push({
      tool: "fact_query",
      inputs: { prompt, language },
      outputs: { intent: factQuery.intent, confidence: factQuery.confidence },
    });
    return finalize(events, steps, toolCalls, factQuery);
  }

  const legacyFact = tryFactLookup(prompt, normalized);
  if (legacyFact) {
    events.push(`handler:${legacyFact.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFactLookup" });
    return finalize(events, steps, toolCalls, legacyFact);
  }

  steps.push({ step: "invoke_tool", detail: "http_fetch" });
  const fetched = await tryFetch(prompt);
  if (fetched) {
    events.push(`handler:${fetched.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFetch" });
    toolCalls.push({
      tool: "http_fetch",
      inputs: { prompt },
      outputs: { intent: fetched.intent, confidence: fetched.confidence, iframeUrl: fetched.iframeUrl || null },
    });
    return finalize(events, steps, toolCalls, fetched);
  }

  steps.push({ step: "invoke_tool", detail: "url_navigate" });
  const navigated = await tryUrlNavigate(prompt);
  if (navigated) {
    events.push(`handler:${navigated.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryUrlNavigate" });
    toolCalls.push({
      tool: "url_navigate",
      inputs: { prompt },
      outputs: { intent: navigated.intent, confidence: navigated.confidence, iframeUrl: navigated.iframeUrl || null },
    });
    return finalize(events, steps, toolCalls, navigated);
  }

  steps.push({ step: "invoke_tool", detail: "web_search" });
  const webSearch = await tryWebSearch(prompt, language);
  if (webSearch) {
    events.push(`handler:${webSearch.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWebSearch" });
    toolCalls.push({
      tool: "web_search",
      inputs: { prompt, language },
      outputs: { intent: webSearch.intent, confidence: webSearch.confidence },
    });
    return finalize(events, steps, toolCalls, webSearch);
  }

  steps.push({ step: "invoke_tool", detail: "wikipedia_lookup" });
  const wiki = await tryWikipediaLookup(prompt, language, preferences);
  if (wiki) {
    events.push(`handler:${wiki.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWikipediaLookup" });
    toolCalls.push({
      tool: "wikipedia_lookup",
      inputs: {
        prompt,
        language,
        guessProbability: numericPreference(
          preferences.guessProbability,
          0.8,
          0,
          1,
        ),
      },
      outputs: { intent: wiki.intent, confidence: wiki.confidence },
    });
    return finalize(events, steps, toolCalls, wiki);
  }
  toolCalls.push({
    tool: "wikipedia_lookup",
    inputs: { prompt, language },
    outputs: { intent: "no_match" },
  });

  // Issue #69: "who is X" prompts that were not resolved by the local
  // knowledge base or Wikipedia should still return a question-typed response
  // (not "unknown") and offer a typo correction when the entity name is close
  // to a known variant.
  const whoIs = tryWhoIsQuestion(prompt);
  if (whoIs) {
    events.push(`handler:${whoIs.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWhoIsQuestion" });
    return finalize(events, steps, toolCalls, whoIs);
  }

  events.push("fallback:unknown");
  steps.push({ step: "fallback", detail: "unknown" });
  return finalize(events, steps, toolCalls, {
    intent: "unknown",
    content: answerFor("unknown", language),
    confidence: 0.1,
    evidence: ["fallback:unknown", `language:${language}`],
  });
}

function finalize(events, steps, toolCalls, answer) {
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const trace = events.map((event) => `trace:${event}`);
  const result = {
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: [...evidence, ...trace],
    steps,
    toolCalls,
  };
  if (answer.iframeUrl) {
    result.iframeUrl = answer.iframeUrl;
  }
  return result;
}

let seedLoaded = false;

async function loadSeed() {
  if (seedLoaded) return;
  seedLoaded = true;
  if (typeof self.FormalAiSeed !== "object" || self.FormalAiSeed === null) {
    return;
  }
  try {
    const seed = await self.FormalAiSeed.loadAll();
    SEED_RAW = (seed && seed.raw) || {};
    if (seed && seed.responses) {
      const merged = {};
      const intents = new Set(
        Object.keys(MULTILINGUAL_ANSWERS).concat(Object.keys(seed.responses)),
      );
      intents.forEach((intent) => {
        const base = MULTILINGUAL_ANSWERS[intent] || {};
        const next = seed.responses[intent] || {};
        const byLanguage = {};
        const langs = new Set(Object.keys(base).concat(Object.keys(next)));
        langs.forEach((language) => {
          const incoming = next[language];
          if (incoming !== undefined) {
            byLanguage[language] = normalizeEntry(incoming, intent);
          } else {
            byLanguage[language] = normalizeEntry(base[language], intent);
          }
        });
        merged[intent] = byLanguage;
      });
      MULTILINGUAL_ANSWERS = merged;
    }
    if (Array.isArray(seed && seed.concepts) && seed.concepts.length > 0) {
      CONCEPTS = seed.concepts;
    }
    if (
      Array.isArray(seed && seed.conceptContexts) &&
      seed.conceptContexts.length > 0
    ) {
      CONCEPT_CONTEXTS = seed.conceptContexts;
    }
    if (Array.isArray(seed && seed.facts) && seed.facts.length > 0) {
      FACTS = seed.facts;
      warmFactCacheFromSeed();
    }
    if (
      seed &&
      seed.brainstormSeeds &&
      Array.isArray(seed.brainstormSeeds.triggers) &&
      seed.brainstormSeeds.triggers.length > 0
    ) {
      BRAINSTORM_SEEDS = seed.brainstormSeeds;
    }
    if (
      seed &&
      seed.personas &&
      Array.isArray(seed.personas.triggers) &&
      seed.personas.triggers.length > 0
    ) {
      PERSONA_SEEDS = seed.personas;
    }
    if (Array.isArray(seed && seed.tools) && seed.tools.length > 0) {
      TOOLS = seed.tools;
    }
    if (seed && seed.agentInfo && typeof seed.agentInfo === "object") {
      AGENT_INFO = Object.assign({}, AGENT_INFO, seed.agentInfo);
    }
    if (Array.isArray(seed && seed.languageRules) && seed.languageRules.length > 0) {
      LANGUAGE_RULES = seed.languageRules
        .filter((rule) => rule && rule.language && rule.start && rule.end)
        .map((rule) => ({
          language: rule.language,
          start: Number(rule.start),
          end: Number(rule.end),
        }));
    }
    if (Array.isArray(seed && seed.promptPatterns) && seed.promptPatterns.length > 0) {
      PROMPT_PATTERNS = seed.promptPatterns;
    }
    if (
      seed &&
      seed.intentRouting &&
      Array.isArray(seed.intentRouting.intents) &&
      seed.intentRouting.intents.length > 0
    ) {
      INTENT_ROUTING = {
        intents: seed.intentRouting.intents,
        articlePrefixes:
          seed.intentRouting.articlePrefixes && seed.intentRouting.articlePrefixes.length
            ? seed.intentRouting.articlePrefixes
            : INTENT_ROUTING.articlePrefixes,
        tracePrefixes:
          seed.intentRouting.tracePrefixes && seed.intentRouting.tracePrefixes.length
            ? seed.intentRouting.tracePrefixes
            : INTENT_ROUTING.tracePrefixes,
      };
    }
  } catch (_error) {
    // Keep fallback tables on error.
  }
}

async function init() {
  if (wasm !== undefined) return;
  await loadSeed();
  try {
    const source = await fetch(withAssetVersion("formal_ai_worker.wasm"));
    const bytes = await source.arrayBuffer();
    const module = await WebAssembly.instantiate(bytes, {});
    wasm = module.instance.exports;
  } catch (_error) {
    wasm = null;
    mode = "js fallback";
  }
  postMessage({
    kind: "ready",
    mode,
    seed: {
      responseIntents: Object.keys(MULTILINGUAL_ANSWERS),
      conceptCount: CONCEPTS.length,
      conceptContextCount: CONCEPT_CONTEXTS.length,
      factCount: FACTS.length,
      brainstormCategoryCount: BRAINSTORM_SEEDS.categories.length,
      personaCount: PERSONA_SEEDS.personas.length,
      toolCount: TOOLS.length,
      files: Object.keys(SEED_RAW),
    },
  });
}

self.onmessage = async (event) => {
  await init();
  const data = event.data || {};
  if (data.kind === "seed_dump") {
    postMessage({
      kind: "seed_dump",
      requestId: data.requestId,
      raw: SEED_RAW,
      responses: MULTILINGUAL_ANSWERS,
      concepts: CONCEPTS,
      conceptContexts: CONCEPT_CONTEXTS,
      facts: FACTS,
      brainstormSeeds: BRAINSTORM_SEEDS,
      personas: PERSONA_SEEDS,
      tools: TOOLS,
      agentInfo: AGENT_INFO,
      languageRules: LANGUAGE_RULES,
      promptPatterns: PROMPT_PATTERNS,
    });
    return;
  }
  const prompt = data.prompt || "";
  const history = Array.isArray(data.history) ? data.history : [];
  const prefs = (data.prefs && typeof data.prefs === "object") ? data.prefs : {};
  const userContext =
    data.userContext && typeof data.userContext === "object"
      ? data.userContext
      : {};
  const answer = attachUserContext(
    await solve(prompt, history, prefs),
    userContext,
  );
  postMessage({
    kind: "message",
    requestId: data.requestId,
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: answer.evidence,
    steps: answer.steps,
    toolCalls: answer.toolCalls,
    iframeUrl: answer.iframeUrl || null,
  });
};

init();
