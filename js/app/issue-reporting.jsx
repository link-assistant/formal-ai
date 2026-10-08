// Issue reporting: assembles report facts and labels and fits the prefilled
// GitHub issue URL within the URL length budget.

import { answerHasDetectedFailure } from "./detected-failure.js";
// Issue #839: the six-section issue-report document is no longer written here.
// `src/issue_report.rs` owns the format for every surface and
// `./issue-report.js` mirrors it for the browser (the wasm worker cannot link
// the Rust core), so this file only assembles the facts and the labels.
import { renderReportBody } from "./issue-report.js";
import { APP_VERSION, ISSUE_LABELS, ISSUE_REPOSITORY } from "./application-constants.jsx";
import { formatDiagnosticPayload, summarizeToolCall } from "./thinking-steps.jsx";
import { userContextFields } from "./user-context.jsx";

// Issue #78 renders the whole dialog as one fenced block with `U:` / `A:` line
// prefixes rather than a Markdown subsection per message, which is what keeps
// the prefilled `?body=` query string under GitHub's cap. Issue #839 moved that
// rendering into the shared builder; what stays here is the mapping from a
// browser message to a turn, including which turn the user asked about.
function reportTurns(messages, effectiveFocus) {
  return messages.map((message) => ({
    role: message.role,
    content: String(message.content ?? ""),
    intent: message.intent ?? "",
    reported: Boolean(effectiveFocus && effectiveFocus.id === message.id),
  }));
}

// Issue #140: GitHub caps the prefilled-issue URL at 8192 characters, so for
// chats that produce a long transcript we have to shrink the body. We keep
// the last two turns intact in shape and replace the rest with summary
// markers: "... omitted N earlier messages ..." for trimmed-out turns,
// "... omitted N lines ..." inside a multi-line message, and
// "... omitted N characters ..." inside a single long line. The exact ceiling
// is `GITHUB_URL_MAX_LENGTH` (documented limit); `URL_SAFETY_MARGIN` keeps a
// small buffer for the encoded `&labels=…` tail.
const GITHUB_URL_MAX_LENGTH = 8192;

const URL_SAFETY_MARGIN = 16;

const URL_BUDGET = GITHUB_URL_MAX_LENGTH - URL_SAFETY_MARGIN;

