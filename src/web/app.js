const {
  createElement: h,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} = React;

const APP_VERSION =
  document.querySelector('meta[name="formal-ai-version"]')?.content || "0.16.0";
const ASSET_VERSION =
  typeof window !== "undefined" ? window.FORMAL_AI_ASSET_VERSION || "" : "";
const ISSUE_REPOSITORY = "link-assistant/formal-ai";
const ISSUE_LABELS = "bug";
const UNKNOWN_ANSWER =
  "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";
const IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI proof of concept that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

// Issue #27: the sidebar advertises every prompt family that has a deterministic
// symbolic rule or seed-backed answer in the engine. The list intentionally
// mirrors the multilingual + hello-world end-to-end tests so any regression in
// the seed surfaces immediately when a user clicks the prompt.
const EXAMPLE_PROMPTS = [
  { label: "Greeting (en)", text: "Hi" },
  { label: "Greeting (ru)", text: "Привет" },
  { label: "Greeting (hi)", text: "नमस्ते" },
  { label: "Greeting (zh)", text: "你好" },
  { label: "Identity (en)", text: "Who are you?" },
  { label: "Identity (zh)", text: "你是谁?" },
  { label: "Hello world (Rust)", text: "Write me hello world program in Rust" },
  { label: "Hello world (Python)", text: "Create a hello world example in Python" },
  { label: "Hello world (JavaScript)", text: "Write hello world in JavaScript" },
  { label: "Hello world (TypeScript)", text: "Write hello world in TypeScript" },
  { label: "Hello world (Go)", text: "Show hello world in Go" },
  { label: "Hello world (C)", text: "Show hello world in C" },
  { label: "Concept (en)", text: "What is Rust?" },
  { label: "Concept (en/Wikipedia)", text: "Who is Donald Trump?" },
  { label: "Concept (ru/Wikipedia)", text: "Кто такой Илон Маск?" },
  { label: "Concept (ru)", text: "Что такое Википедия?" },
  { label: "Concept (zh)", text: "维基百科是什么?" },
  { label: "Concept in context", text: "What is IIR in machine learning?" },
  { label: "Recall (en)", text: "When did I ask about Rust?" },
  { label: "Recall (cross-conv)", text: "Find Wikipedia in another conversation" },
  { label: "Export memory", text: "Export memory" },
  { label: "Import memory", text: "Import memory" },
];

// Issue #27 R5: the demo iterates through the same Example prompts list so
// every advertised feature is exercised. The greeting variants come from
// `EXAMPLE_PROMPTS` (`Greeting (...)` rows) and feature prompts are the
// remainder, minus actions that trigger downloads / file pickers.
const DEMO_GREETING_LABELS = new Set([
  "Greeting (en)",
  "Greeting (ru)",
  "Greeting (hi)",
  "Greeting (zh)",
]);
const DEMO_EXCLUDED_LABELS = new Set(["Export memory", "Import memory"]);

function demoGreetings() {
  return EXAMPLE_PROMPTS.filter((entry) => DEMO_GREETING_LABELS.has(entry.label));
}

function demoFeaturePrompts() {
  return EXAMPLE_PROMPTS.filter(
    (entry) =>
      !DEMO_GREETING_LABELS.has(entry.label) &&
      !DEMO_EXCLUDED_LABELS.has(entry.label),
  );
}

// Persistent cursors so each demo cycle advances through the lists rather
// than repeating the same prompts forever. Wraps when the cursor runs off
// the end.
let demoGreetingCursor = 0;
let demoFeatureCursor = 0;

// Issue #27: typing "Export memory" / "Export your memory" (or a translation)
// in the chat input should trigger the Export memory button so the deterministic
// chat surface stays in sync with the toolbar. Same for Import memory. Each
// phrase is normalised to lower-case ASCII spaces so punctuation and casing
// differences do not break the trigger.
const MEMORY_ACTION_PHRASES = {
  export: [
    "export memory",
    "export your memory",
    "export the memory",
    "export full memory",
    "экспорт памяти",
    "экспортировать память",
    "экспортируй память",
    "экспортируй свою память",
    "स्मृति निर्यात करें",
    "अपनी स्मृति निर्यात करें",
    "导出记忆",
    "导出你的记忆",
    "导出全部记忆",
  ],
  import: [
    "import memory",
    "import new memory",
    "import your new memory",
    "import your memory",
    "импорт памяти",
    "импортировать память",
    "импортируй память",
    "импортируй новую память",
    "स्मृति आयात करें",
    "नई स्मृति आयात करें",
    "अपनी नई स्मृति आयात करें",
    "导入记忆",
    "导入新记忆",
    "导入你的新记忆",
  ],
};

