// Unit coverage for the chat app's ES modules under `src/web/app/` (issue #895).
//
// These files ship to the browser inside `app.js`, but they are plain ES modules
// in the repository, so `node --test` can import and measure them directly
// instead of instrumenting the bundle.

import assert from "node:assert/strict";
import { test } from "node:test";

import { answerHasDetectedFailure } from "../../src/web/app/detected-failure.js";
import {
  COUNT_PLACEHOLDER,
  SECTIONS,
  SECTION_ENVIRONMENT,
  SECTION_REPRODUCTION,
  TITLE_MAX_LENGTH,
  issueTitle,
  renderCount,
  renderReportBody,
  truncateRecords,
  truncateWords,
} from "../../src/web/app/issue-report.js";

test("failure detection fires on solver intents that mean the turn failed", () => {
  for (const intent of ["unknown", "agent_cli_error", "tool_result_failed"]) {
    assert.equal(
      answerHasDetectedFailure({ intent }),
      true,
      `${intent} is a detected failure`,
    );
  }
  assert.equal(answerHasDetectedFailure({ intent: "UNKNOWN" }), true, "intents are case-insensitive");

  for (const intent of ["greeting", "calculation", "identity"]) {
    assert.equal(
      answerHasDetectedFailure({ intent }),
      false,
      `${intent} is a successful turn`,
    );
  }
});

test("failure detection never fires on prose alone", () => {
  // The invitation to file an issue must be driven by structure, not wording;
  // an answer that merely talks about an error is not a failed turn.
  assert.equal(
    answerHasDetectedFailure({
      intent: "explanation",
      content: "The command failed with error: exit code 1 and the request was denied.",
    }),
    false,
    "prose describing a failure is not itself a failure",
  );
});

test("failure detection reads structured tool outputs", () => {
  const failing = [
    { ok: false },
    { success: false },
    { exit_code: 1 },
    { exitCode: 2 },
    { status_code: 500 },
    { statusCode: 404 },
    { status: "error" },
    { error: "boom" },
  ];
  for (const outputs of failing) {
    assert.equal(
      answerHasDetectedFailure({ toolCalls: [{ outputs }] }),
      true,
      `${JSON.stringify(outputs)} is a failed tool call`,
    );
  }

  const succeeding = [{ ok: true }, { exit_code: 0 }, { status: "ok" }, { status: 200 }];
  for (const outputs of succeeding) {
    assert.equal(
      answerHasDetectedFailure({ toolCalls: [{ outputs }] }),
      false,
      `${JSON.stringify(outputs)} is a successful tool call`,
    );
  }
});

test("an expected stop is not a failure", () => {
  // A user who declines a permission prompt has not hit a bug; inviting them to
  // file an issue there would train them to ignore the invitation.
  for (const status of ["refused", "denied", "cancelled", "aborted", "pending", "awaiting_approval"]) {
    assert.equal(
      answerHasDetectedFailure({ toolCalls: [{ outputs: { status } }] }),
      false,
      `${status} is an expected stop, not a failure`,
    );
  }
});

test("failure detection parses JSON-encoded tool outputs", () => {
  assert.equal(
    answerHasDetectedFailure({ toolCalls: [{ outputs: '{"exit_code": 1}' }] }),
    true,
    "a JSON string is inspected as structure",
  );
  assert.equal(
    answerHasDetectedFailure({ toolCalls: [{ outputs: '[{"ok": true}, {"ok": false}]' }] }),
    true,
    "any failing entry in an array fails the turn",
  );
  assert.equal(
    answerHasDetectedFailure({ toolCalls: [{ outputs: "not json at all: error" }] }),
    false,
    "plain text output is not parsed for failure words",
  );
  assert.equal(
    answerHasDetectedFailure({ toolCalls: [{ outputs: "{broken json" }] }),
    false,
    "unparsable JSON is ignored rather than treated as a failure",
  );
});

test("failure detection tolerates missing and malformed answers", () => {
  for (const answer of [undefined, null, "string", 42, {}]) {
    assert.equal(answerHasDetectedFailure(answer), false, `${String(answer)} is not a failure`);
  }
  assert.equal(
    answerHasDetectedFailure({ intent: "greeting", detectedFailure: true }),
    true,
    "an explicit flag wins over the intent",
  );
});

test("the count placeholder is substituted everywhere it appears", () => {
  assert.equal(renderCount(`${COUNT_PLACEHOLDER} earlier turns omitted`, 3), "3 earlier turns omitted");
  assert.equal(
    renderCount(`${COUNT_PLACEHOLDER} of ${COUNT_PLACEHOLDER}`, 7),
    "7 of 7",
    "every occurrence is replaced, not just the first",
  );
  assert.equal(renderCount("no placeholder", 3), "no placeholder");
});

