// Browser mirror of `src/issue_report.rs` (issue #839).
//
// The wasm worker is a standalone `rustc` build that does not link the Rust
// core, so the browser cannot call the canonical builder directly. This module
// is therefore a faithful port, kept honest by
// `tests/integration/issue_839_report_parity.rs`, which renders the same
// fixture through both implementations and asserts the bytes are identical.
//
// Like the Rust module this file owns *format* only: every fact and every
// user-visible phrase is an input, so the i18n catalogue stays with the caller.

export const SECTION_ENVIRONMENT = "## Environment";
export const SECTION_USER_CONTEXT = "## User Context";
export const SECTION_REPRODUCTION = "## Reproduction of dialog";
export const SECTION_REASONING_TRACE = "## Reasoning Trace";
export const SECTION_DESCRIPTION = "## Description";
export const SECTION_ATTACH_MEMORY = "## Attach full memory (optional)";

export const SECTIONS = [
  SECTION_ENVIRONMENT,
  SECTION_USER_CONTEXT,
  SECTION_REPRODUCTION,
  SECTION_REASONING_TRACE,
  SECTION_DESCRIPTION,
  SECTION_ATTACH_MEMORY,
];

export const COUNT_PLACEHOLDER = "{count}";
export const TITLE_MAX_LENGTH = 120;

const text = (value) => String(value ?? "");

function turnPrefix(role) {
  const normalized = text(role).toLowerCase();
  if (normalized === "user") return "U";
  if (normalized === "tool") return "T";
  return "A";
}

export function renderCount(label, count) {
  return text(label).split(COUNT_PLACEHOLDER).join(String(count));
}

function pickFence(samples) {
  let fence = "```";
  while (samples.some((sample) => text(sample).includes(fence))) {
    fence += "`";
  }
  return fence;
}

function pushFields(lines, fields) {
  for (const field of fields) {
    const value = text(field && field.value);
    if (!value) continue;
    lines.push(`- **${text(field && field.label)}**: ${value}`);
  }
}

function pushCodeBlock(lines, language, content) {
  const body = text(content);
  const fence = pickFence([body]);
  lines.push(`${fence}${text(language)}`);
  lines.push(body);
  lines.push(fence);
}

function pushDialog(lines, body) {
  const labels = body.labels || {};
  const turns = Array.isArray(body.turns) ? body.turns : [];
  if (turns.length === 0) {
    lines.push(text(labels.no_messages));
    return;
  }

  lines.push(text(labels.legend));
  lines.push("");
  const fence = pickFence(turns.map((turn) => text(turn.content)));
  lines.push(fence);
  const earlierOmitted = Math.max(0, Number(body.earlier_omitted) || 0);
  if (earlierOmitted > 0) {
    const singular = text(labels.omitted_earlier_one);
    const label = earlierOmitted === 1 && singular ? singular : text(labels.omitted_earlier);
    lines.push(renderCount(label, earlierOmitted));
  }
  for (const turn of turns) {
    const annotations = [];
    const intent = text(turn.intent);
    if (intent === "unknown") {
      annotations.push(`intent: ${intent}`);
    }
    if (turn.reported) {
      if (intent && intent !== "unknown") {
        annotations.push(`intent: ${intent}`);
      }
      annotations.push("reported");
    }
    const head =
      annotations.length > 0 ? `${turnPrefix(turn.role)} (${annotations.join(", ")})` : turnPrefix(turn.role);
    const [first, ...rest] = text(turn.content).split("\n");
    lines.push(`${head}: ${first}`);
    for (const row of rest) lines.push(`   ${row}`);
  }
  lines.push(fence);
}

// Renders the Markdown document. `body` mirrors `ReportBody` field for field,
// in snake_case, so the same JSON fixture feeds both implementations.
export function renderReportBody(body) {
  const safe = body && typeof body === "object" ? body : {};
  const labels = safe.labels || {};
  const environment = Array.isArray(safe.environment) ? safe.environment : [];
  const userContext = Array.isArray(safe.user_context) ? safe.user_context : [];
  const reasoningTrace = Array.isArray(safe.reasoning_trace) ? safe.reasoning_trace : [];
  const attachments = Array.isArray(safe.attachments) ? safe.attachments : [];
  const earlierOmitted = Math.max(0, Number(safe.earlier_omitted) || 0);
  const lines = [];

  lines.push(SECTION_ENVIRONMENT);
  lines.push("");
  pushFields(lines, environment);
  lines.push("");

  if (userContext.some((field) => text(field && field.value))) {
    lines.push(SECTION_USER_CONTEXT);
    lines.push("");
    pushFields(lines, userContext);
    lines.push("");
  }

  lines.push(SECTION_REPRODUCTION);
  lines.push("");
  pushDialog(lines, { ...safe, earlier_omitted: earlierOmitted });

  // Issue #386: a trace is only meaningful beside the complete dialog, so it is
  // dropped as soon as earlier turns had to be omitted.
  if (earlierOmitted === 0 && reasoningTrace.length > 0) {
    lines.push("");
    lines.push(SECTION_REASONING_TRACE);
    lines.push("");
    lines.push(text(labels.trace_heading));
    lines.push("");
    pushCodeBlock(lines, "", reasoningTrace.join("\n"));
    lines.push("");
  }

  lines.push("");
  lines.push(SECTION_DESCRIPTION);
  lines.push("");
  lines.push(text(labels.description_placeholder));
  lines.push("");
  lines.push(SECTION_ATTACH_MEMORY);
  lines.push("");
  lines.push(text(labels.memory_note));
  lines.push("");

  for (const attachment of attachments) {
    lines.push(text(attachment.heading));
    lines.push("");
    const note = text(attachment.note);
    if (note) {
      lines.push(note);
      lines.push("");
    }
    pushCodeBlock(lines, attachment.language, attachment.content);
    lines.push("");
  }

  return lines.join("\n");
}

