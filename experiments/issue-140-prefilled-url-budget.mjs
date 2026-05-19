// Issue #140: smoke-test the prefilled `Report issue` URL truncation budget.
// Mirrors `fitIssueUrl`, `appendDialogBlock`, `appendUserContextBlock`, and
// `createIssueReportBody` from src/web/app.js (kept in sync by hand — those
// helpers live in a React-coupled file that is not Node-importable without a
// bundler).
//
// Usage: node experiments/issue-140-prefilled-url-budget.mjs

const GITHUB_URL_MAX_LENGTH = 8192;
const URL_SAFETY_MARGIN = 16;
const URL_BUDGET = GITHUB_URL_MAX_LENGTH - URL_SAFETY_MARGIN;
const ISSUE_REPOSITORY = "link-assistant/formal-ai";
const ISSUE_LABELS = "bug";

function pickDialogFence(messages) {
  let fence = "```";
  while (messages.some((message) => String(message.content ?? "").includes(fence))) {
    fence += "`";
  }
  return fence;
}

function appendDialogBlock(lines, messages, effectiveFocus, options = {}) {
  if (messages.length === 0) {
    lines.push("No messages have been sent yet.");
    return;
  }
  lines.push("Legend: `U` = user, `A` = agent.");
  lines.push("");
  const fence = pickDialogFence(messages);
  lines.push(fence);
  const earlierOmitted = Math.max(0, Number(options.earlierOmitted) || 0);
  if (earlierOmitted > 0) {
    lines.push(
      `... omitted ${earlierOmitted} earlier ${earlierOmitted === 1 ? "message" : "messages"} ...`,
    );
  }
  messages.forEach((message) => {
    const prefix = message.role === "user" ? "U" : "A";
    const annotations = [];
    if (message.intent === "unknown") {
      annotations.push(`intent: ${message.intent}`);
    }
    if (effectiveFocus && effectiveFocus.id === message.id) {
      if (message.intent && message.intent !== "unknown") {
        annotations.push(`intent: ${message.intent}`);
      }
      annotations.push("reported");
    }
    const head = annotations.length > 0 ? `${prefix} (${annotations.join(", ")})` : prefix;
    const content = String(message.content ?? "");
    const [first, ...rest] = content.split("\n");
    lines.push(`${head}: ${first}`);
    rest.forEach((row) => lines.push(`   ${row}`));
  });
  lines.push(fence);
}

function formatUiLanguagesField(active, browserLanguagesStr) {
  const browserLanguages = browserLanguagesStr
    ? String(browserLanguagesStr)
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean)
    : [];
  const activeStr = String(active || "").trim();
  if (!activeStr && browserLanguages.length === 0) return "unknown";
  const lower = activeStr.toLowerCase();
  const primary = (lang) => String(lang).split(/[-_]/)[0].toLowerCase();
  const matchIndex = browserLanguages.findIndex(
    (lang) => primary(lang) === lower || lang.toLowerCase() === lower,
  );
  if (matchIndex >= 0) {
    return browserLanguages
      .map((lang, idx) => (idx === matchIndex ? `*${lang}*` : lang))
      .join(", ");
  }
  if (!activeStr) return browserLanguages.join(", ");
  if (browserLanguages.length === 0) return `*${activeStr}*`;
  return `*${activeStr}*, ${browserLanguages.join(", ")}`;
}

function formatUiField(context) {
  const parts = [];
  if (context.viewport) parts.push(`${context.viewport} viewport`);
  if (context.screen) parts.push(`${context.screen} screen`);
  if (context.userAgent) parts.push(`${context.userAgent} browser`);
  if (context.platform) parts.push(`${context.platform} platform`);
  return parts.join(", ");
}

function formatLocaleField(context) {
  const locale = context.locale ? String(context.locale).trim() : "";
  const timeZone = context.timeZone ? String(context.timeZone).trim() : "";
  if (locale && timeZone) return `${locale} (${timeZone})`;
  if (locale) return locale;
  if (timeZone) return timeZone;
  return "";
}

function formatThemeField(context) {
  const preference = context.themePreference || "auto";
  const scheme = context.colorScheme || "";
  if (scheme && scheme !== preference) return `${preference} (${scheme})`;
  return preference;
}