function normalizeMemoryPrompt(text) {
  return String(text || "")
    .toLowerCase()
    .replace(/[\s  -​]+/g, " ")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

function recognizeMemoryAction(text) {
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;
  if (MEMORY_ACTION_PHRASES.export.some((phrase) => normalized === phrase)) {
    return "export";
  }
  if (MEMORY_ACTION_PHRASES.import.some((phrase) => normalized === phrase)) {
    return "import";
  }
  return null;
}

// Issue #27 R11: natural-language cross-conversation recall. The user types
// something like "when did I ask about Rust?" or "find Donald Trump in another
// conversation" and the assistant projects the append-only memory log onto
// matching events grouped by conversation. Patterns are prefix-based so the
// remainder of the prompt becomes the search term verbatim (after trimming
// quotes and trailing punctuation). `scope = 'other'` excludes the current
// conversation; `scope = 'all'` searches every conversation including the
// current one.
const RECALL_QUERY_PATTERNS = [
  { prefix: "when did i ask about ", scope: "all" },
  { prefix: "when did i ask ", scope: "all" },
  { prefix: "when did i mention ", scope: "all" },
  { prefix: "when did i talk about ", scope: "all" },
  { prefix: "have i asked about ", scope: "all" },
  { prefix: "have i mentioned ", scope: "all" },
  { prefix: "did i ask about ", scope: "all" },
  { prefix: "did i mention ", scope: "all" },
  { prefix: "search my conversations for ", scope: "all" },
  { prefix: "search conversations for ", scope: "all" },
  { prefix: "search my chats for ", scope: "all" },
  { prefix: "recall ", scope: "all" },
  { prefix: "когда я спрашивал про ", scope: "all" },
  { prefix: "когда я спрашивал о ", scope: "all" },
  { prefix: "когда я спрашивал ", scope: "all" },
  { prefix: "когда я упоминал ", scope: "all" },
  { prefix: "поиск по беседам ", scope: "all" },
  { prefix: "поиск в беседах ", scope: "all" },
  { prefix: "найди в беседах ", scope: "all" },
  { prefix: "我什么时候问过 ", scope: "all" },
  { prefix: "我什么时候问过", scope: "all" },
  { prefix: "我什么时候提到 ", scope: "all" },
  { prefix: "我什么时候提到", scope: "all" },
  { prefix: "搜索我的对话 ", scope: "all" },
  { prefix: "搜索我的对话", scope: "all" },
  { prefix: "在对话中搜索 ", scope: "all" },
  { prefix: "在对话中搜索", scope: "all" },
];

// Suffix forms ("...in another conversation", "...在其他对话中") that mark the
// recall as cross-conversation-only. The remainder before the suffix becomes
// the search term.
const RECALL_OTHER_SUFFIXES = [
  " in another conversation",
  " in other conversations",
  " in my other conversations",
  " in my conversations",
  " in another chat",
  " in other chats",
  " в другой беседе",
  " в других беседах",
  " в других чатах",
  "在其他对话中",
  "在另一个对话中",
];

// "find X in another conversation" — `find ` is the lead-in for the other-scope
// recall when paired with one of the suffixes above.
const RECALL_OTHER_PREFIXES = [
  "find ",
  "search for ",
  "look for ",
  "найди ",
  "поищи ",
  "查找 ",
  "查找",
  "搜索 ",
  "搜索",
];

function stripRecallTerm(term) {
  return String(term || "")
    .replace(/^["'«»『「]+/, "")
    .replace(/["'«»』」]+$/, "")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

// Extract the substring from `original` that corresponds to characters at
// positions [start, end) of the lowercased normalised form. We do not have a
// strict 1:1 character map because normalisation can collapse whitespace, so
// approximate by walking the original and skipping characters that the
// normaliser would also skip. The result is good enough to preserve user
// casing for terms like "Donald Trump" or "Илона Маска".
function recoverOriginalRange(original, normalized, start, end) {
  // Walk through `original` character by character, advancing a normalised
  // cursor whenever we emit a character that would survive normalisation.
  // When the normalised cursor enters [start, end), we capture characters
  // from `original` instead of from `normalized`.
  let nIdx = 0;
  let captured = "";
  let i = 0;
  let prevWasSpace = false;
  while (i < original.length && nIdx < end) {
    const ch = original[i];
    const lower = ch.toLowerCase();
    // Mirror normalizeMemoryPrompt's whitespace collapse: \s plus the
    // unicode-space block U+00A0 / U+2000–U+200B used by the seed corpus.
    if (/[\s\u00A0\u2000-\u200B]/.test(ch)) {
      if (!prevWasSpace) {
        if (nIdx >= start) captured += " ";
        nIdx++;
        prevWasSpace = true;
      }
      i++;
      continue;
    }
    prevWasSpace = false;
    if (nIdx >= start) captured += ch;
    nIdx += lower.length;
    i++;
  }
  return captured.trim();
}

function recognizeRecallQuery(text) {
  const original = String(text || "").trim();
  if (!original) return null;
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;

  // Try "find X in another conversation" — prefix + suffix combo.
  for (const suffix of RECALL_OTHER_SUFFIXES) {
    const suffixIdx = normalized.lastIndexOf(suffix);
    if (suffixIdx < 0) continue;
    const beforeSuffix = normalized.slice(0, suffixIdx);
    for (const prefix of RECALL_OTHER_PREFIXES) {
      if (beforeSuffix.startsWith(prefix)) {
        const normalizedTerm = stripRecallTerm(beforeSuffix.slice(prefix.length));
        if (!normalizedTerm) continue;
        const originalTerm = stripRecallTerm(
          recoverOriginalRange(original, normalized, prefix.length, suffixIdx),
        );
        return { term: originalTerm || normalizedTerm, scope: "other" };
      }
    }
  }

  // Prefix-only patterns ("when did I ask about X", "recall X").
  for (const { prefix, scope } of RECALL_QUERY_PATTERNS) {
    if (normalized.startsWith(prefix)) {
      const normalizedTerm = stripRecallTerm(normalized.slice(prefix.length));
      if (!normalizedTerm) continue;
      const originalTerm = stripRecallTerm(
        recoverOriginalRange(original, normalized, prefix.length, normalized.length),
      );
      return { term: originalTerm || normalizedTerm, scope };
    }
  }
  return null;
}

// Build a Markdown report of every message whose lowercased content contains
// `term`, grouped by conversation. `scope === 'other'` filters out the active
// conversation. `triggerText` is the user's recall request itself — skip
// events whose content equals it so the recall never matches the prompt that
// triggered it. `events` is the full append-only log from FormalAiMemory.
function buildRecallReport({ events, term, scope, currentConversationId, triggerText }) {
  const safeEvents = Array.isArray(events) ? events : [];
  const needle = String(term || "").toLowerCase();
  if (!needle) {
    return {
      content: "No search term recognised in the recall request.",
      matches: [],
    };
  }
  const triggerNormalized = String(triggerText || "").trim().toLowerCase();
  const groups = new Map();
  for (const event of safeEvents) {
    if (!event || (event.kind && event.kind !== "message")) continue;
    const content = String(event.content || "");
    if (!content.toLowerCase().includes(needle)) continue;
    if (triggerNormalized && content.trim().toLowerCase() === triggerNormalized) {
      continue;
    }
    const id = event.conversationId || "legacy";
    if (scope === "other" && id === (currentConversationId || "")) continue;
    let entry = groups.get(id);
    if (!entry) {
      entry = { id, title: "", events: [] };
      groups.set(id, entry);
    }
    if (!entry.title && event.role === "user" && event.conversationTitle) {
      entry.title = event.conversationTitle;
    }
    entry.events.push(event);
  }

  const groupList = Array.from(groups.values());
  // Fill in titles from the first user message of each group when the recorded
  // title is missing (legacy events without a conversationTitle field).
  for (const group of groupList) {
    if (!group.title) {
      const firstUser = group.events.find((e) => e.role === "user");
      if (firstUser && firstUser.content) {
        group.title = deriveConversationTitle(firstUser.content);
      } else if (group.id === "legacy") {
        group.title = "Earlier conversation";
      } else {
        group.title = "Untitled conversation";
      }
    }
    group.events.sort((a, b) => String(a.sentAt || "").localeCompare(String(b.sentAt || "")));
  }
  groupList.sort((left, right) => {
    const lLast = left.events[left.events.length - 1]?.sentAt || "";
    const rLast = right.events[right.events.length - 1]?.sentAt || "";
    return String(rLast).localeCompare(String(lLast));
  });

  const totalMatches = groupList.reduce((sum, g) => sum + g.events.length, 0);
  if (totalMatches === 0) {
    const scopeNote = scope === "other" ? " in any other conversation" : "";
    return {
      content: `No mentions of "${term}" found${scopeNote}.`,
      matches: [],
    };
  }

  const lines = [];
  const conversationCount = groupList.length;
  lines.push(
    `Found **${totalMatches}** mention${totalMatches === 1 ? "" : "s"} of "${term}" across **${conversationCount}** conversation${conversationCount === 1 ? "" : "s"}.`,
  );
  for (const group of groupList) {
    lines.push("");
    lines.push(`### ${group.title}`);
    for (const event of group.events) {
      const stamp = event.sentAt ? event.sentAt : "(no timestamp)";
      const role = event.role === "user" ? "user" : "assistant";
      const snippet = String(event.content || "").replace(/\s+/g, " ").trim();
      const trimmed = snippet.length > 160 ? `${snippet.slice(0, 157)}…` : snippet;
      lines.push(`- ${stamp} — ${role}: ${trimmed}`);
    }
  }
  return { content: lines.join("\n"), matches: groupList };
}

const PREFERENCE_DEFAULTS = {
  demoMode: true,
  diagnosticsMode: false,
  // Issue #27: each sidebar section is a VS Code-style collapsible region; the
  // last expand/collapse state is persisted via FormalAiPreferences so opening
  // the demo never reshuffles the user's layout.
  sidebarPromptsCollapsed: false,
  sidebarToolsCollapsed: false,
  sidebarTraceCollapsed: false,
  sidebarConversationsCollapsed: false,
  // Issue #27: random greeting variations are opt-in but default to on so
  // newcomers see the multilingual surface immediately.
  greetingVariations: true,
  // Issue #27: id of the conversation the user last typed in; on reload the
  // demo restores its event log into the main transcript. Empty string means
  // "no conversation yet — start a fresh one on first user input".
  currentConversationId: "",
  // Issue #27: Chat (single-turn Q&A) vs Agent (multi-step plan + execute) mode.
  // Persisted so the user keeps their preferred operating surface across
  // reloads. Agent mode in the browser sandbox decomposes the prompt into
  // sequential sub-tasks and runs each through the existing solver; a future
  // iteration will wire it to docker / WebVM execution.
  agentMode: false,
};

const MEMORY_EXPORT_FILENAME = "formal-ai-memory.lino";

function withAssetVersion(path) {
  if (!ASSET_VERSION) {
    return path;
  }
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}v=${encodeURIComponent(ASSET_VERSION)}`;
}

function recordMemoryEvent(payload) {
  if (typeof window === "undefined" || !window.FormalAiMemory) {
    return Promise.resolve(null);
  }
  try {
    return window.FormalAiMemory.appendEvent(payload).catch(() => null);
  } catch (_error) {
    return Promise.resolve(null);
  }
}

function downloadTextFile(filename, text) {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return;
  }
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function loadPreferences() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return { ...PREFERENCE_DEFAULTS };
  }
  try {
    return window.FormalAiPreferences.load(PREFERENCE_DEFAULTS);
  } catch (_error) {
    return { ...PREFERENCE_DEFAULTS };
  }
}

function persistPreferences(values) {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return;
  }
  try {
    window.FormalAiPreferences.save(values);
  } catch (_error) {
    // localStorage may be unavailable (private mode, sandboxed iframe); ignore.
  }
}

function randomItem(items) {
  return items[Math.floor(Math.random() * items.length)];
}

// Issue #27: conversations are grouped slices of the append-only event log.
// Each event records the id of the conversation that produced it; the UI then
// filters events on read. New ids are generated locally so they stay portable
// across browsers (no server round-trip required).
function generateConversationId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `conv-${crypto.randomUUID()}`;
  }
  const random = Math.random().toString(16).slice(2, 10);
  return `conv-${Date.now().toString(16)}-${random}`;
}

function deriveConversationTitle(text) {
  const trimmed = String(text || "").trim().replace(/\s+/g, " ");
  if (!trimmed) {
    return "New conversation";
  }
  if (trimmed.length <= 60) {
    return trimmed;
  }
  return `${trimmed.slice(0, 57)}…`;
}

// Group append-only events into per-conversation summaries (id, title,
// timestamps, message count). Events without a conversationId are aggregated
// under the synthetic "legacy" bucket so existing demos remain visible after
// the schema upgrade.
function groupConversations(events) {
  const safe = Array.isArray(events) ? events : [];
  const map = new Map();
  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event || event.kind && event.kind !== "message") {
      continue;
    }
    const id = event.conversationId || "legacy";
    let entry = map.get(id);
    if (!entry) {
      entry = {
        id,
        title: id === "legacy" ? "Earlier conversation" : "",
        firstAt: event.sentAt || "",
        lastAt: event.sentAt || "",
        messageCount: 0,
      };
      map.set(id, entry);
    }
    if (event.role === "user" && !entry.title && event.conversationTitle) {
      entry.title = event.conversationTitle;
    } else if (event.role === "user" && !entry.title) {
      entry.title = deriveConversationTitle(event.content);
    }
    if (event.sentAt && (!entry.firstAt || event.sentAt < entry.firstAt)) {
      entry.firstAt = event.sentAt;
    }
    if (event.sentAt && (!entry.lastAt || event.sentAt > entry.lastAt)) {
      entry.lastAt = event.sentAt;
    }
    entry.messageCount += 1;
  }
  const list = Array.from(map.values());
  list.sort((left, right) => {
    if (left.lastAt && right.lastAt) {
      return right.lastAt.localeCompare(left.lastAt);
    }
    return 0;
  });
  return list;
}

// Issue #27: agent-mode task decomposition. Splits a multi-step prompt into
// sequential sub-tasks on a small, deterministic set of separators that span
// the languages the demo already supports. The split is intentionally
// conservative — if no separator is present we return [trimmedPrompt] so a
// single-step task still runs through the same code path.
const AGENT_STEP_SEPARATORS = [
  /\s*;\s+/,
  /\s+then(?:\s*,)?\s+/i,
  /\s*,\s+(?:and\s+then|then|next)\s+/i,
  /\s*,\s+after\s+that\s+/i,
  /\s+потом\s+/i,
  /\s+затем\s+/i,
  /\s+после\s+этого\s+/i,
  /\s+然后\s*/,
  /\s+接着\s*/,
];

// Issue #27: leading conjunctions ("then", "and then", "потом", "затем",
// "next", "after that", "然后", "接着") are linkers between steps, not part of
// the task itself. Strip them so each split segment is a clean instruction.
const AGENT_LEADING_CONJUNCTIONS =
  /^(?:and\s+then|then|next|after\s+that|потом|затем|после\s+этого|然后|接着)[\s,:]+/i;

function decomposeAgentTask(text) {
  const trimmed = String(text || "").trim();
  if (!trimmed) return [];
  let segments = [trimmed];
  for (const sep of AGENT_STEP_SEPARATORS) {
    const next = [];
    for (const segment of segments) {
      const parts = segment.split(sep);
      for (const part of parts) {
        const cleaned = part.trim();
        if (cleaned) next.push(cleaned);
      }
    }
    segments = next;
  }
  return segments.map((segment) =>
    segment.replace(AGENT_LEADING_CONJUNCTIONS, "").trim(),
  ).filter((segment) => segment.length > 0);
}

function messagesForConversation(events, conversationId) {
  if (!conversationId) {
    return [];
  }
  const safe = Array.isArray(events) ? events : [];
  const out = [];
  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event || event.kind && event.kind !== "message") continue;
    if ((event.conversationId || "legacy") !== conversationId) continue;
    const evidence = Array.isArray(event.evidence) ? event.evidence : [];
    out.push(
      createMessage(event.role || "assistant", String(event.content || ""), {
        intent: event.intent,
        evidence,
      }),
    );
  }
  return out;
}

function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function timeLabel() {
  return new Date().toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function createMessage(role, content, extra = {}) {
  return {
    id: `${role}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    role,
    author: role === "user" ? "You" : "formal-ai",
    content,
    sentAt: timeLabel(),
    ...extra,
  };
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function markdownHtml(value) {
  const text = String(value ?? "");
  if (window.marked && window.DOMPurify) {
    const html = window.marked.parse(text, {
      breaks: true,
      gfm: true,
    });
    return { __html: window.DOMPurify.sanitize(html) };
  }

  return { __html: escapeHtml(text).replaceAll("\n", "<br>") };
}

function normalizePrompt(prompt) {
  return prompt.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function isIdentityPrompt(normalized) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  return (
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
    (has("introduce") && has("yourself"))
  );
}

function localFallbackAnswer(prompt) {
  const normalized = normalizePrompt(prompt);
  if (["hi", "hello", "hey"].includes(normalized)) {
    return {
      intent: "greeting",
      content: "Hi, how may I help you?",
    };
  }

  if (isIdentityPrompt(normalized)) {
    return {
      intent: "identity",
      content: IDENTITY_ANSWER,
    };
  }

  return {
    intent: "unknown",
    content: UNKNOWN_ANSWER,
  };
}

function createDemoTurns() {
  const greetings = demoGreetings();
  const features = demoFeaturePrompts();
  const turns = [];
  if (greetings.length > 0) {
    const greeting = greetings[demoGreetingCursor % greetings.length];
    demoGreetingCursor = (demoGreetingCursor + 1) % greetings.length;
    turns.push({ text: greeting.text, label: greeting.label });
  }
  if (features.length > 0) {
    const feature = features[demoFeatureCursor % features.length];
    demoFeatureCursor = (demoFeatureCursor + 1) % features.length;
    turns.push({ text: feature.text, label: feature.label });
  }
  return turns;
}

function appendCodeBlock(lines, value) {
  const text = String(value ?? "");
  const fence = text.includes("```") ? "````" : "```";
  lines.push(fence);
  lines.push(text);
  lines.push(fence);
}

function shortText(value, limit = 70) {
  const normalized = String(value ?? "").replace(/\s+/g, " ").trim();
  if (normalized.length <= limit) {
    return normalized;
  }

  return `${normalized.slice(0, limit - 3)}...`;
}

function promptBeforeMessage(messages, focusMessage) {
  let prompt = "";
  for (const message of messages) {
    if (message.role === "user") {
      prompt = message.content;
    }
    if (focusMessage && message.id === focusMessage.id) {
      break;
    }
  }
  return prompt;
}

function createIssueTitle(messages, focusMessage) {
  const prompt = promptBeforeMessage(messages, focusMessage);
  if (focusMessage?.intent === "unknown" && prompt) {
    return `Unknown prompt: ${shortText(prompt, 80)}`;
  }
  if (prompt) {
    return `Issue with dialog: ${shortText(prompt, 80)}`;
  }
  return "formal-ai demo issue report";
}

function createIssueReportBody({
  messages,
  focusMessage,
  workerState,
  demoMode,
  demoStatus,
  diagnosticsMode,
}) {
  const lines = [
    "## Environment",
    "",
    `- **Version**: ${APP_VERSION}`,
    `- **URL**: ${window.location.href}`,
    `- **User Agent**: ${navigator.userAgent}`,
    `- **Worker**: ${workerState}`,
    `- **Mode**: ${demoMode ? "demo" : "manual"}`,
    `- **Status**: ${demoStatus}`,
    `- **Diagnostics**: ${diagnosticsMode ? "on" : "off"}`,
    `- **Timestamp**: ${new Date().toISOString()}`,
    "",
    "## Dialog",
    "",
  ];

  if (messages.length === 0) {
    lines.push("No messages have been sent yet.");
  } else {
    messages.forEach((message, index) => {
      const reported = focusMessage?.id === message.id ? " (reported message)" : "";
      lines.push(`### ${index + 1}. ${message.author}${reported}`);
      lines.push("");
      lines.push(`- **Role**: ${message.role}`);
      lines.push(`- **Time**: ${message.sentAt}`);
      if (message.intent) {
        lines.push(`- **Intent**: intent: ${message.intent}`);
      }
      if (message.demoLabel) {
        lines.push(`- **Demo label**: ${message.demoLabel}`);
      }
      lines.push("");
      appendCodeBlock(lines, message.content);
      lines.push("");
    });
  }

  const prompt = promptBeforeMessage(messages, focusMessage);
  lines.push("");
  lines.push("## Reproduction Steps");
  lines.push("");
  lines.push(`1. Open ${window.location.href}`);
  if (prompt) {
    lines.push(`2. Send the prompt "${shortText(prompt, 120)}"`);
    lines.push("3. Click the report link on the dialog message");
  } else {
    lines.push("2. Use the demo until the issue occurs");
    lines.push("3. Click Report issue");
  }
  lines.push("");
  lines.push("## Description");
  lines.push("");
  lines.push("<!-- Please describe what looked wrong or incomplete. -->");
  lines.push("");
  lines.push("## Attach full memory (recommended)");
  lines.push("");
  lines.push(
    "Click **Export memory** in the top bar to save `formal-ai-memory.lino`. The file is the **full memory** of the agent — the entire seed (rules, concepts, tools, multilingual responses), your UI preferences, environment metadata, and the complete append-only event log of this session (every user turn, assistant reply, reasoning step, tool invocation) — so the maintainer can reconstruct the exact session.",
  );
  lines.push("");
  lines.push(
    "**Wrap the export in a `.zip` before attaching.** GitHub's issue uploader does not currently accept `.lino` files (see [supported file types](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files)). On any OS:",
  );
  lines.push("");
  lines.push("- macOS: right-click → *Compress*.");
  lines.push("- Windows: right-click → *Send to* → *Compressed (zipped) folder*.");
  lines.push("- Linux: `zip formal-ai-memory.zip formal-ai-memory.lino`.");
  lines.push("");
  lines.push(
    "**Redact sensitive content first.** The export contains everything you typed into the chat. Open `formal-ai-memory.lino` in any text editor and remove personal names, secrets, API keys, internal URLs, or any pasted code you are not comfortable publishing before zipping and attaching.",
  );
  lines.push("");

  return lines.join("\n");
}

function createIssueUrl(context) {
  const params = new URLSearchParams({
    title: createIssueTitle(context.messages, context.focusMessage),
    body: createIssueReportBody(context),
    labels: ISSUE_LABELS,
  });
  return `https://github.com/${ISSUE_REPOSITORY}/issues/new?${params.toString()}`;
}

function Message({ message, diagnosticsMode, reportIssueUrl }) {
  const evidence = diagnosticsMode ? (message.evidence ?? []) : [];
  const thinkingSteps = diagnosticsMode ? (message.thinkingSteps ?? []) : [];
  const reportLabel = message.intent === "unknown" ? "Report missing rule" : "Report issue";

  return h(
    "article",
    {
      className: `message ${message.role}`,
      "data-testid": "chat-message",
      "data-demo-label": message.demoLabel || null,
    },
    h("div", { className: "avatar", "aria-hidden": "true" }, message.role === "user" ? "Y" : "FA"),
    h(
      "div",
      { className: "message-body" },
      h(
        "div",
        { className: "message-meta" },
        h("strong", null, message.author),
        h("time", null, message.sentAt),
        diagnosticsMode && message.intent
          ? h("span", { className: "intent" }, `intent:${message.intent}`)
          : null,
      ),
      h("div", {
        className: "markdown-body",
        dangerouslySetInnerHTML: markdownHtml(message.content),
      }),
      evidence.length
        ? h(
            "div",
            { className: "evidence-list" },
            evidence.map((item) => h("span", { key: item }, item)),
          )
        : null,
      thinkingSteps.length
        ? h(
            "div",
            { className: "thinking-steps" },
            h("strong", null, "Thinking"),
            h(
              "ol",
              null,
              thinkingSteps.map((item) => h("li", { key: item }, item)),
            ),
          )
        : null,
      reportIssueUrl
        ? h(
            "div",
            { className: "message-actions" },
            h(
              "a",
              {
                href: reportIssueUrl,
                target: "_blank",
                rel: "noopener noreferrer",
              },
              reportLabel,
            ),
          )
        : null,
    ),
  );
}

// Issue #27: a VS Code-style collapsible sidebar section. When `collapsed` is
// false the section participates in the equal-share flex layout and scrolls
// independently; when true only the header remains visible.
function CollapsibleSection({
  title,
  collapsed,
  onToggle,
  testId,
  children,
}) {
  return h(
    "section",
    {
      className: `sidebar-section ${collapsed ? "is-collapsed" : "is-expanded"}`,
      "data-testid": testId,
      "data-collapsed": collapsed ? "true" : "false",
    },
    h(
      "button",
      {
        type: "button",
        className: "sidebar-section-header",
        "aria-expanded": collapsed ? "false" : "true",
        onClick: onToggle,
      },
      h("span", { className: "sidebar-section-caret", "aria-hidden": "true" }, collapsed ? "▶" : "▼"),
      h("h2", null, title),
    ),
    collapsed
      ? null
      : h("div", { className: "sidebar-section-body" }, children),
  );
}

function App() {
  const workerRef = useRef(null);
  const pendingResponses = useRef(new Map());
  const transcriptEndRef = useRef(null);
  const importInputRef = useRef(null);
  const [messages, setMessages] = useState([]);
  const [prompt, setPrompt] = useState("");
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("wasm worker");
  const [memoryStatus, setMemoryStatus] = useState("");
  const [seed, setSeed] = useState({
    raw: {},
    tools: [],
    concepts: [],
    responses: {},
  });
  const initialPreferences = useRef(loadPreferences());
  const [demoMode, setDemoMode] = useState(initialPreferences.current.demoMode);
  const [demoPhase, setDemoPhase] = useState("manual");
  const [demoCountdown, setDemoCountdown] = useState(null);
  const [diagnosticsMode, setDiagnosticsMode] = useState(
    initialPreferences.current.diagnosticsMode,
  );
  // Issue #27: sidebar collapse/expand state per section.
  const [sidebarPromptsCollapsed, setSidebarPromptsCollapsed] = useState(
    initialPreferences.current.sidebarPromptsCollapsed,
  );
  const [sidebarToolsCollapsed, setSidebarToolsCollapsed] = useState(
    initialPreferences.current.sidebarToolsCollapsed,
  );
  const [sidebarTraceCollapsed, setSidebarTraceCollapsed] = useState(
    initialPreferences.current.sidebarTraceCollapsed,
  );
  const [sidebarConversationsCollapsed, setSidebarConversationsCollapsed] = useState(
    initialPreferences.current.sidebarConversationsCollapsed,
  );
  const [greetingVariations, setGreetingVariations] = useState(
    initialPreferences.current.greetingVariations,
  );
  // Issue #27: agent mode runs the user's prompt as a multi-step plan instead
  // of a single Q&A. Persisted across reloads via preferences.
  const [agentMode, setAgentMode] = useState(
    initialPreferences.current.agentMode,
  );
  // Issue #27: a mobile-friendly slide-out menu that hosts the entire sidebar
  // plus the topbar action buttons. On wide screens the menu is hidden via CSS.
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  // Issue #27: conversations. `currentConversationId` is the thread the user is
  // typing in right now; on first user message the demo lazily mints a new id
  // if none is set. `conversations` is the sidebar-visible list of all known
  // threads, derived from the append-only event log and refreshed after every
  // turn.
  const [currentConversationId, setCurrentConversationId] = useState(
    initialPreferences.current.currentConversationId || "",
  );
  const [conversations, setConversations] = useState([]);
  const currentConversationRef = useRef(currentConversationId);
  const conversationTitlesRef = useRef(new Map());

  useEffect(() => {
    currentConversationRef.current = currentConversationId;
  }, [currentConversationId]);

  useEffect(() => {
    if (typeof window === "undefined" || !window.FormalAiSeed) return;
    let cancelled = false;
    window.FormalAiSeed.loadAll().then((loaded) => {
      if (cancelled) return;
      setSeed(loaded);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  // Issue #27: on mount, hydrate the conversation list from the append-only
  // event log and restore the active thread's messages. Operates purely as a
  // projection — no events are mutated.
  const refreshConversations = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      return [];
    }
    try {
      const events = await window.FormalAiMemory.listEvents();
      const list = groupConversations(events);
      list.forEach((entry) => {
        if (entry.title) {
          conversationTitlesRef.current.set(entry.id, entry.title);
        }
      });
      setConversations(list);
      return events;
    } catch (_error) {
      return [];
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    refreshConversations().then((events) => {
      if (cancelled || !Array.isArray(events) || events.length === 0) return;
      const initialId = initialPreferences.current.currentConversationId;
      if (!initialId) return;
      const restored = messagesForConversation(events, initialId);
      if (restored.length > 0) {
        setMessages(restored);
        setDemoMode(false);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [refreshConversations]);

  const handleExportMemory = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus("Memory unavailable");
      return;
    }
    try {
      const events = await window.FormalAiMemory.listEvents();
      const preferences = loadPreferences();
      const text = window.FormalAiMemory.exportFullMemory({
        seed,
        events,
        preferences,
        info: {
          version: APP_VERSION,
          url: window.location.href,
          userAgent: navigator.userAgent,
          workerState,
          mode: demoMode ? "demo" : "manual",
        },
      });
      downloadTextFile(MEMORY_EXPORT_FILENAME, text);
      const seedFileCount = seed && seed.raw ? Object.keys(seed.raw).length : 0;
      setMemoryStatus(
        `Exported full memory: ${events.length} event(s) + ${seedFileCount} seed file(s)`,
      );
    } catch (_error) {
      setMemoryStatus("Export failed");
    }
  }, [seed, workerState, demoMode]);

  const handleImportMemory = useCallback(async (event) => {
    const file = event.target.files && event.target.files[0];
    event.target.value = "";
    if (!file || typeof window === "undefined" || !window.FormalAiMemory) {
      return;
    }
    try {
      const text = await file.text();
      const imported = window.FormalAiMemory.importFullMemory(text);
      const inserted = await window.FormalAiMemory.importEvents(imported.events);
      const current = {
        agentInfo: seed && seed.agentInfo ? seed.agentInfo : {},
        info: { version: APP_VERSION },
      };
      const suggestions = window.FormalAiMemory.suggestMigrations({
        imported,
        current,
      });
      const headline =
        imported.kind === "bundle"
          ? `Imported ${inserted} event(s) from full bundle`
          : `Imported ${inserted} events`;
      if (suggestions.length > 0) {
        setMemoryStatus(
          `${headline}. Migration: ${suggestions.join(" / ")}`,
        );
      } else {
        setMemoryStatus(headline);
      }
    } catch (_error) {
      setMemoryStatus("Import failed");
    }
  }, [seed]);

  const triggerImportMemory = useCallback(() => {
    if (importInputRef.current) {
      importInputRef.current.click();
    }
  }, []);

  useEffect(() => {
    persistPreferences({
      demoMode,
      diagnosticsMode,
      sidebarPromptsCollapsed,
      sidebarToolsCollapsed,
      sidebarTraceCollapsed,
      sidebarConversationsCollapsed,
      greetingVariations,
      currentConversationId,
      agentMode,
    });
  }, [
    demoMode,
    diagnosticsMode,
    sidebarPromptsCollapsed,
    sidebarToolsCollapsed,
    sidebarTraceCollapsed,
    sidebarConversationsCollapsed,
    greetingVariations,
    currentConversationId,
    agentMode,
  ]);

  useEffect(() => {
    const worker = new Worker(withAssetVersion("formal_ai_worker.js"));
    workerRef.current = worker;
    worker.onmessage = (event) => {
      if (event.data.kind === "ready") {
        setWorkerState(event.data.mode);
        return;
      }

      const requestId = event.data.requestId;
      const resolver = pendingResponses.current.get(requestId);
      if (resolver) {
        pendingResponses.current.delete(requestId);
        resolver(event.data);
      }
    };

    return () => worker.terminate();
  }, []);

  useEffect(() => {
    transcriptEndRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  const greetingVariationsRef = useRef(greetingVariations);
  useEffect(() => {
    greetingVariationsRef.current = greetingVariations;
  }, [greetingVariations]);

  const agentModeRef = useRef(agentMode);
  useEffect(() => {
    agentModeRef.current = agentMode;
  }, [agentMode]);

  const requestAnswer = useCallback((text, history = []) => {
    const worker = workerRef.current;
    if (!worker) {
      return Promise.resolve(localFallbackAnswer(text));
    }

    return new Promise((resolve) => {
      const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
      pendingResponses.current.set(requestId, resolve);
      worker.postMessage({
        prompt: text,
        requestId,
        history,
        prefs: { greetingVariations: greetingVariationsRef.current },
      });
    });
  }, []);

  // Issue #27: assign every appended event to the current conversation, lazily
  // minting a fresh id on the first user message of a brand-new chat. The
  // returned object is { conversationId, conversationTitle } so the caller can
  // reuse it for follow-up records within the same turn (assistant reply,
  // reasoning steps, tool calls).
  const ensureConversation = useCallback((seedText) => {
    let id = currentConversationRef.current;
    let isNew = false;
    if (!id) {
      id = generateConversationId();
      isNew = true;
      currentConversationRef.current = id;
      setCurrentConversationId(id);
    }
    let title = conversationTitlesRef.current.get(id);
    if (!title && seedText) {
      title = deriveConversationTitle(seedText);
      conversationTitlesRef.current.set(id, title);
    }
    return { conversationId: id, conversationTitle: title || "", isNew };
  }, []);

  const appendUserMessage = useCallback((text, extra = {}) => {
    const { conversationId, conversationTitle } = ensureConversation(text);
    const message = createMessage("user", text, extra);
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "user",
      content: text,
      sentAt: new Date().toISOString(),
      demoLabel: extra.demoLabel,
      conversationId,
      conversationTitle,
    });
  }, [ensureConversation]);

  const appendAssistantMessage = useCallback((answer) => {
    const source = workerRef.current ? "worker" : "fallback";
    const solverEvidence = Array.isArray(answer.evidence) ? answer.evidence : [];
    const evidence = answer.intent
      ? [`intent:${answer.intent}`, `source:${source}`, ...solverEvidence]
      : solverEvidence;
    const thinkingSteps = Array.isArray(answer.steps) && answer.steps.length > 0
      ? answer.steps.map((entry) => `${entry.step}: ${entry.detail}`)
      : [
          "Normalize prompt text",
          `Select symbolic intent ${answer.intent || "unknown"}`,
          `Render deterministic answer from ${source}`,
        ];
    const message = createMessage("assistant", answer.content, {
      intent: answer.intent,
      evidence,
      thinkingSteps,
    });
    setMessages((current) => [...current, message]);
    const sentAt = new Date().toISOString();
    const { conversationId, conversationTitle } = ensureConversation("");
    if (Array.isArray(answer.steps)) {
      answer.steps.forEach((entry) => {
        recordMemoryEvent({
          kind: "reasoning",
          role: "assistant",
          content: `${entry.step}: ${entry.detail}`,
          intent: answer.intent,
          sentAt,
          conversationId,
          conversationTitle,
        });
      });
    }
    if (Array.isArray(answer.toolCalls)) {
      answer.toolCalls.forEach((call) => {
        recordMemoryEvent({
          kind: "tool_call",
          role: "assistant",
          tool: call.tool,
          inputs: call.inputs,
          outputs: call.outputs,
          content: `tool:${call.tool}`,
          sentAt,
          conversationId,
          conversationTitle,
        });
      });
    }
    recordMemoryEvent({
      kind: "message",
      role: "assistant",
      content: answer.content,
      intent: answer.intent,
      evidence,
      sentAt,
      conversationId,
      conversationTitle,
    }).then(() => {
      // Refresh the sidebar so a brand-new conversation appears immediately.
      refreshConversations();
    });
  }, [ensureConversation, refreshConversations]);

  const conversationHistory = useCallback(
    () =>
      messages.map((message) => ({
        role: message.role,
        content: message.content,
        intent: message.intent,
        evidence: message.evidence,
      })),
    [messages],
  );

  // Issue #27: agent mode — run a decomposed task plan and merge the per-step
  // results into a single assistant message. Each step calls the same solver
  // the chat path uses, so deterministic intents (greeting, identity,
  // arithmetic, concept lookup, etc.) behave identically; the difference is
  // surface presentation, not solver semantics.
  const runAgentPlan = useCallback(
    async (steps, history) => {
      const lines = [];
      lines.push(`## Agent plan (${steps.length} steps)`);
      steps.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
      lines.push("");
      const aggregatedSteps = [];
      const aggregatedToolCalls = [];
      const aggregatedEvidence = [];
      const workingHistory = Array.isArray(history) ? history.slice() : [];
      for (let index = 0; index < steps.length; index += 1) {
        const step = steps[index];
        aggregatedSteps.push({
          step: "agent_plan",
          detail: `${index + 1}/${steps.length} ${step}`,
        });
        const answer = await requestAnswer(step, workingHistory);
        lines.push(`### Step ${index + 1}: ${step}`);
        lines.push(answer.content || "(no output)");
        lines.push("");
        if (Array.isArray(answer.steps)) {
          answer.steps.forEach((entry) => {
            aggregatedSteps.push({
              step: `agent_${index + 1}_${entry.step}`,
              detail: entry.detail,
            });
          });
        }
        if (Array.isArray(answer.toolCalls)) {
          aggregatedToolCalls.push(...answer.toolCalls);
        }
        if (Array.isArray(answer.evidence)) {
          aggregatedEvidence.push(
            ...answer.evidence.map((item) => `step_${index + 1}:${item}`),
          );
        }
        workingHistory.push({ role: "user", content: step });
        workingHistory.push({ role: "assistant", content: answer.content || "" });
      }
      appendAssistantMessage({
        intent: "agent_plan",
        content: lines.join("\n").trim(),
        confidence: 0.85,
        evidence: ["rule:agent_mode", `steps:${steps.length}`, ...aggregatedEvidence],
        steps: aggregatedSteps,
        toolCalls: aggregatedToolCalls,
      });
    },
    [requestAnswer, appendAssistantMessage],
  );

  async function sendText(text, extra = {}) {
    const trimmed = text.trim();
    if (!trimmed || pending) {
      return;
    }

    setPending(true);
    const history = conversationHistory();
    appendUserMessage(trimmed, extra);

    // Issue #27: short-circuit memory-action phrases to the corresponding
    // toolbar button before invoking the worker so the chat surface and the
    // sidebar stay in lock-step.
    const memoryAction = recognizeMemoryAction(trimmed);
    if (memoryAction === "export") {
      await handleExportMemory();
      appendAssistantMessage({
        intent: "memory_export",
        content:
          "Triggered Export memory. Your browser is downloading `formal-ai-memory.lino`.",
        confidence: 1.0,
        evidence: ["rule:memory_export"],
        steps: [{ step: "trigger_button", detail: "memory-export" }],
        toolCalls: [
          {
            tool: "export_memory",
            inputs: { prompt: trimmed },
            outputs: { intent: "memory_export" },
          },
        ],
      });
      setPending(false);
      return;
    }
    if (memoryAction === "import") {
      triggerImportMemory();
      appendAssistantMessage({
        intent: "memory_import",
        content:
          "Triggered Import memory. Pick a `.lino` file in the open dialog to restore the saved memory.",
        confidence: 1.0,
        evidence: ["rule:memory_import"],
        steps: [{ step: "trigger_button", detail: "memory-import" }],
        toolCalls: [
          {
            tool: "import_memory",
            inputs: { prompt: trimmed },
            outputs: { intent: "memory_import" },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27 R11: cross-conversation recall. Phrases like "when did I ask
    // about Rust" / "find Donald Trump in another conversation" search the
    // append-only memory log on the main thread (where FormalAiMemory lives)
    // and emit a Markdown report grouped by conversation. The recognition
    // happens before the worker round-trip so we never have to ferry the full
    // event log across the worker boundary.
    const recallQuery = recognizeRecallQuery(trimmed);
    if (recallQuery && typeof window !== "undefined" && window.FormalAiMemory) {
      let events = [];
      try {
        events = await window.FormalAiMemory.listEvents();
      } catch (_error) {
        events = [];
      }
      const report = buildRecallReport({
        events,
        term: recallQuery.term,
        scope: recallQuery.scope,
        currentConversationId: currentConversationRef.current,
        triggerText: trimmed,
      });
      appendAssistantMessage({
        intent: "conversation_recall",
        content: report.content,
        confidence: 1.0,
        evidence: [
          "rule:conversation_recall",
          `scope:${recallQuery.scope}`,
          `matches:${report.matches.reduce((sum, g) => sum + g.events.length, 0)}`,
        ],
        steps: [
          { step: "extract_term", detail: recallQuery.term },
          { step: "scan_memory", detail: `${events.length} event(s)` },
          { step: "group_by_conversation", detail: `${report.matches.length} group(s)` },
        ],
        toolCalls: [
          {
            tool: "conversation_recall",
            inputs: { term: recallQuery.term, scope: recallQuery.scope },
            outputs: {
              conversations: report.matches.length,
              matches: report.matches.reduce((sum, g) => sum + g.events.length, 0),
            },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27: agent mode decomposes the prompt into sub-tasks and executes
    // them sequentially, producing one consolidated assistant message with a
    // plan preamble and a per-step result list. Chat mode runs the single-step
    // path unchanged.
    if (agentModeRef.current) {
      const steps = decomposeAgentTask(trimmed);
      if (steps.length > 1) {
        await runAgentPlan(steps, history);
        setPending(false);
        return;
      }
    }

    const answer = await requestAnswer(trimmed, history);
    appendAssistantMessage(answer);
    setPending(false);
  }

  async function send() {
    const text = prompt.trim();
    if (!text) {
      return;
    }

    setPrompt("");
    await sendText(text);
  }

  function handleKeyDown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  useEffect(() => {
    if (!demoMode) {
      setDemoPhase("manual");
      setDemoCountdown(null);
      return undefined;
    }

    let cancelled = false;
    let countdownTimer = 0;

    async function runCycle() {
      const turns = createDemoTurns();
      setMessages([]);
      setPending(true);
      setDemoPhase("playing");
      setDemoCountdown(null);

      for (const turn of turns) {
        if (cancelled) {
          return;
        }

        appendUserMessage(turn.text, { demoLabel: turn.label });
        await wait(randomInt(700, 1300));
        const answer = await requestAnswer(turn.text);
        if (cancelled) {
          return;
        }
        appendAssistantMessage(answer);
        await wait(randomInt(900, 1500));
      }

      setPending(false);
      const waitSeconds = randomInt(10, 20);
      let remainingSeconds = waitSeconds;
      setDemoPhase("waiting");
      setDemoCountdown(remainingSeconds);
      countdownTimer = window.setInterval(() => {
        remainingSeconds -= 1;
        if (remainingSeconds <= 0) {
          window.clearInterval(countdownTimer);
          if (!cancelled) {
            runCycle();
          }
          return;
        }
        setDemoCountdown(remainingSeconds);
      }, 1000);
    }

    runCycle();

    return () => {
      cancelled = true;
      window.clearInterval(countdownTimer);
      setPending(false);
    };
  }, [appendAssistantMessage, appendUserMessage, demoMode, requestAnswer]);

  const lastAssistant = useMemo(
    () => [...messages].reverse().find((message) => message.role === "assistant"),
    [messages],
  );

  const demoStatus = demoMode
    ? demoPhase === "waiting" && demoCountdown !== null
      ? `Next dialog in ${demoCountdown}s`
      : "Demo playing"
    : "Manual mode";
  const reportContext = {
    messages,
    workerState,
    demoMode,
    demoStatus,
    diagnosticsMode,
  };
  const currentReportUrl = createIssueUrl(reportContext);

  return h(
    "main",
    { className: "app" },
    h(
      "header",
      { className: "topbar" },
      h("div", { className: "brand" }, h("span", { className: "mark" }, "FA"), h("strong", null, "formal-ai")),
      h(
        "div",
        { className: "topbar-actions" },
        h("span", { className: "demo-status", "data-testid": "demo-status", role: "status" }, demoStatus),
        diagnosticsMode ? h("span", { className: "status" }, workerState) : null,
        h(
          "a",
          {
            className: "report-button",
            "data-testid": "report-issue",
            href: currentReportUrl,
            target: "_blank",
            rel: "noopener noreferrer",
            title:
              "Report issue — open a pre-filled GitHub issue with the current session transcript. Click Export memory to save the full agent state, redact sensitive content, wrap it in a .zip, and attach it (GitHub does not accept .lino directly yet).",
            "aria-label": "Report issue",
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🐛"),
          h("span", { className: "btn-label" }, "Report issue"),
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-export",
            onClick: handleExportMemory,
            title:
              "Export memory — save the full agent state to formal-ai-memory.lino: the entire seed, UI preferences, environment metadata, and the append-only event log. Wrap the file in a .zip before attaching it to a GitHub issue (the issue uploader does not accept .lino directly yet).",
            "aria-label": "Export memory",
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "📤"),
          h("span", { className: "btn-label" }, "Export memory"),
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-import",
            onClick: triggerImportMemory,
            title:
              "Import memory — load a previous export. Accepts both the new full-memory bundle and the legacy demo_memory event-only log. Migration hints are shown next to this bar.",
            "aria-label": "Import memory",
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "📥"),
          h("span", { className: "btn-label" }, "Import memory"),
        ),
        h("input", {
          ref: importInputRef,
          type: "file",
          accept: ".lino,text/plain",
          style: { display: "none" },
          "data-testid": "memory-import-input",
          onChange: handleImportMemory,
        }),
        memoryStatus
          ? h(
              "span",
              {
                className: "memory-status",
                role: "status",
                "data-testid": "memory-status",
              },
              memoryStatus,
            )
          : null,
        h(
          "button",
          {
            type: "button",
            className: "diagnostics-toggle",
            "aria-pressed": diagnosticsMode,
            onClick: () => setDiagnosticsMode((value) => !value),
            title: diagnosticsMode
              ? "Hide reasoning trace, intent, evidence, and thinking-steps panels."
              : "Show reasoning trace, intent, evidence, and thinking-steps panels.",
            "aria-label": diagnosticsMode ? "Diagnostics on" : "Diagnostics",
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🔍"),
          h(
            "span",
            { className: "btn-label" },
            diagnosticsMode ? "Diagnostics on" : "Diagnostics",
          ),
        ),
        h(
          "button",
          {
            type: "button",
            className: "agent-toggle",
            "data-testid": "agent-toggle",
            "aria-pressed": agentMode,
            title: agentMode
              ? "Agent mode is on — switch back to single-turn chat."
              : "Chat mode — switch to agent mode and each message will be decomposed into sequential steps and executed as a plan.",
            "aria-label": agentMode ? "Agent mode" : "Chat mode",
            onClick: () => setAgentMode((value) => !value),
          },
          h(
            "span",
            { className: "btn-icon", "aria-hidden": "true" },
            agentMode ? "🤖" : "💬",
          ),
          h(
            "span",
            { className: "btn-label" },
            agentMode ? "Agent" : "Chat",
          ),
        ),
        h(
          "button",
          {
            type: "button",
            className: "mode-toggle",
            "aria-pressed": demoMode,
            onClick: () => setDemoMode((value) => !value),
            title: demoMode
              ? "Demo is on — stop the scripted dialog and resume manual chat."
              : "Start the scripted demo dialog.",
            "aria-label": demoMode ? "Demo on" : "Demo",
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🎬"),
          h(
            "span",
            { className: "btn-label" },
            demoMode ? "Demo on" : "Demo",
          ),
        ),
        h(
          "button",
          {
            type: "button",
            className: "mobile-menu-toggle",
            "data-testid": "mobile-menu-toggle",
            "aria-pressed": mobileMenuOpen,
            "aria-label": mobileMenuOpen ? "Close menu" : "Open menu",
            title: mobileMenuOpen
              ? "Close the side panel (conversations, prompts, tools)."
              : "Open the side panel (conversations, prompts, tools).",
            onClick: () => setMobileMenuOpen((value) => !value),
          },
          h(
            "span",
            { className: "btn-icon", "aria-hidden": "true" },
            mobileMenuOpen ? "✕" : "☰",
          ),
        ),
      ),
    ),
    mobileMenuOpen
      ? h("div", {
          className: "mobile-menu-backdrop",
          "data-testid": "mobile-menu-backdrop",
          onClick: () => setMobileMenuOpen(false),
        })
      : null,
    h(
      "section",
      { className: "workspace" },
      h(
        "aside",
        {
          className: `context-panel${mobileMenuOpen ? " is-mobile-open" : ""}`,
          "data-testid": "context-panel",
        },
        h(CollapsibleSection, {
          title: "Conversations",
          testId: "sidebar-conversations",
          collapsed: sidebarConversationsCollapsed,
          onToggle: () => setSidebarConversationsCollapsed((value) => !value),
          children: h(
            "div",
            { className: "conversation-list", "data-testid": "conversation-list" },
            h(
              "button",
              {
                type: "button",
                className: "conversation-new",
                "data-testid": "conversation-new",
                onClick: () => {
                  // Drop the current thread id so the next user message mints a
                  // fresh one and assigns its events accordingly.
                  currentConversationRef.current = "";
                  setCurrentConversationId("");
                  setMessages([]);
                  setDemoMode(false);
                  setPrompt("");
                },
              },
              "+ New conversation",
            ),
            conversations.length === 0
              ? h(
                  "p",
                  { className: "conversation-empty" },
                  "Start a new conversation.",
                )
              : h(
                  "ul",
                  {
                    className: "conversation-entries",
                    "data-testid": "conversation-entries",
                  },
                  conversations.map((entry) =>
                    h(
                      "li",
                      {
                        key: entry.id,
                        className:
                          entry.id === currentConversationId
                            ? "conversation-entry is-active"
                            : "conversation-entry",
                      },
                      h(
                        "button",
                        {
                          type: "button",
                          className: "conversation-entry-button",
                          "data-conversation-id": entry.id,
                          "aria-pressed": entry.id === currentConversationId,
                          onClick: async () => {
                            if (entry.id === currentConversationRef.current) {
                              return;
                            }
                            currentConversationRef.current = entry.id;
                            setCurrentConversationId(entry.id);
                            setDemoMode(false);
                            try {
                              const events =
                                await window.FormalAiMemory.listEvents();
                              setMessages(
                                messagesForConversation(events, entry.id),
                              );
                            } catch (_error) {
                              setMessages([]);
                            }
                          },
                        },
                        h(
                          "span",
                          { className: "conversation-entry-title" },
                          entry.title || "(empty)",
                        ),
                        h(
                          "span",
                          { className: "conversation-entry-meta" },
                          `${entry.messageCount} msg`,
                        ),
                      ),
                    ),
                  ),
                ),
          ),
        }),
        h(CollapsibleSection, {
          title: "Example prompts",
          testId: "sidebar-prompts",
          collapsed: sidebarPromptsCollapsed,
          onToggle: () => setSidebarPromptsCollapsed((value) => !value),
          children: h(
            "div",
            { className: "prompt-list", "data-testid": "example-prompts" },
            EXAMPLE_PROMPTS.map((entry) =>
              h(
                "button",
                {
                  key: entry.text,
                  type: "button",
                  "data-prompt-label": entry.label,
                  "data-prompt-text": entry.text,
                  onClick: () => {
                    setDemoMode(false);
                    setPrompt(entry.text);
                  },
                  title: entry.label,
                },
                entry.text,
              ),
            ),
          ),
        }),
        seed.tools && seed.tools.length > 0
          ? h(CollapsibleSection, {
              title: "Tools",
              testId: "sidebar-tools",
              collapsed: sidebarToolsCollapsed,
              onToggle: () => setSidebarToolsCollapsed((value) => !value),
              children: h(
                "div",
                { className: "tool-registry", "data-testid": "tool-registry" },
                h(
                  "ul",
                  { className: "tool-list" },
                  seed.tools.map((tool) =>
                    h(
                      "li",
                      {
                        key: tool.id,
                        className: `tool tool-mode-${tool.mode || "thinking"}`,
                        "data-testid": "tool-entry",
                        "data-tool-id": tool.id,
                        "data-tool-mode": tool.mode || "thinking",
                      },
                      h(
                        "div",
                        { className: "tool-head" },
                        h("strong", null, tool.name || tool.id),
                        h(
                          "span",
                          { className: "tool-mode" },
                          tool.mode === "agent" ? "agent" : "thinking",
                        ),
                      ),
                      tool.description
                        ? h("p", { className: "tool-desc" }, tool.description)
                        : null,
                    ),
                  ),
                ),
              ),
            })
          : null,
        diagnosticsMode
          ? h(CollapsibleSection, {
              title: "Trace",
              testId: "sidebar-trace",
              collapsed: sidebarTraceCollapsed,
              onToggle: () => setSidebarTraceCollapsed((value) => !value),
              children: h(
                "dl",
                { className: "trace-list" },
                h("div", null, h("dt", null, "Model"), h("dd", null, "formal-symbolic-poc")),
                h("div", null, h("dt", null, "Mode"), h("dd", null, demoStatus)),
                h("div", null, h("dt", null, "Intent"), h("dd", null, lastAssistant?.intent ?? "none")),
                h("div", null, h("dt", null, "Data"), h("dd", null, "data/source-index.lino")),
                h(
                  "div",
                  null,
                  h("dt", null, "Seed files"),
                  h(
                    "dd",
                    null,
                    Object.keys(seed.raw || {}).join(", ") || "(loading)",
                  ),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Tools loaded"),
                  h("dd", null, String((seed.tools || []).length)),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Concepts loaded"),
                  h("dd", null, String((seed.concepts || []).length)),
                ),
              ),
            })
          : null,
      ),
      h(
        "section",
        { className: "chat-panel" },
        h(
          "section",
          { className: "messages", "aria-live": "polite", "data-testid": "message-list" },
          messages.map((message) =>
            h(Message, {
              key: message.id,
              message,
              diagnosticsMode,
              reportIssueUrl:
                message.role === "assistant"
                  ? createIssueUrl({ ...reportContext, focusMessage: message })
                  : null,
            }),
          ),
          pending
            ? h(
                "article",
                { className: "message assistant pending" },
                h("div", { className: "avatar", "aria-hidden": "true" }, "FA"),
                h("div", { className: "message-body" }, h("div", { className: "typing" }, "Working")),
              )
            : null,
          h("div", { ref: transcriptEndRef }),
        ),
        h(
          "form",
          {
            className: "composer",
            onSubmit: (event) => {
              event.preventDefault();
              send();
            },
          },
          demoMode
            ? h(
                "p",
                { className: "composer-demo-hint", "data-testid": "composer-demo-hint" },
                "Demo is running — tap ",
                h("span", { className: "composer-demo-hint-icon", "aria-hidden": "true" }, "🎬"),
                " to stop and type your own message.",
              )
            : null,
          h(
            "div",
            { className: "composer-grid" },
            h("textarea", {
              value: prompt,
              rows: 3,
              placeholder: agentMode
                ? "Describe a multi-step task (separate steps with ; or 'then')"
                : "Message formal-ai",
              onChange: (event) => setPrompt(event.target.value),
              onKeyDown: handleKeyDown,
              disabled: demoMode,
              "data-testid": "chat-composer-input",
            }),
            h(
              "button",
              {
                className: "send-button",
                type: "submit",
                disabled: pending || demoMode || !prompt.trim(),
                "data-testid": "chat-composer-submit",
              },
              pending ? "..." : "Send",
            ),
          ),
        ),
      ),
    ),
  );
}

function wait(milliseconds) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, milliseconds);
  });
}

ReactDOM.createRoot(document.getElementById("root")).render(h(App));