function truncateSingleLine(text, maxChars) {
  const str = String(text);
  if (str.length <= maxChars) return str;
  const markerTemplate = "... omitted XXXXX characters ...";
  const reservedForMarker = markerTemplate.length + 12;
  const half = Math.max(8, Math.floor((maxChars - reservedForMarker) / 2));
  if (half * 2 + reservedForMarker >= str.length) {
    // Not enough headroom for a useful trim — fall back to a head-only slice.
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

const REPORT_TRACE_MAX_CHARS = 2400;

const REPORT_TRACE_ITEM_LIMIT = 20;

function compactReportTraceValue(value, limit = 180) {
  const raw =
    value !== null && typeof value === "object"
      ? formatDiagnosticPayload(value)
      : String(value ?? "");
  const compact = raw.replace(/\s+/g, " ").trim();
  return truncateSingleLine(compact, limit);
}

function appendLimitedTraceItems(lines, items, formatter) {
  const safeItems = Array.isArray(items) ? items : [];
  if (safeItems.length <= REPORT_TRACE_ITEM_LIMIT) {
    safeItems.forEach((item) => {
      lines.push(formatter(item));
    });
    return;
  }

  const headCount = Math.ceil(REPORT_TRACE_ITEM_LIMIT / 2);
  const tailCount = REPORT_TRACE_ITEM_LIMIT - headCount;
  safeItems.slice(0, headCount).forEach((item) => {
    lines.push(formatter(item));
  });
  lines.push(`- ... omitted ${safeItems.length - REPORT_TRACE_ITEM_LIMIT} middle trace items ...`);
  safeItems.slice(safeItems.length - tailCount).forEach((item) => {
    lines.push(formatter(item));
  });
}

// Issue #839: the trace is returned as the lines that go inside the fence; the
// shared builder owns the heading and the fence itself.
function reasoningTraceLines(focusMessage) {
  if (!focusMessage || focusMessage.role !== "assistant") return [];

  const trace = [];
  if (focusMessage.intent) {
    trace.push(`intent: ${focusMessage.intent}`);
  }

  if (Array.isArray(focusMessage.evidence) && focusMessage.evidence.length > 0) {
    trace.push("evidence:");
    appendLimitedTraceItems(
      trace,
      focusMessage.evidence,
      (item) => `- ${compactReportTraceValue(item)}`,
    );
  }

  if (
    Array.isArray(focusMessage.diagnosticsSteps) &&
    focusMessage.diagnosticsSteps.length > 0
  ) {
    trace.push("diagnostics_steps:");
    appendLimitedTraceItems(trace, focusMessage.diagnosticsSteps, (entry) => {
      const step = compactReportTraceValue(entry?.step || "step", 80);
      const detail = entry?.formalization?.tuple || entry?.detail || "";
      return `- ${step}: ${compactReportTraceValue(detail)}`;
    });
  } else if (
    Array.isArray(focusMessage.thinkingSteps) &&
    focusMessage.thinkingSteps.length > 0
  ) {
    trace.push("thinking_steps:");
    appendLimitedTraceItems(
      trace,
      focusMessage.thinkingSteps,
      (item) => `- ${compactReportTraceValue(item)}`,
    );
  }

  if (
    Array.isArray(focusMessage.diagnosticsToolCalls) &&
    focusMessage.diagnosticsToolCalls.length > 0
  ) {
    trace.push("tool_calls:");
    appendLimitedTraceItems(trace, focusMessage.diagnosticsToolCalls, (call) => {
      const tool = compactReportTraceValue(call?.tool || "tool", 80);
      const summary = summarizeToolCall(call || {});
      return `- ${tool}: ${compactReportTraceValue(summary)}`;
    });
  }

  if (trace.length === 0) return [];

  return truncateMessageContent(trace.join("\n"), REPORT_TRACE_MAX_CHARS).split("\n");
}

function buildIssueUrl(title, body, labels) {
  const params = new URLSearchParams({ title, body, labels });
  return `https://github.com/${ISSUE_REPOSITORY}/issues/new?${params.toString()}`;
}

function buildIssueUrlForMessages(context, buildBody, title, labels, messages, earlierOmitted) {
  const body = buildBody({ ...context, messages, earlierOmitted });
  return buildIssueUrl(title, body, labels);
}

function fitIssueUrl(context, buildBody) {
  const title = createIssueTitle(context.messages, context.focusMessage);
  const labels = ISSUE_LABELS;
  const messages = Array.isArray(context.messages) ? context.messages : [];

  // Fast path: build with the full transcript and return when it already fits.
  let body = buildBody({ ...context, messages, earlierOmitted: 0 });
  let url = buildIssueUrl(title, body, labels);
  if (url.length <= URL_BUDGET) return url;

  // Step 1: keep the last two messages as the minimum useful reproduction,
  // then backfill older turns while URL budget remains.
  let includedMessages = messages.slice(-Math.min(2, messages.length));
  let earlierOmitted = messages.length - includedMessages.length;
  url = buildIssueUrlForMessages(
    context,
    buildBody,
    title,
    labels,
    includedMessages,
    earlierOmitted,
  );

  // If the final exchange itself is too large, shrink it first so the link
  // stays usable before trying to preserve any earlier context.
  if (url.length > URL_BUDGET) {
    for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
      const truncatedMessages = includedMessages.map((message) => ({
        ...message,
        content: truncateMessageContent(message.content, perMessageBudget),
      }));
      url = buildIssueUrlForMessages(
        context,
        buildBody,
        title,
        labels,
        truncatedMessages,
        earlierOmitted,
      );
      if (url.length <= URL_BUDGET) return url;
    }
    return url;
  }

  let bestUrl = url;

  while (earlierOmitted > 0) {
    const boundaryIndex = earlierOmitted - 1;
    const candidateMessages = [messages[boundaryIndex], ...includedMessages];
    const candidateOmitted = boundaryIndex;
    url = buildIssueUrlForMessages(
      context,
      buildBody,
      title,
      labels,
      candidateMessages,
      candidateOmitted,
    );
    if (url.length <= URL_BUDGET) {
      includedMessages = candidateMessages;
      earlierOmitted = candidateOmitted;
      bestUrl = url;
      continue;
    }

    // The next earlier turn does not fit in full. Keep a truncated version
    // instead of dropping all context before the last two messages.
    for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
      const truncatedBoundary = {
        ...messages[boundaryIndex],
        content: truncateMessageContent(messages[boundaryIndex].content, perMessageBudget),
      };
      url = buildIssueUrlForMessages(
        context,
        buildBody,
        title,
        labels,
        [truncatedBoundary, ...includedMessages],
        candidateOmitted,
      );
      if (url.length <= URL_BUDGET) return url;
    }

    return bestUrl;
  }

  // Final defensive pass: if the transcript had fewer than two messages and
  // still overflowed, shrink whatever was available.
  for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
    const truncatedMessages = includedMessages.map((message) => ({
      ...message,
      content: truncateMessageContent(message.content, perMessageBudget),
    }));
    url = buildIssueUrlForMessages(
      context,
      buildBody,
      title,
      labels,
      truncatedMessages,
      earlierOmitted,
    );
    if (url.length <= URL_BUDGET) return url;
  }

  return bestUrl;
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

