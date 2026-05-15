// Universal solver implementation for the demo worker.
//
// Every reasoning path here mirrors the Rust `FormalAiEngine` in
// `src/solver.rs` so the website, CLI, Telegram bot, library, and HTTP server
// all produce the same answers for the same prompts. The answer the user
// sees is always a projection of an append-only event log — there is no
// hardcoded prompt→answer table.

let wasm;
let mode = "wasm worker";

const IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI proof of concept that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

const UNKNOWN_ANSWER =
  "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";

const MULTILINGUAL_ANSWERS = {
  greeting: {
    ru: "Здравствуйте! Чем могу помочь?",
    hi: "नमस्ते! मैं आपकी क्या मदद कर सकता हूँ?",
    zh: "你好!请问有什么可以帮您的?",
  },
  identity: {
    ru:
      "Я formal-ai — детерминированный символьный AI proof of concept, который отвечает на основе локальных правил Links Notation и совместимых OpenAI-форматов. В этой демонстрации я не выполняю нейросетевой инференс.",
    hi:
      "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI proof of concept, जो स्थानीय Links Notation नियमों और OpenAI-संगत API आकारों से उत्तर देता है। इस डेमो में मैं कोई न्यूरल इन्फेरेन्स नहीं करता।",
    zh:
      "我是 formal-ai —— 一个确定性的符号化 AI 概念验证项目,根据本地的 Links Notation 规则和兼容 OpenAI 的 API 形式作答。本演示不进行任何神经网络推理。",
  },
  unknown: {
    ru:
      "Я пока не знаю символьного правила для этого запроса. Добавьте факт или правило в Links Notation и повторите запрос.",
    hi:
      "मेरे पास इस संकेत के लिए अभी कोई सीखा हुआ प्रतीकात्मक नियम नहीं है। Links Notation में एक तथ्य या नियम जोड़ें और फिर अनुरोध दोबारा भेजें।",
    zh:
      "我目前还没有针对该提示的符号规则。请用 Links Notation 添加事实或规则,然后再次发送请求。",
  },
};

function detectLanguage(prompt) {
  const text = String(prompt || "");
  for (const ch of text) {
    const code = ch.codePointAt(0);
    if (code >= 0x0400 && code <= 0x04ff) return "ru";
    if (code >= 0x0900 && code <= 0x097f) return "hi";
    if (code >= 0x4e00 && code <= 0x9fff) return "zh";
  }
  if (/[a-zA-Z]/.test(text)) return "en";
  return "en";
}

