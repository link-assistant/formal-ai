import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const [expectedPath, evidenceDir] = process.argv.slice(2);
assert.ok(
  expectedPath && evidenceDir,
  "usage: verify_issue_707_generalization.mjs EXPECTED_JSONL EVIDENCE_DIR",
);

/** Enum variant names as the plan serializes them, to the advertised tool names. */
const PRIMITIVES = {
  FsRead: "fs.read",
  FsWrite: "fs.write",
  FsList: "fs.list",
  FsMove: "fs.move",
  ShellRun: "shell.run",
  HttpFetch: "http.fetch",
  HttpPost: "http.post",
  DomQuery: "dom.query",
  DomExtract: "dom.extract",
  ArchivePack: "archive.pack",
  ArchiveUnpack: "archive.unpack",
  ProcessStatus: "process.status",
};

function readEvents(file) {
  return fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter(Boolean)
    .flatMap((line) => {
      try {
        return [JSON.parse(line)];
      } catch {
        return [];
      }
    });
}

const cases = readEvents(expectedPath).map((entry) => ({
  ...entry,
  primitives: entry.steps.map((variant) => {
    const dotted = PRIMITIVES[variant];
    assert.ok(dotted, `unknown primitive variant ${variant}`);
    return dotted;
  }),
}));
assert.ok(cases.length >= 12, "expected at least twelve held-out cases");
for (const entry of cases) {
  assert.ok(
    entry.plan_id.startsWith("synthesized-"),
    `${entry.case_id} must be synthesized, not recalled`,
  );
}

function caseEvidence(phase, entry) {
  const events = readEvents(path.join(evidenceDir, phase, `${entry.case_id}.jsonl`));
  const seenToolUses = new Set();
  const tools = [];
  for (const event of events) {
    if (
      event.type !== "tool_use" ||
      !String(event.name || "").startsWith("formal_ai_") ||
      !event.input ||
      Object.keys(event.input).length === 0 ||
      seenToolUses.has(event.tool_use_id)
    ) {
      continue;
    }
    seenToolUses.add(event.tool_use_id);
    tools.push(event.name);
  }
  assert.deepEqual(
    tools,
    entry.primitives.map((name) => `formal_ai_${name.replaceAll(".", "_")}`),
    `${phase}/${entry.case_id} primitive sequence`,
  );

  const clientErrors = events.filter(
    (event) => event.type === "result" && event.status === "error",
  );
  assert.deepEqual(clientErrors, [], `${phase}/${entry.case_id} client tool errors`);

  const finalText = events
    .filter((event) => event.type === "message" && event.role === "assistant")
    .flatMap((event) => event.content || [])
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n");
  assert.ok(
    finalText.includes(`computer_use_complete:`) && finalText.includes(entry.plan_id),
    `${phase}/${entry.case_id} must report the synthesized plan ${entry.plan_id}`,
  );

  const idle = events.find(
    (event) => event.type === "idle" && /^ses_/.test(event.session_id || ""),
  );
  assert.ok(idle, `${phase}/${entry.case_id} session id`);
  return { session_id: idle.session_id, tools };
}

function auditEvidence(phase) {
  const records = readEvents(path.join(evidenceDir, phase, "audit.jsonl"));
  const expectedCount = cases.reduce((sum, entry) => sum + entry.primitives.length, 0);
  assert.equal(records.length, expectedCount, `${phase} audit record count`);
  const byPlan = {};
  for (const entry of cases) {
    const planRecords = records.filter((record) => record.plan_id === entry.plan_id);
    assert.deepEqual(
      planRecords.map((record) => record.primitive),
      entry.primitives,
      `${phase}/${entry.case_id} audited primitive sequence`,
    );
    for (const record of planRecords) {
      assert.equal(record.verified, true, `${phase}/${record.step_id} verified`);
      assert.deepEqual(
        record.events.map((event) => event.phase),
        ["precondition", "effect", "postcondition"],
        `${phase}/${record.step_id} event phases`,
      );
      assert.ok(
        record.events.every((event) => event.passed === true),
        `${phase}/${record.step_id} event verification`,
      );
    }
    byPlan[entry.case_id] = {
      plan_id: entry.plan_id,
      steps: planRecords.length,
      verification_events: planRecords.reduce((sum, record) => sum + record.events.length, 0),
    };
  }
  return byPlan;
}

const manifest = {
  issue: 707,
  slice: "held_out_generalization",
  client: "@link-assistant/agent",
  case_count: cases.length,
  record: {},
  replay: {},
};
for (const entry of cases) {
  manifest.record[entry.case_id] = caseEvidence("record", entry);
  manifest.replay[entry.case_id] = caseEvidence("replay", entry);
  assert.deepEqual(
    manifest.record[entry.case_id].tools,
    manifest.replay[entry.case_id].tools,
    `${entry.case_id} record/replay sequence`,
  );
  assert.notEqual(
    manifest.record[entry.case_id].session_id,
    manifest.replay[entry.case_id].session_id,
    `${entry.case_id} replay must be a fresh Agent CLI session`,
  );
}
manifest.record_audit = auditEvidence("record");
manifest.replay_audit = auditEvidence("replay");
fs.writeFileSync(
  path.join(evidenceDir, "manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
