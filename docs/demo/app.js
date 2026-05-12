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

function initialMessages() {
  return [
    createMessage("user", "Hi", { intent: "greeting" }),
    createMessage("assistant", "Hi, how may I help you?", {
      intent: "greeting",
      evidence: ["response:greeting", "intent:greeting"],
    }),
  ];
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
      kind: "greeting",
      text: greeting,
      label: "Greeting",
    },
    {
      kind: "hello-world",
      text: randomItem(language.prompts),
      label: language.language,
    },
  ];
}

function Message({ message }) {
  const evidence = message.evidence ?? [];

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
        message.intent ? h("span", { className: "intent" }, message.intent) : null,
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
    ),
  );
}

function App() {
  const workerRef = useRef(null);
  const pendingResponses = useRef(new Map());
  const transcriptEndRef = useRef(null);
  const [messages, setMessages] = useState(initialMessages);
  const [prompt, setPrompt] = useState("");
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("wasm worker");
  const [demoMode, setDemoMode] = useState(false);
  const [demoState, setDemoState] = useState("manual");
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
    const evidence = answer.intent
      ? [`intent:${answer.intent}`, `source:${workerRef.current ? "worker" : "fallback"}`]
      : [];
    const message = createMessage("assistant", answer.content, {
      intent: answer.intent,
      evidence,
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
      setDemoState("manual");
      return undefined;
    }

    let cancelled = false;
    let cycleTimer = 0;

    async function runCycle() {
      const turns = createDemoTurns();
      setMessages([]);
      setPending(true);
      setDemoState("playing");

      for (const turn of turns) {
        if (cancelled) {
          return;
        }

        appendUserMessage(turn.text, { intent: turn.kind, demoLabel: turn.label });
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
      setDemoState(`next dialog in ${waitSeconds}s`);
      cycleTimer = window.setTimeout(runCycle, waitSeconds * 1000);
    }

    runCycle();

    return () => {
      cancelled = true;
      window.clearTimeout(cycleTimer);
      setPending(false);
    };
  }, [appendAssistantMessage, appendUserMessage, demoMode, requestAnswer]);

  const lastAssistant = useMemo(
    () => [...messages].reverse().find((message) => message.role === "assistant"),
    [messages],
  );

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
        h("span", { className: "status" }, workerState),
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
        h("h2", null, "Trace"),
        h(
          "dl",
          { className: "trace-list" },
          h("div", null, h("dt", null, "Model"), h("dd", null, "formal-symbolic-poc")),
          h("div", null, h("dt", null, "Mode"), h("dd", null, demoMode ? demoState : "manual")),
          h("div", null, h("dt", null, "Intent"), h("dd", null, lastAssistant?.intent ?? "none")),
          h("div", null, h("dt", null, "Data"), h("dd", null, "data/source-index.lino")),
        ),
      ),
      h(
        "section",
        { className: "chat-panel" },
        h(
          "section",
          { className: "messages", "aria-live": "polite", "data-testid": "message-list" },
          messages.map((message) => h(Message, { key: message.id, message })),
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