const CONCEPTS = [
  {
    slug: "concept_universal_solver",
    term: "universal solver",
    aliases: ["the universal solver", "universal problem solver"],
    category: "algorithm",
    summary:
      "The universal solver is formal-ai's deterministic 11-step loop: impulse, formalization, context, history, decomposition, TDD, synthesis, combination, verification, simplification, documentation. Every interface routes through the same loop.",
    source: "docs/case-studies/issue-12/README.md",
    sourceKind: "project-docs",
  },
  {
    slug: "concept_event_log",
    term: "event log",
    aliases: ["the event log", "eventlog", "append-only log"],
    category: "data-structure",
    summary:
      "The event log is formal-ai's append-only system of record. Every step in the universal solver loop appends an Event with a stable content-addressed id; the user-facing answer is, by construction, a projection of this log.",
    source: "docs/NON-GOALS.md",
    sourceKind: "project-docs",
  },
  {
    slug: "concept_links_notation",
    term: "Links Notation",
    aliases: ["links notation", "lino", "the links notation format"],
    category: "data-format",
    summary:
      "Links Notation is an indentation-based, untyped serialization format used by the Deep Theory project to represent links and link networks as portable text.",
    source: "https://github.com/linksplatform/Documentation",
    sourceKind: "project-docs",
  },
  {
    slug: "concept_doublet",
    term: "doublet",
    aliases: ["doublet link", "a doublet", "two-link"],
    category: "data-structure",
    summary:
      "A doublet is a link with exactly two endpoints. In Deep Theory it is the canonical reduction target for higher-arity links because every higher arity can be encoded as a chain of doublets.",
    source: "docs/VISION.md",
    sourceKind: "project-docs",
  },
  {
    slug: "concept_wikipedia",
    term: "Wikipedia",
    aliases: [
      "wikipedia",
      "the wikipedia",
      "en.wikipedia",
      "википедия",
      "विकिपीडिया",
      "维基百科",
      "維基百科",
    ],
    category: "encyclopedia",
    summary:
      "Wikipedia is a free, multilingual online encyclopedia written and maintained by a community of volunteer contributors through a model of open collaboration.",
    source: "https://en.wikipedia.org/wiki/Wikipedia",
    sourceKind: "wikipedia",
  },
  {
    slug: "concept_wikidata",
    term: "Wikidata",
    aliases: [
      "wikidata",
      "the wikidata knowledge graph",
      "викидата",
      "विकिडेटा",
      "维基数据",
      "維基數據",
    ],
    category: "structured-knowledge",
    summary:
      "Wikidata is a collaboratively edited multilingual knowledge graph hosted by the Wikimedia Foundation. It stores structured data items that power Wikipedia infoboxes and external knowledge applications.",
    source: "https://en.wikipedia.org/wiki/Wikidata",
    sourceKind: "wikipedia",
  },
  {
    slug: "concept_wiktionary",
    term: "Wiktionary",
    aliases: [
      "wiktionary",
      "the wiktionary dictionary",
      "викисловарь",
      "विक्षनरी",
      "维基词典",
      "維基辭典",
    ],
    category: "dictionary",
    summary:
      "Wiktionary is a multilingual, web-based free-content dictionary, available in many languages and including thesaurus, rhymes, translations, audio pronunciations, etymologies, and definitions.",
    source: "https://en.wikipedia.org/wiki/Wiktionary",
    sourceKind: "wikipedia",
  },
  {
    slug: "concept_webassembly",
    term: "WebAssembly",
    aliases: ["webassembly", "wasm", "the wasm runtime"],
    category: "runtime",
    summary:
      "WebAssembly (Wasm) is a binary instruction format for a stack-based virtual machine. It is designed as a portable compilation target for programming languages, enabling deployment on the web for client and server applications.",
    source: "https://en.wikipedia.org/wiki/WebAssembly",
    sourceKind: "wikipedia",
  },
  {
    slug: "concept_rust",
    term: "Rust",
    aliases: [
      "rust",
      "rust programming language",
      "the rust language",
      "rust-lang",
      "раст",
      "язык раст",
      "रस्ट",
      "रस्ट प्रोग्रामिंग",
      "rust 语言",
      "rust语言",
      "rust 程序设计语言",
    ],
    category: "programming-language",
    summary:
      "Rust is a multi-paradigm, general-purpose programming language that emphasises performance, type safety, and concurrency. It enforces memory safety without using a garbage collector.",
    source: "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    sourceKind: "wikipedia",
  },
];

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