function appendUserContextBlock(lines, context) {
  const safe = context && typeof context === "object" ? context : {};
  const entries = [];
  const push = (label, value) => {
    if (value === undefined || value === null) return;
    const text = String(value).trim();
    if (!text) return;
    entries.push(`- **${label}**: ${text}`);
  };
  push("UI languages", formatUiLanguagesField(safe.uiLanguage, safe.browserLanguages));
  push("Theme", formatThemeField(safe));
  push("UI", formatUiField(safe));
  push("Locale", formatLocaleField(safe));
  if (safe.preferredLocation) {
    push("Preferred location", safe.preferredLocation);
  }
  push("Guess probability", `${safe.guessProbability || "unknown"}%`);
  push("Temperature", safe.temperature);
  if (safe.locationInference && !safe.preferredLocation) {
    push("Location", `inferred from ${safe.locationInference.replace(/;.*$/, "").trim()}`);
  }

  if (entries.length === 0) return;
  lines.push("## User Context");
  lines.push("");
  for (const entry of entries) lines.push(entry);
  lines.push("");
}

function truncateSingleLine(text, maxChars) {
  const str = String(text);
  if (str.length <= maxChars) return str;
  const markerTemplate = "... omitted XXXXX characters ...";
  const reservedForMarker = markerTemplate.length + 12;
  const half = Math.max(8, Math.floor((maxChars - reservedForMarker) / 2));
  if (half * 2 + reservedForMarker >= str.length) {
    const headOnly = str.slice(0, Math.max(8, maxChars - reservedForMarker));
    const omitted = str.length - headOnly.length;
    return `${headOnly}... omitted ${omitted} characters ...`;
  }
  const start = str.slice(0, half);
  const end = str.slice(str.length - half);
  const omitted = str.length - start.length - end.length;
  return `${start}... omitted ${omitted} characters ...${end}`;
}

function truncateMessageContent(content, maxChars) {
  const str = String(content ?? "");
  if (str.length <= maxChars) return str;
  const lines = str.split("\n");
  if (lines.length > 2) {
    const first = lines[0];
    const last = lines[lines.length - 1];
    const omitted = lines.length - 2;
    const combined = `${first}\n... omitted ${omitted} lines ...\n${last}`;
    if (combined.length <= maxChars) return combined;
    return `${truncateSingleLine(first, Math.floor((maxChars - 32) / 2))}\n... omitted ${omitted} lines ...\n${truncateSingleLine(last, Math.floor((maxChars - 32) / 2))}`;
  }
  return truncateSingleLine(str, maxChars);
}

function lastUnknownAssistantMessage(messages) {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    if (messages[i].role === "assistant" && messages[i].intent === "unknown") {
      return messages[i];
    }
  }
  return null;
}

function shortText(value, limit = 70) {
  const normalized = String(value ?? "").replace(/\s+/g, " ").trim();
  if (normalized.length <= limit) return normalized;
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
  const effectiveFocus = focusMessage ?? lastUnknownAssistantMessage(messages);
  const prompt = promptBeforeMessage(messages, effectiveFocus);
  if (effectiveFocus?.intent === "unknown" && prompt) {
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
  userContext,
  earlierOmitted = 0,
}) {
  const effectiveFocus = focusMessage ?? lastUnknownAssistantMessage(messages);
  const lines = [
    "## Environment",
    "",
    `- **Version**: dev`,
    `- **URL**: https://link-assistant.github.io/formal-ai/`,
    `- **Worker**: ${workerState}`,
    `- **Mode**: ${demoMode ? "demo" : "manual"}`,
    `- **Status**: ${demoStatus}`,
    `- **Diagnostics**: ${diagnosticsMode ? "on" : "off"}`,
    `- **Timestamp**: 2026-05-19T17:58:16.000Z`,
    "",
  ];
  appendUserContextBlock(lines, userContext);
  lines.push("## Dialog");
  lines.push("");
  appendDialogBlock(lines, messages, effectiveFocus, { earlierOmitted });

  const prompt = promptBeforeMessage(messages, effectiveFocus);
  lines.push("");
  lines.push("## Reproduction Steps");
  lines.push("");
  lines.push(`1. Open https://link-assistant.github.io/formal-ai/`);
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
  lines.push("## Attach full memory (optional)");
  lines.push("");
  lines.push(
    "Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.",
  );
  lines.push("");
  return lines.join("\n");
}

function buildIssueUrl(title, body, labels) {
  const params = new URLSearchParams({ title, body, labels });
  return `https://github.com/${ISSUE_REPOSITORY}/issues/new?${params.toString()}`;
}

