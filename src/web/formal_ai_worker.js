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
  "I am formal-ai, a deterministic symbolic AI proof of concept that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

const FALLBACK_GREETING_ANSWER = "Hi, how may I help you?";

const FALLBACK_UNKNOWN_ANSWER =
  "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";

// Mutable runtime tables — populated from seed at init(). Each entry is
// `{ text, variants }` so the worker can return either the canonical phrase
// (for deterministic tests and tool calls) or a random variant (for greeting
// randomisation introduced in issue #27). Non-greeting intents currently ship
// a single phrase, so `variants` is `[text]` and randomisation is a no-op.
let MULTILINGUAL_ANSWERS = {
  greeting: {
    en: { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] },
  },
  identity: {
    en: { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] },
  },
  unknown: {
    en: { text: FALLBACK_UNKNOWN_ANSWER, variants: [FALLBACK_UNKNOWN_ANSWER] },
  },
};
let CONCEPTS = [];
let CONCEPT_CONTEXTS = [];
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
  if (intent === "identity") {
    return { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] };
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

function detectLanguage(prompt) {
  const text = String(prompt || "");
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
  return prompt.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
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

function extractConceptQuery(prompt) {
  const trimmedRaw = String(prompt || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!trimmedRaw) return null;

  const suffixes = conceptPatternsByKind("suffix");
  for (const suffix of suffixes) {
    if (trimmedRaw.endsWith(suffix)) {
      return finalizeConceptBody(
        trimmedRaw.slice(0, -suffix.length).trim(),
      );
    }
  }

  const lower = trimmedRaw.toLowerCase();
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

function evaluateArithmetic(expression) {
  const lower = expression.toLowerCase();
  const normalized = lower
    .replace(/\s+multiplied by\s+/g, " * ")
    .replace(/\s+divided by\s+/g, " / ")
    .replace(/\s+times\s+/g, " * ")
    .replace(/\s+plus\s+/g, " + ")
    .replace(/\s+minus\s+/g, " - ")
    .replace(/\s+modulo\s+/g, " % ")
    .replace(/\s+mod\s+/g, " % ");
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

function extractArithmeticExpression(prompt) {
  const trimmed = String(prompt || "").trim();
  if (!trimmed) return null;
  const lower = trimmed.toLowerCase();
  const prefixes = [
    "what is ",
    "what's ",
    "what does ",
    "calculate ",
    "compute ",
    "evaluate ",
    "how much is ",
    "solve ",
  ];
  let working = trimmed;
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      working = trimmed.slice(prefix.length);
      break;
    }
  }
  working = working.replace(/[?.!]+$/g, "").trim();
  working = working
    .replace(/\s+equals?$/i, "")
    .replace(/\s+=$/g, "")
    .trim();
  if (!working) return null;
  const workingLower = working.toLowerCase();
  const hasSymbolic = /[+\-*/%×·÷−]/.test(working);
  const hasWord =
    / plus | minus | times | multiplied by | divided by | modulo | mod /.test(
      ` ${workingLower} `,
    );
  const hasDigit = /[0-9]/.test(working);
  if (!hasDigit) return null;
  if (!hasSymbolic && !hasWord) return null;
  const allowed = /^[0-9+\-*/%().\s_×·÷−,a-zA-Z]+$/;
  if (!allowed.test(working)) return null;
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
    const value = evaluateArithmetic(expression);
    const formatted = formatArithmeticResult(value);
    return {
      intent: "calculation",
      content: `${expression.trim()} = ${formatted}`,
      confidence: 1.0,
      evidence: [`calculation:${expression.trim()}=${formatted}`],
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
        return { slug: data.pages[0].key, language: lang };
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

async function tryWikipediaLookup(prompt, language) {
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
  const humanUrl = humanizeUrl(summary.url);
  const body =
    `${summary.title}: ${summary.extract}\n\n` +
    `Source: [${humanUrl}](${summary.url}) (wikipedia).`;
  const evidence = [
    `wikipedia_lookup:${summary.title}`,
    `source:${humanUrl}`,
    `language:${summary.language}`,
  ];
  if (wikiContext) evidence.push(`wikipedia_lookup:context:${wikiContext}`);
  return {
    intent: "wikipedia_lookup",
    content: body,
    confidence: 0.85,
    evidence,
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

function extractFetchUrl(normalized) {
  const rest = normalized.slice("fetch ".length).trim();
  if (!rest || !rest.includes(".")) return null;
  if (rest.startsWith("http://") || rest.startsWith("https://")) return rest;
  return `https://${rest}`;
}

async function tryFetch(prompt) {
  const normalized = normalizePrompt(prompt);
  if (!isFetchPrompt(normalized)) return null;
  const url = extractFetchUrl(normalized);
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
      "The page is shown in the embedded frame below. Use the expand button to view it full-screen.",
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

async function solve(prompt, history, prefs) {
  const preferences = prefs || {};
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

  if (isGreetingPrompt(normalized, prompt)) {
    events.push("rule:greeting");
    steps.push({ step: "match_rule", detail: "greeting" });
    const randomize = preferences.greetingVariations !== false;
    return finalize(events, steps, toolCalls, {
      intent: "greeting",
      content: answerFor("greeting", language, { randomize: randomize }),
      confidence: 1.0,
      evidence: [
        "rule:greeting",
        `language:${language}`,
        `variation:${randomize ? "random" : "canonical"}`,
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
    });
  }

  const syncHandlers = [
    { name: "tryHistorical", run: () => tryHistorical(prompt, history) },
    { name: "tryArithmetic", run: () => tryArithmetic(prompt) },
    { name: "tryJavaScriptExecution", run: () => tryJavaScriptExecution(prompt) },
    { name: "tryConceptLookup", run: () => tryConceptLookup(prompt) },
    { name: "tryHelloWorld", run: () => tryHelloWorld(prompt) },
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
        hit.intent === "concept_lookup_in_context"
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

  steps.push({ step: "invoke_tool", detail: "wikipedia_lookup" });
  const wiki = await tryWikipediaLookup(prompt, language);
  if (wiki) {
    events.push(`handler:${wiki.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWikipediaLookup" });
    toolCalls.push({
      tool: "wikipedia_lookup",
      inputs: { prompt, language },
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
  const answer = await solve(prompt, history, prefs);
  postMessage({
    kind: "message",
    requestId: data.requestId,
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: answer.evidence,
    steps: answer.steps,
    toolCalls: answer.toolCalls,
  });
};

init();