function extractConceptTerm(prompt) {
  const trimmedRaw = String(prompt || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!trimmedRaw) return null;

  const hindiSuffixes = [" क्या है", " क्या होता है", " कौन है", " कौन हैं"];
  for (const suffix of hindiSuffixes) {
    if (trimmedRaw.endsWith(suffix)) {
      return finalizeConceptBody(
        trimmedRaw.slice(0, -suffix.length).trim(),
      );
    }
  }
  const chineseSuffixes = ["是什么", "是甚麼", "是谁", "是誰"];
  for (const suffix of chineseSuffixes) {
    if (trimmedRaw.endsWith(suffix)) {
      return finalizeConceptBody(
        trimmedRaw.slice(0, -suffix.length).trim(),
      );
    }
  }

  const lower = trimmedRaw.toLowerCase();
  const prefixes = [
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

function isIdentityPrompt(normalized, rawPrompt) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  const englishMatch =
    [
      "who are you",
      "what are you",
      "who is formal ai",
      "what is formal ai",
      "who is formalai",
      "what is formalai",
      "tell me about yourself",
      "introduce yourself",
    ].includes(normalized) ||
    (has("who") && has("you")) ||
    (has("what") && has("you")) ||
    ((has("who") || has("what")) && has("formal") && has("ai")) ||
    (has("tell") && has("yourself")) ||
    (has("introduce") && has("yourself"));
  if (englishMatch) return true;
  const raw = String(rawPrompt || "")
    .toLowerCase()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  return [
    "кто ты",
    "что ты",
    "तुम कौन हो",
    "你是谁",
    "你是誰",
  ].includes(raw);
}

function isGreetingPrompt(normalized, rawPrompt) {
  if (["hi", "hello", "hey"].includes(normalized)) return true;
  const raw = String(rawPrompt || "")
    .toLowerCase()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  return [
    "привет",
    "здравствуйте",
    "नमस्ते",
    "你好",
    "您好",
  ].includes(raw);
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
  const events = [`impulse:${prompt}`];
  const normalized = normalizePrompt(prompt);
  events.push(`formalization:${normalized || "(empty)"}`);
  const language = detectLanguage(prompt);
  events.push(`language:${language}`);

  if (isGreetingPrompt(normalized, prompt)) {
    events.push("rule:greeting");
    const content =
      MULTILINGUAL_ANSWERS.greeting[language] || "Hi, how may I help you?";
    return finalize(events, {
      intent: "greeting",
      content,
      confidence: 1.0,
      evidence: ["rule:greeting", `language:${language}`],
    });
  }
  if (isIdentityPrompt(normalized, prompt)) {
    events.push("rule:identity");
    const content = MULTILINGUAL_ANSWERS.identity[language] || IDENTITY_ANSWER;
    return finalize(events, {
      intent: "identity",
      content,
      confidence: 1.0,
      evidence: ["rule:identity", `language:${language}`],
    });
  }

  const syncHandlers = [
    () => tryHistorical(prompt, history),
    () => tryArithmetic(prompt),
    () => tryJavaScriptExecution(prompt),
    () => tryConceptLookup(prompt),
    () => tryHelloWorld(prompt),
  ];
  for (const handler of syncHandlers) {
    const hit = handler();
    if (hit) {
      events.push(`handler:${hit.intent}`);
      return finalize(events, hit);
    }
  }

  const wiki = await tryWikipediaLookup(prompt, language);
  if (wiki) {
    events.push(`handler:${wiki.intent}`);
    return finalize(events, wiki);
  }

  events.push("fallback:unknown");
  const unknownContent =
    MULTILINGUAL_ANSWERS.unknown[language] || UNKNOWN_ANSWER;
  return finalize(events, {
    intent: "unknown",
    content: unknownContent,
    confidence: 0.1,
    evidence: ["fallback:unknown", `language:${language}`],
  });
}

function finalize(events, answer) {
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const trace = events.map((event) => `trace:${event}`);
  return {
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: [...evidence, ...trace],
  };
}

async function init() {
  if (wasm !== undefined) return;
  try {
    const source = await fetch("formal_ai_worker.wasm");
    const bytes = await source.arrayBuffer();
    const module = await WebAssembly.instantiate(bytes, {});
    wasm = module.instance.exports;
  } catch (_error) {
    wasm = null;
    mode = "js fallback";
  }
  postMessage({ kind: "ready", mode });
}

self.onmessage = async (event) => {
  await init();
  const prompt = event.data.prompt || "";
  const history = Array.isArray(event.data.history) ? event.data.history : [];
  const answer = await solve(prompt, history);
  postMessage({
    kind: "message",
    requestId: event.data.requestId,
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: answer.evidence,
  });
};

init();