test("word truncation cuts on a boundary and keeps whole characters", () => {
  assert.equal(truncateWords("short", 20), "short");
  assert.equal(truncateWords("  padded  ", 20), "padded", "the value is trimmed first");
  assert.equal(
    truncateWords("one two three four five", 12),
    "one two…",
    "the cut lands on a word boundary in the second half of the budget",
  );

  // Issue #826: titles are quoted in any script, so the budget counts characters
  // rather than UTF-16 code units.
  const cyrillic = truncateWords("проверка длинного заголовка на русском языке", 20);
  assert.ok(Array.from(cyrillic).length <= 20, "the Cyrillic title fits the character budget");
  assert.match(cyrillic, /…$/, "the truncation is marked with an ellipsis");

  const emoji = truncateWords("🙂".repeat(30), 10);
  assert.ok(
    !emoji.includes("�"),
    "an astral-plane character is never split into a broken surrogate",
  );
});

test("issue titles combine the first and last subject within the length limit", () => {
  const turns = [
    { role: "user", content: "search hive-mind on desktop" },
    { role: "assistant", content: "no answer", intent: "unknown" },
    { role: "user", content: "try the second query" },
  ];
  const title = issueTitle(turns, { prefix: "web: ", default_title: "web: report" });

  assert.ok(title.startsWith("web: "), "the configured prefix is applied");
  assert.ok(
    Array.from(title).length <= TITLE_MAX_LENGTH,
    "the title never exceeds the GitHub-friendly limit",
  );
  assert.match(title, /`/, "the quoted subject is fenced so punctuation cannot break the title");

  assert.equal(
    issueTitle([], { prefix: "web: ", default_title: "web: report" }),
    "web: report",
    "an empty dialog falls back to the configured default title",
  );
});

test("a long single subject is truncated to fit the title limit", () => {
  const long = Array.from({ length: 60 }, (_, index) => `word${index}`).join(" ");
  const title = issueTitle([{ role: "user", content: long }], {
    prefix: "web: ",
    default_title: "web: report",
  });
  assert.ok(
    Array.from(title).length <= TITLE_MAX_LENGTH,
    `the title is ${Array.from(title).length} characters, within the limit`,
  );
  assert.match(title, /…/, "the truncation is visible to the reader");
});

test("the rendered report contains every declared section in order", () => {
  const body = {
    labels: {},
    environment: [
      { label: "Version", value: "0.328.0 (wasm)" },
      { label: "URL", value: "https://formal-ai.dev/" },
    ],
    user_context: [{ label: "UI languages", value: "en (browser: en-US)" }],
    turns: [
      { role: "user", content: "search hive-mind" },
      { role: "assistant", content: "no answer", intent: "unknown" },
    ],
    earlier_omitted: 0,
    reasoning_trace: ["intent: unknown", "evidence:", "- surface:web"],
    attachments: [],
  };

  const document = renderReportBody(body);
  assert.ok(document.startsWith(SECTION_ENVIRONMENT), "the environment comes first");
  assert.ok(document.includes(SECTION_REPRODUCTION), "the dialog is reproduced");
  assert.match(document, /0\.328\.0 \(wasm\)/, "environment fields are rendered as a list");
  assert.match(document, /search hive-mind/, "the user's turn is quoted");

  let cursor = -1;
  for (const section of SECTIONS) {
    const position = document.indexOf(section);
    if (position === -1) continue;
    assert.ok(position > cursor, `${section} appears after the previous section`);
    cursor = position;
  }
});

test("a fence in the dialog is escaped by a longer fence", () => {
  const document = renderReportBody({
    labels: {},
    environment: [{ label: "Version", value: "0.328.0" }],
    turns: [{ role: "user", content: "```\ncode\n```" }],
  });
  assert.match(
    document,
    /````/,
    "a turn that already contains a fence is wrapped in a longer one so the block cannot break out",
  );
});

test("the reporter tolerates a missing or empty body", () => {
  for (const body of [undefined, null, {}, "not an object"]) {
    const document = renderReportBody(body);
    assert.equal(typeof document, "string", `${String(body)} still renders a document`);
    assert.ok(document.includes(SECTION_ENVIRONMENT), "the skeleton is always present");
  }
});

test("record truncation drops whole records and reports how many", () => {
  const record = (index) => `  record\n    id "${index}"\n    note "some text"`;
  const document = ["root", ...Array.from({ length: 20 }, (_, index) => record(index))].join("\n");

  const untouched = truncateRecords(document, 1_000_000, "… {count} omitted …");
  assert.equal(untouched.omitted, 0, "a document within budget is returned unchanged");
  assert.equal(untouched.text, document);

  const trimmed = truncateRecords(document, 400, "… {count} omitted …");
  assert.ok(trimmed.omitted > 0, "records were dropped");
  assert.ok(
    new TextEncoder().encode(trimmed.text).length <= 400,
    "the result honours the byte budget",
  );
  assert.ok(
    trimmed.text.includes('id "0"'),
    "the opening records survive so the reader sees what the session was about",
  );
  assert.ok(
    trimmed.text.includes('id "19"'),
    "the closing records survive so the reader sees where it went wrong",
  );
  // Issue #838: the point of record-aware truncation is that no record is ever
  // cut open, so every `record` header that survives keeps both of its fields.
  const surviving = trimmed.text.split("\n  record\n").slice(1);
  for (const block of surviving) {
    assert.match(block, /^ {4}id "\d+"\n {4}note "some text"/, "each kept record is intact");
  }
});