function normalizeSingleLine(value) {
  return text(value).split(/\s+/u).filter(Boolean).join(" ");
}

// Rule 1 of §4: the turn that asked for the report is never the subject. Only a
// trailing run is dropped — an earlier report-shaped turn that the agent
// answered (issue #826's `Зарепорти баг`) is part of the reported story. Rule 4
// wins over rule 1 when nothing else remains.
function titleSubjects(turns) {
  const subjects = (Array.isArray(turns) ? turns : [])
    .filter((turn) => text(turn && turn.role).toLowerCase() === "user")
    .map((turn) => ({
      text: normalizeSingleLine(turn.content),
      invoking: Boolean(turn.report_invoking),
    }))
    .filter((entry) => entry.text);
  while (subjects.length > 1 && subjects[subjects.length - 1].invoking) {
    subjects.pop();
  }
  const deduped = [];
  for (const entry of subjects) {
    if (deduped.length === 0 || deduped[deduped.length - 1] !== entry.text) {
      deduped.push(entry.text);
    }
  }
  return deduped;
}

export function truncateWords(value, max) {
  const trimmed = text(value).trim();
  const characters = Array.from(trimmed);
  if (characters.length <= max) return trimmed;
  // Character indices, not UTF-16 offsets: the convention quotes user turns in
  // any script, and issue #826's title is Cyrillic.
  const head = characters.slice(0, Math.max(0, max - 1));
  let boundary = -1;
  for (let index = head.length - 1; index >= 0; index -= 1) {
    if (/\s/u.test(head[index])) {
      boundary = index;
      break;
    }
  }
  const cut =
    boundary >= 0 && boundary >= Math.floor(max / 2)
      ? head.slice(0, boundary).join("")
      : head.join("");
  return `${cut.replace(/\s+$/u, "")}…`;
}

export function issueTitle(turns, settings) {
  const safe = settings && typeof settings === "object" ? settings : {};
  const prefix = text(safe.prefix);
  const subjects = titleSubjects(turns);
  const first = subjects[0];
  if (!first) return text(safe.default_title);

  const last = subjects[subjects.length - 1];
  if (last && last !== first) {
    const combined = `${prefix}\`${first}\` + \`${last}\``;
    if (Array.from(combined).length <= TITLE_MAX_LENGTH) return combined;
  }

  const budget = Math.max(0, TITLE_MAX_LENGTH - (Array.from(prefix).length + 2));
  return `${prefix}\`${truncateWords(first, budget)}\``;
}

const byteLength = (value) => new TextEncoder().encode(text(value)).length;

function indentOf(line) {
  return line.length - line.replace(/^ +/u, "").length;
}

function joinedLength(lines) {
  return lines.reduce((total, line) => total + byteLength(line) + 1, 0);
}

function truncateLines(lines, maxBytes, omittedLabel) {
  const kept = [];
  let used = byteLength(omittedLabel) + 1;
  for (const line of lines) {
    if (used + byteLength(line) + 1 > maxBytes) break;
    used += byteLength(line) + 1;
    kept.push(line);
  }
  const omitted = lines.length - kept.length;
  let result = kept.join("\n");
  if (omitted > 0) {
    result += `\n${renderCount(omittedLabel, omitted)}`;
  }
  return { text: `${result}\n`, omitted };
}

// Shrinks a Links Notation document without ever cutting inside a record; see
// the Rust doc comment on `truncate_records` for the #838 background.
export function truncateRecords(value, maxBytes, omittedLabel) {
  const document = text(value);
  if (byteLength(document) <= maxBytes) return { text: document, omitted: 0 };

  const lines = document.split("\n");
  const indents = lines
    .filter((line) => line.trim())
    .map(indentOf)
    .filter((indent) => indent > 0);
  if (indents.length === 0) return truncateLines(lines, maxBytes, omittedLabel);
  const baseIndent = Math.min(...indents);

  const header = [];
  const records = [];
  for (const line of lines) {
    if (line.trim() && indentOf(line) === baseIndent) {
      records.push([line]);
    } else if (records.length > 0) {
      records[records.length - 1].push(line);
    } else {
      header.push(line);
    }
  }

  const marker = `${" ".repeat(baseIndent)}${text(omittedLabel)}`;
  const budget = Math.max(0, maxBytes - (joinedLength(header) + byteLength(marker) + 1));
  const sizes = records.map(joinedLength);
  let headCount = 0;
  let tailCount = 0;
  let used = 0;
  // Half the budget goes to the opening records (what the session was about)
  // and the rest to the closing ones (where it went wrong).
  while (headCount < records.length && used + sizes[headCount] <= Math.floor(budget / 2)) {
    used += sizes[headCount];
    headCount += 1;
  }
  while (
    headCount + tailCount < records.length &&
    used + sizes[records.length - 1 - tailCount] <= budget
  ) {
    used += sizes[records.length - 1 - tailCount];
    tailCount += 1;
  }

  const omitted = records.length - headCount - tailCount;
  if (omitted === 0) return { text: document, omitted: 0 };

  const kept = [...header];
  for (const record of records.slice(0, headCount)) kept.push(...record);
  kept.push(`${" ".repeat(baseIndent)}${renderCount(omittedLabel, omitted)}`);
  for (const record of records.slice(records.length - tailCount)) kept.push(...record);
  return { text: `${kept.join("\n")}\n`, omitted };
}
