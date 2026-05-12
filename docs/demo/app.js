const {
  createElement: h,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} = React;

const UNKNOWN_ANSWER =
  "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";

const QUICK_PROMPTS = [
  "Hi",
  "Write me hello world program in Rust",
  "Create a hello world example in Python",
  "Write hello world in JavaScript",
  "Show hello world in Go",
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

function localFallbackAnswer(prompt) {
  const normalized = prompt.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
  if (["hi", "hello", "hey"].includes(normalized)) {
    return {
      intent: "greeting",
      content: "Hi, how may I help you?",
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

function Message({ message, diagnosticsMode }) {
  const evidence = diagnosticsMode ? (message.evidence ?? []) : [];
  const thinkingSteps = diagnosticsMode ? (message.thinkingSteps ?? []) : [];

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
    ),
  );
}

function App() {
  const workerRef = useRef(null);
  const pendingResponses = useRef(new Map());
  const transcriptEndRef = useRef(null);
  const [messages, setMessages] = useState([]);
  const [prompt, setPrompt] = useState("");
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("wasm worker");
  const [demoMode, setDemoMode] = useState(true);
  const [demoPhase, setDemoPhase] = useState("manual");
  const [demoCountdown, setDemoCountdown] = useState(null);
  const [diagnosticsMode, setDiagnosticsMode] = useState(false);
  const [previewInput, setPreviewInput] = useState(false);

  useEffect(() => {
    const worker = new Worker("formal_ai_worker.js");
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

  const requestAnswer = useCallback((text) => {
    const worker = workerRef.current;
    if (!worker) {
      return Promise.resolve(localFallbackAnswer(text));
    }

    return new Promise((resolve) => {
      const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
      pendingResponses.current.set(requestId, resolve);
      worker.postMessage({ prompt: text, requestId });
    });
  }, []);

  const appendUserMessage = useCallback((text, extra = {}) => {
    const message = createMessage("user", text, extra);
    setMessages((current) => [...current, message]);
  }, []);

  const appendAssistantMessage = useCallback((answer) => {
    const source = workerRef.current ? "worker" : "fallback";
    const evidence = answer.intent
      ? [`intent:${answer.intent}`, `source:${source}`]
      : [];
    const thinkingSteps = [
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
  }, []);

  async function sendText(text, extra = {}) {
    const trimmed = text.trim();
    if (!trimmed || pending) {
      return;
    }

    setPending(true);
    appendUserMessage(trimmed, extra);
    const answer = await requestAnswer(trimmed);
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
        { className: "context-panel" },
        h("h2", null, "Prompts"),
        h(
          "div",
          { className: "prompt-list" },
          QUICK_PROMPTS.map((item) =>
            h(
              "button",
              {
                key: item,
                type: "button",
                onClick: () => {
                  setDemoMode(false);
                  setPrompt(item);
                },
              },
              item,
            ),
          ),
        ),
        diagnosticsMode ? h("h2", null, "Trace") : null,
        diagnosticsMode
          ? h(
              "dl",
              { className: "trace-list" },
              h("div", null, h("dt", null, "Model"), h("dd", null, "formal-symbolic-poc")),
              h("div", null, h("dt", null, "Mode"), h("dd", null, demoStatus)),
              h("div", null, h("dt", null, "Intent"), h("dd", null, lastAssistant?.intent ?? "none")),
              h("div", null, h("dt", null, "Data"), h("dd", null, "data/source-index.lino")),
            )
          : null,
      ),
      h(
        "section",
        { className: "chat-panel" },
        h(
          "section",
          { className: "messages", "aria-live": "polite", "data-testid": "message-list" },
          messages.map((message) =>
            h(Message, { key: message.id, message, diagnosticsMode }),
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
            { className: "composer-toolbar" },
            h(
              "button",
              {
                type: "button",
                className: "preview-toggle",
                "aria-pressed": previewInput,
                onClick: () => setPreviewInput((value) => !value),
              },
              previewInput ? "Write" : "Preview",
            ),
          ),
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
            previewInput
              ? h("div", {
                  className: "composer-preview markdown-body",
                  dangerouslySetInnerHTML: markdownHtml(prompt || " "),
                })
              : null,
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
