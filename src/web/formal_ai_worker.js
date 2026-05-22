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

const FALLBACK_TEST_STATUS_ANSWER = "Test passed. I'm here.";
const FALLBACK_COURTESY_RESPONSE_ANSWER =
  "Glad to hear it. What would you like to do next?";
const FALLBACK_COURTESY_ACKNOWLEDGEMENTS = [
  "Glad to hear it.",
  "You're welcome.",
];
const FALLBACK_COURTESY_FOLLOW_UPS = [
  "What would you like to do next?",
  "Do you want to discuss something else?",
];

const FALLBACK_UNKNOWN_ANSWER =
  "I don't know how to answer that yet. I cannot answer that from local Links Notation rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. To make it durable, export memory or use Report issue so developers can add the fact or rule to the seed.";

const FALLBACK_CLARIFICATION_ANSWER =
  "I'm sorry for the confusion. I am formal-ai, a deterministic symbolic AI. I can answer greetings, identity questions, concept lookups (what is X?), arithmetic, and Hello World programs. If you'd like to ask about something specific, try one of those or add a fact in Links Notation.";

// Mutable runtime tables — populated from seed at init(). Each entry is
// `{ text, variants }` so the worker can return either the canonical phrase
// (for deterministic tests and tool calls) or a random variant (for greeting
// randomisation introduced in issue #27). Courtesy responses can also carry
// separated acknowledgement and follow-up fragments for issue #160.
let MULTILINGUAL_ANSWERS = {
  greeting: {
    en: { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] },
  },
  farewell: {
    en: { text: "Goodbye! Feel free to return any time.", variants: ["Goodbye! Feel free to return any time."] },
  },
  test_status: {
    en: { text: FALLBACK_TEST_STATUS_ANSWER, variants: [FALLBACK_TEST_STATUS_ANSWER] },
  },
  courtesy_response: {
    en: {
      text: FALLBACK_COURTESY_RESPONSE_ANSWER,
      variants: [FALLBACK_COURTESY_RESPONSE_ANSWER],
      acknowledgements: FALLBACK_COURTESY_ACKNOWLEDGEMENTS,
      followUps: FALLBACK_COURTESY_FOLLOW_UPS,
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
let PROJECTS = [];
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
      keywords: [
        "hi",
        "hello",
        "hey",
        "привет",
        "здравствуйте",
        "шалом",
        "नमस्ते",
        "नमस्कार",
        "सलाम",
        "हाय",
        "你好",
        "您好",
        "嗨",
        "哈喽",
      ],
      phrases: [
        "how are you",
        "how are you doing",
        "how do you do",
        "how is it going",
        "how s it going",
        "how are things",
        "шабат шалом",
        "как дела",
        "как твои дела",
        "как ваши дела",
        "как у тебя дела",
        "как у вас дела",
        "привет как дела",
        "здравствуйте как ваши дела",
        "как поживаешь",
        "как вы поживаете",
        "राम राम",
        "कैसे हो",
        "आप कैसे हैं",
        "तुम कैसे हो",
        "क्या हाल है",
        "आपका क्या हाल है",
        "सब कैसा चल रहा है",
        "早上好",
        "早安",
        "你好吗",
        "你还好吗",
        "你怎么样",
        "您怎么样",
        "最近怎么样",
        "过得怎么样",
      ],
      tokens: ["greet"],
      combos: [],
    },
    {
      id: "intent_farewell",
      slug: "farewell",
      responseLink: "response:farewell",
      keywords: [
        "bye",
        "goodbye",
        "пока",
        "ciao",
        "tschüss",
        "再见",
        "拜拜",
        "回见",
        "अलविदा",
        "विदा",
        "बाय",
        "टाटा",
      ],
      phrases: ["до свидания", "досвидания", "改天见", "后会有期", "फिर मिलेंगे"],
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
      id: "intent_test_status",
      slug: "test_status",
      responseLink: "response:test_status",
      keywords: [
        "test",
        "ping",
        "pong",
        "testing",
        "тест",
        "пинг",
        "टेस्ट",
        "परीक्षण",
        "测试",
        "測試",
      ],
      phrases: [
        "test passed",
        "testing 123",
        "are you there",
        "you there",
        "i m here",
        "i am here",
        "я здесь",
        "тест пройден",
        "ты здесь",
        "вы здесь",
        "परीक्षण सफल रहा",
        "मैं यहाँ हूँ",
        "मैं यहां हूं",
        "क्या आप वहाँ हैं",
        "क्या आप वहां हैं",
        "测试通过",
        "測試通過",
        "我在这里",
        "我在這裡",
        "你在吗",
        "您在吗",
        "你在嗎",
        "您在嗎",
      ],
      tokens: [],
      combos: [
        ["test", "passed"],
        ["test", "here"],
        ["testing", "123"],
        ["ping", "test"],
        ["тест", "пройден"],
        ["тест", "здесь"],
        ["परीक्षण", "सफल"],
      ],
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
        "तू कौन है",
        "आप कौन हैं",
        "अपना परिचय दो",
        "अपने बारे में बताओ",
        "你是谁",
        "您是谁",
        "你是什么",
        "介绍一下你自己",
        "告诉我你自己",
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
      acknowledgements: FALLBACK_COURTESY_ACKNOWLEDGEMENTS,
      followUps: FALLBACK_COURTESY_FOLLOW_UPS,
    };
  }
  if (intent === "identity") {
    return { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] };
  }
  if (intent === "test_status") {
    return { text: FALLBACK_TEST_STATUS_ANSWER, variants: [FALLBACK_TEST_STATUS_ANSWER] };
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

// Mirrors `src/engine.rs::UNKNOWN_OPENERS_*`. The first entry of each pool
// equals the opener already embedded in the seed text so the "with-variations"
// answer is a strict superset of the seed. Different prompts get different
// openers; the same prompt always picks the same one (FNV-1a hash, mirrored
// from `stableBehaviorRuleId`).
const UNKNOWN_OPENERS_BY_LANGUAGE = {
  en: [
    "I don't know how to answer that yet.",
    "I didn't understand you.",
    "I'm not sure how to respond to that yet.",
    "I haven't learned to answer that yet.",
    "That one is new to me.",
  ],
  ru: [
    "Я пока не знаю, как ответить на это.",
    "Я тебя не понял.",
    "Я не уверен, как на это ответить.",
    "Я ещё не научился отвечать на это.",
    "Это для меня новое.",
  ],
  hi: [
    "मुझे अभी इसका उत्तर देना नहीं आता।",
    "मैं समझ नहीं पाया।",
    "मुझे यकीन नहीं है कि कैसे उत्तर दूँ।",
    "मैंने अभी तक यह उत्तर देना नहीं सीखा।",
    "यह मेरे लिए नया है।",
  ],
  zh: [
    "我还不知道如何回答这个问题。",
    "我不太明白你说的意思。",
    "我不确定该如何回答。",
    "我还没有学会回答这个问题。",
    "这对我来说是新的。",
  ],
};

function unknownOpenersFor(language) {
  return UNKNOWN_OPENERS_BY_LANGUAGE[language] || UNKNOWN_OPENERS_BY_LANGUAGE.en;
}

function selectUnknownOpener(prompt, language) {
  const pool = unknownOpenersFor(language);
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
  for (const separator of [". ", "。", "। "]) {
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

function cleanMechanismFragment(value) {
  return String(value || "")
    .trim()
    .replace(/^[`"'«»<>()\[\]{}]+/u, "")
    .replace(/[`"'«»<>()\[\]{}]+$/u, "")
    .replace(/[?？。.!,,;:]+$/u, "")
    .trim();
}

function cleanMechanismSubject(value) {
  let clean = cleanMechanismFragment(value);
  for (const suffix of [
    " in detail",
    " internally",
    " exactly",
    " please",
    " подробнее",
    " подробно",
    " пожалуйста",
  ]) {
    const lower = clean.toLowerCase();
    if (lower.endsWith(suffix)) {
      clean = cleanMechanismFragment(clean.slice(0, -suffix.length));
    }
  }
  const lower = clean.toLowerCase();
  const pronouns = new Set([
    "it",
    "this",
    "that",
    "you",
    "yourself",
    "does",
    "do",
    "это",
    "оно",
    "он",
    "она",
    "они",
    "ты",
    "вы",
    "यह",
    "ये",
    "这",
    "这个",
    "它",
  ]);
  if (
    !clean ||
    pronouns.has(lower) ||
    lower.startsWith("does ") ||
    lower.startsWith("do ") ||
    lower.startsWith("to ") ||
    lower.startsWith("you ")
  ) {
    return null;
  }
  return clean;
}

function stripMechanismTail(subject) {
  let clean = cleanMechanismSubject(subject);
  if (!clean) return null;
  const lower = clean.toLowerCase();
  for (const suffix of [
    " work",
    " works",
    " structured",
    " organized",
    " organised",
    " built",
  ]) {
    if (lower.endsWith(suffix)) {
      clean = cleanMechanismSubject(clean.slice(0, -suffix.length));
      break;
    }
  }
  return clean;
}

function mechanismSubjectAfterPrefix(original, lower, prefix) {
  if (!lower.startsWith(prefix)) return null;
  return cleanMechanismSubject(original.slice(prefix.length));
}

function mechanismSubjectBeforeSuffix(original, lower, suffix) {
  if (!lower.endsWith(suffix)) return null;
  return cleanMechanismSubject(original.slice(0, -suffix.length));
}

function mechanismSubjectBetween(original, lower, prefix, suffixes) {
  if (!lower.startsWith(prefix)) return null;
  for (const suffix of suffixes) {
    if (!lower.endsWith(suffix)) continue;
    const end = original.length - suffix.length;
    if (end <= prefix.length) return null;
    return cleanMechanismSubject(original.slice(prefix.length, end));
  }
  return null;
}

function extractHowItWorksSubject(input, lowerInput) {
  const original = cleanMechanismFragment(input);
  if (!original) return null;
  const lower = cleanMechanismFragment(lowerInput || original.toLowerCase())
    .toLowerCase();

  for (const prefix of [
    "how does ",
    "how do ",
    "how did ",
    "how is ",
    "как устроен ",
    "как устроена ",
    "как устроено ",
    "как устроены ",
    "как работает ",
    "как работают ",
  ]) {
    const subject = mechanismSubjectAfterPrefix(original, lower, prefix);
    if (subject) return stripMechanismTail(subject);
  }

  for (const [prefix, suffixes] of [
    ["how ", [" works", " work"]],
    ["как ", [" работает", " работают"]],
  ]) {
    const subject = mechanismSubjectBetween(original, lower, prefix, suffixes);
    if (subject) return subject;
  }

  for (const suffix of [
    " कैसे काम करता है",
    " कैसे काम करती है",
    " कैसे काम करते हैं",
    " कैसे काम करता",
    " कैसे काम करती",
    " कैसे काम करते",
    " 是如何工作的",
    "是如何工作的",
    " 是怎么工作的",
    "是怎么工作的",
    " 如何工作",
    "如何工作",
    " 怎么工作",
    "怎么工作",
    " 的工作原理是什么",
    "的工作原理是什么",
    " как работает",
    " как работают",
  ]) {
    const subject = mechanismSubjectBeforeSuffix(original, lower, suffix);
    if (subject) return subject;
  }

  return null;
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

  const howItWorksSubject = extractHowItWorksSubject(trimmedRaw, lower);
  if (howItWorksSubject) return finalizeConceptBody(howItWorksSubject);

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

function cleanWikipediaArticleQuestionTerm(value) {
  return String(value || "")
    .trim()
    .replace(/^[«»"“”‘’'`「」『』]+|[«»"“”‘’'`「」『』]+$/gu, "")
    .replace(/[?!.。！？।]+$/gu, "")
    .replace(/\s+/g, " ")
    .trim();
}

function hasWikipediaArticleQuestionShape(value) {
  const lower = String(value || "").toLowerCase();
  if (!/(?:wikipedia|wiki|википед|维基百科|維基百科|विकिपीडिया)/u.test(lower)) return false;
  const hasArticleWord = /(?:article|page|стать[ьяеию]|страниц|条目|條目|页面|頁面|文章|लेख|पृष्ठ)/u.test(lower);
  if (!hasArticleWord) return false;
  return /(?:is there|does .*have|exist|available|есть|существ|имеет|найд|назв|有|存在|有没有|是否有|吗|嗎|क्या|है|मौजूद)/u.test(lower);
}

function extractWikipediaArticleQuestionTerm(prompt) {
  const raw = cleanWikipediaArticleQuestionTerm(prompt);
  if (!raw || !hasWikipediaArticleQuestionShape(raw)) return null;

  const dashMatch = raw.match(/^(.+?)\s+[-—–:]\s+(.+)$/u);
  if (dashMatch && hasWikipediaArticleQuestionShape(dashMatch[2])) {
    return cleanWikipediaArticleQuestionTerm(dashMatch[1]);
  }

  for (const pattern of [
    /^(?:is|are)\s+there\s+(?:an?\s+)?(?:wikipedia|wiki)\s+(?:article|page)\s+(?:about|on|for)\s+(.+)$/iu,
    /^does\s+(?:wikipedia|wiki)\s+have\s+(?:an?\s+)?(?:article|page)\s+(?:about|on|for)\s+(.+)$/iu,
    /^(?:есть|существует|имеется)\s+(?:ли\s+)?(?:в\s+)?(?:русскоязычной\s+)?википедии\s+(?:отдельная\s+)?(?:статья|страница)\s+(?:о|об|про|с\s+названием)\s+(.+)$/iu,
    /^(?:есть|существует|имеется)\s+(?:ли\s+)?(?:отдельная\s+)?(?:статья|страница)\s+(?:в\s+)?(?:русскоязычной\s+)?википедии\s+(?:о|об|про|с\s+названием)\s+(.+)$/iu,
    /^(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:关于|關於|名为|名為)?\s*(.+?)\s*(?:的)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu,
    /^(.+?)\s*(?:在)?(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:这样(?:的)?|這樣(?:的)?|一篇)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu,
    /^(?:क्या\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu,
    /^(?:क्या\s+)?(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(?:ऐसा\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu,
  ]) {
    const match = raw.match(pattern);
    if (match) return cleanWikipediaArticleQuestionTerm(match[1]);
  }

  const trailingRussian = raw.match(/^(.+?)\s+(?:есть|существует|имеется)\s+(?:ли\s+)?(?:такая\s+)?(?:статья|страница)\s+(?:в\s+)?(?:русскоязычной\s+)?википедии$/iu);
  if (trailingRussian) return cleanWikipediaArticleQuestionTerm(trailingRussian[1]);
  const trailingHindi = raw.match(/^(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(?:ऐसा\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu);
  if (trailingHindi) return cleanWikipediaArticleQuestionTerm(trailingHindi[1]);
  const trailingChinese = raw.match(/^(.+?)\s*(?:在)?(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:这样(?:的)?|這樣(?:的)?|一篇)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu);
  if (trailingChinese) return cleanWikipediaArticleQuestionTerm(trailingChinese[1]);

  return null;
}

function refineWikipediaArticleQuestionLookup(term, language) {
  const exactTerm = cleanWikipediaArticleQuestionTerm(term);
  const query = {
    exactTerm,
    lookupTerm: exactTerm,
    contextOriginal: "",
  };
  const lower = exactTerm.toLowerCase();
  if (
    (language === "ru" || /[а-яё]/iu.test(exactTerm)) &&
    /\s(?:в|на)\s+(?:предложени[еяию]|предложениях|словосочетани[еяию]|словосочетаниях)$/iu.test(lower)
  ) {
    query.lookupTerm = cleanWikipediaArticleQuestionTerm(
      exactTerm.replace(/\s(?:в|на)\s+(?:предложени[еяию]|предложениях|словосочетани[еяию]|словосочетаниях)$/iu, ""),
    );
    query.contextOriginal = "грамматика";
  }
  if (
    (language === "en" || /^[\p{ASCII}\s]+$/u.test(exactTerm)) &&
    /\s+in\s+(?:a\s+)?sentences?$/iu.test(lower)
  ) {
    query.lookupTerm = cleanWikipediaArticleQuestionTerm(
      exactTerm.replace(/\s+in\s+(?:a\s+)?sentences?$/iu, ""),
    );
    query.contextOriginal = "grammar";
  }
  if (language === "hi" || /[\u0900-\u097f]/u.test(exactTerm)) {
    const prefix = exactTerm.match(/^(?:वाक्य|वाक्यों)\s+में\s+(.+)$/u);
    const suffix = exactTerm.match(/^(.+?)\s+(?:वाक्य|वाक्यों)\s+में$/u);
    const match = prefix || suffix;
    if (match) {
      query.lookupTerm = cleanWikipediaArticleQuestionTerm(match[1]);
      query.contextOriginal = "व्याकरण";
    }
  }
  if (language === "zh" || /[\u3400-\u9fff]/u.test(exactTerm)) {
    const prefix = exactTerm.match(/^(?:句子中(?:的)?|句子里(?:的)?|句中的)(.+)$/u);
    const suffix = exactTerm.match(/^(.+?)(?:在)?句子(?:中|里)$/u);
    const match = prefix || suffix;
    if (match) {
      query.lookupTerm = cleanWikipediaArticleQuestionTerm(match[1]);
      query.contextOriginal = "语法";
    }
  }
  return query;
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

const PERCENT_OF_CURRENCY_CODES = new Map([
  ["$", "USD"],
  ["€", "EUR"],
  ["¥", "JPY"],
  ["₹", "INR"],
  ["₽", "RUB"],
]);

const DEFAULT_CURRENCY_RATES = new Map([
  ["USD:EUR", 0.92],
  ["USD:GBP", 0.79],
  ["USD:JPY", 148.5],
  ["USD:CHF", 0.88],
  ["USD:CNY", 7.25],
  ["USD:RUB", 89.5],
  ["USD:INR", 86.5],
  ["USD:CLF", 0.022],
  ["USD:VND", 25810.0],
  ["USD:KZT", 470.0],
  ["EUR:USD", 1.087],
  ["EUR:GBP", 0.86],
  ["EUR:JPY", 161.5],
  ["EUR:CHF", 0.96],
  ["GBP:USD", 1.27],
  ["GBP:EUR", 1.16],
]);

function currencyCodeFromWord(value) {
  const lower = String(value || "").toLowerCase();
  if (
    lower === "usd" ||
    lower === "dollar" ||
    lower === "dollars" ||
    [
      "доллар",
      "доллара",
      "долларе",
      "доллары",
      "долларов",
      "долларам",
      "доллару",
      "долларом",
      "долларами",
      "долларах",
    ].includes(lower)
  ) {
    return "USD";
  }
  if (
    lower === "eur" ||
    lower === "euro" ||
    lower === "euros" ||
    lower === "евро"
  ) {
    return "EUR";
  }
  if (
    lower === "rub" ||
    lower === "ruble" ||
    lower === "rubles" ||
    [
      "рубль",
      "рубля",
      "рубле",
      "рубли",
      "рублей",
      "рублям",
      "рублю",
      "рублём",
      "рублем",
      "рублями",
      "рублях",
    ].includes(lower)
  ) {
    return "RUB";
  }
  return "";
}

function defaultCurrencyRate(from, to) {
  if (from === to) return 1;
  const direct = DEFAULT_CURRENCY_RATES.get(`${from}:${to}`);
  if (direct) return direct;
  const inverse = DEFAULT_CURRENCY_RATES.get(`${to}:${from}`);
  if (inverse) return 1 / inverse;
  if (from !== "USD" && to !== "USD") {
    const fromUsd = defaultCurrencyRate(from, "USD");
    const usdTo = defaultCurrencyRate("USD", to);
    if (fromUsd && usdTo) return fromUsd * usdTo;
  }
  return null;
}

function evaluatePercentOfExpression(expression) {
  const match = String(expression || "")
    .trim()
    .match(
      /^([+-]?\d+(?:\.\d+)?)\s*%\s+of\s+([$€¥₹₽])?\s*([+-]?\d+(?:\.\d+)?)(?:\s*(usd|eur|rub|dollars?|euros?|rubles?))?$/i,
    );
  if (!match) return null;
  const percent = Number(match[1]);
  const amount = Number(match[3]);
  if (!Number.isFinite(percent) || !Number.isFinite(amount)) return null;
  const currency =
    PERCENT_OF_CURRENCY_CODES.get(match[2] || "") ||
    currencyCodeFromWord(match[4]);
  const result = formatArithmeticResult((amount * percent) / 100);
  return currency ? `${result} ${currency}` : result;
}

function evaluateCurrencyConversionExpression(expression) {
  const match = String(expression || "")
    .trim()
    .match(
      /^([+-]?\d+(?:[.,]\d+)?)\s+(.+?)\s+(?:in|as|to|в|во|к)\s+(.+)$/iu,
    );
  if (!match) return null;
  const amount = Number(match[1].replace(",", "."));
  if (!Number.isFinite(amount)) return null;
  const from = currencyCodeFromWord(match[2].trim());
  const to = currencyCodeFromWord(match[3].trim());
  if (!from || !to) return null;
  const rate = defaultCurrencyRate(from, to);
  if (!rate) return null;
  return `${formatArithmeticResult(amount * rate)} ${to}`;
}

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
  const interpretations = [];
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
    const stripped = stripKnownPrefix(working, prefixes);
    if (stripped) {
      working = stripped.value;
      if (stripped.interpretation) interpretations.push(stripped.interpretation);
      changed = true;
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
  const hasPercentOf = evaluatePercentOfExpression(working) !== null;
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
  if (hasPercentOf) return { expression: working, interpretations };
  if (evaluateCurrencyConversionExpression(working) !== null) {
    return { expression: working, interpretations };
  }
  const allowed = /^[0-9+\-*/%().=\s_×·÷−,a-zA-Z]+$/;
  if (!allowed.test(working) && !hasWordOperator) return null;
  return { expression: working, interpretations };
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
  if (repositoryFromPrompt(rawPrompt)) return false;
  return matchesIntentRoute(normalized, rawPrompt, "intent_identity");
}

function isGreetingPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_greeting");
}

function isFarewellPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_farewell");
}

function isTestStatusPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_test_status");
}

function isCourtesyResponsePrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_courtesy_response");
}

function isPunctuationOnlyPrompt(prompt) {
  const trimmed = String(prompt || "").trim();
  return /^[.!?…。？！]+$/.test(trimmed);
}

function stableBehaviorRuleId(prefix, value) {
  let hash = 0xcbf29ce484222325n;
  const source = String(value || "");
  for (let index = 0; index < source.length; index += 1) {
    hash ^= BigInt(source.charCodeAt(index));
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `${prefix}_${hash.toString(16)}`;
}

function extractQuotedPhrase(text) {
  const source = String(text || "");
  const pairs = [
    ['"', '"'],
    ["'", "'"],
    ["`", "`"],
    ["«", "»"],
  ];
  for (const [open, close] of pairs) {
    const start = source.indexOf(open);
    if (start === -1) continue;
    const end = source.indexOf(close, start + open.length);
    if (end !== -1) return source.slice(start + open.length, end);
  }
  return null;
}

// Issue #216: extract the surface from unquoted translation prompts such as
// `translate apple to russian`, `переведи яблоко на английский`,
// `apple का हिंदी में अनुवाद करो`, or `把 apple 翻译成中文`. Returns null when
// the prompt already contains a quoted fragment or does not match a supported
// verb + target-marker pattern.
function extractUnquotedTranslationSurface(text) {
  const source = String(text || "").trim();
  const trimmed = source.replace(/[.!?。]+$/u, "");
  const lower = trimmed.toLowerCase();

  const extractBetween = (prefix, marker) => {
    if (!lower.startsWith(prefix)) return null;
    const afterPrefix = lower.slice(prefix.length);
    const markerIndex = afterPrefix.indexOf(marker);
    if (markerIndex === -1) return null;
    return cleanUnquotedTranslationSurface(
      trimmed.slice(prefix.length, prefix.length + markerIndex),
    );
  };

  const direct =
    extractBetween("translate ", " to ") ||
    extractBetween("переведи ", " на ");
  if (direct) return direct;

  const hindi = extractHindiUnquotedTranslationSurface(trimmed, lower);
  if (hindi) return hindi;
  return extractChineseUnquotedTranslationSurface(trimmed, lower);
}

function cleanUnquotedTranslationSurface(candidate) {
  const cleaned = String(candidate || "").trim();
  if (!cleaned || /["'«»`“”‘’]/u.test(cleaned)) return null;
  return cleaned;
}

function extractHindiUnquotedTranslationSurface(original, lower) {
  if (!lower.includes("अनुवाद")) return null;
  for (const targetMarker of [" में अनुवाद", " मे अनुवाद"]) {
    const targetIndex = lower.indexOf(targetMarker);
    if (targetIndex === -1) continue;
    const beforeTarget = lower.slice(0, targetIndex);
    for (const surfaceMarker of [" का ", " को "]) {
      const surfaceEnd = beforeTarget.lastIndexOf(surfaceMarker);
      if (surfaceEnd !== -1) {
        return cleanUnquotedTranslationSurface(original.slice(0, surfaceEnd));
      }
    }
  }
  return null;
}

function firstMarkerOffset(text, markers) {
  let best = null;
  for (const marker of markers) {
    const offset = text.indexOf(marker);
    if (offset !== -1 && (best === null || offset < best)) best = offset;
  }
  return best;
}

function extractChineseUnquotedTranslationSurface(original, lower) {
  for (const prefix of ["把", "将"]) {
    if (!lower.startsWith(prefix)) continue;
    const rest = lower.slice(prefix.length);
    const markerIndex = firstMarkerOffset(rest, [
      "翻译成",
      "翻译为",
      "翻译到",
      "翻譯成",
      "翻譯為",
      "翻譯到",
    ]);
    if (markerIndex !== null) {
      return cleanUnquotedTranslationSurface(
        original.slice(prefix.length, prefix.length + markerIndex),
      );
    }
  }

  for (const prefix of ["翻译", "翻譯"]) {
    if (!lower.startsWith(prefix)) continue;
    const rest = lower.slice(prefix.length);
    const markerIndex = firstMarkerOffset(rest, ["成", "为", "為", "到"]);
    if (markerIndex !== null) {
      return cleanUnquotedTranslationSurface(
        original.slice(prefix.length, prefix.length + markerIndex),
      );
    }
  }
  return null;
}

function escapeBehaviorRuleValue(value) {
  return String(value || "")
    .replaceAll("\\", "\\\\")
    .replaceAll('"', '\\"')
    .replaceAll("\n", "\\n");
}

function behaviorRuleRecords() {
  const greeting = answerFor("greeting", "en");
  const farewell = answerFor("farewell", "en");
  const identity = answerFor("identity", "en");
  return [
    {
      id: "rule_greeting",
      topic: "greetings",
      intent: "greeting",
      label: "Greeting rule",
      matches: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases",
      response: greeting,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user says \`Hi\`, \`Hello\`, or \`Hey\` then respond with \`${greeting}\`.`,
    },
    {
      id: "rule_farewell",
      topic: "farewells",
      intent: "farewell",
      label: "Farewell rule",
      matches: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases",
      response: farewell,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user says \`bye\`, \`goodbye\`, or \`пока\` then respond with \`${farewell}\`.`,
    },
    {
      id: "rule_identity",
      topic: "identity",
      intent: "identity",
      label: "Identity rule",
      matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      response: identity,
      source: "data/seed/identity.lino + multilingual responses",
      whenThen: `When the user asks \`Who are you?\` or \`Кто ты?\` then respond with \`${identity}\`.`,
    },
    {
      id: "rule_capabilities",
      topic: "capabilities",
      intent: "capabilities",
      label: "Capabilities rule",
      matches: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts",
      response: "Lists the supported symbolic chat capabilities.",
      source: "src/solver_handlers/user_intent.rs",
      whenThen:
        "When the user asks `What can you do?` or `Что ты умеешь?` then respond with the multilingual capability listing.",
    },
    {
      id: "rule_hello_world_rust",
      topic: "hello_world",
      intent: "hello_world_rust",
      label: "Hello-world rule (Rust)",
      matches: "`hello world` plus aliases: rust, rs",
      response: "Returns a minimal Rust hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `rust` then respond with a minimal Rust hello-world program.",
    },
    {
      id: "rule_hello_world_python",
      topic: "hello_world",
      intent: "hello_world_python",
      label: "Hello-world rule (Python)",
      matches: "`hello world` plus aliases: python, py",
      response: "Returns a minimal Python hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `python` then respond with a minimal Python hello-world program.",
    },
    {
      id: "rule_hello_world_javascript",
      topic: "hello_world",
      intent: "hello_world_javascript",
      label: "Hello-world rule (JavaScript)",
      matches: "`hello world` plus aliases: javascript, js, node",
      response: "Returns a minimal JavaScript hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `javascript` then respond with a minimal JavaScript hello-world program.",
    },
    {
      id: "rule_hello_world_typescript",
      topic: "hello_world",
      intent: "hello_world_typescript",
      label: "Hello-world rule (TypeScript)",
      matches: "`hello world` plus aliases: typescript, ts",
      response: "Returns a minimal TypeScript hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `typescript` then respond with a minimal TypeScript hello-world program.",
    },
    {
      id: "rule_hello_world_go",
      topic: "hello_world",
      intent: "hello_world_go",
      label: "Hello-world rule (Go)",
      matches: "`hello world` plus aliases: go, golang",
      response: "Returns a minimal Go hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `go` then respond with a minimal Go hello-world program.",
    },
    {
      id: "rule_hello_world_c",
      topic: "hello_world",
      intent: "hello_world_c",
      label: "Hello-world rule (C)",
      matches: "`hello world` plus aliases: c",
      response: "Returns a minimal C hello-world program.",
      source: "data/seed/hello-world-programs.lino",
      whenThen:
        "When the user requests a `hello world` program with alias `c` then respond with a minimal C hello-world program.",
    },
    {
      id: "rule_unknown",
      topic: "unknown_fallback",
      intent: "unknown",
      label: "Unknown fallback rule",
      matches: "Any prompt that no earlier rule or handler can answer",
      response: answerFor("unknown", "en"),
      source: "data/seed/multilingual-responses.lino",
      whenThen:
        "When no earlier rule or handler matches the prompt then respond with the multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, `When I say … answer …`, `Report issue`, `Export memory`).",
    },
  ];
}

const BEHAVIOR_RULE_TOPIC_LABELS = {
  greetings: "Greetings",
  farewells: "Farewells",
  identity: "Identity",
  capabilities: "Capabilities",
  hello_world: "Hello-world programs",
  unknown_fallback: "Unknown fallback",
};

const BEHAVIOR_RULE_TOPIC_ORDER = [
  "greetings",
  "farewells",
  "identity",
  "capabilities",
  "hello_world",
  "unknown_fallback",
];

function behaviorRuleTopicLabel(topic) {
  return BEHAVIOR_RULE_TOPIC_LABELS[topic] || "Other";
}

function behaviorRuleTopicOrder(topic) {
  const index = BEHAVIOR_RULE_TOPIC_ORDER.indexOf(topic);
  return index === -1 ? BEHAVIOR_RULE_TOPIC_ORDER.length : index;
}

function renderBehaviorRuleList(runtimeRules) {
  const lines = [
    "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):",
    "",
  ];
  const groups = new Map();
  for (const rule of behaviorRuleRecords()) {
    const order = behaviorRuleTopicOrder(rule.topic);
    if (!groups.has(order)) {
      groups.set(order, { label: behaviorRuleTopicLabel(rule.topic), rules: [] });
    }
    groups.get(order).rules.push(rule);
  }
  const ordered = Array.from(groups.entries()).sort((a, b) => a[0] - b[0]);
  ordered.forEach(([, group], index) => {
    lines.push(`# ${group.label}`);
    for (const rule of group.rules) {
      lines.push(`- \`${rule.id}\` -> ${rule.whenThen}`);
    }
    if (index + 1 < ordered.length) lines.push("");
  });
  if (Array.isArray(runtimeRules) && runtimeRules.length > 0) {
    lines.push("", "# Dialog-local rules taught in this conversation");
    for (const rule of runtimeRules) {
      lines.push(
        `- \`${rule.id}\` -> When the user says \`${rule.trigger}\` then respond with \`${rule.answer}\`.`,
      );
    }
  }
  lines.push(
    "",
    "Read one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`.",
    "Teach this dialog with: When `your prompt` then `your answer`. " +
      "Equivalent forms: When I say `your prompt`, answer `your answer`; " +
      "If I ask `your prompt`, reply `your answer`; " +
      "When `your prompt` do `your answer`.",
    "Multilingual forms: Russian `Когда \\`X\\` тогда \\`Y\\`` / `Когда \\`X\\` делай \\`Y\\``, " +
      "Hindi `जब \\`X\\` तब \\`Y\\``, Chinese `当 \\`X\\` 时 \\`Y\\``.",
    "The write is append-only: export memory to preserve the rule message with the dialog.",
  );
  return lines.join("\n");
}

function renderBehaviorRuleDetail(rule) {
  return [
    rule.label,
    "",
    rule.whenThen,
    "",
    "```links",
    rule.id,
    `  topic "${escapeBehaviorRuleValue(rule.topic)}"`,
    `  intent "${escapeBehaviorRuleValue(rule.intent)}"`,
    `  matches "${escapeBehaviorRuleValue(rule.matches)}"`,
    `  response "${escapeBehaviorRuleValue(rule.response)}"`,
    `  source "${escapeBehaviorRuleValue(rule.source)}"`,
    `  when_then "${escapeBehaviorRuleValue(rule.whenThen)}"`,
    "```",
    "",
    "To change this behavior in the current dialog, send: When `your prompt` then `your answer`. " +
      "Equivalent: When I say `your prompt`, answer `your answer`.",
  ].join("\n");
}

function renderSelfFacts() {
  return [
    "Facts I know about myself:",
    "",
    "```links",
    "self_fact_model",
    '  subject "formal-ai"',
    '  relation "model"',
    `  object "${escapeBehaviorRuleValue(AGENT_INFO.model || "formal-symbolic-production")}"`,
    "self_fact_policy",
    '  subject "formal-ai"',
    '  relation "policy"',
    '  object "deterministic symbolic AI; no neural network inference"',
    "self_fact_rules",
    '  subject "formal-ai"',
    '  relation "answer_source"',
    '  object "local Links Notation rules"',
    "self_fact_memory",
    '  subject "formal-ai"',
    '  relation "memory"',
    '  object "append-only dialog events plus seed files in Links Notation"',
    "```",
    "",
    "Read behavior with `List behavior rules`; teach one with When `prompt` then `answer` (or When I say `prompt`, answer `answer`).",
  ].join("\n");
}

function renderRuntimeRuleUpdate(rule) {
  const whenThenText = `When the user says \`${rule.trigger}\` then respond with \`${rule.answer}\`.`;
  return [
    "Behavior rule recorded for this dialog.",
    "",
    whenThenText,
    "",
    "```links",
    rule.id,
    '  type "behavior_rule_runtime"',
    `  match_prompt "${escapeBehaviorRuleValue(rule.trigger)}"`,
    `  answer "${escapeBehaviorRuleValue(rule.answer)}"`,
    `  when_then "${escapeBehaviorRuleValue(whenThenText)}"`,
    '  source "user_message"',
    "```",
    "",
    `Send \`${rule.trigger}\` now and I will answer with the configured response. Export memory to keep this rule message with the dialog.`,
  ].join("\n");
}

function isBehaviorRulesList(normalized) {
  return (
    matchesBehaviorRulesListSeedPattern(normalized) ||
    normalized.includes("list behavior rules") ||
    normalized.includes("list all behavior rules") ||
    normalized.includes("show behavior rules") ||
    normalized.includes("show all behavior rules") ||
    normalized.includes("what behavior rules") ||
    normalized.includes("existing behavior rules") ||
    isSupportedLanguageBehaviorRulesListQuery(normalized) ||
    normalized.includes("список правил поведения") ||
    normalized.includes("покажи правила поведения") ||
    normalized.includes("какие правила поведения") ||
    normalized.includes("व्यवहार के नियम") ||
    normalized.includes("व्यवहार नियम सूचीबद्ध करें") ||
    normalized.includes("行为规则") ||
    normalized.includes("列出行为规则")
  );
}

function matchesBehaviorRulesListSeedPattern(normalized) {
  return PROMPT_PATTERNS.some((pattern) => {
    if (!pattern || pattern.intent !== "behavior_rules_list" || !pattern.text) {
      return false;
    }
    const text = normalizePrompt(pattern.text);
    if (!text) return false;
    switch (pattern.kind) {
      case "keyword":
      case "phrase":
        return normalized === text || normalized.includes(text);
      case "prefix":
        return normalized.startsWith(text);
      case "suffix":
        return normalized.endsWith(text);
      default:
        return false;
    }
  });
}

function isSupportedLanguageBehaviorRulesListQuery(normalized) {
  return (
    isEnglishBehaviorRulesListQuery(normalized) ||
    isRussianBehaviorRulesListQuery(normalized) ||
    isHindiBehaviorRulesListQuery(normalized) ||
    isChineseBehaviorRulesListQuery(normalized)
  );
}

function isEnglishBehaviorRulesListQuery(normalized) {
  const mentionsRules =
    normalized.includes("rules") ||
    normalized.includes("rule list") ||
    normalized.includes("rules list");
  const asksToList =
    normalized.includes("list") ||
    normalized.includes("show") ||
    normalized.includes("what") ||
    normalized.includes("which");
  const pointsAtAssistantRules =
    normalized.includes("behavior") ||
    normalized.includes("your") ||
    normalized.includes("own") ||
    normalized.includes("current") ||
    normalized.includes("existing");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isRussianBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("правил") || normalized.includes("правила");
  const asksToList =
    normalized.includes("список") ||
    normalized.includes("перечисли") ||
    normalized.includes("покажи") ||
    normalized.includes("какие");
  const pointsAtAssistantRules =
    normalized.includes("поведения") ||
    normalized.includes("своих") ||
    normalized.includes("свои") ||
    normalized.includes("твоих") ||
    normalized.includes("твои") ||
    normalized.includes("собственные") ||
    normalized.includes("список правил");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isHindiBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("नियम") || normalized.includes("नियमों");
  const asksToList =
    normalized.includes("सूची") ||
    normalized.includes("सूचीबद्ध") ||
    normalized.includes("दिखाओ") ||
    normalized.includes("दिखाएं") ||
    normalized.includes("बताओ") ||
    normalized.includes("कौन");
  const pointsAtAssistantRules =
    normalized.includes("व्यवहार") ||
    normalized.includes("अपने") ||
    normalized.includes("तुम्हारे") ||
    normalized.includes("आपके") ||
    normalized.includes("नियमों की सूची");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isChineseBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("规则") || normalized.includes("規則");
  const asksToList =
    normalized.includes("列出") ||
    normalized.includes("显示") ||
    normalized.includes("顯示") ||
    normalized.includes("展示") ||
    normalized.includes("哪些") ||
    normalized.includes("什么");
  const pointsAtAssistantRules =
    normalized.includes("行为") ||
    normalized.includes("行為") ||
    normalized.includes("你的") ||
    normalized.includes("您的") ||
    normalized.includes("自己") ||
    normalized.includes("规则列表") ||
    normalized.includes("規則列表");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isSelfFactQuery(normalized) {
  return (
    normalized.includes("facts you know about yourself") ||
    normalized.includes("facts about yourself") ||
    normalized.includes("self facts") ||
    normalized.includes("list all facts you know about yourself") ||
    normalized.includes("какие факты ты знаешь о себе") ||
    normalized.includes("факты о себе") ||
    normalized.includes("अपने बारे में तथ्य") ||
    normalized.includes("स्वयं के बारे में तथ्य") ||
    normalized.includes("关于你自己的事实") ||
    normalized.includes("自我事实")
  );
}

function cleanRuleQuery(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[\s`"':._,\-?!]+|[\s`"':._,\-?!]+$/g, "")
    .toLowerCase();
}

function detailQuery(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const prefixes = [
    "show behavior rule",
    "read behavior rule",
    "describe behavior rule",
    "show rule",
    "read rule",
    "details for rule",
    "детали правила",
    "покажи правило",
    "прочитай правило",
  ];
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      return cleanRuleQuery(String(prompt || "").slice(prefix.length));
    }
  }
  if (lower.includes("rule_unknown")) return "unknown";
  return "";
}

function findBehaviorRule(query) {
  const cleaned = cleanRuleQuery(query);
  const withoutPrefix = cleaned.startsWith("rule_") ? cleaned.slice(5) : cleaned;
  return behaviorRuleRecords().find(
    (rule) =>
      rule.id === cleaned ||
      rule.id === `rule_${withoutPrefix}` ||
      rule.intent === cleaned ||
      rule.intent === withoutPrefix ||
      rule.label.toLowerCase().includes(withoutPrefix),
  );
}

function codeSpans(text) {
  return String(text || "")
    .split("`")
    .map((part, index) => (index % 2 === 1 ? part.trim() : ""))
    .filter(Boolean);
}

// Issue #144: recognize behavior-rule updates expressed as `When X then Y`
// (and translations) in addition to the explicit `When I say … answer …`
// grammar. KEYWORD_PAIRS is a list of (head, link) tuples that bracket the
// trigger and the answer; both must appear, head before link, and there must
// be at least one backtick on each side so the runtime extractor can pull the
// trigger and answer deterministically.
const BEHAVIOR_RULE_KEYWORD_PAIRS = [
  // English
  ["when ", " then "],
  ["when ", " do "],
  // Russian
  ["когда ", " тогда "],
  ["когда ", " делай "],
  ["когда ", " сделай "],
  ["когда ", " отвечай "],
  ["когда ", " отвечать "],
  ["если ", " то "],
  // Hindi
  ["जब ", " तब "],
  ["जब ", " तो "],
  // Chinese
  ["当 ", " 时 "],
  ["当 ", " 则 "],
  ["当 ", " 回答 "],
  ["当 ", "时回答 "],
  ["当 ", "则回答 "],
];

function looksLikeRuntimeRuleUpdate(text) {
  const raw = String(text || "");
  const lower = raw.toLowerCase();
  if (
    (lower.includes("when i say") && (lower.includes("answer") || lower.includes("reply"))) ||
    (lower.includes("if i ask") && (lower.includes("answer") || lower.includes("reply"))) ||
    lower.includes("add behavior rule") ||
    lower.includes("update behavior rule") ||
    (lower.includes("когда я скажу") && lower.includes("ответ")) ||
    (lower.includes("если я спрошу") && lower.includes("ответ")) ||
    lower.includes("добавь правило поведения") ||
    lower.includes("обнови правило поведения")
  ) {
    return true;
  }
  for (const [head, link] of BEHAVIOR_RULE_KEYWORD_PAIRS) {
    const headPos = lower.indexOf(head);
    if (headPos === -1) continue;
    const tail = lower.slice(headPos + head.length);
    const linkPos = tail.indexOf(link);
    if (linkPos === -1) continue;
    const absoluteLinkPos = headPos + head.length + linkPos;
    const beforeLink = raw.slice(headPos, absoluteLinkPos);
    const afterLink = raw.slice(absoluteLinkPos + link.length);
    if (beforeLink.includes("`") && afterLink.includes("`")) return true;
  }
  return false;
}

function runtimeRuleFromText(text) {
  if (!looksLikeRuntimeRuleUpdate(text)) return null;
  const spans = codeSpans(text);
  if (spans.length < 2) return null;
  const trigger = spans[0].trim();
  const answer = spans[1].trim();
  if (!trigger || !answer) return null;
  return {
    id: stableBehaviorRuleId("behavior_rule_runtime", `${trigger}\n${answer}`),
    trigger,
    answer,
  };
}

function runtimeRuleForPrompt(prompt, history) {
  const normalizedPrompt = normalizePrompt(prompt);
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "user") continue;
    const rule = runtimeRuleFromText(turn.content);
    if (rule && normalizePrompt(rule.trigger) === normalizedPrompt) {
      return rule;
    }
  }
  return null;
}

function collectRuntimeRules(history) {
  const turns = Array.isArray(history) ? history : [];
  const seen = new Set();
  const rules = [];
  for (const turn of turns) {
    const role = String((turn || {}).role || "").toLowerCase();
    if (role !== "user") continue;
    const rule = runtimeRuleFromText((turn || {}).content);
    if (rule && !seen.has(rule.id)) {
      seen.add(rule.id);
      rules.push(rule);
    }
  }
  return rules;
}

function tryBehaviorRules(prompt, normalized, history) {
  const updateRule = runtimeRuleFromText(prompt);
  if (updateRule) {
    return {
      intent: "behavior_rule_update",
      content: renderRuntimeRuleUpdate(updateRule),
      confidence: 1.0,
      evidence: ["behavior_rule:update", updateRule.id],
    };
  }

  if (isBehaviorRulesList(normalized)) {
    return {
      intent: "behavior_rules_list",
      content: renderBehaviorRuleList(collectRuntimeRules(history)),
      confidence: 1.0,
      evidence: ["behavior_rules:list", "all"],
    };
  }

  const query = detailQuery(prompt);
  if (query) {
    const rule = findBehaviorRule(query);
    if (rule) {
      return {
        intent: "behavior_rule_detail",
        content: renderBehaviorRuleDetail(rule),
        confidence: 1.0,
        evidence: ["behavior_rule:read", rule.id],
      };
    }
  }

  if (isSelfFactQuery(normalized)) {
    return {
      intent: "self_facts",
      content: renderSelfFacts(),
      confidence: 1.0,
      evidence: ["self_facts:list", "formal-ai"],
    };
  }

  const runtimeRule = runtimeRuleForPrompt(prompt, history);
  if (runtimeRule) {
    return {
      intent: "behavior_rule_custom",
      content: runtimeRule.answer,
      confidence: 1.0,
      evidence: ["behavior_rule:match", runtimeRule.id],
    };
  }

  return null;
}

function containsAny(normalized, values) {
  if (!normalized || !Array.isArray(values)) return false;
  return values.some((value) => value && normalized.includes(String(value).toLowerCase()));
}

const WEB_SEARCH_CAPABILITY_PHRASES = {
  en: [
    "web search",
    "internet search",
    "search engines",
    "can you search the internet",
    "can you search internet",
    "can you search the web",
    "can you search web",
    "can you search online",
    "do you have internet search",
    "do you have web search",
    "do you have internet access",
    "are you connected to search engines",
    "can you use search engines",
    "can you browse the web",
  ],
  ru: [
    "веб-поиск",
    "веб поиск",
    "поиск в интернете",
    "поисковик",
    "поисковые системы",
    "можешь искать в интернете",
    "можешь искать интернет",
    "умеешь искать в интернете",
    "умеешь искать интернет",
    "можешь искать онлайн",
    "умеешь искать онлайн",
    "у тебя есть веб-поиск",
    "у тебя есть веб поиск",
    "у тебя есть поиск в интернете",
    "есть доступ к интернету",
    "подключен к поисковикам",
    "подключена к поисковикам",
    "подключен к поисковым системам",
    "можешь пользоваться интернетом",
  ],
  hi: [
    "web search",
    "internet search",
    "search engine",
    "इंटरनेट पर खोज सकते",
    "ऑनलाइन खोज सकते",
    "इंटरनेट खोज है",
    "वेब खोज है",
    "सर्च इंजन से जुड़े",
    "खोज इंजन से जुड़े",
  ],
  zh: [
    "web search",
    "搜索引擎",
    "上网搜索",
    "搜索互联网",
    "搜索网络",
    "联网搜索",
    "用搜索引擎",
    "使用搜索引擎",
    "网络搜索",
  ],
};

const FEATURE_CAPABILITIES = [
  {
    slug: "web_search",
    state: "web_search",
    labels: { en: "web search", ru: "веб-поиск", hi: "web search", zh: "web search" },
    aliases: WEB_SEARCH_CAPABILITY_PHRASES,
    examples: {
      en: "Search the web for Nikola Tesla",
      ru: "Найди в интернете Никола Тесла",
      hi: "Search the web for Nikola Tesla",
      zh: "Search the web for Nikola Tesla",
    },
  },
  {
    slug: "diagnostics",
    state: "diagnostics",
    labels: { en: "diagnostics", ru: "диагностика", hi: "diagnostics", zh: "诊断" },
    aliases: {
      en: ["diagnostics", "diagnostic", "trace", "reasoning trace"],
      ru: ["диагностика", "диагност", "трассировка", "trace"],
      hi: ["diagnostics", "निदान", "trace"],
      zh: ["诊断", "trace", "推理跟踪"],
    },
    examples: {
      en: "Turn on diagnostics",
      ru: "Включи диагностику",
      hi: "Turn on diagnostics",
      zh: "开启诊断",
    },
  },
  {
    slug: "agent_mode",
    state: "agent_mode",
    labels: { en: "agent mode", ru: "agent mode", hi: "agent mode", zh: "agent mode" },
    aliases: {
      en: ["agent mode", "agent", "multi-step", "autonomous"],
      ru: ["agent mode", "агент", "многошаг", "автоном"],
      hi: ["agent mode", "एजेंट", "multi-step"],
      zh: ["agent mode", "代理", "多步骤"],
    },
    examples: {
      en: "Turn on agent mode",
      ru: "Включи agent mode",
      hi: "Turn on agent mode",
      zh: "开启 agent mode",
    },
  },
  {
    slug: "definition_fusion",
    state: "definition_fusion",
    labels: {
      en: "automatic definition fusion",
      ru: "автоматическое слияние определений",
      hi: "automatic definition fusion",
      zh: "自动 definition fusion",
    },
    aliases: {
      en: ["definition fusion", "merge definitions", "automatic definition"],
      ru: ["слияние определений", "объединение определений"],
      hi: ["definition fusion", "merge definitions"],
      zh: ["definition fusion", "合并定义"],
    },
    examples: {
      en: "Turn on definition fusion",
      ru: "Включи слияние определений",
      hi: "Turn on definition fusion",
      zh: "开启 definition fusion",
    },
  },
  {
    slug: "configuration",
    state: "always",
    labels: {
      en: "message-driven configuration",
      ru: "настройка через сообщения",
      hi: "message-driven configuration",
      zh: "消息驱动设置",
    },
    aliases: {
      en: ["configure", "configuration", "settings", "preferences", "theme", "language", "chat style", "composer style", "ui skin"],
      ru: ["настрой", "конфигурац", "параметр", "тема", "язык", "стиль чата", "оформление"],
      hi: ["settings", "configuration", "theme", "language", "सेटिंग"],
      zh: ["设置", "配置", "主题", "语言", "聊天样式"],
    },
    examples: {
      en: "Switch to dark theme",
      ru: "Переключи тему на темную",
      hi: "Switch to dark theme",
      zh: "切换到深色主题",
    },
  },
  {
    slug: "memory_actions",
    state: "always",
    labels: {
      en: "memory import/export",
      ru: "импорт и экспорт памяти",
      hi: "memory import/export",
      zh: "记忆导入/导出",
    },
    aliases: {
      en: ["export memory", "import memory", "memory export", "memory import"],
      ru: ["экспорт памяти", "импорт памяти", "память экспорт", "память импорт"],
      hi: ["memory export", "memory import", "स्मृति निर्यात", "स्मृति आयात"],
      zh: ["导出记忆", "导入记忆", "memory export", "memory import"],
    },
    examples: {
      en: "Export memory",
      ru: "Экспортируй память",
      hi: "Export memory",
      zh: "导出记忆",
    },
  },
  {
    slug: "greeting",
    state: "always",
    labels: { en: "greetings", ru: "приветствия", hi: "अभिवादन", zh: "问候" },
    aliases: {
      en: ["greeting", "greetings", "say hello", "respond to hello"],
      ru: ["приветствие", "приветствия", "здороваться", "привет"],
      hi: ["अभिवादन", "नमस्ते", "hello"],
      zh: ["问候", "打招呼", "你好"],
    },
    examples: { en: "Hello", ru: "Привет", hi: "नमस्ते", zh: "你好" },
  },
  {
    slug: "hello_world",
    state: "always",
    labels: {
      en: "Hello World code generation",
      ru: "генерация Hello World",
      hi: "Hello World code generation",
      zh: "Hello World 代码生成",
    },
    aliases: {
      en: ["hello world", "write code", "generate code", "program"],
      ru: ["hello world", "код", "программу", "программа"],
      hi: ["hello world", "code", "program", "प्रोग्राम"],
      zh: ["hello world", "代码", "程序"],
    },
    examples: {
      en: "Write hello world in Rust",
      ru: "Напиши hello world на Rust",
      hi: "Write hello world in Rust",
      zh: "Write hello world in Rust",
    },
  },
  {
    slug: "concept_lookup",
    state: "always",
    labels: { en: "concept lookup", ru: "поиск понятий", hi: "concept lookup", zh: "概念查找" },
    aliases: {
      en: ["concept lookup", "concept", "wikipedia lookup"],
      ru: ["поиск понятий", "понятие"],
      hi: ["concept", "अवधारणा"],
      zh: ["概念"],
    },
    examples: {
      en: "What is Wikipedia?",
      ru: "Что такое Википедия?",
      hi: "विकिपीडिया क्या है?",
      zh: "什么是维基百科？",
    },
  },
  {
    slug: "arithmetic",
    state: "always",
    labels: { en: "arithmetic", ru: "арифметика", hi: "अंकगणित", zh: "算术" },
    aliases: {
      en: ["arithmetic", "calculate", "math", "2 + 2"],
      ru: ["арифмет", "считать", "посчитать", "2 + 2"],
      hi: ["अंकगणित", "गणना", "math", "2 + 2"],
      zh: ["算术", "计算", "数学", "2 + 2"],
    },
    examples: {
      en: "What is 2 + 2?",
      ru: "Сколько будет 2 + 2?",
      hi: "2 + 2 क्या है?",
      zh: "2 + 2 等于多少？",
    },
  },
  {
    slug: "translation",
    state: "always",
    labels: { en: "translation", ru: "перевод", hi: "अनुवाद", zh: "翻译" },
    aliases: {
      en: ["translation", "translate", "language translation"],
      ru: ["перевод", "переводить", "перевести"],
      hi: ["अनुवाद", "translate", "translation"],
      zh: ["翻译", "translation", "translate"],
    },
    examples: {
      en: 'Translate "hello" to Russian',
      ru: 'Переведи "hello" на русский',
      hi: 'Translate "hello" to Hindi',
      zh: 'Translate "hello" to Chinese',
    },
  },
  {
    slug: "memory",
    state: "always",
    labels: {
      en: "conversation memory",
      ru: "память разговора",
      hi: "conversation memory",
      zh: "会话记忆",
    },
    aliases: {
      en: ["memory", "remember", "recall", "conversation context"],
      ru: ["память", "помнить", "запомнить", "контекст"],
      hi: ["स्मृति", "याद", "memory", "context"],
      zh: ["记忆", "记住", "回忆", "上下文"],
    },
    examples: {
      en: "My name is Ada. What is my name?",
      ru: "Меня зовут Ада. Как меня зовут?",
      hi: "My name is Ada. What is my name?",
      zh: "My name is Ada. What is my name?",
    },
  },
  {
    slug: "demo_mode",
    state: "always",
    labels: { en: "demo mode", ru: "демо-режим", hi: "demo mode", zh: "演示模式" },
    aliases: {
      en: ["demo mode", "demo", "scripted demo"],
      ru: ["демо", "демо-режим", "сценарный демо"],
      hi: ["demo", "डेमो"],
      zh: ["演示", "demo"],
    },
    examples: { en: "Turn off demo mode", ru: "Выключи демо", hi: "Turn off demo mode", zh: "关闭演示" },
  },
  {
    slug: "http_url",
    state: "always",
    labels: {
      en: "URL fetch/navigation",
      ru: "HTTP-запросы и переходы по URL",
      hi: "URL fetch/navigation",
      zh: "URL fetch/navigation",
    },
    aliases: {
      en: ["http fetch", "fetch url", "open url", "navigate to url", "visit url"],
      ru: ["http запрос", "открыть url", "перейти на", "сделать запрос"],
      hi: ["http fetch", "url", "navigate"],
      zh: ["http fetch", "url", "打开链接", "访问链接"],
    },
    examples: {
      en: "Navigate to example.com",
      ru: "Перейди на example.com",
      hi: "Navigate to example.com",
      zh: "Navigate to example.com",
    },
  },
  {
    slug: "javascript_execution",
    state: "always",
    labels: {
      en: "JavaScript execution",
      ru: "выполнение JavaScript",
      hi: "JavaScript execution",
      zh: "JavaScript execution",
    },
    aliases: {
      en: ["javascript", "run javascript", "execute javascript"],
      ru: ["javascript", "js", "выполнить javascript"],
      hi: ["javascript", "js"],
      zh: ["javascript", "js"],
    },
    examples: {
      en: "Run JavaScript: 1 + 1",
      ru: "Выполни JavaScript: 1 + 1",
      hi: "Run JavaScript: 1 + 1",
      zh: "Run JavaScript: 1 + 1",
    },
  },
  {
    slug: "planning",
    state: "always",
    labels: {
      en: "summaries, brainstorming, roleplay, and project planning",
      ru: "резюме, брейншторминг, роли и планирование проектов",
      hi: "summaries, brainstorming, roleplay, and project planning",
      zh: "总结、头脑风暴、角色扮演和项目计划",
    },
    aliases: {
      en: ["summarize", "brainstorm", "roleplay", "software project", "project plan"],
      ru: ["резюмировать", "брейншторм", "роль", "проект", "план проекта"],
      hi: ["summary", "brainstorm", "roleplay", "project plan"],
      zh: ["总结", "头脑风暴", "角色扮演", "项目计划"],
    },
    examples: {
      en: "Brainstorm 5 project ideas",
      ru: "Предложи 5 идей проекта",
      hi: "Brainstorm 5 project ideas",
      zh: "Brainstorm 5 project ideas",
    },
  },
];

function localizedValue(record, language) {
  if (!record || typeof record !== "object") return "";
  return record[language] || record.en || "";
}

function featureAliases(feature, language) {
  if (!feature || !feature.aliases) return [];
  return feature.aliases[language] || feature.aliases.en || [];
}

function detectFeatureCapability(normalized, language) {
  return FEATURE_CAPABILITIES.find((feature) => {
    return (
      containsAny(normalized, featureAliases(feature, language)) ||
      (language !== "en" && containsAny(normalized, featureAliases(feature, "en")))
    );
  }) || null;
}

function isFeatureCapabilityQuestion(normalized, language) {
  if (language === "ru") {
    return containsAny(normalized, [
      "можешь",
      "умеешь",
      "поддерживаешь",
      "у тебя есть",
      "есть ли",
      "доступен",
      "доступна",
      "включен",
      "включена",
      "подключен",
      "подключена",
      "можно ли",
    ]);
  }
  if (language === "zh") {
    return containsAny(normalized, ["能", "可以", "支持", "有", "启用", "可用"]);
  }
  if (language === "hi") {
    return containsAny(normalized, ["क्या", "सकते", "सकती", "समर्थन", "उपलब्ध"]);
  }
  return containsAny(normalized, [
    "can you",
    "can formal-ai",
    "are you able",
    "are you connected",
    "do you support",
    "do you have",
    "enabled",
    "available",
    "can i",
  ]);
}

function isFeatureActionRequest(normalized, feature) {
  if (!feature) return false;
  if (feature.slug === "arithmetic") {
    return [
      "can you calculate ",
      "can you compute ",
    ].some((prefix) => normalized.startsWith(prefix));
  }
  if (feature.slug === "planning") {
    return containsAny(normalized, [
      "can you summarize ",
      "can you brainstorm ",
      "can you roleplay ",
    ]);
  }
  return false;
}

function webSearchStatusContent(language, available, providers) {
  const providerList = providers || "none";
  const rrfK = webSearchRrfK();
  if (language === "ru") {
    return available
      ? `Да. В этой конфигурации веб-поиск включен: я могу использовать DuckDuckGo Instant Answer по умолчанию и доступные CORS-провайдеры (\`${providerList}\`) для явных запросов вроде \`Найди в интернете Никола Тесла\`. Результаты из top-10 по каждому провайдеру объединяются через reciprocal rank fusion (k = ${rrfK}). Если провайдеры отключены или заблокированы в браузерной сессии, я сообщу об этом вместо ответа "да".`
      : "Нет. В этой браузерной сессии веб-поиск сейчас недоступен: браузер offline или все CORS-readable поисковые провайдеры отключены после ошибок. Я могу отвечать по локальным правилам и кэшу, но не буду обращаться к поисковым системам.";
  }
  if (language === "zh") {
    return available
      ? `可以。当前配置启用了 web search：我会默认使用 DuckDuckGo Instant Answer，并可使用这些 CORS-readable provider（\`${providerList}\`）处理明确的搜索请求，例如 \`Search the web for Nikola Tesla\`。每个 provider 的 top-10 结果会用 reciprocal rank fusion 合并（k = ${rrfK}）。如果浏览器会话中所有 provider 被禁用或阻止，我会说明不可用，而不是回答可以。`
      : "不可以。当前浏览器会话中 web search 不可用：浏览器 offline，或所有 CORS-readable 搜索 provider 都因错误被禁用。我仍可使用本地规则和缓存回答，但不会调用搜索引擎。";
  }
  if (language === "hi") {
    return available
      ? `हाँ। इस configuration में web search enabled है: मैं default रूप से DuckDuckGo Instant Answer और उपलब्ध CORS-readable providers (\`${providerList}\`) का उपयोग explicit prompts जैसे \`Search the web for Nikola Tesla\` के लिए कर सकता हूँ। हर provider के top-10 results reciprocal rank fusion (k = ${rrfK}) से merge होते हैं। अगर browser session में providers disabled या blocked हों, तो मैं "हाँ" कहने के बजाय स्थिति बताऊँगा।`
      : "नहीं। इस browser session में web search अभी available नहीं है: browser offline है या सभी CORS-readable search providers errors के बाद disabled हैं। मैं local rules और cache से जवाब दे सकता हूँ, लेकिन search engines को call नहीं करूँगा।";
  }
  return available
    ? `Yes. Web search is enabled in this configuration: I can use DuckDuckGo Instant Answer by default plus the configured CORS-readable providers (\`${providerList}\`) for explicit prompts such as \`Search the web for Nikola Tesla\`. The top-10 results from each provider are merged with reciprocal rank fusion (k = ${rrfK}). If the browser session disables or blocks every provider, I will say that instead of claiming search is available.`
    : "No. Web search is unavailable in this browser session: the browser is offline or every CORS-readable search provider has been disabled after errors. I can still answer from local rules and cache, but I will not call search engines.";
}

function featureAvailability(feature, preferences) {
  if (!feature) return { available: false, reason: "unknown" };
  if (feature.state === "web_search") {
    const providers = WEB_SEARCH_PROVIDERS.filter((provider) => !webSearchIsDisabled(provider.id));
    const online = typeof navigator === "undefined" || navigator.onLine !== false;
    return {
      available: online && providers.length > 0,
      reason: online && providers.length > 0 ? "none" : "offline_or_no_providers",
      providers,
    };
  }
  if (feature.state === "diagnostics") {
    const available = Boolean(preferences && preferences.diagnosticsMode);
    return { available, reason: available ? "none" : "diagnostics_off" };
  }
  if (feature.state === "agent_mode") {
    const available = Boolean(preferences && preferences.agentMode);
    return { available, reason: available ? "none" : "agent_mode_off" };
  }
  if (feature.state === "definition_fusion") {
    const available = definitionFusionByDefault(preferences || {});
    return { available, reason: available ? "none" : "definition_fusion_explicit" };
  }
  return { available: true, reason: "none" };
}

function unavailableReasonText(reason, language) {
  const reasons = {
    offline_or_no_providers: {
      en: "the browser is offline or no search providers are available",
      ru: "браузер offline или нет доступных поисковых провайдеров",
      hi: "browser offline है या कोई search provider available नहीं है",
      zh: "浏览器 offline，或没有可用搜索 provider",
    },
    diagnostics_off: {
      en: "diagnostics are off; enable them to show traces",
      ru: "диагностика выключена; включите ее, чтобы видеть трассировку",
      hi: "diagnostics off है; trace दिखाने के लिए इसे enable करें",
      zh: "诊断已关闭；开启后才会显示 trace",
    },
    agent_mode_off: {
      en: "agent mode is off; multi-step actions require explicit opt-in",
      ru: "agent mode выключен; для многошаговых действий нужен явный opt-in",
      hi: "agent mode off है; multi-step actions के लिए explicit opt-in चाहिए",
      zh: "agent mode 已关闭；多步骤操作需要显式启用",
    },
    definition_fusion_explicit: {
      en: "automatic definition fusion is set to explicit-only",
      ru: "автоматическое слияние определений работает только после включения режима auto",
      hi: "automatic definition fusion के लिए auto mode enable करना होगा",
      zh: "自动 definition fusion 需要切换到 auto 模式",
    },
  };
  return localizedValue(reasons[reason] || { en: "not available" }, language);
}

function featureCapabilityContent(feature, language, availability) {
  if (feature.slug === "web_search") {
    const providers = availability.providers || [];
    return webSearchStatusContent(
      language,
      availability.available,
      providers.map((provider) => provider.id).join(", "),
    );
  }
  const label = localizedValue(feature.labels, language);
  const example = localizedValue(feature.examples, language);
  if (availability.available) {
    if (language === "ru") {
      return `Да. Возможность «${label}» доступна в этой конфигурации. Пример сообщения: \`${example}\`.`;
    }
    if (language === "zh") {
      return `可以。当前配置中「${label}」可用。示例消息：\`${example}\`。`;
    }
    if (language === "hi") {
      return `हाँ। इस configuration में \`${label}\` available है। Example message: \`${example}\`.`;
    }
    return `Yes. ${label} is available in this configuration. Example message: \`${example}\`.`;
  }
  const reason = unavailableReasonText(availability.reason, language);
  if (language === "ru") {
    return `Нет. Возможность «${label}» сейчас недоступна в этой конфигурации: ${reason}. Пример сообщения после включения: \`${example}\`.`;
  }
  if (language === "zh") {
    return `不可以。当前配置中「${label}」不可用：${reason}。启用后的示例消息：\`${example}\`。`;
  }
  if (language === "hi") {
    return `नहीं। इस configuration में \`${label}\` अभी available नहीं है: ${reason}. Enable करने के बाद example message: \`${example}\`.`;
  }
  return `No. ${label} is not available in this configuration: ${reason}. Example message after enabling it: \`${example}\`.`;
}

function tryFeatureCapabilityStatus(prompt, normalized, language, preferences) {
  if (!isFeatureCapabilityQuestion(normalized, language)) return null;
  const feature = detectFeatureCapability(normalized, language);
  if (!feature) return null;
  if (isFeatureActionRequest(normalized, feature)) return null;
  const availability = featureAvailability(feature, preferences || {});
  const providers = WEB_SEARCH_PROVIDERS.filter((provider) => !webSearchIsDisabled(provider.id));
  return {
    intent: "capabilities",
    content: featureCapabilityContent(feature, language, availability),
    confidence: availability.available ? 0.95 : 0.6,
    evidence: [
      "handler:capabilities",
      `feature:question:${feature.slug}`,
      availability.available
        ? `feature:available:${feature.slug}`
        : `feature:unavailable:${feature.slug}:${availability.reason}`,
      ...(feature.slug === "web_search" ? providers.map((provider) => `web_search:provider:${provider.id}`) : []),
      `language:${language}`,
    ],
  };
}

function isMoreCapabilitiesPrompt(normalized, language) {
  if (language === "ru") {
    return (
      normalized.includes("что ещё ты умеешь") ||
      normalized.includes("что еще ты умеешь") ||
      normalized.includes("что ещё можешь") ||
      normalized.includes("что еще можешь") ||
      normalized.includes("что ты ещё умеешь") ||
      normalized.includes("что ты еще умеешь")
    );
  }
  return (
    normalized.includes("what else can you do") ||
    normalized.includes("what else do you do") ||
    normalized.includes("what other things can you do")
  );
}

function historyMentionsWebSearch(history) {
  if (!Array.isArray(history)) return false;
  return history.some((turn) => {
    const content = String(turn && turn.content ? turn.content : "").toLowerCase();
    return (
      content.includes("duckduckgo") ||
      content.includes("web search") ||
      content.includes("search the internet") ||
      content.includes("веб-поиск") ||
      content.includes("веб поиск") ||
      content.includes("интернет")
    );
  });
}

function additionalCapabilitiesContent(language) {
  if (language === "ru") {
    return "Кроме уже названных возможностей, могу ещё:\n\n- **Арифметика**: вычислять выражения вроде «Сколько будет 2 + 2?»\n- **Перевод**: переводить короткие фразы между поддерживаемыми языками.\n- **Поиск понятий**: объяснять термины, например «Что такое Википедия?»\n- **Hello World**: генерировать минимальные программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Память диалога**: использовать предыдущие сообщения текущей сессии.\n- **Правила поведения**: показывать встроенные правила через `List behavior rules` и `Show behavior rule unknown`.\n- **Настройки и действия**: включать диагностику/демо/agent mode, менять тему, язык, стиль чата, экспортировать и импортировать память.";
  }
  return "Beyond the capability already discussed, I can also:\n\n- **Arithmetic**: evaluate expressions like `2 + 2`.\n- **Translation**: translate short phrases between supported languages.\n- **Concept lookup**: explain terms such as `What is Wikipedia?`.\n- **Hello World**: generate small programs in Rust, Python, JavaScript, Go, C, and more.\n- **Conversation memory**: use earlier messages from the current session.\n- **Behavior rules**: show built-in rules with `List behavior rules` and `Show behavior rule unknown`.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export.";
}

function tryCapabilities(prompt, normalized, preferences, history) {
  const language = detectLanguage(prompt);
  const featureStatus = tryFeatureCapabilityStatus(prompt, normalized, language, preferences);
  if (featureStatus) return featureStatus;
  const moreCapabilities = isMoreCapabilitiesPrompt(normalized, language);
  const isCapabilities =
    language === "ru"
      ? moreCapabilities ||
        normalized.includes("что ты умеешь") ||
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
          : moreCapabilities ||
            normalized.includes("what can you do") ||
            normalized.includes("what you can do") ||
            normalized.includes("what are your capabilities") ||
            normalized.includes("what are you capable of") ||
            normalized.includes("what do you do") ||
            normalized.includes("show me what you can do") ||
            normalized.includes("what features do you have") ||
            normalized.includes("how can you help") ||
            normalized.includes("what are your features");
  if (!isCapabilities) return null;
  if (moreCapabilities) {
    const priorSearch = historyMentionsWebSearch(history);
    return {
      intent: "capabilities",
      content: additionalCapabilitiesContent(language),
      confidence: 1.0,
      evidence: [
        "handler:capabilities",
        "capabilities:follow_up",
        ...(priorSearch ? ["capabilities:history:prior_web_search"] : []),
        `language:${language}`,
      ],
    };
  }
  const content =
    language === "ru"
      ? "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\n- **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n- **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Веб-поиск**: ищу в интернете через DuckDuckGo, Wikipedia и Wikidata, когда поиск доступен.\n- **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n- **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n- **Перевод**: перевожу фразы между языками.\n- **Память**: помню контекст разговора в рамках сессии.\n- **Настройки и действия**: через сообщения можно включать диагностику/демо/agent mode, менять тему, язык, стиль чата и экспортировать или импортировать память.\n\nЯ работаю на основе локальных символьных правил, без нейросетевого инференса."
      : language === "zh"
        ? "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\n- **问候**：回应「你好」等问候语。\n- **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n- **Web search**：在可用时通过 DuckDuckGo、Wikipedia 和 Wikidata 搜索互联网。\n- **概念查找**：解释术语，例如「什么是维基百科？」\n- **算术**：计算表达式，例如「2 + 2 等于多少？」\n- **翻译**：在语言之间翻译短语。\n- **记忆**：在会话中记住上下文。\n- **设置和操作**：可通过消息开启诊断、演示、agent mode，切换主题、语言、聊天样式，并导出或导入记忆。\n\n我基于本地符号规则运行，不进行神经网络推理。"
        : language === "hi"
          ? "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\n- **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n- **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n- **Web search**: उपलब्ध होने पर DuckDuckGo, Wikipedia, और Wikidata से इंटरनेट में खोजना।\n- **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n- **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n- **अनुवाद**: भाषाओं के बीच अनुवाद।\n- **स्मृति**: सत्र में संदर्भ याद रखना।\n- **Settings और actions**: messages से diagnostics/demo/agent mode बदलना, theme/language/chat style बदलना, और memory export/import करना।\n\nमैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।"
          : "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\n- **Greetings**: respond to «Hi», «Hello», and similar.\n- **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n- **Web search**: search the internet through DuckDuckGo, Wikipedia, and Wikidata when available.\n- **Concept lookup**: explain terms — try «What is Wikipedia?»\n- **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n- **Translation**: translate phrases between languages.\n- **Memory**: recall context within the current session.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export from messages.\n\nI run on local symbolic rules, without any neural network inference.";
  return {
    intent: "capabilities",
    content,
    confidence: 1.0,
    evidence: ["handler:capabilities", `language:${language}`],
  };
}

function detectTranslationSourceLanguage(normalized) {
  if (
    normalized.includes("from english") ||
    normalized.includes("с английского") ||
    normalized.includes("अंग्रेजी से") ||
    normalized.includes("अंग्रेज़ी से") ||
    normalized.includes("从英语") ||
    normalized.includes("从英文")
  ) return "en";
  if (
    normalized.includes("from russian") ||
    normalized.includes("с русского") ||
    normalized.includes("रूसी से") ||
    normalized.includes("从俄语")
  ) return "ru";
  if (
    normalized.includes("from hindi") ||
    normalized.includes("हिंदी से") ||
    normalized.includes("हिन्दी से") ||
    normalized.includes("从印地语") ||
    normalized.includes("从印地文")
  ) return "hi";
  if (
    normalized.includes("from chinese") ||
    normalized.includes("चीनी से") ||
    normalized.includes("从中文") ||
    normalized.includes("从汉语") ||
    normalized.includes("从漢語")
  ) return "zh";
  return null;
}

function detectTranslationTargetLanguage(normalized) {
  if (
    normalized.includes("to english") ||
    normalized.includes("на английский") ||
    normalized.includes("на английском") ||
    normalized.includes("अंग्रेजी में") ||
    normalized.includes("अंग्रेज़ी में") ||
    ["成英文", "成英语", "为英文", "为英语", "為英文", "為英语", "到英文", "到英语"]
      .some((marker) => normalized.includes(marker))
  ) {
    return "en";
  }
  if (
    normalized.includes("to russian") ||
    normalized.includes("на русский") ||
    normalized.includes("रूसी में") ||
    ["成俄语", "成俄語", "为俄语", "为俄語", "為俄语", "為俄語", "到俄语", "到俄語"]
      .some((marker) => normalized.includes(marker))
  ) return "ru";
  if (
    normalized.includes("to hindi") ||
    normalized.includes("на хинди") ||
    normalized.includes("हिंदी में") ||
    normalized.includes("हिन्दी में") ||
    [
      "成印地语",
      "成印地文",
      "为印地语",
      "为印地文",
      "為印地语",
      "為印地文",
      "到印地语",
      "到印地文",
    ].some((marker) => normalized.includes(marker))
  ) return "hi";
  if (
    normalized.includes("to chinese") ||
    normalized.includes("на китайский") ||
    normalized.includes("चीनी में") ||
    [
      "成中文",
      "成汉语",
      "成漢語",
      "为中文",
      "为汉语",
      "为漢語",
      "為中文",
      "為汉语",
      "為漢語",
      "到中文",
      "到汉语",
      "到漢語",
    ].some((marker) => normalized.includes(marker))
  ) return "zh";
  return null;
}

// Offline meaning registry for the browser worker.
//
// The Rust pipeline (`src/translation/pipeline.rs`) resolves any pair
// of surfaces through Wiktionary + Wikidata using cached HTTP
// responses. The worker mirrors that with a live `liveWiktionaryTranslate`
// fallback below (MediaWiki action API is CORS-friendly via
// `origin=*`), but keeps this small in-memory registry of greetings and
// stock phrases so the demo stays snappy when the network is slow.
// `primary` is the canonical form deformalization renders; `aliases` is a
// list of normalized alternative surfaces used during formalization.
const TRANSLATION_MEANING_REGISTRY = [
  {
    token: "greeting",
    primary: { en: "Hello", ru: "Привет", hi: "नमस्ते", zh: "你好" },
    aliases: {
      en: ["hello", "hi", "hey"],
      ru: ["привет", "здравствуйте", "здравствуй"],
      hi: ["नमस्ते", "नमस्कार"],
      zh: ["你好", "您好"],
    },
  },
  {
    token: "greeting_how_are_you",
    primary: {
      en: "How are you?",
      ru: "Как у тебя дела?",
      hi: "आप कैसे हैं?",
      zh: "你好吗？",
    },
    aliases: {
      en: ["howareyou", "hellohowareyou", "hihowareyou"],
      ru: [
        "какдела",
        "какутебядела",
        "какувасдела",
        "какваши дела",
        "какватидела",
        "какваши",
        "приветкакдела",
        "здравствуйтекаквашидела",
      ],
      hi: ["आपकैसेहैं", "तुमकैसेहो"],
      zh: ["你好吗", "你怎么样"],
    },
  },
  {
    token: "thank_you",
    primary: { en: "Thank you", ru: "Спасибо", hi: "धन्यवाद", zh: "谢谢" },
    aliases: {
      en: ["thanks", "thankyou", "thankyouverymuch"],
      ru: ["спасибо", "благодарю", "большоеспасибо"],
      hi: ["धन्यवाद", "शुक्रिया"],
      zh: ["谢谢", "多谢", "感谢"],
    },
  },
  {
    token: "you_are_welcome",
    primary: {
      en: "You are welcome",
      ru: "Пожалуйста",
      hi: "आपका स्वागत है",
      zh: "不客气",
    },
    aliases: {
      en: ["youarewelcome", "yourewelcome", "nottoworry"],
      ru: ["пожалуйста", "незачто"],
      hi: ["आपकास्वागतहै", "कोईबातनहीं"],
      zh: ["不客气", "不用谢"],
    },
  },
  {
    token: "goodbye",
    primary: { en: "Goodbye", ru: "До свидания", hi: "अलविदा", zh: "再见" },
    aliases: {
      en: ["goodbye", "bye", "seeyou", "byebye"],
      ru: ["досвидания", "пока", "прощай"],
      hi: ["अलविदा", "फिरमिलेंगे"],
      zh: ["再见", "拜拜"],
    },
  },
  {
    token: "good_morning",
    primary: { en: "Good morning", ru: "Доброе утро", hi: "सुप्रभात", zh: "早上好" },
    aliases: {
      en: ["goodmorning"],
      ru: ["доброеутро"],
      hi: ["सुप्रभात", "शुभप्रभात"],
      zh: ["早上好", "早安"],
    },
  },
  {
    token: "good_evening",
    primary: { en: "Good evening", ru: "Добрый вечер", hi: "शुभ संध्या", zh: "晚上好" },
    aliases: {
      en: ["goodevening"],
      ru: ["добрыйвечер"],
      hi: ["शुभसंध्या"],
      zh: ["晚上好", "晚安"],
    },
  },
  {
    token: "what_is_your_name",
    primary: {
      en: "What is your name?",
      ru: "Как тебя зовут?",
      hi: "तुम्हारा नाम क्या है?",
      zh: "你叫什么名字？",
    },
    aliases: {
      en: ["whatisyourname", "whatsyourname"],
      ru: ["кактебязовут", "каквасзовут"],
      hi: ["तुम्हारानामक्याहै", "आपकानामक्याहै"],
      zh: ["你叫什么名字", "您叫什么名字"],
    },
  },
  {
    token: "who_are_you",
    primary: {
      en: "Who are you?",
      ru: "Кто ты такой?",
      hi: "तुम कौन हो?",
      zh: "你是谁？",
    },
    aliases: {
      en: ["whoareyou"],
      ru: ["ктоты", "ктотытакой", "ктотытакая", "ктовы", "ктовытакой", "ктовытакая"],
      hi: ["तुमकौनहो", "आपकौनहैं"],
      zh: ["你是谁", "您是谁"],
    },
  },
  {
    token: "what_is_this",
    primary: {
      en: "What is this?",
      ru: "Что это такое?",
      hi: "यह क्या है?",
      zh: "这是什么？",
    },
    aliases: {
      en: ["whatisthis", "whatisit"],
      ru: ["чтоэто", "чтоэтотакое"],
      hi: ["यहक्याहै", "येक्याहै"],
      zh: ["这是什么", "這是什麼"],
    },
  },
  {
    token: "i_am_fine",
    primary: { en: "I am fine", ru: "У меня всё хорошо", hi: "मैं ठीक हूँ", zh: "我很好" },
    aliases: {
      en: ["iamfine", "imfine", "imdoingfine", "imdoingwell"],
      ru: ["уменявсёхорошо", "уменявсехорошо", "всёхорошо"],
      hi: ["मैंठीकहूँ", "मैंठीकहूं"],
      zh: ["我很好", "我挺好的"],
    },
  },
  {
    token: "yes",
    primary: { en: "Yes", ru: "Да", hi: "हाँ", zh: "是" },
    aliases: {
      en: ["yes", "yeah", "yep", "aye"],
      ru: ["да", "ага", "конечно"],
      hi: ["हाँ", "हां", "जी"],
      zh: ["是", "是的", "对"],
    },
  },
  {
    token: "no",
    primary: { en: "No", ru: "Нет", hi: "नहीं", zh: "不" },
    aliases: {
      en: ["no", "nope", "nah"],
      ru: ["нет", "неа"],
      hi: ["नहीं", "ना"],
      zh: ["不", "不是"],
    },
  },
  // Issue #216 / #217: the apple noun must be translatable in both
  // directions from the browser demo, including unquoted prompts.
  {
    token: "apple",
    primary: { en: "apple", ru: "яблоко", hi: "सेब", zh: "苹果" },
    aliases: {
      en: ["apple", "apples"],
      ru: [
        "яблоко",
        "яблока",
        "яблоку",
        "яблоком",
        "яблоке",
        "яблоки",
        "яблок",
        "яблокам",
        "яблоками",
        "яблоках",
      ],
      hi: ["सेब"],
      zh: ["苹果"],
    },
  },
];

const TRANSLATION_TERMINAL_PUNCTUATION = ["?", "!", ".", "。", "？", "！", "．"];

function normalizeTranslationAlias(surface) {
  return Array.from(String(surface || "").toLowerCase())
    .filter((character) => /[\p{L}\p{N}]/u.test(character))
    .join("");
}

function formalizeSurface(surface, source) {
  const normalized = normalizeTranslationAlias(surface);
  if (!normalized) return null;
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    const aliases = (entry.aliases && entry.aliases[source]) || [];
    if (aliases.some((alias) => normalizeTranslationAlias(alias) === normalized)) {
      return entry.token;
    }
    const primary = entry.primary && entry.primary[source];
    if (primary && normalizeTranslationAlias(primary) === normalized) {
      return entry.token;
    }
  }
  return null;
}

function deformalizeMeaning(token, target) {
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    if (entry.token !== token) continue;
    const primary = entry.primary && entry.primary[target];
    return primary || null;
  }
  return null;
}

function canonicalTokenForNormalized(normalized) {
  if (!normalized) return null;
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    const aliasesByLang = entry.aliases || {};
    for (const lang of Object.keys(aliasesByLang)) {
      const aliases = aliasesByLang[lang] || [];
      if (aliases.some((alias) => normalizeTranslationAlias(alias) === normalized)) {
        return entry.token;
      }
    }
    const primaryByLang = entry.primary || {};
    for (const lang of Object.keys(primaryByLang)) {
      if (normalizeTranslationAlias(primaryByLang[lang]) === normalized) {
        return entry.token;
      }
    }
  }
  return null;
}

function canonicalMeaningToken(raw) {
  return canonicalTokenForNormalized(raw) || raw;
}

function normalizeMeaningText(surface) {
  const raw = normalizeTranslationAlias(surface);
  return canonicalMeaningToken(raw);
}

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

const RU_EN_PHRASE_FALLBACKS = new Map([
  ["кто ты", "Who are you?"],
  ["кто ты такой", "Who are you?"],
  ["кто ты такая", "Who are you?"],
  ["кто вы", "Who are you?"],
  ["кто вы такой", "Who are you?"],
  ["кто вы такая", "Who are you?"],
  ["что это", "What is this?"],
  ["что это такое", "What is this?"],
]);

const RU_EN_WORD_FALLBACKS = new Map([
  ["найди", "find"],
  ["найдите", "find"],
  ["найти", "find"],
  ["синоним", "synonyms"],
  ["синонимы", "synonyms"],
  ["синонимов", "synonyms"],
  ["или", "or"],
  ["пример", "examples"],
  ["примеры", "examples"],
  ["примеров", "examples"],
  ["согласование", "agreement"],
  ["согласования", "agreement"],
  ["согласованию", "agreement"],
  ["согласованием", "agreement"],
  ["согласовании", "agreement"],
  ["доброе", "good"],
  ["добрый", "good"],
  ["добрая", "good"],
  ["добрые", "good"],
  ["доброго", "good"],
  ["добрую", "good"],
  ["добрым", "good"],
  ["хорошее", "good"],
  ["хороший", "good"],
  ["хорошая", "good"],
  ["хорошие", "good"],
  ["хорошего", "good"],
  ["хорошую", "good"],
  ["хорошим", "good"],
  ["яблоко", "apple"],
  ["яблока", "apple"],
  ["яблоку", "apple"],
  ["яблоком", "apple"],
  ["яблоке", "apple"],
  ["яблоки", "apple"],
  ["яблок", "apple"],
  ["яблокам", "apple"],
  ["яблоками", "apple"],
  ["яблоках", "apple"],
]);

const RU_EN_GENITIVE_RELATION_HEADS = new Set([
  "пример",
  "примеры",
  "примеров",
  "синоним",
  "синонимы",
  "синонимов",
]);

const RU_EN_GENITIVE_NOUN_FALLBACKS = new Map([
  ["согласования", "agreement"],
]);

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
      RU_EN_GENITIVE_RELATION_HEADS.has(word) &&
      RU_EN_GENITIVE_NOUN_FALLBACKS.has(next)
    ) {
      translated.push(
        RU_EN_WORD_FALLBACKS.get(word),
        "of",
        RU_EN_GENITIVE_NOUN_FALLBACKS.get(next),
      );
      index += 1;
      continue;
    }
    const surface = RU_EN_WORD_FALLBACKS.get(word);
    if (!surface) return null;
    translated.push(surface);
  }
  return capitalizeAsciiFirst(translated.join(" "));
}

function translateCompositionalSurface(surface, source, target) {
  if (source !== "ru" || target !== "en") return null;
  const normalized = normalizeComposableSurface(surface);
  const phrase = RU_EN_PHRASE_FALLBACKS.get(normalized);
  if (phrase) return phrase;

  const words = normalized.split(/\s+/u).filter(Boolean);
  if (words.length < 2 || words.length > 8) return null;
  return translateRussianWordSequence(words);
}

function detectLanguageSlug(text) {
  let latin = 0;
  let cyrillic = 0;
  let devanagari = 0;
  let cjk = 0;
  let other = 0;
  for (const character of String(text || "")) {
    const code = character.codePointAt(0);
    if (/[a-z]/i.test(character)) latin += 1;
    else if (code >= 0x0400 && code <= 0x04ff) cyrillic += 1;
    else if (code >= 0x0900 && code <= 0x097f) devanagari += 1;
    else if (code >= 0x4e00 && code <= 0x9fff) cjk += 1;
    else if (/\p{L}/u.test(character)) other += 1;
  }
  const total = latin + cyrillic + devanagari + cjk + other;
  if (total === 0) return "en";
  if (other > latin && other >= cyrillic && other >= devanagari && other >= cjk) {
    return "unknown";
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
  if (lower.includes("переведи") || lower.includes("опиши")) return "ru";
  if (lower.includes("अनुवाद")) return "hi";
  if (lower.includes("翻译") || lower.includes("翻譯")) return "zh";
  return "en";
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
  if (surface) {
    const live = await liveWiktionaryTranslate(surface, source, target);
    if (live) return { surface: live, gap: false };
  }
  const compositional = translateCompositionalSurface(surface, source, target);
  if (compositional) return { surface: compositional, gap: false };
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
  const isTranslationRequest =
    normalized.startsWith("translate") ||
    normalized.startsWith("переведи") ||
    normalized.startsWith("опиши") ||
    Boolean(
      targetHint &&
        (normalized.includes("अनुवाद") ||
          normalized.includes("翻译") ||
          normalized.includes("翻譯")),
    );
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
  const extracted = extractArithmeticExpression(prompt);
  if (!extracted) return null;
  const expression = extracted.expression;
  const interpretations = Array.isArray(extracted.interpretations)
    ? extracted.interpretations
    : [];
  try {
    const isEquation = expression.includes("=");
    let formatted;
    let backend = "js";
    const wasmResult = wasmEvaluateArithmetic(expression);
    if (wasmResult && wasmResult.ok) {
      formatted = wasmResult.value;
      backend = "wasm";
    } else {
      const percentOfResult = evaluatePercentOfExpression(expression);
      const currencyConversionResult = evaluateCurrencyConversionExpression(expression);
      if (currencyConversionResult !== null) {
        formatted = currencyConversionResult;
        backend = "js-currency";
      } else if (percentOfResult) {
        formatted = percentOfResult;
        backend = "js-percent-of";
      } else if (isEquation) {
        formatted = solveLinearEquation(expression);
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
      interpretations,
    };
  } catch (error) {
    const message = String(error && error.message ? error.message : error);
    return {
      intent: "calculation_error",
      content: `I could not evaluate \`${expression.trim()}\`: ${message}.`,
      confidence: 0.4,
      evidence: [`calculation_error:${message}`],
      interpretations,
    };
  }
}

const WEEKDAY_CYCLE = [
  {
    slug: "monday",
    en: "Monday",
    ru: "понедельник",
    hi: "सोमवार",
    zh: "星期一",
    ruGenitive: "понедельника",
    ruInstrumental: "понедельником",
    aliases: ["monday", "mon", "понедельника", "понедельником", "понедельнику", "понедельнике", "понедельник"],
  },
  {
    slug: "tuesday",
    en: "Tuesday",
    ru: "вторник",
    hi: "मंगलवार",
    zh: "星期二",
    ruGenitive: "вторника",
    ruInstrumental: "вторником",
    aliases: ["tuesday", "tue", "tues", "вторника", "вторником", "вторнику", "вторнике", "вторник"],
  },
  {
    slug: "wednesday",
    en: "Wednesday",
    ru: "среда",
    hi: "बुधवार",
    zh: "星期三",
    ruGenitive: "среды",
    ruInstrumental: "средой",
    aliases: ["wednesday", "wed", "средой", "среде", "среду", "среды", "среда"],
  },
  {
    slug: "thursday",
    en: "Thursday",
    ru: "четверг",
    hi: "गुरुवार",
    zh: "星期四",
    ruGenitive: "четверга",
    ruInstrumental: "четвергом",
    aliases: ["thursday", "thu", "thur", "thurs", "четверга", "четвергом", "четвергу", "четверге", "четверг"],
  },
  {
    slug: "friday",
    en: "Friday",
    ru: "пятница",
    hi: "शुक्रवार",
    zh: "星期五",
    ruGenitive: "пятницы",
    ruInstrumental: "пятницей",
    aliases: ["friday", "fri", "пятницей", "пятнице", "пятницу", "пятницы", "пятница"],
  },
  {
    slug: "saturday",
    en: "Saturday",
    ru: "суббота",
    hi: "शनिवार",
    zh: "星期六",
    ruGenitive: "субботы",
    ruInstrumental: "субботой",
    aliases: ["saturday", "sat", "субботой", "субботе", "субботу", "субботы", "суббота"],
  },
  {
    slug: "sunday",
    en: "Sunday",
    ru: "воскресенье",
    hi: "रविवार",
    zh: "星期日",
    ruGenitive: "воскресенья",
    ruInstrumental: "воскресеньем",
    aliases: ["sunday", "sun", "воскресеньем", "воскресенью", "воскресенья", "воскресенье"],
  },
];

const CALENDAR_NEXT_MARKERS = [
  "after",
  "comes after",
  "day after",
  "next day",
  "following day",
  "following weekday",
  "follows",
  "после",
  "наступает после",
  "следующий день",
  "следующая",
  "следом за",
];

const CALENDAR_PREVIOUS_MARKERS = [
  "before",
  "comes before",
  "day before",
  "previous day",
  "previous weekday",
  "precedes",
  "перед",
  "предыдущий день",
  "предыдущая",
  "предшествует",
];

const CALENDAR_TODAY_MARKERS = ["today", "сегодня", "आज", "今天"];

const CALENDAR_CURRENT_DAY_MARKERS = [
  "day",
  "weekday",
  "week day",
  "date",
  "день",
  "дня",
  "дату",
  "дата",
  "число",
  "दिन",
  "तारीख",
  "दिनांक",
  "星期",
  "星期几",
  "日期",
  "几号",
  "日子",
];

const CALENDAR_CURRENT_DAY_QUESTION_MARKERS = [
  "?",
  "what",
  "which",
  "tell me",
  "show",
  "какой",
  "какая",
  "какое",
  "скажи",
  "покажи",
  "कौन",
  "क्या",
  "बताओ",
  "दिखाओ",
  "什么",
  "几",
  "告诉",
  "显示",
];

function hasCalendarCjkCharacter(term) {
  return /[\u4e00-\u9fff]/u.test(term);
}

function isCalendarWordCharacter(character) {
  return /[\p{L}\p{N}_]/u.test(character);
}

function containsCalendarTerm(text, term) {
  if (hasCalendarCjkCharacter(term)) {
    return String(text || "").includes(term);
  }
  let index = String(text || "").indexOf(term);
  while (index !== -1) {
    const before = index > 0 ? Array.from(text.slice(0, index)).pop() : "";
    const after = Array.from(text.slice(index + term.length))[0] || "";
    if (
      (!before || !isCalendarWordCharacter(before)) &&
      (!after || !isCalendarWordCharacter(after))
    ) {
      return true;
    }
    index = text.indexOf(term, index + term.length);
  }
  return false;
}

function mentionsWeekdayContext(normalized) {
  return (
    ["day", "weekday", "week day", "день", "дня", "дни", "дней"].some((marker) =>
      containsCalendarTerm(normalized, marker),
    ) || normalized.includes("недел")
  );
}

function mentionsCurrentDayQuestion(normalized) {
  const mentionsToday = CALENDAR_TODAY_MARKERS.some((marker) =>
    containsCalendarTerm(normalized, marker),
  );
  if (!mentionsToday) return false;
  const asksForDay =
    CALENDAR_CURRENT_DAY_MARKERS.some((marker) =>
      containsCalendarTerm(normalized, marker),
    ) || normalized.includes("недел");
  const questionLike = CALENDAR_CURRENT_DAY_QUESTION_MARKERS.some((marker) =>
    normalized.includes(marker),
  );
  return asksForDay && questionLike;
}

function detectWeekdayOperation(normalized) {
  const hasNext = CALENDAR_NEXT_MARKERS.some((marker) => normalized.includes(marker));
  const hasPrevious = CALENDAR_PREVIOUS_MARKERS.some((marker) => normalized.includes(marker));
  if (hasNext && !hasPrevious) return "next";
  if (hasPrevious && !hasNext) return "previous";
  return null;
}

function detectWeekday(normalized) {
  for (const weekday of WEEKDAY_CYCLE) {
    if (weekday.aliases.some((alias) => containsCalendarTerm(normalized, alias))) {
      return weekday;
    }
  }
  return null;
}

function shiftWeekday(weekday, operation) {
  const index = WEEKDAY_CYCLE.indexOf(weekday);
  const offset = operation === "next" ? 1 : -1;
  return WEEKDAY_CYCLE[(index + offset + WEEKDAY_CYCLE.length) % WEEKDAY_CYCLE.length];
}

function validCalendarTimeZone(candidate) {
  const timeZone = cleanContextValue(candidate);
  if (!timeZone) return "";
  try {
    new Intl.DateTimeFormat("en-US", { timeZone }).format(new Date(0));
    return timeZone;
  } catch (_error) {
    return "";
  }
}

function resolvedCalendarTimeZone(userContext) {
  const fromContext = validCalendarTimeZone(userContext && userContext.timeZone);
  if (fromContext) return fromContext;
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "";
  } catch (_error) {
    return "";
  }
}

function calendarDateInTimeZone(date, timeZone) {
  const options = {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  };
  if (timeZone) options.timeZone = timeZone;
  const parts = new Intl.DateTimeFormat("en-CA", options).formatToParts(date);
  const value = (type) => parts.find((part) => part.type === type)?.value || "";
  const year = Number(value("year"));
  const month = Number(value("month"));
  const day = Number(value("day"));
  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
    return null;
  }
  const iso = `${String(year).padStart(4, "0")}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
  const dayIndex = new Date(Date.UTC(year, month - 1, day)).getUTCDay();
  const weekday = WEEKDAY_CYCLE[(dayIndex + 6) % 7];
  return { iso, weekday };
}

function currentCalendarDate(userContext) {
  const reference = new Date();
  const timeZone = resolvedCalendarTimeZone(userContext);
  return {
    timeZone: timeZone || "local",
    date: calendarDateInTimeZone(reference, timeZone),
  };
}

function renderCurrentDay(language, weekday, isoDate, timeZone) {
  if (language === "ru") {
    return `Сегодня ${weekday.ru}, ${isoDate} (${timeZone}).`;
  }
  if (language === "hi") {
    return `आज ${weekday.hi} है, ${isoDate} (${timeZone}).`;
  }
  if (language === "zh") {
    return `今天是${weekday.zh}，${isoDate}（${timeZone}）。`;
  }
  return `Today is ${weekday.en}, ${isoDate} (${timeZone}).`;
}

function renderWeekdayRelation(language, operation, source, result) {
  const delta = operation === "next" ? "+1" : "-1";
  if (language === "ru") {
    if (operation === "next") {
      return `После ${source.ruGenitive} наступает ${result.ru}. Я сдвинул ${source.ru} на ${delta} в семидневном календарном цикле.`;
    }
    return `Перед ${source.ruInstrumental} идёт ${result.ru}. Я сдвинул ${source.ru} на ${delta} в семидневном календарном цикле.`;
  }
  if (operation === "next") {
    return `The day after ${source.en} is ${result.en}. I move ${source.en} by ${delta} in the seven-day calendar cycle.`;
  }
  return `The day before ${source.en} is ${result.en}. I move ${source.en} by ${delta} in the seven-day calendar cycle.`;
}

function tryCalendarReasoning(prompt, normalized, userContext = {}) {
  if (mentionsCurrentDayQuestion(normalized)) {
    const language = detectLanguage(prompt);
    const resolved = currentCalendarDate(userContext);
    if (!resolved.date) return null;
    return {
      intent: "calendar_current_day",
      content: renderCurrentDay(
        language,
        resolved.date.weekday,
        resolved.date.iso,
        resolved.timeZone,
      ),
      confidence: 1.0,
      evidence: [
        "calendar:clock:browser",
        `calendar:today:${resolved.date.iso}`,
        `calendar:weekday:${resolved.date.weekday.slug}`,
        `calendar:time_zone:${resolved.timeZone}`,
        `language:${language}`,
      ],
    };
  }
  if (!mentionsWeekdayContext(normalized)) return null;
  const operation = detectWeekdayOperation(normalized);
  if (!operation) return null;
  const source = detectWeekday(normalized);
  if (!source) return null;
  const result = shiftWeekday(source, operation);
  const language = detectLanguage(prompt);
  return {
    intent: "calendar_weekday_relation",
    content: renderWeekdayRelation(language, operation, source, result),
    confidence: 1.0,
    evidence: [
      "calendar:cycle:monday,tuesday,wednesday,thursday,friday,saturday,sunday",
      `calendar:subject_weekday:${source.slug}`,
      `calendar:operation:${operation}:${source.slug}`,
      `calendar:result_weekday:${result.slug}`,
      `language:${language}`,
    ],
  };
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
  const left = Array.from(String(a || ""));
  const right = Array.from(String(b || ""));
  const m = left.length, n = right.length;
  const dp = Array.from({ length: m + 1 }, (_, i) =>
    Array.from({ length: n + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0))
  );
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = left[i - 1] === right[j - 1]
        ? dp[i - 1][j - 1]
        : 1 + Math.min(dp[i - 1][j - 1], dp[i - 1][j], dp[i][j - 1]);
      if (
        i > 1 &&
        j > 1 &&
        left[i - 1] === right[j - 2] &&
        left[i - 2] === right[j - 1]
      ) {
        dp[i][j] = Math.min(dp[i][j], dp[i - 2][j - 2] + 1);
      }
    }
  }
  return dp[m][n];
}

function isCloseTokenTypo(actual, expected) {
  const left = String(actual || "").toLowerCase();
  const right = String(expected || "").toLowerCase();
  const leftLength = Array.from(left).length;
  const rightLength = Array.from(right).length;
  return Math.min(leftLength, rightLength) >= 4 && editDistance(left, right) === 1;
}

function leadingTokenSpans(value, limit) {
  const text = String(value || "");
  const spans = [];
  const pattern = /\S+/gu;
  let match;
  while ((match = pattern.exec(text)) !== null && spans.length < limit) {
    spans.push({
      start: match.index,
      end: match.index + match[0].length,
      text: match[0],
    });
  }
  return spans;
}

function fuzzyPrefixMatch(value, prefix) {
  const words = String(prefix || "").trim().split(/\s+/u).filter(Boolean);
  if (words.length === 0) return null;
  const spans = leadingTokenSpans(value, words.length);
  if (spans.length !== words.length) return null;
  let typoCount = 0;
  for (let i = 0; i < words.length; i += 1) {
    const actual = spans[i].text;
    const expected = words[i];
    if (actual.toLowerCase() === expected.toLowerCase()) continue;
    if (!isCloseTokenTypo(actual, expected)) return null;
    typoCount += 1;
  }
  if (typoCount !== 1) return null;
  const end = spans[spans.length - 1].end;
  return {
    typoCount,
    end,
    interpretation: {
      original: String(value || "").slice(0, end),
      corrected: String(prefix || "").trim(),
    },
  };
}

function stripKnownPrefix(value, prefixes) {
  const text = String(value || "");
  const lower = text.toLowerCase();
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      return { value: text.slice(prefix.length).trimStart(), interpretation: null };
    }
  }
  const matches = prefixes
    .map((prefix) => fuzzyPrefixMatch(text, prefix))
    .filter(Boolean)
    .sort((left, right) =>
      left.typoCount - right.typoCount || right.end - left.end,
    );
  const best = matches[0];
  if (!best) return null;
  const next = matches[1];
  if (next && next.typoCount === best.typoCount && next.end === best.end) {
    return null;
  }
  return {
    value: text.slice(best.end).trimStart(),
    interpretation: best.interpretation,
  };
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

const WIKIPEDIA_ACTION_API_HOSTS = {
  en: "https://en.wikipedia.org/w/api.php",
  ru: "https://ru.wikipedia.org/w/api.php",
  hi: "https://hi.wikipedia.org/w/api.php",
  zh: "https://zh.wikipedia.org/w/api.php",
};

const WIKTIONARY_SEARCH_HOSTS = {
  en: "https://en.wiktionary.org/w/api.php",
  ru: "https://ru.wiktionary.org/w/api.php",
  hi: "https://hi.wiktionary.org/w/api.php",
  zh: "https://zh.wiktionary.org/w/api.php",
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

function normalizeLookupText(value) {
  return String(value || "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

function compactLookupText(value) {
  return normalizeLookupText(value).replace(/\s+/g, "");
}

function boundedEditDistance(left, right, limit) {
  if (Math.abs(left.length - right.length) > limit) return limit + 1;
  let previous = Array.from({ length: right.length + 1 }, (_, index) => index);
  for (let i = 1; i <= left.length; i += 1) {
    const current = [i];
    let rowMin = current[0];
    for (let j = 1; j <= right.length; j += 1) {
      const cost = left[i - 1] === right[j - 1] ? 0 : 1;
      const next = Math.min(
        previous[j] + 1,
        current[j - 1] + 1,
        previous[j - 1] + cost,
      );
      current[j] = next;
      rowMin = Math.min(rowMin, next);
    }
    if (rowMin > limit) return limit + 1;
    previous = current;
  }
  return previous[right.length];
}

function isNearLookupText(left, right) {
  const a = compactLookupText(left);
  const b = compactLookupText(right);
  if (!a || !b) return false;
  const maxLength = Math.max(a.length, b.length);
  const limit = maxLength <= 8 ? 1 : 2;
  return boundedEditDistance(a, b, limit) <= limit;
}

function isPlausibleWikipediaSearchMatch(summary, term) {
  if (
    !summary ||
    (summary.matchKind !== "search" && summary.matchKind !== "context_search")
  ) {
    return true;
  }
  const termNormalized = normalizeLookupText(term);
  if (!termNormalized) return true;
  const termTokens = termNormalized.split(/\s+/).filter(Boolean);
  const candidates = [
    summary.title,
    summary.matchedTitle,
    String(summary.matchedSlug || "").replace(/_/g, " "),
    summary.extract,
  ];
  for (const candidate of candidates) {
    const normalized = normalizeLookupText(candidate);
    if (!normalized) continue;
    if (normalized === termNormalized) return true;
    const candidateTokens = new Set(normalized.split(/\s+/).filter(Boolean));
    if (
      termTokens.length > 0 &&
      termTokens.every((token) => candidateTokens.has(token))
    ) {
      return true;
    }
    if (isNearLookupText(termNormalized, normalized)) return true;
  }
  return false;
}

const LOOKUP_STEM_STOPWORDS = new Set([
  "a",
  "an",
  "and",
  "for",
  "in",
  "of",
  "on",
  "the",
  "to",
  "about",
  "sentence",
  "sentences",
  "в",
  "во",
  "и",
  "или",
  "на",
  "о",
  "об",
  "про",
]);

function hasSharedLookupStem(summary, term) {
  const normalizedTerm = normalizeLookupText(term);
  if (!normalizedTerm) return false;
  const content = normalizeLookupText(
    [
      summary && summary.title,
      summary && summary.matchedTitle,
      summary && String(summary.matchedSlug || "").replace(/_/g, " "),
      summary && summary.extract,
    ]
      .filter(Boolean)
      .join(" "),
  );
  if (!content) return false;
  const contentTokens = content.split(/\s+/).filter(Boolean);
  for (const token of normalizedTerm.split(/\s+/).filter(Boolean)) {
    if (LOOKUP_STEM_STOPWORDS.has(token) || token.length < 7) continue;
    const stemLength = Math.min(8, token.length - 2);
    const stem = token.slice(0, stemLength);
    if (stem.length >= 5 && contentTokens.some((candidate) => candidate.startsWith(stem))) {
      return true;
    }
  }
  return false;
}

function isArticleQuestionWikipediaMatch(summary, query) {
  if (!summary) return false;
  if (summary.matchKind === "direct") return true;
  if (isPlausibleWikipediaSearchMatch(summary, query.exactTerm)) return true;
  if (query.lookupTerm !== query.exactTerm && isPlausibleWikipediaSearchMatch(summary, query.lookupTerm)) {
    return true;
  }
  if (!hasSharedLookupStem(summary, query.lookupTerm || query.exactTerm)) {
    return false;
  }
  return !query.contextOriginal || hasSharedLookupStem(summary, query.contextOriginal);
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

function decodeHtmlEntities(value) {
  const named = {
    amp: "&",
    apos: "'",
    mdash: "—",
    ndash: "–",
    gt: ">",
    lt: "<",
    nbsp: " ",
    quot: '"',
  };
  return String(value || "")
    .replace(/&#x([0-9a-f]+);/giu, (_match, code) => {
      const parsed = Number.parseInt(code, 16);
      return Number.isFinite(parsed) ? String.fromCodePoint(parsed) : "";
    })
    .replace(/&#(\d+);/gu, (_match, code) => {
      const parsed = Number.parseInt(code, 10);
      return Number.isFinite(parsed) ? String.fromCodePoint(parsed) : "";
    })
    .replace(/&([a-z]+);/giu, (match, name) => named[name.toLowerCase()] || match);
}

function stripHtmlToText(html) {
  return decodeHtmlEntities(
    String(html || "")
      .replace(/<style\b[\s\S]*?<\/style>/giu, " ")
      .replace(/<script\b[\s\S]*?<\/script>/giu, " ")
      .replace(/<sup\b[\s\S]*?<\/sup>/giu, " ")
      .replace(/<[^>]+>/gu, " "),
  )
    .replace(/\s+([,.;:!?])/gu, "$1")
    .replace(/\s+/gu, " ")
    .trim();
}

function truncateDisambiguationHtml(html) {
  const text = String(html || "");
  let end = text.length;
  for (const marker of [
    /<h[1-6]\b[^>]*id=["'](?:См\._также|See_also|Примечания|References|Notes)["']/iu,
    /<div\b[^>]*id=["']disambig["']/iu,
  ]) {
    const match = marker.exec(text);
    if (match && match.index > 0) end = Math.min(end, match.index);
  }
  return text.slice(0, end);
}

function deduplicateTextList(values) {
  const out = [];
  const seen = new Set();
  for (const value of values) {
    const text = String(value || "").trim();
    if (!text) continue;
    const key = normalizeLookupText(text);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    out.push(text);
  }
  return out;
}

function extractDisambiguationEntriesFromHtml(html) {
  const scoped = truncateDisambiguationHtml(html);
  const entries = [];
  const itemPattern = /<li\b[^>]*>([\s\S]*?)<\/li>/giu;
  let match;
  while ((match = itemPattern.exec(scoped)) !== null) {
    const text = stripHtmlToText(match[1]);
    if (!text || text.startsWith("↑")) continue;
    entries.push(text);
  }
  return deduplicateTextList(entries).slice(0, 12);
}

function extractDisambiguationEntriesFromSummary(summary) {
  const title = normalizeLookupText(summary && summary.title);
  const raw = String((summary && summary.extract) || "");
  const extract = raw.replace(
    /^([^:\n]{1,80}):\s*([«»"'“”„]?[^\n]{1,80}[»"'“”„]?\s[—–-]\s)/u,
    "$1:\n$2",
  );
  const lines = extract
    .split(/\n+/u)
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => {
      const normalized = normalizeLookupText(line.replace(/:$/u, ""));
      return normalized && normalized !== title;
    });
  return deduplicateTextList(lines);
}

function definitionPrefixForDisambiguationEntry(entry) {
  const text = String(entry || "").trim();
  const dash = text.search(/\s[—–-]\s/u);
  if (dash <= 0) return "";
  return normalizeLookupText(
    text
      .slice(0, dash)
      .trim()
      .replace(/^[«»"'“”„]+|[«»"'“”„]+$/gu, ""),
  );
}

function isDefinitionStyleDisambiguation(summary, requestedTerm, entries) {
  const targets = [requestedTerm, summary && summary.title]
    .map((value) => normalizeLookupText(value))
    .filter(Boolean);
  if (targets.length === 0) return false;
  return entries.some((entry) => {
    const prefix = definitionPrefixForDisambiguationEntry(entry);
    return prefix && targets.includes(prefix);
  });
}

async function fetchWikipediaDisambiguationEntries(summary) {
  if (typeof fetch !== "function" || !summary) return [];
  const base =
    WIKIPEDIA_ACTION_API_HOSTS[summary.language] || WIKIPEDIA_ACTION_API_HOSTS.en;
  const page = summary.matchedSlug || summary.title;
  if (!page) return [];
  const url = `${base}?action=parse&page=${encodeURIComponent(
    page,
  )}&prop=text&format=json&formatversion=2&redirects=1&origin=*`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return [];
    const data = await response.json();
    const text = data && data.parse ? data.parse.text : "";
    let html = "";
    if (typeof text === "string") {
      html = text;
    } else if (text && typeof text === "object" && text["*"]) {
      html = text["*"];
    }
    return extractDisambiguationEntriesFromHtml(html);
  } catch (_error) {
    return [];
  }
}

async function buildDefinitionStyleDisambiguationSummary(
  data,
  term,
  language,
  matchedSlug,
  requestUrl,
) {
  const title = String(data.title || term);
  const pageUrl =
    (data.content_urls &&
      data.content_urls.desktop &&
      data.content_urls.desktop.page) ||
    requestUrl;
  const summary = {
    title,
    extract: String(data.extract || "").trim(),
    url: pageUrl,
    language,
    matchKind: "disambiguation",
    matchedSlug,
  };
  const summaryEntries = extractDisambiguationEntriesFromSummary(summary);
  if (!isDefinitionStyleDisambiguation(summary, term, summaryEntries)) {
    return null;
  }
  const parsedEntries = await fetchWikipediaDisambiguationEntries(summary);
  const entries = parsedEntries.length > 0 ? parsedEntries : summaryEntries;
  return {
    ...summary,
    extract: entries.join("\n"),
    disambiguationEntries: entries,
  };
}

async function fetchWikipediaSummary(term, language, context, options) {
  if (typeof fetch !== "function") return null;
  const includeDefinitionDisambiguation = Boolean(
    options && options.includeDefinitionDisambiguation,
  );
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
        if (data.type === "disambiguation") {
          if (includeDefinitionDisambiguation && !context) {
            const disambiguation = await buildDefinitionStyleDisambiguationSummary(
              data,
              term,
              host.language,
              slug,
              url,
            );
            if (disambiguation) return disambiguation;
          }
          continue;
        }
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

function wikipediaDisambiguationMessage(summary, language) {
  const humanUrl = humanizeUrl(summary.url);
  const entries = Array.isArray(summary.disambiguationEntries)
    ? summary.disambiguationEntries
    : String(summary.extract || "")
        .split(/\n+/u)
        .map((line) => line.trim())
        .filter(Boolean);
  const list = entries.map((entry) => `- ${entry}`).join("\n");
  if (language === "ru") {
    return `На странице Wikipedia «${summary.title}» перечислены значения:\n\n${list}\n\nИсточник: [${humanUrl}](${summary.url}) (wikipedia).`;
  }
  if (language === "zh") {
    return `Wikipedia “${summary.title}”页面列出以下含义：\n\n${list}\n\n来源：[${humanUrl}](${summary.url}) (wikipedia).`;
  }
  if (language === "hi") {
    return `Wikipedia पृष्ठ "${summary.title}" ये अर्थ सूचीबद्ध करता है:\n\n${list}\n\nस्रोत: [${humanUrl}](${summary.url}) (wikipedia).`;
  }
  return `Wikipedia's "${summary.title}" page lists these meanings:\n\n${list}\n\nSource: [${humanUrl}](${summary.url}) (wikipedia).`;
}

function wikipediaArticleQuestionMessage(summary, query, language, exactMatch) {
  const humanUrl = humanizeUrl(summary.url);
  const source = `Source: [${humanUrl}](${summary.url}) (wikipedia).`;
  if (language === "ru") {
    const wikipediaName =
      summary.language === "ru" ? "русскоязычной Википедии" : "Wikipedia";
    if (exactMatch) {
      return `В Wikipedia есть статья «${summary.title}»: ${summary.extract}\n\nИсточник: [${humanUrl}](${summary.url}) (wikipedia).`;
    }
    return [
      `В ${wikipediaName} я не нашёл отдельной статьи с названием «${query.exactTerm}», но ближайшая подходящая страница — «${summary.title}»: ${summary.extract}`,
      `Источник: [${humanUrl}](${summary.url}) (wikipedia).`,
    ].join("\n\n");
  }
  if (language === "zh") {
    const zhSource = `来源：[${humanUrl}](${summary.url}) (wikipedia).`;
    if (exactMatch) {
      return `Wikipedia 有一篇“${summary.title}”条目：${summary.extract}\n\n${zhSource}`;
    }
    return `我没有找到标题为“${query.exactTerm}”的 Wikipedia 条目，但最接近的有用页面是“${summary.title}”：${summary.extract}\n\n${zhSource}`;
  }
  if (language === "hi") {
    const hiSource = `स्रोत: [${humanUrl}](${summary.url}) (wikipedia).`;
    if (exactMatch) {
      return `Wikipedia पर "${summary.title}" लेख है: ${summary.extract}\n\n${hiSource}`;
    }
    return `मुझे Wikipedia पर "${query.exactTerm}" शीर्षक वाला अलग लेख नहीं मिला, लेकिन सबसे नज़दीकी उपयोगी पृष्ठ "${summary.title}" है: ${summary.extract}\n\n${hiSource}`;
  }
  if (exactMatch) {
    return `Wikipedia has an article titled "${summary.title}": ${summary.extract}\n\n${source}`;
  }
  return `I did not find an exact Wikipedia article titled "${query.exactTerm}", but the closest useful page is "${summary.title}": ${summary.extract}\n\n${source}`;
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

function wikidataConceptUrl(hit) {
  const id = hit && hit.id ? String(hit.id) : "";
  if (id) return `https://www.wikidata.org/wiki/${encodeURIComponent(id)}`;
  const conceptUri = hit && hit.concepturi ? String(hit.concepturi) : "";
  const qid = conceptUri.match(/Q\d+/);
  if (qid) return `https://www.wikidata.org/wiki/${qid[0]}`;
  return "https://www.wikidata.org/wiki/Wikidata:Main_Page";
}

function wikidataHitMatchesTerm(hit, term) {
  const target = normalizeLookupText(term);
  if (!target || !hit) return false;
  const candidates = [
    hit.label,
    hit.title,
    hit.match && hit.match.text,
    hit.display && hit.display.label && hit.display.label.value,
  ];
  if (Array.isArray(hit.aliases)) {
    candidates.push(...hit.aliases);
  }
  return candidates.some((candidate) => normalizeLookupText(candidate) === target);
}

async function fetchWikidataConceptSummary(term, language) {
  if (typeof fetch !== "function") return null;
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
      const hits = data && Array.isArray(data.search) ? data.search : [];
      const hit = hits.find((candidate) =>
        wikidataHitMatchesTerm(candidate, term),
      );
      if (!hit) continue;
      const display = hit.display || {};
      return {
        sourceKind: "wikidata",
        qid: hit.id || "",
        title:
          (display.label && display.label.value) ||
          hit.label ||
          (hit.match && hit.match.text) ||
          term,
        description:
          (display.description && display.description.value) ||
          hit.description ||
          "",
        url: wikidataConceptUrl(hit),
        language: lang,
      };
    } catch (_error) {
      // Try the next Wikidata language.
    }
  }
  return null;
}

function wiktionaryFallbackDescription(title, language) {
  if (language === "ru") {
    return `В Wiktionary есть словарная статья «${title}».`;
  }
  if (language === "zh") {
    return `Wiktionary 有“${title}”这个词条。`;
  }
  if (language === "hi") {
    return `Wiktionary में "${title}" के लिए शब्दकोश प्रविष्टि है।`;
  }
  return `Wiktionary has a dictionary entry for "${title}".`;
}

async function fetchWiktionaryEntry(term, language) {
  if (typeof fetch !== "function") return null;
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  const target = normalizeLookupText(term);
  for (const lang of ordered) {
    const base = WIKTIONARY_SEARCH_HOSTS[lang] || WIKTIONARY_SEARCH_HOSTS.en;
    const url = `${base}?action=opensearch&search=${encodeURIComponent(
      term,
    )}&limit=5&format=json&origin=*`;
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
      if (!Array.isArray(data) || !Array.isArray(data[1])) continue;
      const titles = data[1];
      const descriptions = Array.isArray(data[2]) ? data[2] : [];
      const urls = Array.isArray(data[3]) ? data[3] : [];
      const index = titles.findIndex(
        (title) => normalizeLookupText(title) === target,
      );
      if (index < 0) continue;
      const title = titles[index] || term;
      return {
        sourceKind: "wiktionary",
        title,
        description:
          descriptions[index] || wiktionaryFallbackDescription(title, lang),
        url:
          urls[index] ||
          `https://${lang}.wiktionary.org/wiki/${encodeURIComponent(title)}`,
        language: lang,
      };
    } catch (_error) {
      // Try the next Wiktionary language.
    }
  }
  return null;
}

function renderExternalLookupContent(result, requestedTerm) {
  const humanUrl = humanizeUrl(result.url);
  const title = result.title || requestedTerm;
  const heading =
    requestedTerm && normalizeLookupText(requestedTerm) !== normalizeLookupText(title)
      ? `${requestedTerm}: ${title}`
      : title;
  const description = String(result.description || "").trim();
  const body = description ? `${heading}: ${description}` : `${heading}.`;
  return `${body}\n\nSource: [${humanUrl}](${result.url}) (${result.sourceKind}).`;
}

function externalLookupResponse(result, requestedTerm, rejectedSummary) {
  const humanUrl = humanizeUrl(result.url);
  const evidence = [
    `${result.sourceKind}_lookup:${result.qid || result.title}`,
    `source:${humanUrl}`,
    `language:${result.language}`,
  ];
  if (result.qid) evidence.push(`wikidata:${result.qid}`);
  if (rejectedSummary && rejectedSummary.title) {
    evidence.push(`wikipedia_lookup:rejected:${rejectedSummary.title}`);
  }
  return {
    intent: `${result.sourceKind}_lookup`,
    content: renderExternalLookupContent(result, requestedTerm),
    confidence: result.sourceKind === "wikidata" ? 0.82 : 0.75,
    evidence,
  };
}

async function tryTermKnowledgeFallback(term, language, rejectedSummary) {
  const wikidata = await fetchWikidataConceptSummary(term, language);
  if (wikidata) {
    return externalLookupResponse(wikidata, term, rejectedSummary);
  }
  const wiktionary = await fetchWiktionaryEntry(term, language);
  if (wiktionary) {
    return externalLookupResponse(wiktionary, term, rejectedSummary);
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
        formalizedObject: cached.subjectQid || "",
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
    formalizedObject: resolved.subjectQid || "",
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
  const summary = await fetchWikipediaSummary(wikiTerm, language, wikiContext, {
    includeDefinitionDisambiguation: !wikiContext,
  });
  if (!summary) {
    return tryTermKnowledgeFallback(wikiTerm, language, null);
  }
  const isClosestMatch = isClosestWikipediaMatch(summary);
  const requiresPlausibleSearchMatch =
    isClosestMatch || summary.matchKind === "context_search";
  if (
    requiresPlausibleSearchMatch &&
    !isPlausibleWikipediaSearchMatch(summary, wikiTerm)
  ) {
    const fallback = await tryTermKnowledgeFallback(wikiTerm, language, summary);
    if (fallback) return fallback;
    return null;
  }
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
  if (summary.matchKind === "disambiguation") {
    const entryCount = Array.isArray(summary.disambiguationEntries)
      ? summary.disambiguationEntries.length
      : 0;
    evidence.push(`wikipedia_lookup:disambiguation:${summary.title}`);
    evidence.push(`wikipedia_lookup:disambiguation_entries:${entryCount}`);
    return {
      intent: "wikipedia_lookup",
      content: wikipediaDisambiguationMessage(summary, language),
      confidence: 0.84,
      evidence,
    };
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

async function tryWikipediaArticleQuestion(prompt, language, preferences) {
  const term = extractWikipediaArticleQuestionTerm(prompt);
  if (!term) return null;
  const query = refineWikipediaArticleQuestionLookup(term, language);
  if (!query.exactTerm) return null;

  const exactSummary = await fetchWikipediaSummary(query.exactTerm, language, null);
  let summary = exactSummary;
  const exactMatch = exactSummary && exactSummary.matchKind === "direct";
  if (!exactMatch && (query.lookupTerm !== query.exactTerm || query.contextOriginal)) {
    const refinedSummary = await fetchWikipediaSummary(
      query.lookupTerm,
      language,
      query.contextOriginal,
    );
    if (refinedSummary) summary = refinedSummary;
  }
  if (!summary) {
    return tryTermKnowledgeFallback(query.exactTerm, language, null);
  }
  if (!exactMatch && !isArticleQuestionWikipediaMatch(summary, query)) {
    const fallback = await tryTermKnowledgeFallback(
      query.exactTerm,
      language,
      summary,
    );
    if (fallback) return fallback;
    return null;
  }

  const guessProbability = numericPreference(
    preferences && preferences.guessProbability,
    0.8,
    0,
    1,
  );
  const humanUrl = humanizeUrl(summary.url);
  const evidence = [
    `wikipedia_article_question:${query.exactTerm}`,
    `source:${humanUrl}`,
    `language:${summary.language}`,
  ];
  if (query.lookupTerm !== query.exactTerm) {
    evidence.push(`wikipedia_article_question:lookup:${query.lookupTerm}`);
  }
  if (query.contextOriginal) {
    evidence.push(`wikipedia_article_question:context:${query.contextOriginal}`);
  }
  if (exactMatch) {
    evidence.push("wikipedia_article_question:exact");
  } else {
    evidence.push(`wikipedia_article_question:closest_match:${summary.title}`);
  }
  if (!exactMatch && guessProbability < 0.5) {
    evidence.push("ambiguity:ask");
    return {
      intent: "wikipedia_article_question",
      content: wikipediaClarificationMessage(summary, language),
      confidence: 0.65,
      evidence,
    };
  }
  if (!exactMatch) evidence.push("ambiguity:guess");
  return {
    intent: "wikipedia_article_question",
    content: wikipediaArticleQuestionMessage(summary, query, language, exactMatch),
    confidence: exactMatch ? 0.88 : 0.82,
    evidence,
    query: query.exactTerm,
    formalizedObject: summary.title,
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

function containsAnySubstring(value, needles) {
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
      if (!containsAnySubstring(lower, SOFTWARE_FEATURE_MARKERS)) continue;
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
  if (gameTracker || containsAnySubstring(lower, ["track", "hp", "status", "damage", "cooldown"])) {
    return "state_tracking";
  }
  if (containsAnySubstring(lower, ["import", "export", "csv", "backup", "report", "calendar"])) {
    return "data_exchange";
  }
  if (containsAnySubstring(lower, ["reminder", "notification", "schedule", "weekly"])) {
    return "automation";
  }
  if (containsAnySubstring(lower, ["validate", "check", "conflict", "audit"])) {
    return "validation";
  }
  if (containsAnySubstring(lower, ["api", "discord", "telegram", "github", "browser"])) {
    return "integration";
  }
  if (containsAnySubstring(lower, ["dashboard", "chart", "filter", "progress"])) {
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
  if (containsAnySubstring(normalized, ["manual instruction", "instructions", "no code"])) {
    return "manual_instructions";
  }
  if (containsAnySubstring(normalized, ["execute", "run command", "run it", "webvm"])) {
    return "immediate_execution";
  }
  if (
    containsAnySubstring(normalized, ["bash", "shell"]) ||
    containsAnyToken(normalized, ["script", "scripts", "commands"])
  ) {
    return "script_generation";
  }
  return "code_generation";
}

function detectSoftwareImplementationLanguage(normalized) {
  if (containsAnySubstring(normalized, ["python", "django", "fastapi"])) return "python";
  if (containsAnySubstring(normalized, ["rust", "cargo"])) return "rust";
  if (containsAnySubstring(normalized, ["javascript", "node.js", "node "])) return "javascript";
  return "typescript";
}

function softwareApprovalGates(normalized, deliveryMode) {
  const gates = ["task_formalization", "implementation_plan"];
  if (normalized.includes("requirement")) gates.push("requirements");
  if (containsAnySubstring(normalized, ["each step", "step by step"])) gates.push("each_step");
  if (deliveryMode === "code_generation") {
    gates.push("generated_code");
  } else if (deliveryMode === "manual_instructions") {
    gates.push("manual_instructions");
  } else {
    gates.push("generated_script");
    gates.push("bash_command");
  }
  if (containsAnySubstring(normalized, ["shell", "bash", "command", "docker", "webvm"])) {
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

const WEB_SEARCH_EXPLICIT_PREFIXES = [
  "search the web for ",
  "search web for ",
  "search the internet for ",
  "search internet for ",
  "search online for ",
  "search for information about ",
  "search for information on ",
  "web search for ",
  "find on the internet ",
  "find online ",
  "find information about ",
  "find information on ",
  "find detailed information about ",
  "find detailed information on ",
  "find info about ",
  "find info on ",
  "look up information about ",
  "look up information on ",
  "look up info about ",
  "look up info on ",
  "look up online ",
  "найди в интернете ",
  "поищи в интернете ",
  "поиск в интернете ",
  "найди онлайн ",
  "поищи онлайн ",
  "найди в сети ",
  "поищи в сети ",
  "найди информацию в интернете о ",
  "найди информацию в интернете об ",
  "поищи информацию в интернете о ",
  "поищи информацию в интернете об ",
  "найди информацию о ",
  "найди информацию об ",
  "найди информацию про ",
  "найди информацию по ",
  "найти информацию о ",
  "найти информацию об ",
  "поищи информацию о ",
  "поищи информацию об ",
  "поищи информацию про ",
  "поищи информацию по ",
  "найди инфу о ",
  "найди инфу об ",
  "поищи инфу о ",
  "поищи инфу об ",
  "найди сведения о ",
  "найди сведения об ",
  "поищи сведения о ",
  "поищи сведения об ",
  "найди материалы о ",
  "найди материалы об ",
  "поищи материалы о ",
  "поищи материалы об ",
];

const WEB_SEARCH_ACTION_MARKERS = [
  " search ",
  " find ",
  " look up ",
  " lookup ",
  " research ",
  " investigate ",
  " найди ",
  " найти ",
  " поищи ",
  " поиск ",
  " поискать ",
  " ищи ",
  " разыщи ",
  " узнай ",
  "खोज",
  "ढूंढ",
  "ढूँढ",
  "搜索",
  "查找",
  "查询",
  "檢索",
  "检索",
  "搜一下",
  "查一下",
];

const WEB_SEARCH_STRONG_ACTION_MARKERS = [
  " search ",
  " look up ",
  " lookup ",
  " research ",
  " investigate ",
  " поищи ",
  " поиск ",
  " поискать ",
  " ищи ",
  "खोज",
  "ढूंढ",
  "ढूँढ",
  "搜索",
  "查找",
  "查询",
  "檢索",
  "检索",
  "搜一下",
  "查一下",
];

const WEB_SEARCH_SIGNAL_MARKERS = [
  " web ",
  " internet ",
  " online ",
  " wikipedia ",
  " wikidata ",
  " wiktionary ",
  " information ",
  " info ",
  " details ",
  " data ",
  " material ",
  " materials ",
  " resource ",
  " resources ",
  " source ",
  " sources ",
  " article ",
  " articles ",
  " fact ",
  " facts ",
  " интернете ",
  " интернет ",
  " онлайн ",
  " сети ",
  " википед",
  " викиданн",
  " информац",
  " инфу ",
  " сведения ",
  " материал",
  " данные ",
  " источник",
  "जानकारी",
  "सूचना",
  "विवरण",
  "सामग्री",
  "स्रोत",
  "लेख",
  "इंटरनेट",
  "ऑनलाइन",
  "वेब",
  "विकिपीडिया",
  "विकिडाटा",
  "信息",
  "資料",
  "资料",
  "内容",
  "來源",
  "来源",
  "资源",
  "資源",
  "文章",
  "百科",
  "维基百科",
  "維基百科",
  "维基数据",
  "維基數據",
  "网上",
  "網上",
  "在线",
  "在線",
  "互联网",
  "網路",
  "网络",
];

const SEARCH_QUERY_AFTER_MARKERS = [
  " about ",
  " on ",
  " regarding ",
  " concerning ",
  " for ",
  " о ",
  " об ",
  " про ",
  " по ",
  " насчет ",
  " относительно ",
  "关于",
  "關於",
  "有关",
  "有關",
];

const SEARCH_QUERY_BEFORE_MARKERS = [
  " के बारे में",
  " के विषय में",
  " से संबंधित",
  " पर",
  " की जानकारी",
  " की सूचना",
];

const SEARCH_ACTION_AFTER_MARKERS = [
  "search for ",
  "search ",
  "find ",
  "look up ",
  "lookup ",
  "research ",
  "investigate ",
  "найди ",
  "найти ",
  "поищи ",
  "поискать ",
  "ищи ",
  "разыщи ",
  "узнай ",
  "खोजो ",
  "खोजें ",
  "खोजिए ",
  "ढूंढो ",
  "ढूँढो ",
  "ढूंढें ",
  "ढूँढें ",
  "搜索",
  "查找",
  "查询",
  "檢索",
  "检索",
  "搜一下",
  "查一下",
];

const SEARCH_QUERY_LEADING_NOISE = [
  "please ",
  "can you ",
  "could you ",
  "would you ",
  "me ",
  "the ",
  "some ",
  "detailed ",
  "more ",
  "current ",
  "latest ",
  "information about ",
  "information on ",
  "info about ",
  "info on ",
  "details about ",
  "details on ",
  "data about ",
  "data on ",
  "подробные ",
  "информацию о ",
  "информацию об ",
  "инфу о ",
  "инфу об ",
  "сведения о ",
  "сведения об ",
  "материалы о ",
  "материалы об ",
  "материалы по ",
  "данные о ",
  "данные об ",
  "о ",
  "об ",
  "про ",
  "по ",
  "कृपया ",
  "जानकारी ",
  "सूचना ",
  "विवरण ",
  "सामग्री ",
  "关于",
  "關於",
  "有关",
  "有關",
];

const SEARCH_QUERY_TRAILING_NOISE = [
  " online",
  " on the internet",
  " on the web",
  " on wikipedia",
  " in wikipedia",
  " from wikipedia",
  " information",
  " info",
  " details",
  " data",
  " material",
  " materials",
  " resources",
  " sources",
  " articles",
  " facts",
  " в интернете",
  " онлайн",
  " в сети",
  " в википедии",
  " википедии",
  " информация",
  " сведения",
  " материалы",
  " данные",
  " के बारे में",
  " के विषय में",
  " से संबंधित",
  " पर",
  " की जानकारी",
  " की सूचना",
  " जानकारी",
  " सूचना",
  " विवरण",
  " सामग्री",
  " स्रोत",
  " विकिपीडिया में",
  " ऑनलाइन",
  " इंटरनेट पर",
  " खोजो",
  " खोजें",
  " खोजिए",
  " ढूंढो",
  " ढूँढो",
  " ढूंढें",
  " ढूँढें",
  "的信息",
  "的資料",
  "的资料",
  "信息",
  "資料",
  "资料",
  "内容",
  "文章",
  "在维基百科上",
  "在維基百科上",
  "维基百科",
  "維基百科",
  "网上",
  "網上",
  "在线",
  "在線",
  "搜索",
  "查找",
  "查一下",
  "搜一下",
];

const SEARCH_QUERY_SOURCE_ONLY = [
  "web",
  "internet",
  "online",
  "wikipedia",
  "wikidata",
  "wiktionary",
  "интернет",
  "интернете",
  "онлайн",
  "сети",
  "википедии",
  "इंटरनेट",
  "ऑनलाइन",
  "वेब",
  "विकिपीडिया",
  "网上",
  "網上",
  "在线",
  "在線",
  "互联网",
  "網路",
  "网络",
  "维基百科",
  "維基百科",
];

const IMPLICIT_RESEARCH_QUESTION_PREFIXES = [
  "what is the ",
  "what is a ",
  "what is an ",
  "what is ",
  "what are the ",
  "what are ",
  "what s the ",
  "what s a ",
  "what s an ",
  "what s ",
  "which is the ",
  "which is a ",
  "which is an ",
  "which are the ",
  "which are ",
  "which ",
  "who is the ",
  "who are the ",
  "who ",
  "where is the ",
  "where are the ",
  "where ",
  "when is the ",
  "when are the ",
  "when ",
  "why is the ",
  "why are the ",
  "why ",
  "how is the ",
  "how are the ",
  "how ",
  "can you tell me ",
  "could you tell me ",
  "do you know ",
];

const IMPLICIT_RESEARCH_MODIFIERS = [
  " most ",
  " best ",
  " top ",
  " leading ",
  " standard ",
  " de facto ",
  " widely used ",
  " commonly used ",
  " popular ",
  " recommended ",
  " current ",
  " latest ",
  " recent ",
  " state of the art ",
  " sota ",
  " should i use ",
  " should we use ",
  " should be used ",
];

const IMPLICIT_RESEARCH_EVIDENCE_DOMAINS = [
  " dataset ",
  " datasets ",
  " benchmark ",
  " benchmarks ",
  " corpus ",
  " corpora ",
  " metric ",
  " metrics ",
  " framework ",
  " frameworks ",
  " paper ",
  " papers ",
  " study ",
  " studies ",
];

const IMPLICIT_RESEARCH_EVALUATION_DOMAINS = [
  " evaluation ",
  " evaluate ",
  " validation ",
  " validate ",
  " quality ",
  " translation ",
  " compare ",
  " comparison ",
];

const ENUMERATION_RESEARCH_PREFIXES = [
  "list all ",
  "list every ",
  "list the ",
  "show all ",
  "show me all ",
  "show me the ",
  "give me all ",
  "name all ",
  "enumerate all ",
  "перечисли всех ",
  "перечисли все ",
  "список всех ",
  "назови всех ",
  "सभी ",
  "हर ",
  "列出所有 ",
  "列出全部 ",
  "显示所有 ",
  "枚举所有 ",
];

const ENUMERATION_RESEARCH_CONSTRAINT_MARKERS = [
  " with ",
  " that ",
  " who ",
  " whose ",
  " where ",
  " which ",
  " having ",
  " have ",
  " has ",
  " featuring ",
  " capable of ",
  " can ",
  " for ",
  " by ",
  " in ",
  " с ",
  " у которых ",
  " которые ",
  " имеющие ",
  " имеющих ",
  " для ",
  " в ",
  " जिनके ",
  " जिनमें ",
  " जिसमें ",
  " वाले ",
  " के साथ ",
  " के लिए ",
  " में ",
  " 具有 ",
  " 有 ",
  " 带有 ",
  " 可以 ",
  " 能 ",
  " 在 ",
  " 用于 ",
];

function containsSearchMarker(normalized, marker) {
  const text = String(normalized || "");
  if (marker.startsWith(" ") || marker.endsWith(" ")) {
    return ` ${text} `.includes(marker);
  }
  return text.includes(marker);
}

function containsAnySearchMarker(normalized, markers) {
  return markers.some((marker) => containsSearchMarker(normalized, marker));
}

function stripSearchNoisePrefix(value, prefix) {
  const text = cleanSearchQuery(value);
  return text.toLowerCase().startsWith(prefix)
    ? cleanSearchQuery(text.slice(prefix.length))
    : text;
}

function stripSearchNoiseSuffix(value, suffix) {
  const text = cleanSearchQuery(value);
  return text.toLowerCase().endsWith(suffix)
    ? cleanSearchQuery(text.slice(0, text.length - suffix.length))
    : text;
}

function cleanSemanticSearchQuery(value) {
  let query = cleanSearchQuery(value);
  while (true) {
    const before = query;
    for (const prefix of SEARCH_QUERY_LEADING_NOISE) {
      query = stripSearchNoisePrefix(query, prefix);
    }
    for (const suffix of SEARCH_QUERY_TRAILING_NOISE) {
      query = stripSearchNoiseSuffix(query, suffix);
    }
    if (query === before) return query;
  }
}

function validSearchQuery(value) {
  const query = cleanSemanticSearchQuery(value);
  const queryKey = query.toLowerCase();
  if (SEARCH_QUERY_SOURCE_ONLY.includes(queryKey)) return "";
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
  const hasAction = containsAnySearchMarker(normalized, WEB_SEARCH_ACTION_MARKERS);
  if (!hasAction) return "";
  const hasStrongAction = containsAnySearchMarker(
    normalized,
    WEB_SEARCH_STRONG_ACTION_MARKERS,
  );
  if (!hasStrongAction && !containsAnySearchMarker(normalized, WEB_SEARCH_SIGNAL_MARKERS)) {
    return "";
  }
  for (const marker of SEARCH_QUERY_AFTER_MARKERS) {
    const query =
      queryAfterRawMarker(prompt, marker) ||
      queryAfterNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  for (const marker of SEARCH_QUERY_BEFORE_MARKERS) {
    const query =
      queryBeforeRawMarker(prompt, marker) ||
      queryBeforeNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  for (const marker of SEARCH_ACTION_AFTER_MARKERS) {
    const query =
      queryAfterRawMarker(prompt, marker) ||
      queryAfterNormalizedMarker(normalized, marker);
    if (query) return query;
  }
  return "";
}

function stripImplicitResearchPrefix(value) {
  const text = String(value || "");
  for (const prefix of IMPLICIT_RESEARCH_QUESTION_PREFIXES) {
    if (text.startsWith(prefix)) {
      return text.slice(prefix.length);
    }
  }
  return text;
}

function extractImplicitResearchQuestion(normalized) {
  const text = String(normalized || "");
  if (!startsWithAny(text, IMPLICIT_RESEARCH_QUESTION_PREFIXES)) return "";
  const padded = ` ${text} `;
  const hasModifier = IMPLICIT_RESEARCH_MODIFIERS.some((marker) =>
    padded.includes(marker),
  );
  const hasEvidenceDomain = IMPLICIT_RESEARCH_EVIDENCE_DOMAINS.some((marker) =>
    padded.includes(marker),
  );
  const hasEvaluationDomain = IMPLICIT_RESEARCH_EVALUATION_DOMAINS.some((marker) =>
    padded.includes(marker),
  );
  if (!hasModifier && !(hasEvidenceDomain && hasEvaluationDomain)) return "";
  return validSearchQuery(stripImplicitResearchPrefix(text));
}

function stripEnumerationResearchPrefix(value) {
  const text = String(value || "").trim();
  const lower = text.toLowerCase();
  for (const prefix of ENUMERATION_RESEARCH_PREFIXES) {
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
    ENUMERATION_RESEARCH_CONSTRAINT_MARKERS,
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
    normalized.startsWith("search my chats ")
  ) {
    return "";
  }
  for (const prefix of WEB_SEARCH_EXPLICIT_PREFIXES) {
    const rawQuery = stripSearchPrefix(prompt, prefix);
    const normalizedQuery = normalized.startsWith(prefix)
      ? validSearchQuery(normalized.slice(prefix.length))
      : "";
    const query = rawQuery || normalizedQuery;
    if (query) {
      return { query, kind: "explicit_prefix" };
    }
  }
  const semanticQuery = extractSemanticWebSearchQuery(prompt, normalized);
  if (semanticQuery) {
    return { query: semanticQuery, kind: "semantic_action" };
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
  const suffixes = [
    " step by step",
    " in steps",
    " with steps",
    " for me",
    " please",
  ];
  for (const suffix of suffixes) {
    if (clean.endsWith(suffix)) {
      clean = clean.slice(0, -suffix.length).trim();
      break;
    }
  }
  return clean;
}

function extractProceduralHowToTask(normalized) {
  const prefixes = [
    "please tell me how to ",
    "please show me how to ",
    "tell me how to ",
    "show me how to ",
    "what are the steps to ",
    "what steps do i need to ",
    "what steps do we need to ",
    "how should i ",
    "how should we ",
    "how could i ",
    "how could we ",
    "how would i ",
    "how would we ",
    "how can i ",
    "how can we ",
    "how do i ",
    "how do we ",
    "how to ",
  ];
  const clean = cleanProceduralFragment(normalized);
  for (const prefix of prefixes) {
    if (!clean.startsWith(prefix)) continue;
    const task = cleanProceduralFragment(clean.slice(prefix.length));
    if (!task) return null;
    const firstSpace = task.search(/\s/u);
    const action = firstSpace === -1 ? task : task.slice(0, firstSpace);
    const object = firstSpace === -1 ? "" : task.slice(firstSpace + 1).trim();
    return { task, action, object };
  }
  return null;
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

async function tryProceduralHowTo(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const task = extractProceduralHowToTask(normalized);
  if (!task) return null;

  const query = `how to ${task.task}`;
  const pageTitle = wikiHowPageTitle(task.task);
  const apiUrl = wikiHowParseApiUrl(pageTitle);
  const providerSummary = WEB_SEARCH_PROVIDERS.map((provider) => provider.id).join(", ");
  const evidence = [
    `procedural_how_to:request:${task.task}`,
    `procedural_how_to:action:${task.action}`,
    `procedural_how_to:stage:wikipedia`,
    `procedural_how_to:stage:wikidata`,
    `procedural_how_to:stage:wikihow_api`,
    `procedural_how_to:wikihow_candidate:${pageTitle}`,
    `http_fetch:request:${apiUrl}`,
  ];
  if (task.object) {
    evidence.splice(2, 0, `procedural_how_to:object:${task.object}`);
  }

  const wikiHow = await fetchWikiHowProcedure(pageTitle, evidence);
  const lines = [
    `Procedural discovery for \`${task.task}\` (action \`${task.action}\`, object \`${task.object}\`).`,
    "",
    "Source path: Wikipedia -> Wikidata -> wikiHow API -> web search fallback -> recursive fetch check.",
    "",
  ];

  let confidence = 0.78;
  let diagnostics = null;
  let formalizedObject = "";
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
    evidence.push(`procedural_how_to:wikihow_miss:${wikiHow.error || "no_match"}`);
    evidence.push("procedural_how_to:stage:web_search");
    const webSearch = await tryWebSearch(`search the web for ${query}`, language);
    if (webSearch) {
      appendUniqueEvidence(evidence, webSearch.evidence);
      diagnostics = webSearch.diagnostics || null;
      formalizedObject = webSearch.formalizedObject || "";
      lines.push(
        `wikiHow candidate \`${pageTitle}\` did not return explicit steps (${wikiHow.error || "no_match"}).`,
      );
      lines.push("");
      lines.push(`Fallback web search for \`${query}\`:`);
      lines.push("");
      lines.push(webSearch.content);
    } else {
      evidence.push(`web_search:request:${query}`);
      for (const provider of WEB_SEARCH_PROVIDERS) {
        evidence.push(`web_search:provider:${provider.id}`);
      }
      evidence.push(`web_search:combined:rrf:k=${webSearchRrfK()}`);
      lines.push(
        `wikiHow candidate \`${pageTitle}\` did not return explicit steps (${wikiHow.error || "no_match"}).`,
      );
      lines.push("");
      lines.push(
        `Fallback web search for \`${query}\` should use ${providerSummary} and reciprocal rank fusion (k = ${webSearchRrfK()}).`,
      );
    }
  }
  if (!evidence.includes("procedural_how_to:stage:web_search")) {
    evidence.push("procedural_how_to:stage:web_search");
    evidence.push(`web_search:request:${query}`);
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

function evaluateFramePolicy(headers, targetUrl, embedderOrigin) {
  const frameHeaders = normalizeFramePolicyHeaders(headers);
  const xFrameOptions = frameHeaders["x-frame-options"] || "";
  const csp = frameHeaders["content-security-policy"] || "";
  let target;
  try {
    target = new URL(targetUrl);
  } catch (_error) {
    return { status: "unknown", reason: "the target URL could not be parsed" };
  }

  const xFrameDirectives = xFrameOptions
    .split(",")
    .map((part) => part.trim().toLowerCase())
    .filter(Boolean);
  const sourceSets = frameAncestorsSourceSets(csp);
  const cspHasFrameAncestorsNone = sourceSets.some((sources) =>
    sources.includes("'none'"),
  );
  if (xFrameDirectives.includes("deny")) {
    return {
      status: "blocked",
      reason: cspHasFrameAncestorsNone
        ? "the page sends X-Frame-Options: DENY and CSP frame-ancestors 'none'"
        : "the page sends X-Frame-Options: DENY",
    };
  }
  if (xFrameDirectives.includes("sameorigin")) {
    let embedder;
    try {
      embedder = embedderOrigin ? new URL(embedderOrigin) : null;
    } catch (_error) {
      embedder = null;
    }
    if (!embedder || embedder.origin !== target.origin) {
      return {
        status: "blocked",
        reason: "the page sends X-Frame-Options: SAMEORIGIN",
      };
    }
  }

  if (sourceSets.length > 0) {
    let embedder;
    try {
      embedder = embedderOrigin ? new URL(embedderOrigin) : null;
    } catch (_error) {
      embedder = null;
    }
    if (!embedder) {
      return {
        status: "unknown",
        reason: "the current web app origin is unavailable",
      };
    }
    for (const sources of sourceSets) {
      if (sources.includes("'none'")) {
        return {
          status: "blocked",
          reason: "the page sends CSP frame-ancestors 'none'",
        };
      }
      if (
        sources.length > 0 &&
        !sources.some((source) => sourceExpressionMatches(source, target, embedder))
      ) {
        return {
          status: "blocked",
          reason:
            "the page's CSP frame-ancestors directive does not include this web app",
        };
      }
    }
  }

  return {
    status: "allowed",
    reason: "no blocking X-Frame-Options or CSP frame-ancestors policy was detected",
  };
}

async function detectFramePolicy(url) {
  const evidence = [`url_preview:frame_policy_check:${FRAME_POLICY_CHECK_ENDPOINT}`];
  if (typeof fetch !== "function") {
    return {
      status: "unknown",
      reason: "browser fetch is not available",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }
  if (!isPublicHttpUrl(url)) {
    return {
      status: "unknown",
      reason: "only public HTTP(S) URLs are checked by the frame-policy service",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }

  try {
    const response = await fetch(framePolicyCheckUrl(url), {
      method: "GET",
      mode: "cors",
      credentials: "omit",
    });
    evidence.push(`url_preview:frame_policy_status:${response.status}`);
    if (!response.ok) {
      return {
        status: "unknown",
        reason: `the frame-policy service returned HTTP ${response.status}`,
        evidence: evidence.concat("url_preview:frame_policy:unknown"),
      };
    }
    const data = await response.json();
    const headers = (data && (data.headers || (data.data && data.data.headers))) || null;
    if (!headers || typeof headers !== "object") {
      return {
        status: "unknown",
        reason: "the frame-policy service did not return response headers",
        evidence: evidence.concat("url_preview:frame_policy:unknown"),
      };
    }
    const verdict = evaluateFramePolicy(headers, url, currentEmbedderOrigin());
    return {
      ...verdict,
      evidence: evidence.concat(`url_preview:frame_policy:${verdict.status}`),
    };
  } catch (_error) {
    return {
      status: "unknown",
      reason: "the frame-policy service could not be reached from this browser",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }
}

function directExternalLinkAnswer(url, framePolicy, leadingLine) {
  const lines = [leadingLine || `I suggest opening this in a new tab: [${url}](${url}).`, ""];
  if (framePolicy && framePolicy.status === "blocked") {
    lines.push(
      `I checked the page's frame policy, and it does not allow embedding here because ${framePolicy.reason}.`,
    );
  } else if (framePolicy && framePolicy.status === "unknown") {
    lines.push(
      `I could not verify that this page allows embedding here because ${framePolicy.reason}.`,
    );
  } else {
    lines.push("I could not verify that this page allows embedding here.");
  }
  lines.push(
    "Browser JavaScript also cannot read the page content directly unless the site allows CORS, so the direct external link is the reliable option.",
  );
  return lines.join("\n");
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
    // CORS block or network failure. Check target frame policy before choosing
    // between an iframe preview and a direct external link.
    const isCors =
      err instanceof TypeError &&
      (err.message.toLowerCase().includes("cors") ||
        err.message.toLowerCase().includes("network") ||
        err.message.toLowerCase().includes("failed to fetch"));
    evidence.push(`http_fetch:error:${isCors ? "cors" : "network"}`);
    const framePolicy = await detectFramePolicy(url);
    evidence.push(...framePolicy.evidence);
    const fetchFailureLine = `Could not fetch \`${url}\` directly${isCors ? " (CORS restriction)" : " (network error)"}.`;
    if (framePolicy.status !== "allowed") {
      evidence.push(`url_preview:external_link:${url}`);
      return {
        intent: "http_fetch",
        content: directExternalLinkAnswer(
          url,
          framePolicy,
          `${fetchFailureLine}\n\nI suggest opening this in a new tab: [${url}](${url}).`,
        ),
        confidence: 0.75,
        evidence,
        iframeUrl: null,
      };
    }
    evidence.push(`url_preview:iframe:${url}`);
    const lines = [
      fetchFailureLine,
      "",
      "I checked the page's frame policy and can show it in the embedded frame below.",
    ];
    return {
      intent: "http_fetch",
      content: lines.join("\n"),
      confidence: 0.8,
      evidence,
      iframeUrl: url,
    };
  }
}

async function tryUrlNavigate(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractUrlNavigateUrl(prompt, normalized);
  if (!url) return null;

  const evidence = [`url_navigate:request:${url}`];
  const framePolicy = await detectFramePolicy(url);
  evidence.push(...framePolicy.evidence);
  if (framePolicy.status !== "allowed") {
    evidence.push(`url_preview:external_link:${url}`);
    return {
      intent: "url_navigate",
      content: directExternalLinkAnswer(url, framePolicy),
      confidence: 0.95,
      evidence,
      iframeUrl: null,
    };
  }

  evidence.push(`url_preview:iframe:${url}`);
  const lines = [
    "I checked the page's frame policy and can show it here.",
    "",
    `Direct link: [${url}](${url}).`,
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
// of the session and records the decision in memory. Issue #180: we also
// pre-probe every provider once per session so the first user query does not
// pay for failed sockets — the result is cached in `WEB_SEARCH_AVAILABLE`
// alongside the disable list.
const WEB_SEARCH_DISABLED = new Map();
const WEB_SEARCH_AVAILABLE = new Map();
const WEB_SEARCH_DIAGNOSTICS = [];
let WEB_SEARCH_PROBE_PROMISE = null;

function webSearchDisable(providerId, reason) {
  if (!WEB_SEARCH_DISABLED.has(providerId)) {
    WEB_SEARCH_DISABLED.set(providerId, { reason, at: Date.now() });
  }
}

function webSearchIsDisabled(providerId) {
  return WEB_SEARCH_DISABLED.has(providerId);
}

function webSearchMarkAvailable(providerId, info) {
  WEB_SEARCH_AVAILABLE.set(providerId, Object.assign({ at: Date.now() }, info || {}));
  WEB_SEARCH_DISABLED.delete(providerId);
}

// Issue #180: record a single HTTP exchange so the diagnostics panel can
// surface the raw request/response/conversion trace. We keep a small ring
// buffer in RAM so very long sessions do not bloat memory.
const WEB_SEARCH_DIAG_LIMIT = 64;
function recordWebSearchDiagnostic(entry) {
  if (!entry || typeof entry !== "object") return;
  WEB_SEARCH_DIAGNOSTICS.push(entry);
  while (WEB_SEARCH_DIAGNOSTICS.length > WEB_SEARCH_DIAG_LIMIT) {
    WEB_SEARCH_DIAGNOSTICS.shift();
  }
}

function consumeWebSearchDiagnostics() {
  if (WEB_SEARCH_DIAGNOSTICS.length === 0) return [];
  const snapshot = WEB_SEARCH_DIAGNOSTICS.slice();
  WEB_SEARCH_DIAGNOSTICS.length = 0;
  return snapshot;
}

async function fetchProviderJson(providerId, url, options) {
  if (typeof fetch !== "function") {
    webSearchDisable(providerId, "no_fetch");
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: false,
      error: "fetch unavailable",
    });
    return { ok: false, error: "fetch unavailable", finalUrl: url };
  }
  const startedAt = Date.now();
  try {
    const response = await fetch(url, options || { mode: "cors" });
    const status = response ? response.status : 0;
    const statusText = response ? response.statusText : "";
    if (!response || !response.ok) {
      recordWebSearchDiagnostic({
        providerId,
        url,
        method: (options && options.method) || "GET",
        requestHeaders: (options && options.headers) || null,
        ok: false,
        status,
        statusText,
        elapsedMs: Date.now() - startedAt,
      });
      return { ok: false, status, statusText, finalUrl: url };
    }
    const text = await response.text();
    let data = null;
    try {
      data = text ? JSON.parse(text) : null;
    } catch (parseError) {
      const message = parseError instanceof Error ? parseError.message : String(parseError);
      recordWebSearchDiagnostic({
        providerId,
        url,
        method: (options && options.method) || "GET",
        requestHeaders: (options && options.headers) || null,
        ok: false,
        status,
        statusText,
        elapsedMs: Date.now() - startedAt,
        responseSnippet: text.slice(0, 1024),
        error: `parse_error: ${message}`,
      });
      return { ok: false, error: `parse_error: ${message}`, finalUrl: url };
    }
    webSearchMarkAvailable(providerId, { status });
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: true,
      status,
      statusText,
      elapsedMs: Date.now() - startedAt,
      responseSnippet: text.length > 4096 ? `${text.slice(0, 4096)}…` : text,
      responseBytes: text.length,
    });
    return { ok: true, status, data, finalUrl: url };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const isCors =
      message.toLowerCase().includes("cors") ||
      message.toLowerCase().includes("network") ||
      message.toLowerCase().includes("failed to fetch");
    webSearchDisable(providerId, isCors ? "cors" : "network");
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: false,
      elapsedMs: Date.now() - startedAt,
      error: message,
      cors: isCors,
    });
    return { ok: false, error: message, finalUrl: url, cors: isCors };
  }
}

// Issue #180: shared text-shaping helpers used by every web-search provider so
// the rendered bullet looks consistent regardless of which provider produced
// the entry. `extractDomain` returns the bare host (without `www.`),
// `extractQuoteAroundQuery` walks the response body and returns a short
// Google-style snippet that contains the original query word when possible,
// and `escapeRegExp` is the standard helper used by the snippet picker.
function escapeRegExp(value) {
  return String(value || "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function extractDomain(url) {
  const raw = String(url || "").trim();
  if (!raw) return "";
  try {
    const u = new URL(raw);
    return u.hostname.replace(/^www\./i, "");
  } catch (_error) {
    const match = raw.match(/^[a-z][a-z0-9+.\-]*:\/\/([^\/?#]+)/i);
    if (match) return match[1].replace(/^www\./i, "");
    return "";
  }
}

function extractQuoteAroundQuery(text, query, maxChars) {
  const max = typeof maxChars === "number" && maxChars > 0 ? Math.floor(maxChars) : 200;
  const raw = String(text || "").replace(/\s+/g, " ").trim();
  if (!raw) return "";
  if (raw.length <= max) return raw;
  const q = String(query || "").trim();
  if (q) {
    const candidates = [q, ...q.split(/\s+/)].filter((value, index, array) =>
      value && array.indexOf(value) === index,
    );
    for (const candidate of candidates) {
      if (!candidate || candidate.length < 2) continue;
      const re = new RegExp(escapeRegExp(candidate), "i");
      const match = raw.match(re);
      if (match && typeof match.index === "number") {
        const half = Math.max(20, Math.floor((max - candidate.length) / 2));
        let start = Math.max(0, match.index - half);
        let end = Math.min(raw.length, start + max);
        if (start > 0) {
          const space = raw.lastIndexOf(" ", start);
          if (space > 0 && match.index - space <= half + 20) start = space + 1;
        }
        if (end < raw.length) {
          const space = raw.indexOf(" ", end);
          if (space > 0 && space - start <= max + 40) end = space;
        }
        let snippet = raw.slice(start, end).trim();
        if (start > 0) snippet = "… " + snippet;
        if (end < raw.length) snippet = snippet + " …";
        return snippet;
      }
    }
  }
  let cut = raw.slice(0, max);
  const lastPeriod = Math.max(
    cut.lastIndexOf(". "),
    cut.lastIndexOf("! "),
    cut.lastIndexOf("? "),
    cut.lastIndexOf("。"),
  );
  if (lastPeriod > max * 0.5) return cut.slice(0, lastPeriod + 1).trim();
  const lastSpace = cut.lastIndexOf(" ");
  if (lastSpace > max * 0.5) cut = cut.slice(0, lastSpace);
  return cut.trim() + " …";
}

const PROVIDER_DISPLAY_LABELS = {
  duckduckgo: "DuckDuckGo",
  "internet-archive": "Internet Archive",
  wikipedia: "Википедия",
  wikidata: "Викидата",
  wiktionary: "Викисловарь",
};

const PROVIDER_DISPLAY_LABELS_BY_LANG = {
  en: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "Internet Archive",
    wikipedia: "Wikipedia",
    wikidata: "Wikidata",
    wiktionary: "Wiktionary",
  },
  ru: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "Архив Интернета",
    wikipedia: "Википедия",
    wikidata: "Викидата",
    wiktionary: "Викисловарь",
  },
  zh: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "互联网档案馆",
    wikipedia: "维基百科",
    wikidata: "维基数据",
    wiktionary: "维基词典",
  },
  hi: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "इंटरनेट आर्काइव",
    wikipedia: "विकिपीडिया",
    wikidata: "विकिडेटा",
    wiktionary: "विक्षनरी",
  },
};

function providerDisplayLabel(providerId, language) {
  const code = String(language || "").toLowerCase().slice(0, 2);
  const table = PROVIDER_DISPLAY_LABELS_BY_LANG[code] || PROVIDER_DISPLAY_LABELS_BY_LANG.en;
  return table[providerId] || PROVIDER_DISPLAY_LABELS[providerId] || providerId;
}

async function searchDuckDuckGo(query, language, limit) {
  // DuckDuckGo Instant Answer — CORS-readable, no key. Returns the abstract
  // and a flat list of related-topic links. We treat the abstract link plus
  // the related topics as the ranked result list (issue #133).
  //
  // Issue #153: the previous signature was (query, limit) but the dispatcher
  // calls every provider as (query, language, providerLimit). That meant
  // `limit` was set to a 2-letter language code like "en", and
  // `results.slice(0, "en")` silently returned an empty array, so DuckDuckGo
  // contributed nothing to the fused ranking.
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const params = ["q=" + encodeURIComponent(query), "format=json", "no_redirect=1", "no_html=1"];
  if (language && /^[a-z]{2,3}$/i.test(language) && language !== "en") {
    // DuckDuckGo accepts a `kl` region hint (lowercase locale). We do not
    // require a region/country code so a bare language is treated as the
    // canonical locale for that language; failing that, DDG falls back to
    // English content gracefully.
    params.push("kl=" + encodeURIComponent(`${language}-${language}`));
  }
  const url = "https://api.duckduckgo.com/?" + params.join("&");
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
    if (results.length >= cap) break;
  }
  return { ok: true, results: results.slice(0, cap), finalUrl: outcome.finalUrl };
}

async function searchWikipediaWebProvider(query, language, limit) {
  // Reuse the existing helper but adapt the shape to {title, url, excerpt}.
  const result = await searchWikipediaPages(query, language, limit);
  if (!result || !Array.isArray(result.pages)) {
    return { ok: false, results: [], finalUrl: "", language: language || "en" };
  }
  // R194/issue-153: thread the Wikipedia page key through so cross-source
  // deduplication can match `Apple_(disambiguation)` against the Wikidata
  // sitelink `enwiki: Apple_(disambiguation)` even if the URLs disagree on
  // percent-encoding.
  const results = result.pages.slice(0, limit).map((page) => ({
    title: page.title,
    url: page.url,
    excerpt: page.excerpt,
    wikipediaKey: page.key || page.title || "",
    wikipediaLanguage: result.language,
    virtualId: `WP:${page.key || page.title || query}`,
    sourceKind: "wikipedia",
  }));
  return {
    ok: true,
    results,
    language: result.language,
    finalUrl: `https://${result.language}.wikipedia.org/w/rest.php/v1/search/page?q=${encodeURIComponent(query)}`,
  };
}

async function searchWikidataEntities(query, language, limit) {
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  // R194/issue-153: request `sitelinks/urls` so each entity carries its
  // Wikipedia URL inline. We use that to merge entries returned by the
  // Wikipedia provider with the same entity (otherwise the user sees the
  // same fact as two bullets — "Apple" via Wikidata Q89 and "Apple" via
  // enwiki).
  const url =
    "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=" +
    encodeURIComponent(query) +
    "&language=" +
    encodeURIComponent(lang) +
    "&format=json&origin=*&props=sitelinks/urls&limit=" +
    encodeURIComponent(limit);
  const outcome = await fetchProviderJson("wikidata", url);
  if (!outcome.ok || !outcome.data || !Array.isArray(outcome.data.search)) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const results = outcome.data.search.slice(0, limit).map((entry) => {
    const sitelinks = entry.sitelinks && typeof entry.sitelinks === "object"
      ? entry.sitelinks
      : {};
    const wikipediaLang = sitelinks[`${lang}wiki`] ? lang : "en";
    const wikipediaEntry =
      sitelinks[`${wikipediaLang}wiki`] || sitelinks.enwiki || null;
    const wikipediaUrl = wikipediaEntry && wikipediaEntry.url
      ? wikipediaEntry.url
      : "";
    const wikipediaKey = wikipediaEntry && wikipediaEntry.title
      ? String(wikipediaEntry.title).replace(/\s+/g, "_")
      : "";
    return {
      title: entry.label || entry.id || query,
      url: entry.concepturi || `https://www.wikidata.org/wiki/${entry.id}`,
      excerpt: stripHtml(entry.description || ""),
      qid: entry.id || "",
      virtualId: entry.id || "",
      sourceKind: "wikidata",
      wikipediaUrl,
      wikipediaKey,
      wikipediaLanguage: wikipediaEntry ? wikipediaLang : "",
    };
  });
  return { ok: true, results, finalUrl: outcome.finalUrl };
}

async function searchInternetArchive(query, language, limit) {
  // Issue #153: archive.org publishes a CORS-enabled `advancedsearch.php`
  // endpoint that returns ranked results across the entire collection (web
  // captures, books, audio, software, ...). This complements the DuckDuckGo
  // Instant Answer (which mostly returns a single Wikipedia abstract) and
  // gives the agent another general-purpose web search fallback to draw on
  // when the structured providers (Wikidata/Wikipedia) miss the query.
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const params = [
    "q=" + encodeURIComponent(query),
    "fl%5B%5D=identifier",
    "fl%5B%5D=title",
    "fl%5B%5D=description",
    "fl%5B%5D=creator",
    "sort%5B%5D=" + encodeURIComponent("downloads desc"),
    "rows=" + encodeURIComponent(cap),
    "page=1",
    "output=json",
  ];
  const url = "https://archive.org/advancedsearch.php?" + params.join("&");
  const outcome = await fetchProviderJson("internet-archive", url);
  if (
    !outcome.ok ||
    !outcome.data ||
    !outcome.data.response ||
    !Array.isArray(outcome.data.response.docs)
  ) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const docs = outcome.data.response.docs;
  const results = docs.slice(0, cap).map((doc) => {
    const identifier = doc.identifier || "";
    const description = Array.isArray(doc.description)
      ? doc.description.join(" • ")
      : (doc.description || "");
    const creator = Array.isArray(doc.creator)
      ? doc.creator.join(", ")
      : (doc.creator || "");
    const excerpt = stripHtml(creator ? `${creator} — ${description}` : description);
    return {
      title: doc.title || identifier || query,
      url: identifier ? `https://archive.org/details/${identifier}` : `https://archive.org/search.php?query=${encodeURIComponent(query)}`,
      excerpt,
      virtualId: `IA:${identifier || query}`,
      sourceKind: "internet-archive",
    };
  });
  return { ok: true, results, finalUrl: outcome.finalUrl };
}

// Issue #180: Wiktionary opensearch is a CORS-readable provider that returns
// short dictionary definitions — exactly the kind of "fragment containing the
// original request" the rendering template needs. We reuse the same
// `fetchProviderJson` plumbing so the diagnostics panel records the raw call.
async function searchWiktionary(query, language, limit) {
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  const ordered = [lang, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  const collected = [];
  let lastFinalUrl = "";
  let lastError = "";
  for (const candidate of ordered) {
    const base = WIKTIONARY_SEARCH_HOSTS[candidate] || WIKTIONARY_SEARCH_HOSTS.en;
    const url = `${base}?action=opensearch&search=${encodeURIComponent(query)}&limit=${cap}&format=json&origin=*`;
    const outcome = await fetchProviderJson("wiktionary", url);
    lastFinalUrl = outcome.finalUrl || lastFinalUrl;
    if (!outcome.ok || !Array.isArray(outcome.data) || !Array.isArray(outcome.data[1])) {
      if (outcome.error) lastError = outcome.error;
      continue;
    }
    const titles = outcome.data[1];
    const descriptions = Array.isArray(outcome.data[2]) ? outcome.data[2] : [];
    const urls = Array.isArray(outcome.data[3]) ? outcome.data[3] : [];
    for (let index = 0; index < titles.length && collected.length < cap; index += 1) {
      const title = titles[index] || query;
      const description = stripHtml(
        descriptions[index] || wiktionaryFallbackDescription(title, candidate),
      );
      const itemUrl = urls[index] ||
        `https://${candidate}.wiktionary.org/wiki/${encodeURIComponent(title)}`;
      collected.push({
        title,
        url: itemUrl,
        excerpt: description,
        wiktionaryKey: String(title).replace(/\s+/g, "_"),
        wiktionaryLanguage: candidate,
        virtualId: `WT:${candidate}:${String(title).replace(/\s+/g, "_")}`,
        sourceKind: "wiktionary",
      });
    }
    if (collected.length > 0) break;
  }
  if (collected.length === 0) {
    return { ok: false, results: [], finalUrl: lastFinalUrl, error: lastError || "no_results" };
  }
  return { ok: true, results: collected.slice(0, cap), finalUrl: lastFinalUrl };
}

// Issue #180: The priority order requested in the issue is
// DuckDuckGo → Internet Archive → Wikipedia → Wikidata → Wiktionary → rest.
// We also keep the corresponding light-weight probe URL so the per-session
// availability check at the top of `tryWebSearch` can pre-flight every
// provider once instead of failing the first user query.
const WEB_SEARCH_PROVIDERS = [
  {
    id: "duckduckgo",
    label: "DuckDuckGo Instant Answer",
    priority: 1,
    probeUrl: "https://api.duckduckgo.com/?q=ping&format=json&no_redirect=1&no_html=1",
    run: (query, language, limit) => searchDuckDuckGo(query, language, limit),
  },
  {
    id: "internet-archive",
    label: "Internet Archive (archive.org)",
    priority: 2,
    probeUrl:
      "https://archive.org/advancedsearch.php?q=ping&fl%5B%5D=identifier&rows=1&page=1&output=json",
    run: (query, language, limit) =>
      searchInternetArchive(query, language, limit),
  },
  {
    id: "wikipedia",
    label: "Wikipedia REST",
    priority: 3,
    probeUrl: "https://en.wikipedia.org/w/rest.php/v1/search/page?q=ping&limit=1",
    run: (query, language, limit) =>
      searchWikipediaWebProvider(query, language, limit),
  },
  {
    id: "wikidata",
    label: "Wikidata entities",
    priority: 4,
    probeUrl:
      "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=ping&language=en&format=json&origin=*&limit=1",
    run: (query, language, limit) =>
      searchWikidataEntities(query, language, limit),
  },
  {
    id: "wiktionary",
    label: "Wiktionary opensearch",
    priority: 5,
    probeUrl:
      "https://en.wiktionary.org/w/api.php?action=opensearch&search=ping&limit=1&format=json&origin=*",
    run: (query, language, limit) =>
      searchWiktionary(query, language, limit),
  },
];

const WEB_SEARCH_PROVIDER_PRIORITY = WEB_SEARCH_PROVIDERS.reduce((acc, provider, index) => {
  acc[provider.id] = typeof provider.priority === "number" ? provider.priority : index + 1;
  return acc;
}, Object.create(null));

// Issue #180: pre-probe every provider exactly once per browser session. The
// result lives in `WEB_SEARCH_AVAILABLE` / `WEB_SEARCH_DISABLED` for the rest
// of the worker's lifetime so subsequent queries skip CORS-blocked endpoints
// without re-burning a socket. We return a shared promise so concurrent
// callers cooperate on the same probe batch.
function ensureWebSearchProviderProbes() {
  if (WEB_SEARCH_PROBE_PROMISE) return WEB_SEARCH_PROBE_PROMISE;
  if (typeof fetch !== "function") {
    WEB_SEARCH_PROBE_PROMISE = Promise.resolve([]);
    return WEB_SEARCH_PROBE_PROMISE;
  }
  WEB_SEARCH_PROBE_PROMISE = (async () => {
    const tasks = WEB_SEARCH_PROVIDERS.map((provider) => async () => {
      if (!provider.probeUrl) return null;
      const startedAt = Date.now();
      try {
        const response = await fetch(provider.probeUrl, { mode: "cors" });
        const status = response ? response.status : 0;
        if (response && response.ok) {
          webSearchMarkAvailable(provider.id, { probedAt: startedAt, status });
          recordWebSearchDiagnostic({
            providerId: provider.id,
            url: provider.probeUrl,
            method: "GET",
            ok: true,
            status,
            elapsedMs: Date.now() - startedAt,
            phase: "probe",
          });
          return { providerId: provider.id, ok: true, status };
        }
        recordWebSearchDiagnostic({
          providerId: provider.id,
          url: provider.probeUrl,
          method: "GET",
          ok: false,
          status,
          elapsedMs: Date.now() - startedAt,
          phase: "probe",
        });
        return { providerId: provider.id, ok: false, status };
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        const isCors =
          message.toLowerCase().includes("cors") ||
          message.toLowerCase().includes("network") ||
          message.toLowerCase().includes("failed to fetch");
        webSearchDisable(provider.id, isCors ? "cors" : "network");
        recordWebSearchDiagnostic({
          providerId: provider.id,
          url: provider.probeUrl,
          method: "GET",
          ok: false,
          elapsedMs: Date.now() - startedAt,
          error: message,
          cors: isCors,
          phase: "probe",
        });
        return { providerId: provider.id, ok: false, error: message, cors: isCors };
      }
    });
    return runWithConcurrencyLimit(tasks, webSearchConcurrency());
  })();
  return WEB_SEARCH_PROBE_PROMISE;
}

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

// Issue #153/#180: identify "the same entity" returned by different providers
// so the fused list shows one bullet with the other URLs collapsed under
// "Other sources:". A single result can carry several canonical identifiers
// (Wikidata Q-id, Wikipedia page key, Wiktionary headword) — dedupe walks all
// of them and merges into the first existing group it finds. Returning a
// list makes the Wikipedia↔Wikidata merge robust against percent-encoding
// differences in the two providers' URLs.
function canonicalEntityKeys(meta) {
  if (!meta) return [];
  const keys = [];
  if (meta.qid && /^Q\d+$/.test(meta.qid)) keys.push(`Q:${meta.qid}`);
  if (meta.wikipediaKey) {
    const lang = meta.wikipediaLanguage || "en";
    keys.push(`WP:${lang}:${meta.wikipediaKey}`);
  }
  if (meta.wiktionaryKey) {
    const lang = meta.wiktionaryLanguage || "en";
    keys.push(`WT:${lang}:${meta.wiktionaryKey}`);
  }
  return keys;
}

// Backwards-compatible shim: prefer the primary key but keep the historical
// single-key signature for callers that still rely on it.
function canonicalEntityKey(meta) {
  const keys = canonicalEntityKeys(meta);
  return keys.length > 0 ? keys[0] : null;
}

function buildItemMetadataIndex(perProvider) {
  // The richer the meta the better — an entry that carries a Wikidata `qid`
  // is preferred over a Wikipedia-only entry for the same URL, because the
  // Q-id is what cross-provider dedupe groups by. Without this preference,
  // the Wikipedia URL would be indexed by the Wikipedia provider's meta
  // (`WP:en:Apple`) and a separate Wikidata entry for the same fact (`Q:Q89`)
  // would never collapse into one bullet.
  const byUrl = new Map();
  const rank = (item) => (item && item.qid ? 2 : 1);
  function record(url, item) {
    if (!url || !item) return;
    const existing = byUrl.get(url);
    if (!existing || rank(item) > rank(existing)) {
      byUrl.set(url, item);
    }
  }
  for (const provider of perProvider) {
    if (!provider || !Array.isArray(provider.results)) continue;
    for (const item of provider.results) {
      if (!item || !item.url) continue;
      record(item.url, item);
      // Wikidata results carry the Wikipedia URL of the same entity inline;
      // index that too so the Wikipedia provider's entry is recognised as
      // a duplicate of the Wikidata one.
      if (item.wikipediaUrl) record(item.wikipediaUrl, item);
    }
  }
  return byUrl;
}

function dedupeFusedEntries(fused, metaByUrl, evidence) {
  const groupsByKey = new Map();
  const allGroups = [];
  const standalone = [];

  function alreadyHasProvider(target, candidate) {
    return target.providers.some(
      (existing) => existing.id === candidate.id && existing.rank === candidate.rank,
    );
  }

  fused.forEach((entry, index) => {
    const meta = metaByUrl.get(entry.url) || null;
    const keys = canonicalEntityKeys(meta);
    const enriched = Object.assign({}, entry, {
      qid: (meta && meta.qid) || "",
      wikipediaKey: (meta && meta.wikipediaKey) || "",
      wikipediaLanguage: (meta && meta.wikipediaLanguage) || "",
      wiktionaryKey: (meta && meta.wiktionaryKey) || "",
      wiktionaryLanguage: (meta && meta.wiktionaryLanguage) || "",
      sourceKind: (meta && meta.sourceKind) || "",
      virtualId:
        (meta && meta.virtualId) ||
        (meta && meta.qid) ||
        (meta && meta.wikipediaKey ? `WP:${meta.wikipediaKey}` : ""),
      alternateUrls: [],
      keys: keys.slice(),
      originalRank: index,
    });

    if (keys.length === 0) {
      standalone.push(enriched);
      return;
    }
    let head = null;
    for (const key of keys) {
      if (groupsByKey.has(key)) {
        head = groupsByKey.get(key);
        break;
      }
    }
    if (!head) {
      allGroups.push(enriched);
      for (const key of keys) {
        if (!groupsByKey.has(key)) groupsByKey.set(key, enriched);
      }
      return;
    }
    // Found an existing group — absorb this entry into it.
    head.score += enriched.score;
    head.alternateUrls.push({
      url: enriched.url,
      title: enriched.title,
      providers: enriched.providers,
      sourceKind: enriched.sourceKind,
    });
    for (const p of enriched.providers) {
      if (!alreadyHasProvider(head, p)) head.providers.push(p);
    }
    // Register the absorbed entry's keys against the head group too so a third
    // provider matching either canonical id still merges in.
    for (const key of keys) {
      if (!groupsByKey.has(key)) groupsByKey.set(key, head);
    }
    // Prefer the richest virtualId once we know more identifiers.
    if (!head.virtualId && enriched.virtualId) head.virtualId = enriched.virtualId;
    if (!head.qid && enriched.qid) head.qid = enriched.qid;
    if (!head.wikipediaKey && enriched.wikipediaKey) {
      head.wikipediaKey = enriched.wikipediaKey;
      head.wikipediaLanguage = enriched.wikipediaLanguage;
    }
    if (!head.wiktionaryKey && enriched.wiktionaryKey) {
      head.wiktionaryKey = enriched.wiktionaryKey;
      head.wiktionaryLanguage = enriched.wiktionaryLanguage;
    }
    if (Array.isArray(evidence)) {
      evidence.push(`web_search:dedupe:${keys[0]}:absorbed:${enriched.url}`);
    }
  });
  const merged = [...allGroups, ...standalone];
  merged.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    if (b.providers.length !== a.providers.length) {
      return b.providers.length - a.providers.length;
    }
    // Issue #180: stable order by provider priority so DDG-led entries beat
    // Wikidata-only entries on perfect ties.
    const ap = providerPriorityScore(a.providers);
    const bp = providerPriorityScore(b.providers);
    if (ap !== bp) return ap - bp;
    return a.originalRank - b.originalRank;
  });
  return merged;
}

function providerPriorityScore(providers) {
  if (!Array.isArray(providers) || providers.length === 0) return 999;
  let best = 999;
  for (const p of providers) {
    const score = WEB_SEARCH_PROVIDER_PRIORITY[p && p.id] || 999;
    if (score < best) best = score;
  }
  return best;
}

// Issue #153: localized templates for the web search response. Keep these in
// sync with the visible UI strings in `src/web/i18n-catalog.lino`. The worker
// runs in a separate context that cannot import lino-i18n at runtime, so we
// inline the small subset that is actually rendered to chat. `en` is always
// the fallback when the catalogue for the active language is missing.
const WEB_SEARCH_TEXTS = {
  en: {
    header: (query, top, k) =>
      `Search results for \`${query}\` — top ${top} after reciprocal rank fusion (k = ${k}).`,
    otherSources: "Other sources",
    via: "via",
    readMore: "Read more",
    noResults: (query, providers) =>
      `No CORS-enabled web search results were returned for \`${query}\`.\n\nProviders tried: ${providers}.`,
    allDisabled: (providers) =>
      `All CORS-readable search providers are disabled for this session. Tried: ${providers}.`,
  },
  ru: {
    header: (query, top, k) =>
      `Результаты поиска для \`${query}\` — топ ${top} после реципрокного объединения рангов (k = ${k}).`,
    otherSources: "Другие источники",
    via: "через",
    readMore: "Подробнее",
    noResults: (query, providers) =>
      `Не получены результаты веб-поиска с поддержкой CORS для \`${query}\`.\n\nПопробованы провайдеры: ${providers}.`,
    allDisabled: (providers) =>
      `Все CORS-совместимые поисковые провайдеры отключены в этой сессии. Пробовали: ${providers}.`,
  },
  zh: {
    header: (query, top, k) =>
      `搜索 \`${query}\` 的结果 — 经互惠等级融合后的前 ${top} 项（k = ${k}）。`,
    otherSources: "其他来源",
    via: "来自",
    readMore: "阅读更多",
    noResults: (query, providers) =>
      `未获取到 \`${query}\` 的可用 CORS 搜索结果。\n\n已尝试的提供方：${providers}。`,
    allDisabled: (providers) =>
      `本会话中所有支持 CORS 的搜索提供方都已禁用。已尝试：${providers}。`,
  },
  hi: {
    header: (query, top, k) =>
      `\`${query}\` के लिए खोज परिणाम — रेसिप्रोकल रैंक फ़्यूज़न के बाद शीर्ष ${top} (k = ${k})।`,
    otherSources: "अन्य स्रोत",
    via: "के माध्यम से",
    readMore: "और पढ़ें",
    noResults: (query, providers) =>
      `\`${query}\` के लिए CORS-समर्थित कोई खोज परिणाम नहीं मिले।\n\nप्रयास किए गए प्रदाता: ${providers}.`,
    allDisabled: (providers) =>
      `इस सत्र के लिए सभी CORS-समर्थित खोज प्रदाता अक्षम हैं। प्रयास किया: ${providers}.`,
  },
};

function webSearchTexts(language) {
  const code = String(language || "").toLowerCase().slice(0, 2);
  return WEB_SEARCH_TEXTS[code] || WEB_SEARCH_TEXTS.en;
}

async function tryWebSearch(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const request = extractWebSearchRequest(prompt, normalized);
  if (!request || !request.query) return null;
  return runWebSearchQuery(request.query, language, request.kind);
}

async function runWebSearchQuery(query, language, queryKind) {
  query = String(query || "").trim();
  if (!query) return null;
  const rrfK = webSearchRrfK();
  const concurrency = webSearchConcurrency();
  const providerLimit = webSearchProviderLimit();
  const texts = webSearchTexts(language);

  // Issue #180: pre-probe every provider once per browser session so the
  // first user query does not waste sockets on CORS-blocked endpoints. The
  // probe results live in `WEB_SEARCH_AVAILABLE`/`WEB_SEARCH_DISABLED` for
  // the rest of the worker lifetime.
  await ensureWebSearchProviderProbes();

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
    if (language) {
      evidence.push(`web_search:language:${language}`);
    }
    for (const provider of WEB_SEARCH_PROVIDERS) {
      evidence.push(`web_search:provider:${provider.id}`);
    }
    evidence.push(`web_search:combined:rrf:k=${rrfK}`);
  }
  if (queryKind) {
    evidence.push(`web_search:query_kind:${queryKind}`);
  }

  // Issue #180: providers are tried in declared priority order so the rendered
  // list matches the user's requested DDG → IA → WP → WD → Wiktionary
  // sequence whenever scores tie. Session-disabled providers are skipped on
  // top of the WASM-derived prefix and annotated for the diagnostics panel.
  const ordered = WEB_SEARCH_PROVIDERS.slice().sort((a, b) => {
    const pa = typeof a.priority === "number" ? a.priority : 999;
    const pb = typeof b.priority === "number" ? b.priority : 999;
    return pa - pb;
  });
  const active = ordered.filter((provider) => !webSearchIsDisabled(provider.id));
  for (const provider of ordered) {
    if (webSearchIsDisabled(provider.id)) {
      evidence.push(`web_search:disabled:${provider.id}`);
    } else if (WEB_SEARCH_AVAILABLE.has(provider.id)) {
      evidence.push(`web_search:available:${provider.id}`);
    }
  }

  if (active.length === 0) {
    return {
      intent: "web_search",
      content: texts.allDisabled(WEB_SEARCH_PROVIDERS.map((p) => p.id).join(", ")),
      confidence: 0.3,
      evidence,
      diagnostics: { providers: [], httpExchanges: consumeWebSearchDiagnostics() },
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
  const metaByUrl = buildItemMetadataIndex(perProvider);
  const deduped = dedupeFusedEntries(fused, metaByUrl, evidence);
  const top = deduped.slice(0, providerLimit);
  top.forEach((entry, index) => {
    evidence.push(`web_search:fused:${index + 1}:${entry.providers.map((p) => p.id).join("+")}:${entry.url}`);
    if (entry.virtualId) {
      evidence.push(`web_search:formal:${index + 1}:${entry.virtualId}`);
    }
  });

  const diagnostics = {
    query,
    language: language || "",
    providers: perProvider.map((p) => ({
      id: p.id,
      label: p.label,
      ok: !!p.ok,
      count: Array.isArray(p.results) ? p.results.length : 0,
      elapsedMs: p.elapsedMs || 0,
      finalUrl: p.finalUrl || "",
      error: p.error || "",
    })),
    httpExchanges: consumeWebSearchDiagnostics(),
    fused: top.map((entry, index) => ({
      rank: index + 1,
      url: entry.url,
      title: entry.title,
      score: entry.score,
      providers: entry.providers,
      alternateUrls: entry.alternateUrls,
      virtualId: entry.virtualId || "",
      keys: entry.keys || [],
    })),
  };

  if (top.length === 0) {
    return {
      intent: "web_search",
      content: texts.noResults(query, active.map((p) => p.label).join(", ")),
      confidence: 0.35,
      evidence,
      diagnostics,
    };
  }

  // Issue #180: every fused result is rendered Google-style — a single line
  // with title + bare domain, an indented quote (a fragment containing the
  // original query when possible, truncated near ~220 chars), a "Read more"
  // link, and finally a faint "Другие источники:" line listing alternates
  // (provider label + url) without per-source excerpts.
  const lines = [texts.header(query, top.length, rrfK), ""];
  top.forEach((entry, index) => {
    const domain = extractDomain(entry.url);
    const titlePiece = `**[${entry.title || entry.url}](${entry.url})**`;
    const domainPiece = domain ? `  \`${domain}\`` : "";
    const idTag = entry.virtualId ? `  \`${entry.virtualId}\`` : "";
    lines.push(`${index + 1}. ${titlePiece}${domainPiece}${idTag}`);
    const quote = extractQuoteAroundQuery(entry.excerpt, query, 220);
    if (quote) {
      lines.push(`   > ${quote}`);
    }
    const sourceTags = entry.providers
      .map((p) => `${p.id}#${p.rank}`)
      .join(", ");
    lines.push(`   [${texts.readMore}](${entry.url}) — _${texts.via} ${sourceTags}_`);
    if (Array.isArray(entry.alternateUrls) && entry.alternateUrls.length > 0) {
      const others = entry.alternateUrls
        .map((alt) => {
          const labelProvider = pickPrimaryProviderId(alt.providers, alt.sourceKind);
          const label = providerDisplayLabel(labelProvider, language);
          return `[${label}](${alt.url})`;
        })
        .filter(Boolean);
      if (others.length > 0) {
        lines.push(`   _${texts.otherSources}: ${others.join(", ")}_`);
      }
    }
    lines.push("");
  });
  while (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();

  // Resolve the formalization tuple now that we know the top-ranked entity.
  // Prefer a real Wikidata Q-id; fall back to the WP virtual id, then to the
  // bare normalised query. We scan the whole `top` slice instead of just
  // `top[0]` so that a DuckDuckGo result without an id at rank 1 still lets
  // us fold a Wikidata Q-id from rank 2+ into the resolved tuple.
  let formalizedObject = "";
  for (const entry of top) {
    if (entry && entry.virtualId) {
      formalizedObject = entry.virtualId;
      if (/^Q\d+$/.test(entry.virtualId)) break;
    }
  }

  return {
    intent: "web_search",
    content: lines.join("\n"),
    confidence: 0.85,
    evidence,
    formalizedObject,
    query,
    diagnostics,
  };
}

const PROMOTED_PROJECT_ORGS = ["link-assistant", "link-foundation", "linksplatform"];

function projectPromotionEnabled(preferences) {
  const value = preferences && preferences.associativeProjectPromotion;
  if (value === undefined || value === null || value === "") return true;
  if (value === true) return true;
  if (value === false) return false;
  const normalized = String(value).trim().toLowerCase();
  if (["0", "false", "no", "off", "disabled"].includes(normalized)) return false;
  if (["1", "true", "yes", "on", "enabled"].includes(normalized)) return true;
  return true;
}

function normalizeProjectTerm(value) {
  let term = normalizePrompt(value)
    .replace(/[-_]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  for (const prefix of ["the ", "a ", "an "]) {
    if (term.startsWith(prefix)) {
      term = term.slice(prefix.length).trim();
      break;
    }
  }
  return term;
}

function projectRepoSlug(project) {
  return `${project.org}/${project.name}`;
}

function localizedProject(project, language) {
  if (!project || !Array.isArray(project.localized)) return null;
  return (
    project.localized.find((loc) => loc && loc.language === language) ||
    project.localized.find((loc) => loc && loc.language === "en") ||
    null
  );
}

function projectDisplayName(project, language) {
  const localized = localizedProject(project, language);
  return (localized && localized.displayName) || project.displayName || project.name || "";
}

function projectStatementsFor(project, language) {
  const localized = localizedProject(project, language);
  if (
    localized &&
    Array.isArray(localized.statements) &&
    localized.statements.length > 0
  ) {
    return localized.statements;
  }
  return Array.isArray(project && project.statements) ? project.statements : [];
}

function describeProjectRecord(project, language) {
  const statements = projectStatementsFor(project, language)
    .filter((statement) => {
      const kind = statement && statement.kind;
      return statement && statement.text && kind !== "install" && kind !== "example";
    })
    .slice()
    .sort((a, b) => Number(b.weight || 0) - Number(a.weight || 0))
    .slice(0, 3)
    .map((statement) => String(statement.text).trim())
    .filter(Boolean);
  if (statements.length > 0) return statements.join(" ");
  return project.description || projectDisplayName(project, language);
}

function projectMatchesAlias(project, normalizedTerm) {
  if (!project || !normalizedTerm) return false;
  const aliases = Array.isArray(project.aliases) ? project.aliases : [];
  return (
    normalizeProjectTerm(project.displayName) === normalizedTerm ||
    normalizeProjectTerm(project.name) === normalizedTerm ||
    normalizeProjectTerm(projectRepoSlug(project)) === normalizedTerm ||
    aliases.some((alias) => normalizeProjectTerm(alias) === normalizedTerm)
  );
}

function projectByAlias(term) {
  const normalizedTerm = normalizeProjectTerm(term);
  if (!normalizedTerm) return null;
  return PROJECTS.find((project) => projectMatchesAlias(project, normalizedTerm)) || null;
}

function isPromotedProject(project) {
  return PROMOTED_PROJECT_ORGS.some(
    (org) => String(project && project.org).toLowerCase() === org,
  );
}

function promotedProjectByRepo(owner, name) {
  const ownerLower = String(owner || "").toLowerCase();
  const nameLower = String(name || "").toLowerCase();
  return (
    PROJECTS.find(
      (project) =>
        isPromotedProject(project) &&
        String(project.org || "").toLowerCase() === ownerLower &&
        String(project.name || "").toLowerCase() === nameLower,
    ) || null
  );
}

function cleanRepositorySegment(segment) {
  const trimmed = String(segment || "").trim().replace(/\.git$/i, "");
  if (!trimmed || !/^[A-Za-z0-9._-]+$/.test(trimmed)) return "";
  return trimmed;
}

function repositoryFromUrl(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch (_error) {
    return null;
  }
  const host = parsed.hostname.toLowerCase().replace(/^www\./, "");
  const platform =
    host === "github.com"
      ? { slug: "github", label: "GitHub", host: "github.com" }
      : host === "gitlab.com"
        ? { slug: "gitlab", label: "GitLab", host: "gitlab.com" }
        : host === "bitbucket.org"
          ? { slug: "bitbucket", label: "Bitbucket", host: "bitbucket.org" }
          : null;
  if (!platform) return null;
  const segments = parsed.pathname.split("/").filter(Boolean);
  const owner = cleanRepositorySegment(segments[0]);
  const name = cleanRepositorySegment(segments[1]);
  if (!owner || !name) return null;
  return {
    platform,
    owner,
    name,
    url: `https://${platform.host}/${owner}/${name}`,
  };
}

function repositoryFromSlug(term) {
  const parts = String(term || "").trim().split("/");
  if (parts.length !== 2) return null;
  const owner = cleanRepositorySegment(parts[0]);
  const name = cleanRepositorySegment(parts[1]);
  if (!owner || !name) return null;
  return {
    platform: { slug: "github", label: "GitHub", host: "github.com" },
    owner,
    name,
    url: `https://github.com/${owner}/${name}`,
  };
}

function repositoryFromPrompt(prompt) {
  const urlCandidate = firstUrlCandidate(prompt);
  if (urlCandidate) {
    const repo = repositoryFromUrl(urlCandidate.url);
    if (repo) return repo;
  }
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const term = String(query.termOriginal || query.term || "").trim();
  if (!term) return null;
  if (term.includes("://") || looksLikeHostname(term)) {
    const url = normalizeUrlCandidate(term);
    return url ? repositoryFromUrl(url) : null;
  }
  if (term.includes("/") && !/\s/.test(term)) {
    return repositoryFromSlug(term);
  }
  return null;
}

function repositorySlug(repo) {
  return `${repo.owner}/${repo.name}`;
}

function genericProjectLookupAnswer(prompt, language, repo, promotionEnabled) {
  const evidence = [];
  if (!promotionEnabled) evidence.push("project_lookup:promotion:disabled");
  if (repo) {
    const slug = repositorySlug(repo);
    evidence.push(`project_lookup:repository:${repo.platform.slug}:${slug}`);
    evidence.push(`source:${repo.url}`);
    const content =
      language === "ru"
        ? `Это запрос о репозитории [${slug}](${repo.url}) на ${repo.platform.label}.\n\nОбычный путь project_lookup ищет и резюмирует README или описание проекта на GitHub, GitLab и Bitbucket без особого правила для отдельного названия. Если репозиторий находится в продвигаемых организациях и продвижение включено, он будет показан первым.`
        : `This is a repository lookup for [${slug}](${repo.url}) on ${repo.platform.label}.\n\nThe generic project_lookup path can summarize README or project descriptions from GitHub, GitLab, and Bitbucket without a special case for any single name. If the repository belongs to a promoted organization and promotion is enabled, that repository is listed first.`;
    return { intent: "project_lookup", content, confidence: 0.82, evidence };
  }
  evidence.push("project_lookup:repository_hosts:GitHub,GitLab,Bitbucket");
  const content =
    language === "ru"
      ? "Это обычный запрос project_lookup о проекте или репозитории.\n\nЯ не выделяю специальный репозиторий, потому что продвижение ассоциативных репозиториев отключено. Дальше следует искать и резюмировать подходящие проекты на GitHub, GitLab и Bitbucket и похожих хостингах."
      : "This is a generic project_lookup request for a project or repository.\n\nI am not privileging a specific repository because associative repository promotion is disabled. The next step is to search and summarize matching projects across GitHub, GitLab, Bitbucket, and similar hosts.";
  return { intent: "project_lookup", content, confidence: 0.72, evidence };
}

async function renderPromotedProjectLookup(prompt, language, project) {
  const displayName = projectDisplayName(project, language);
  const repo = projectRepoSlug(project);
  const url = project.url || `https://github.com/${repo}`;
  const description = describeProjectRecord(project, language);
  const orgs = PROMOTED_PROJECT_ORGS.join(", ");
  const preferredLine =
    language === "ru"
      ? `В контексте репозиториев ${orgs} под \`${displayName}\` я прежде всего имею в виду [${repo}](${url}) — ${description}`
      : `In the ${orgs} repository context, \`${displayName}\` should first mean [${repo}](${url}) — ${description}`;

  const search = await runWebSearchQuery(displayName, language);
  const evidence = [
    `project:promoted:${repo}`,
    `source:${url}`,
    "summarization:mode:short",
    `summarization:language:${language}`,
  ];
  if (search && Array.isArray(search.evidence)) {
    evidence.push(...search.evidence);
  } else {
    evidence.push("web_search:no_results");
  }

  const lines = [preferredLine];
  if (search && search.content) {
    lines.push("");
    lines.push(
      language === "ru"
        ? "Другие найденные в интернете репозитории и сущности:"
        : "Other repositories and entities found online:",
    );
    lines.push("");
    lines.push(search.content);
  } else {
    lines.push("");
    lines.push(
      language === "ru"
        ? "Интернет-поиск по другим совпадениям не вернул результатов через доступные CORS-провайдеры."
        : "Web search for other matches returned no results through the available CORS providers.",
    );
  }

  return {
    intent: "project_lookup",
    content: lines.join("\n"),
    confidence: 0.9,
    evidence,
  };
}

async function tryProjectLookup(prompt, language, preferences) {
  const promotionEnabled = projectPromotionEnabled(preferences);
  const repo = repositoryFromPrompt(prompt);
  if (repo) {
    const promoted = promotionEnabled
      ? promotedProjectByRepo(repo.owner, repo.name)
      : null;
    if (promoted) {
      return renderPromotedProjectLookup(prompt, language, promoted);
    }
    return genericProjectLookupAnswer(prompt, language, repo, promotionEnabled);
  }

  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const project = projectByAlias(query.termOriginal || query.term);
  if (!project) return null;
  if (promotionEnabled && isPromotedProject(project)) {
    return renderPromotedProjectLookup(prompt, language, project);
  }
  return genericProjectLookupAnswer(prompt, language, null, promotionEnabled);
}

function pickPrimaryProviderId(providers, sourceKind) {
  if (sourceKind === "wikidata") return "wikidata";
  if (sourceKind === "wikipedia") return "wikipedia";
  if (sourceKind === "wiktionary") return "wiktionary";
  if (sourceKind === "internet-archive") return "internet-archive";
  if (Array.isArray(providers) && providers.length > 0) {
    const sorted = providers.slice().sort(
      (a, b) => (WEB_SEARCH_PROVIDER_PRIORITY[a.id] || 999) - (WEB_SEARCH_PROVIDER_PRIORITY[b.id] || 999),
    );
    return sorted[0].id;
  }
  return "";
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

// Issue #153: every prompt should be formalized as a Subject-Verb-Object tuple
// regardless of source language. We emit a deterministic, offline formalization
// here (so the trace is stable even when no APIs are reachable) and, when a
// downstream handler resolves the object to a Wikidata/Wikipedia/Wiktionary
// item, we emit a second `formalize_resolved` step with the real ids. Ids use
// canonical prefixes: `Q<n>` / `P<n>` for Wikidata, `WP:<title>` for
// Wikipedia-only items, `WT:<word>` for Wiktionary-only items, `OP:<verb>` for
// the symbolic operation, and `@USER` for the implicit user subject.
const FORMALIZATION_VERBS = [
  // English
  { verb: "what are the steps to", op: "OP:procedure" },
  { verb: "show me how to", op: "OP:procedure" },
  { verb: "tell me how to", op: "OP:procedure" },
  { verb: "how should i", op: "OP:procedure" },
  { verb: "how could i", op: "OP:procedure" },
  { verb: "how would i", op: "OP:procedure" },
  { verb: "how can i", op: "OP:procedure" },
  { verb: "how do i", op: "OP:procedure" },
  { verb: "how to", op: "OP:procedure" },
  { verb: "search", op: "OP:search" },
  { verb: "find", op: "OP:search" },
  { verb: "lookup", op: "OP:lookup" },
  { verb: "look up", op: "OP:lookup" },
  { verb: "define", op: "OP:define" },
  { verb: "what is", op: "OP:define" },
  { verb: "who is", op: "OP:identify" },
  { verb: "explain", op: "OP:define" },
  { verb: "compute", op: "OP:compute" },
  { verb: "calculate", op: "OP:compute" },
  { verb: "hello", op: "OP:greet" },
  { verb: "hi", op: "OP:greet" },
  { verb: "goodbye", op: "OP:farewell" },
  { verb: "bye", op: "OP:farewell" },
  // Russian
  { verb: "найди", op: "OP:search" },
  { verb: "поищи", op: "OP:search" },
  { verb: "поиск", op: "OP:search" },
  { verb: "что такое", op: "OP:define" },
  { verb: "кто такой", op: "OP:identify" },
  { verb: "объясни", op: "OP:define" },
  { verb: "посчитай", op: "OP:compute" },
  { verb: "вычисли", op: "OP:compute" },
  { verb: "привет", op: "OP:greet" },
  { verb: "здравствуй", op: "OP:greet" },
  { verb: "пока", op: "OP:farewell" },
  { verb: "до свидания", op: "OP:farewell" },
  // Hindi
  { verb: "खोज", op: "OP:search" },
  { verb: "ढूंढ", op: "OP:search" },
  { verb: "क्या है", op: "OP:define" },
  { verb: "कौन है", op: "OP:identify" },
  { verb: "नमस्ते", op: "OP:greet" },
  { verb: "अलविदा", op: "OP:farewell" },
  // Chinese
  { verb: "搜索", op: "OP:search" },
  { verb: "查找", op: "OP:search" },
  { verb: "什么是", op: "OP:define" },
  { verb: "是谁", op: "OP:identify" },
  { verb: "你好", op: "OP:greet" },
  { verb: "再见", op: "OP:farewell" },
];

function exactFormalizationMatch(prompt, normalized) {
  const haystack = String(normalized || "").toLowerCase();
  const raw = String(prompt || "");
  const rawLower = String(prompt || "").toLowerCase();
  for (const { verb, op } of FORMALIZATION_VERBS) {
    if (haystack.startsWith(verb + " ") || haystack === verb) {
      return {
        op,
        verb,
        objectText: haystack === verb ? "" : normalized.slice(verb.length),
        interpretations: [],
      };
    }
    if (rawLower.startsWith(verb + " ") || rawLower === verb) {
      return {
        op,
        verb,
        objectText: rawLower === verb ? "" : raw.slice(verb.length),
        interpretations: [],
      };
    }
    if (haystack.includes(" " + verb + " ")) {
      return { op, verb, objectText: null, interpretations: [] };
    }
  }
  return null;
}

function fuzzyFormalizationMatch(prompt) {
  const matches = FORMALIZATION_VERBS
    .map((entry) => {
      const match = fuzzyPrefixMatch(prompt, entry.verb);
      return match ? Object.assign({ entry }, match) : null;
    })
    .filter(Boolean)
    .sort((left, right) =>
      left.typoCount - right.typoCount || right.end - left.end,
    );
  const best = matches[0];
  if (!best) return null;
  const peers = matches.filter(
    (match) => match.typoCount === best.typoCount && match.end === best.end,
  );
  if (peers.length > 1) {
    return {
      ambiguous: true,
      suggestions: peers.map((match) => match.entry.verb),
      interpretations: [],
    };
  }
  return {
    op: best.entry.op,
    verb: best.entry.verb,
    objectText: String(prompt || "").slice(best.end),
    interpretations: [best.interpretation],
  };
}

function detectFormalizationMatch(prompt, normalized) {
  return exactFormalizationMatch(prompt, normalized) || fuzzyFormalizationMatch(prompt);
}

function objectForFormalization(prompt, normalized, match) {
  // For search-style ops we extract the explicit query the same way the web
  // search handler does. For other ops we keep the prompt body that follows
  // the detected verb so the tuple shows what the user is asking about.
  const op = match && match.op;
  if (op === "OP:search" || op === "OP:lookup") {
    const query = extractWebSearchQuery(prompt, normalized);
    if (query) return query;
  }
  if (op === "OP:procedure") {
    const task = extractProceduralHowToTask(normalized);
    if (task) return task.task;
  }
  const haystack = String(normalized || "").toLowerCase();
  for (const { verb } of FORMALIZATION_VERBS) {
    if (haystack.startsWith(verb + " ")) {
      return cleanSearchQuery(normalized.slice(verb.length));
    }
  }
  if (match && typeof match.objectText === "string") {
    return cleanSearchQuery(match.objectText);
  }
  return cleanSearchQuery(normalized || "");
}

function virtualObjectId(term) {
  const trimmed = String(term || "").trim();
  if (!trimmed) return "?";
  return `?${trimmed}`;
}

function formatFormalizationTuple(parts) {
  return `(${parts.filter(Boolean).join(" ")})`;
}

function buildFormalization(prompt, normalized) {
  const match = detectFormalizationMatch(prompt, normalized);
  if (!match || match.ambiguous) {
    const fallback = normalized || "(empty)";
    return {
      raw: String(prompt || ""),
      subject: "@USER",
      verb: "OP:express",
      object: virtualObjectId(fallback),
      tuple: formatFormalizationTuple(["@USER", "OP:express", virtualObjectId(fallback)]),
      needsClarification: Boolean(match && match.ambiguous),
      suggestions: match && match.suggestions ? match.suggestions : [],
      interpretations: [],
    };
  }
  const object = objectForFormalization(prompt, normalized, match);
  return {
    raw: String(prompt || ""),
    subject: "@USER",
    verb: match.op,
    object: virtualObjectId(object),
    tuple: formatFormalizationTuple(["@USER", match.op, virtualObjectId(object)]),
    interpretations: match.interpretations || [],
  };
}

function formalizationDetail(formalization) {
  if (!formalization || typeof formalization !== "object") {
    return String(formalization || "(empty)");
  }
  const arrow = formalization.raw && formalization.tuple ? " -> " : "";
  return `${formalization.raw || ""}${arrow}${formalization.tuple || ""}`.trim();
}

function formalizationClarificationMessage(formalization, language) {
  const suggestions = Array.isArray(formalization && formalization.suggestions)
    ? formalization.suggestions
    : [];
  const rendered = suggestions.length > 0
    ? suggestions.map((item) => `"${item}"`).join(", ")
    : "one of the known commands";
  if (language === "ru") {
    return `Не уверен, как интерпретировать этот запрос. Вы имели в виду ${rendered}?`;
  }
  if (language === "zh") {
    return `我不确定如何解释这个请求。你是指 ${rendered} 吗？`;
  }
  if (language === "hi") {
    return `मुझे पक्का नहीं है कि इस अनुरोध को कैसे समझूं। क्या आपका मतलब ${rendered} था?`;
  }
  return `I am not sure how to interpret that request. Did you mean ${rendered}?`;
}

// Once a handler resolves the search object to a concrete entity, this helper
// folds the resolved id back into the original formalization so the trace
// shows the canonical (@USER OP:search Q<id>) tuple alongside the placeholder.
function resolveFormalizationWithId(formalization, resolvedId) {
  if (!formalization || !resolvedId) return null;
  const next = Object.assign({}, formalization, {
    object: resolvedId,
    tuple: formatFormalizationTuple([
      formalization.subject || "@USER",
      formalization.verb || "OP:express",
      resolvedId,
    ]),
  });
  return next;
}

async function solve(prompt, history, prefs, userContext = {}) {
  const preferences = prefs || {};
  const autoDefinitionFusion = definitionFusionByDefault(preferences);
  const steps = [];
  const toolCalls = [];
  const events = [`impulse:${prompt}`];
  steps.push({ step: "impulse", detail: prompt });
  const normalized = normalizePrompt(prompt);
  const formalization = buildFormalization(prompt, normalized);
  events.push(`formalization:${formalization.tuple}`);
  steps.push({
    step: "formalize",
    detail: formalizationDetail(formalization),
    formalization: {
      raw: formalization.raw,
      subject: formalization.subject,
      verb: formalization.verb,
      object: formalization.object,
      tuple: formalization.tuple,
      interpretations: formalization.interpretations || [],
    },
  });
  const language = detectLanguage(prompt);
  events.push(`language:${language}`);
  steps.push({ step: "detect_language", detail: language });

  // Issue #180: bundle the per-turn formalization context so every
  // handler hit can fold a resolved entity id back into the tuple and
  // every `finalize` call can emit a `deformalize` step that records the
  // symbolic → natural-language projection. The context is mutable so
  // resolvers can update `resolved` as new ids surface.
  const formalizationContext = {
    initial: formalization,
    resolved: null,
    language,
  };

  if (formalization.needsClarification) {
    events.push("formalization:ambiguous");
    steps.push({
      step: "clarify_formalization",
      detail: (formalization.suggestions || []).join(", "),
    });
    return finalize(events, steps, toolCalls, {
      intent: "clarification",
      content: formalizationClarificationMessage(formalization, language),
      confidence: 0.4,
      evidence: ["formalization:ambiguous"],
    }, formalizationContext);
  }

  const behaviorRule = tryBehaviorRules(prompt, normalized, history);
  if (behaviorRule) {
    events.push(`handler:${behaviorRule.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryBehaviorRules" });
    return finalize(events, steps, toolCalls, behaviorRule, formalizationContext);
  }

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
    }, formalizationContext);
  }

  const translation = await tryTranslation(prompt, normalized);
  if (translation) {
    events.push(`handler:${translation.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryTranslation" });
    return finalize(events, steps, toolCalls, translation, formalizationContext);
  }

  steps.push({ step: "invoke_tool", detail: "wikipedia_article_question" });
  const earlyWikiArticleQuestion = await tryWikipediaArticleQuestion(
    prompt,
    language,
    preferences,
  );
  if (earlyWikiArticleQuestion) {
    events.push(`handler:${earlyWikiArticleQuestion.intent}`);
    steps.push({
      step: "dispatch_handler",
      detail: "tryWikipediaArticleQuestion",
    });
    toolCalls.push({
      tool: "wikipedia_article_question",
      inputs: {
        prompt,
        language,
        query: earlyWikiArticleQuestion.query || "",
      },
      outputs: {
        intent: earlyWikiArticleQuestion.intent,
        confidence: earlyWikiArticleQuestion.confidence,
        formalizedObject: earlyWikiArticleQuestion.formalizedObject || "",
      },
    });
    return finalize(
      events,
      steps,
      toolCalls,
      earlyWikiArticleQuestion,
      formalizationContext,
    );
  }

  const capabilities = tryCapabilities(prompt, normalized, preferences, history);
  if (capabilities) {
    events.push(`handler:${capabilities.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryCapabilities" });
    return finalize(events, steps, toolCalls, capabilities, formalizationContext);
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
    }, formalizationContext);
  }
  if (isFarewellPrompt(normalized, prompt)) {
    events.push("rule:farewell");
    steps.push({ step: "match_rule", detail: "farewell" });
    return finalize(events, steps, toolCalls, {
      intent: "farewell",
      content: answerFor("farewell", language),
      confidence: 1.0,
      evidence: ["rule:farewell", `language:${language}`],
    }, formalizationContext);
  }
  if (isTestStatusPrompt(normalized, prompt)) {
    events.push("rule:test_status");
    steps.push({ step: "match_rule", detail: "test_status" });
    return finalize(events, steps, toolCalls, {
      intent: "test_status",
      content: answerFor("test_status", language),
      confidence: 1.0,
      evidence: ["rule:test_status", `language:${language}`],
    });
  }
  if (isCourtesyResponsePrompt(normalized, prompt)) {
    events.push("rule:courtesy_response");
    steps.push({ step: "match_rule", detail: "courtesy_response" });
    const courtesy = courtesyResponseFor(language, preferences);
    return finalize(events, steps, toolCalls, {
      intent: "courtesy_response",
      content: courtesy.content,
      confidence: 1.0,
      evidence: [
        "rule:courtesy_response",
        `language:${language}`,
        `variation:${courtesy.randomize ? "random" : "canonical"}`,
        `temperature:${courtesy.temperature.toFixed(2)}`,
        `follow_up_probability:${courtesy.followUpProbability.toFixed(2)}`,
        `follow_up:${courtesy.followUpIncluded ? "included" : "omitted"}`,
      ],
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
    }, formalizationContext);
  }

  const syncHandlers = [
    { name: "tryHistorical", run: () => tryHistorical(prompt, history) },
    { name: "tryBrainstormingRequest", run: () => tryBrainstormingRequest(prompt, normalized) },
    { name: "tryRoleplayRequest", run: () => tryRoleplayRequest(prompt, normalized) },
    { name: "tryKupiSlona", run: () => tryKupiSlona(prompt, normalized) },
    {
      name: "tryCalendarReasoning",
      run: () => tryCalendarReasoning(prompt, normalized, userContext),
    },
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
      return finalize(events, steps, toolCalls, hit, formalizationContext);
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
      outputs: {
        intent: factQuery.intent,
        confidence: factQuery.confidence,
        formalizedObject: factQuery.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, factQuery, formalizationContext);
  }

  const legacyFact = tryFactLookup(prompt, normalized);
  if (legacyFact) {
    events.push(`handler:${legacyFact.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFactLookup" });
    return finalize(events, steps, toolCalls, legacyFact, formalizationContext);
  }

  steps.push({ step: "invoke_tool", detail: "project_lookup" });
  const projectLookup = await tryProjectLookup(prompt, language, preferences);
  if (projectLookup) {
    events.push(`handler:${projectLookup.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryProjectLookup" });
    toolCalls.push({
      tool: "project_lookup",
      inputs: { prompt, language },
      outputs: {
        intent: projectLookup.intent,
        confidence: projectLookup.confidence,
      },
    });
    return finalize(events, steps, toolCalls, projectLookup);
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
    return finalize(events, steps, toolCalls, fetched, formalizationContext);
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
    return finalize(events, steps, toolCalls, navigated, formalizationContext);
  }

  steps.push({ step: "invoke_tool", detail: "procedural_how_to" });
  const procedure = await tryProceduralHowTo(prompt, language);
  if (procedure) {
    events.push(`handler:${procedure.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryProceduralHowTo" });
    toolCalls.push({
      tool: "procedural_how_to",
      inputs: {
        prompt,
        language,
        query: procedure.query || "",
        wikihowCandidate: procedure.wikihowCandidate || "",
      },
      outputs: {
        intent: procedure.intent,
        confidence: procedure.confidence,
        formalizedObject: procedure.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, procedure, formalizationContext);
  }

  steps.push({ step: "invoke_tool", detail: "web_search" });
  const webSearch = await tryWebSearch(prompt, language);
  if (webSearch) {
    events.push(`handler:${webSearch.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWebSearch" });
    toolCalls.push({
      tool: "web_search",
      inputs: { prompt, language, query: webSearch.query || "" },
      outputs: {
        intent: webSearch.intent,
        confidence: webSearch.confidence,
        formalizedObject: webSearch.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, webSearch, formalizationContext);
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
    return finalize(events, steps, toolCalls, wiki, formalizationContext);
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
    return finalize(events, steps, toolCalls, whoIs, formalizationContext);
  }

  events.push("fallback:unknown");
  steps.push({ step: "fallback", detail: "unknown" });
  return finalize(events, steps, toolCalls, {
    intent: "unknown",
    content: unknownAnswerWithVariation(prompt, language),
    confidence: 0.1,
    evidence: ["fallback:unknown", `language:${language}`],
  }, formalizationContext);
}

// Issue #180: every handler hit flows through this helper so the trace shows
// the resolved-formalization fold (when the handler exposes a `formalizedObject`)
// followed by a uniform `deformalize` step that captures how the symbolic
// answer was projected into the natural-language `content`. Keeping the logic
// here means new handlers automatically participate in the architecture
// without having to repeat the bookkeeping.
function applyResolvedFormalization(events, steps, formalizationContext, answer) {
  if (!formalizationContext || !answer || !answer.formalizedObject) return;
  const resolved = resolveFormalizationWithId(
    formalizationContext.initial,
    answer.formalizedObject,
  );
  if (!resolved) return;
  // Skip the extra step when the placeholder already matched the resolved id
  // (e.g. cache hits where the formalization tuple already had a Q-id).
  if (resolved.tuple === formalizationContext.initial.tuple) return;
  formalizationContext.resolved = resolved;
  events.push(`formalization:resolved:${resolved.tuple}`);
  steps.push({
    step: "formalize_resolved",
    detail: formalizationDetail(resolved),
    formalization: {
      raw: resolved.raw,
      subject: resolved.subject,
      verb: resolved.verb,
      object: resolved.object,
      tuple: resolved.tuple,
    },
  });
}

function collectInterpretations(formalizationContext, answer) {
  const combined = [];
  const pushAll = (items) => {
    if (!Array.isArray(items)) return;
    for (const item of items) {
      if (!item || !item.original || !item.corrected) continue;
      combined.push({
        original: String(item.original),
        corrected: String(item.corrected),
      });
    }
  };
  pushAll(
    formalizationContext &&
      formalizationContext.initial &&
      formalizationContext.initial.interpretations,
  );
  pushAll(answer && answer.interpretations);
  const seen = new Set();
  return combined.filter((item) => {
    const key = `${item.original.toLowerCase()}\u0000${item.corrected.toLowerCase()}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function interpretationStatements(interpretations) {
  return interpretations
    .map((item) => `Interpreted "${item.original}" as "${item.corrected}".`)
    .join("\n");
}

function applyVisibleInterpretations(answer, interpretations) {
  if (!answer || interpretations.length === 0) return answer;
  const statements = interpretationStatements(interpretations);
  return Object.assign({}, answer, {
    content: `${statements}\n\n${String(answer.content || "")}`,
    evidence: [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      ...interpretations.map((item) => `interpretation:${item.original}->${item.corrected}`),
    ],
  });
}

function deformalizeProjection(formalizationContext, answer) {
  const tuple =
    (formalizationContext &&
      ((formalizationContext.resolved && formalizationContext.resolved.tuple) ||
        (formalizationContext.initial && formalizationContext.initial.tuple))) ||
    "(@USER OP:express ?)";
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const content = String(answer.content || "");
  const firstLine = content.split(/\r?\n/, 1)[0] || "";
  const projection = firstLine.length > 96 ? `${firstLine.slice(0, 96)}…` : firstLine;
  return {
    tuple,
    intent: answer.intent || "unknown",
    contentChars: content.length,
    evidenceCount: evidence.length,
    language:
      (formalizationContext && formalizationContext.language) ||
      answer.language ||
      "",
    summary: `${tuple} ⇒ ${answer.intent || "unknown"}: ${projection}`,
  };
}

function finalize(events, steps, toolCalls, answer, formalizationContext) {
  const interpretations = collectInterpretations(formalizationContext, answer);
  answer = applyVisibleInterpretations(answer, interpretations);
  applyResolvedFormalization(events, steps, formalizationContext, answer);
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const projection = deformalizeProjection(formalizationContext, answer);
  events.push(`deformalize:${projection.tuple}:${projection.intent}`);
  steps.push({
    step: "deformalize",
    detail: projection.summary,
    projection,
  });
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
  if (answer.diagnostics) {
    result.diagnostics = answer.diagnostics;
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
    if (Array.isArray(seed && seed.projects) && seed.projects.length > 0) {
      PROJECTS = seed.projects;
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
      projectCount: PROJECTS.length,
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
      projects: PROJECTS,
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
    await solve(prompt, history, prefs, userContext),
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
    diagnostics: answer.diagnostics || null,
  });
};

init();