function fitIssueUrl(context, buildBody) {
  const title = createIssueTitle(context.messages, context.focusMessage);
  const labels = ISSUE_LABELS;
  const messages = Array.isArray(context.messages) ? context.messages : [];

  let body = buildBody({ ...context, messages, earlierOmitted: 0 });
  let url = buildIssueUrl(title, body, labels);
  if (url.length <= URL_BUDGET) return { url, body, strategy: "full" };

  const lastTwo = messages.slice(-2);
  const earlierOmitted = messages.length - lastTwo.length;
  body = buildBody({ ...context, messages: lastTwo, earlierOmitted });
  url = buildIssueUrl(title, body, labels);
  if (url.length <= URL_BUDGET) return { url, body, strategy: "last-two" };

  for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
    const truncatedMessages = lastTwo.map((message) => ({
      ...message,
      content: truncateMessageContent(message.content, perMessageBudget),
    }));
    body = buildBody({ ...context, messages: truncatedMessages, earlierOmitted });
    url = buildIssueUrl(title, body, labels);
    if (url.length <= URL_BUDGET) {
      return { url, body, strategy: `truncated-${perMessageBudget}` };
    }
  }
  return { url, body, strategy: "exhausted" };
}

const baseUserContext = {
  uiLanguage: "ru",
  uiLanguagePreference: "auto",
  themePreference: "auto",
  uiSkin: "flat",
  chatStyle: "cards",
  composerStyle: "flat",
  composerAction: "attach",
  browserLanguage: "ru",
  browserLanguages: "ru, en-US, en, ru-RU",
  locale: "ru",
  timeZone: "Europe/Samara",
  colorScheme: "light",
  viewport: "1536x730",
  screen: "1536x864 @1.25x",
  userAgent:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36",
  platform: "Windows",
  preferredLocation: "",
  guessProbability: "100",
  temperature: "0.7",
  locationInference:
    "time zone / locale only; exact geolocation was not requested",
};

const unknownAnswer =
  "Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.";

function makeMessage(id, role, content, intent) {
  return { id, role, content, intent };
}

function makeUnknownTurnPair(prefix, count, prompt = "ва") {
  const messages = [];
  for (let i = 0; i < count; i += 1) {
    messages.push(makeMessage(`u${prefix}${i}`, "user", prompt));
    messages.push(makeMessage(`a${prefix}${i}`, "assistant", unknownAnswer, "unknown"));
  }
  return messages;
}

function extractDialogBlock(body) {
  const dialogStart = body.indexOf("## Dialog");
  if (dialogStart < 0) return "(no dialog)";
  const after = body.slice(dialogStart);
  const nextHeading = after.indexOf("\n## ", 1);
  return nextHeading < 0 ? after : after.slice(0, nextHeading);
}

function runCase(name, messages, options = {}) {
  const context = {
    messages,
    focusMessage: options.focusMessage ?? null,
    workerState: "wasm worker",
    demoMode: false,
    demoStatus: "Ручной режим",
    diagnosticsMode: false,
    userContext: baseUserContext,
  };
  const result = fitIssueUrl(context, createIssueReportBody);
  const fits = result.url.length <= GITHUB_URL_MAX_LENGTH;
  const preview = result.body.split("\n").slice(0, 24).join("\n");
  const dialog = extractDialogBlock(result.body);
  const dialogLines = dialog.split("\n");
  const dialogPreview =
    dialogLines.length <= 30
      ? dialog
      : [...dialogLines.slice(0, 15), `... [${dialogLines.length - 30} dialog lines omitted in this preview] ...`, ...dialogLines.slice(-15)].join("\n");
  console.log(`\n=== ${name} ===`);
  console.log(`URL length: ${result.url.length} (limit ${GITHUB_URL_MAX_LENGTH}, budget ${URL_BUDGET})`);
  console.log(`Strategy: ${result.strategy}`);
  console.log(`Fits under cap: ${fits ? "YES" : "NO"}`);
  console.log(`Body length: ${result.body.length} chars`);
  console.log("--- body header (first 24 lines) ---");
  console.log(preview);
  console.log("--- dialog block ---");
  console.log(dialogPreview);
  console.log("--- end ---");
  return result;
}

runCase("empty conversation", []);
runCase(
  "issue #140 reproduction (20 repeated unknown turns)",
  makeUnknownTurnPair("a", 20),
);
runCase(
  "extreme dialog (200 unknown turns)",
  makeUnknownTurnPair("b", 200),
);

const huge = [
  makeMessage("u1", "user", "Hi"),
  makeMessage("a1", "assistant", "Hi, how may I help you?"),
];
huge.push(
  makeMessage(
    "u2",
    "user",
    Array.from({ length: 60 }, (_, idx) => `Line ${idx + 1}: ${"x".repeat(80)}`).join("\n"),
  ),
);
huge.push(
  makeMessage(
    "a2",
    "assistant",
    `Reply with a very long single line: ${"y".repeat(6000)}.`,
    "unknown",
  ),
);
runCase("multi-line message + very long single line", huge);

console.log("\nAll cases finished.");
