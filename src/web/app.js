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
  { label: "Export memory", text: "Export memory" },
  { label: "Import memory", text: "Import memory" },
];

const DEMO_LANGUAGES = [
  {
    language: "Rust",
    prompts: [
      "Write me hello world program in Rust",
      "Create a Rust hello world example",
    ],
  },
  {
    language: "Python",
    prompts: [
      "Create a hello world example in Python",
      "Write hello world in Python",
    ],
  },
  {
    language: "JavaScript",
    prompts: [
      "Write hello world in JavaScript",
      "Create a JavaScript hello world program",
    ],
  },
  {
    language: "TypeScript",
    prompts: [
      "Write hello world in TypeScript",
      "Create a TypeScript hello world program",
    ],
  },
  {
    language: "Go",
    prompts: ["Show hello world in Go", "Write a Go hello world program"],
  },
  {
    language: "C",
    prompts: ["Show hello world in C", "Write a C hello world program"],
  },
];

const DEMO_GREETINGS = ["Hi", "Hello"];

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
  const greeting = randomItem(DEMO_GREETINGS);
  const language = randomItem(DEMO_LANGUAGES);
  return [
    {
      text: greeting,
      label: "Greeting",
    },
    {
      text: randomItem(language.prompts),
      label: language.language,
    },
  ];
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
    { className: `message ${message.role}`, "data-testid": "chat-message" },
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
  // Issue #27: a mobile-friendly slide-out menu that hosts the entire sidebar
  // plus the topbar action buttons. On wide screens the menu is hidden via CSS.
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

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
    });
  }, [
    demoMode,
    diagnosticsMode,
    sidebarPromptsCollapsed,
    sidebarToolsCollapsed,
    sidebarTraceCollapsed,
    sidebarConversationsCollapsed,
    greetingVariations,
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

  const appendUserMessage = useCallback((text, extra = {}) => {
    const message = createMessage("user", text, extra);
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "user",
      content: text,
      sentAt: new Date().toISOString(),
      demoLabel: extra.demoLabel,
    });
  }, []);

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
    if (Array.isArray(answer.steps)) {
      answer.steps.forEach((entry) => {
        recordMemoryEvent({
          kind: "reasoning",
          role: "assistant",
          content: `${entry.step}: ${entry.detail}`,
          intent: answer.intent,
          sentAt,
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
    });
  }, []);

  const conversationHistory = useCallback(
    () =>
      messages.map((message) => ({
        role: message.role,
        content: message.content,
      })),
    [messages],
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
              "Open a pre-filled GitHub issue with the current session transcript. Click Export memory to save the full agent state, redact sensitive content, wrap it in a .zip, and attach it (GitHub does not accept .lino directly yet).",
          },
          "Report issue",
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-export",
            onClick: handleExportMemory,
            title:
              "Save the full agent state to formal-ai-memory.lino: the entire seed, UI preferences, environment metadata, and the append-only event log. Wrap the file in a .zip before attaching it to a GitHub issue (the issue uploader does not accept .lino directly yet).",
          },
          "Export memory",
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-import",
            onClick: triggerImportMemory,
            title:
              "Load a previous export. Accepts both the new full-memory bundle and the legacy demo_memory event-only log. Migration hints are shown next to this bar.",
          },
          "Import memory",
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
          },
          diagnosticsMode ? "Diagnostics on" : "Diagnostics",
        ),
        h(
          "button",
          {
            type: "button",
            className: "mode-toggle",
            "aria-pressed": demoMode,
            onClick: () => setDemoMode((value) => !value),
          },
          demoMode ? "Demo on" : "Demo",
        ),
      ),
    ),
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
                  setMessages([]);
                  setDemoMode(false);
                  setPrompt("");
                },
              },
              "+ New conversation",
            ),
            h(
              "p",
              { className: "conversation-empty" },
              messages.length === 0
                ? "Start a new conversation."
                : `Current conversation: ${messages.length} message(s)`,
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
          h(
            "div",
            { className: "composer-grid" },
            h("textarea", {
              value: prompt,
              rows: 3,
              placeholder: "Message formal-ai",
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
