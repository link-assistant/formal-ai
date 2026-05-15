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

// Mutable runtime tables — populated from seed at init().
let MULTILINGUAL_ANSWERS = {
  greeting: { en: FALLBACK_GREETING_ANSWER },
  identity: { en: FALLBACK_IDENTITY_ANSWER },
  unknown: { en: FALLBACK_UNKNOWN_ANSWER },
};
let CONCEPTS = [];
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

function answerFor(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  return (
    table[language] ||
    table.en ||
    (intent === "greeting"
      ? FALLBACK_GREETING_ANSWER
      : intent === "identity"
      ? FALLBACK_IDENTITY_ANSWER
      : FALLBACK_UNKNOWN_ANSWER)
  );
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

function lookupConcept(term) {
  const normalized = normalizeConceptTerm(term);
  if (!normalized) {
    return null;
  }
  return (
    CONCEPTS.find(
      (record) =>
        normalizeConceptTerm(record.term) === normalized ||
        normalizeConceptTerm(record.slug) === normalized ||
        record.aliases.some(
          (alias) => normalizeConceptTerm(alias) === normalized,
        ),
    ) || null
  );
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
  return kind === "suffix" ? DEFAULT_CONCEPT_SUFFIXES : DEFAULT_CONCEPT_PREFIXES;
}

function extractConceptTerm(prompt) {
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

function finalizeConceptBody(body) {
  let trimmed = String(body || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim()
    .toLowerCase();
  if (!trimmed) return null;
  for (const suffix of [" mean", " stand for"]) {
    if (trimmed.endsWith(suffix)) {
      trimmed = trimmed.slice(0, -suffix.length).trim();
      break;
    }
  }
  return trimmed || null;
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

function trySummarizeConversation(history) {
  if (!Array.isArray(history) || history.length === 0) return null;
  const bullets = history
    .filter((turn) => turn && turn.content)
    .map((turn) => `- ${turn.role}: ${turn.content}`);
  if (bullets.length === 0) return null;
  return {
    intent: "summarize_conversation",
    content: `Conversation so far:\n${bullets.join("\n")}`,
    confidence: 0.85,
    evidence: ["summarize_conversation", "prior_turn:user"],
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

function tryConceptLookup(prompt) {
  const term = extractConceptTerm(prompt);
  if (!term) return null;
  const record = lookupConcept(term);
  if (!record) return null;
  const body = `${record.term} (${record.category}): ${record.summary}\n\nSource: ${record.source} (${record.sourceKind}).`;
  return {
    intent: "concept_lookup",
    content: body,
    confidence: 0.9,
    evidence: [
      `concept_lookup:${record.slug}`,
      `source:${record.source}`,
    ],
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

async function fetchWikipediaSummary(term, language) {
  if (typeof fetch !== "function") return null;
  const hosts = wikipediaHostsFor(language);
  for (const host of hosts) {
    const slug = term
      .trim()
      .replace(/\s+/g, "_")
      .replace(/_+/g, "_");
    const url = `${host.url}/${encodeURIComponent(slug)}`;
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
      // Swallow network/parse errors and continue to the next host.
    }
  }
  return null;
}

async function tryWikipediaLookup(prompt, language) {
  const term = extractConceptTerm(prompt);
  if (!term) return null;
  // Avoid hitting the network for terms that already resolved in CONCEPTS;
  // that path is handled by `tryConceptLookup`.
  if (lookupConcept(term)) return null;
  const summary = await fetchWikipediaSummary(term, language);
  if (!summary) return null;
  const body = `${summary.title}: ${summary.extract}\n\nSource: ${summary.url} (wikipedia).`;
  return {
    intent: "wikipedia_lookup",
    content: body,
    confidence: 0.85,
    evidence: [
      `wikipedia_lookup:${summary.title}`,
      `source:${summary.url}`,
      `language:${summary.language}`,
    ],
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
  if (
    normalized.startsWith("summarize the conversation") ||
    normalized.startsWith("summarise the conversation") ||
    normalized === "summarize so far"
  ) {
    return trySummarizeConversation(history);
  }
  return null;
}

async function solve(prompt, history) {
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
    return finalize(events, steps, toolCalls, {
      intent: "greeting",
      content: answerFor("greeting", language),
      confidence: 1.0,
      evidence: ["rule:greeting", `language:${language}`],
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
      if (hit.intent === "concept_lookup") {
        toolCalls.push({
          tool: "concept_lookup",
          inputs: { prompt },
          outputs: { intent: hit.intent, confidence: hit.confidence },
        });
      }
      return finalize(events, steps, toolCalls, hit);
    }
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
  return {
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: [...evidence, ...trace],
    steps,
    toolCalls,
  };
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
      const merged = {
        greeting: Object.assign({}, MULTILINGUAL_ANSWERS.greeting, seed.responses.greeting || {}),
        identity: Object.assign({}, MULTILINGUAL_ANSWERS.identity, seed.responses.identity || {}),
        unknown: Object.assign({}, MULTILINGUAL_ANSWERS.unknown, seed.responses.unknown || {}),
      };
      Object.keys(seed.responses).forEach((key) => {
        if (!merged[key]) merged[key] = seed.responses[key];
      });
      MULTILINGUAL_ANSWERS = merged;
    }
    if (Array.isArray(seed && seed.concepts) && seed.concepts.length > 0) {
      CONCEPTS = seed.concepts;
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
      tools: TOOLS,
      agentInfo: AGENT_INFO,
      languageRules: LANGUAGE_RULES,
      promptPatterns: PROMPT_PATTERNS,
    });
    return;
  }
  const prompt = data.prompt || "";
  const history = Array.isArray(data.history) ? data.history : [];
  const answer = await solve(prompt, history);
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