function lastUnknownAssistantMessage(messages) {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    if (messages[i].role === "assistant" && messages[i].intent === "unknown") {
      return messages[i];
    }
  }
  return null;
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

// Issue #386: the worker kind (`wasm worker`) used to occupy its own
// Environment line. Folding it into the version (`0.174.0 (wasm)`) keeps the
// header compact. The trailing " worker" word is redundant inside the parens.
function formatVersionWithWorker(version, workerState) {
  const worker = String(workerState || "").trim();
  if (!worker) return version;
  const short = worker.replace(/\s*workers?$/i, "").trim() || worker;
  return `${version} (${short})`;
}

// Issue #839: every phrase the report document contains comes from
// `data/seed/agent-info.lino`, the same file the CLI reads, so the two surfaces
// cannot say different things. The seed reaches the browser through
// `seed.agentInfo`; the values below are its shipped English text, used while
// the seed is still loading.
const REPORT_LABEL_DEFAULTS = {
  issue_report_dialog_legend: "Legend: `U` = user, `A` = agent, `T` = tool result.",
  issue_report_no_messages: "No messages have been sent yet.",
  issue_report_omitted_messages: "... omitted {count} earlier messages ...",
  issue_report_omitted_message: "... omitted {count} earlier message ...",
  issue_report_trace_heading: "Focused assistant turn:",
  issue_report_description_placeholder:
    "<!-- Please describe what looked wrong or incomplete. -->",
  issue_report_memory_note:
    "Click **Export memory** to save `formal-ai-memory.lino`, redact it, and attach it (as a `.zip` if needed). See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md).",
};

function reportLabels(agentInfo) {
  const seeded = agentInfo && typeof agentInfo === "object" ? agentInfo : {};
  const label = (key) => String(seeded[key] || REPORT_LABEL_DEFAULTS[key]);
  return {
    legend: label("issue_report_dialog_legend"),
    no_messages: label("issue_report_no_messages"),
    omitted_earlier: label("issue_report_omitted_messages"),
    omitted_earlier_one: label("issue_report_omitted_message"),
    trace_heading: label("issue_report_trace_heading"),
    description_placeholder: label("issue_report_description_placeholder"),
    memory_note: label("issue_report_memory_note"),
  };
}

// Issue #386: fold the worker into the version (`0.174.0 (wasm)`) and drop
// settings that sit at their default. Manual mode is the interactive default,
// so Mode/Status are only worth reporting while a demo is playing, and
// Diagnostics is only reported when it has been turned on.
function environmentFields({ workerState, demoMode, demoStatus, diagnosticsMode }) {
  const fields = [
    { label: "Version", value: formatVersionWithWorker(APP_VERSION, workerState) },
    { label: "URL", value: window.location.href },
  ];
  if (demoMode) {
    fields.push({ label: "Mode", value: "demo" });
    fields.push({ label: "Status", value: demoStatus });
  }
  if (diagnosticsMode) {
    fields.push({ label: "Diagnostics", value: "on" });
  }
  fields.push({ label: "Timestamp", value: new Date().toISOString() });
  return fields;
}

// Issue #839: this function now only gathers facts. `renderReportBody` — the
// browser mirror of `src/issue_report.rs` — turns them into the document, so
// the web report and the agentic report are the same six sections by
// construction rather than by review.
function createIssueReportBody(context) {
  const { messages, focusMessage, userContext, agentInfo, earlierOmitted = 0 } = context;
  const effectiveFocus = focusMessage ?? lastUnknownAssistantMessage(messages);
  return renderReportBody({
    labels: reportLabels(agentInfo),
    environment: environmentFields(context),
    user_context: userContextFields(userContext),
    turns: reportTurns(messages, effectiveFocus),
    earlier_omitted: earlierOmitted,
    // Issue #386: the reasoning trace is only meaningful next to the full
    // dialog. When earlier turns had to be dropped to fit GitHub's URL cap the
    // dialog is no longer complete, so the shared builder drops the trace too.
    reasoning_trace: reasoningTraceLines(effectiveFocus),
    attachments: [],
  });
}

export function createIssueUrl(context) {
  return fitIssueUrl(context, (effectiveContext) => createIssueReportBody(effectiveContext));
}

export function shouldOfferMessageReport(message) {
  return message?.role === "assistant" && answerHasDetectedFailure(message);
}
