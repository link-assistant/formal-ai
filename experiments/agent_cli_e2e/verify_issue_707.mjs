import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const [seedPath, evidenceDir] = process.argv.slice(2);
assert.ok(seedPath && evidenceDir, "usage: verify_issue_707.mjs SEED EVIDENCE_DIR");

const seed = fs.readFileSync(seedPath, "utf8");
const tasks = [];
let current = null;
for (const line of seed.split(/\r?\n/)) {
  const task = /^  task (.+)$/.exec(line);
  if (task) {
    current = { id: task[1], steps: [] };
    tasks.push(current);
    continue;
  }
  const step = /^    step ([\w.]+) /.exec(line);
  if (step && current) current.steps.push(step[1]);
}
assert.equal(tasks.length, 10, "benchmark must contain exactly ten tasks");

function readEvents(file) {
  return fs.readFileSync(file, "utf8")
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

function taskEvidence(phase, task) {
  const file = path.join(evidenceDir, phase, `${task.id}.jsonl`);
  const events = readEvents(file);
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
  const expected = task.steps.map((name) => `formal_ai_${name.replaceAll(".", "_")}`);
  assert.deepEqual(tools, expected, `${phase}/${task.id} primitive sequence`);
  const clientErrors = events.filter(
    (event) => event.type === "result" && event.status === "error",
  );
  assert.deepEqual(clientErrors, [], `${phase}/${task.id} client tool errors`);

  const finalText = events
    .filter((event) => event.type === "message" && event.role === "assistant")
    .flatMap((event) => event.content || [])
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n");
  assert.match(finalText, new RegExp(`computer_use_complete:.*${task.id}`), `${phase}/${task.id} final`);

  const idle = events.find((event) => event.type === "idle" && /^ses_/.test(event.session_id || ""));
  assert.ok(idle, `${phase}/${task.id} session id`);
  return { session_id: idle.session_id, tools };
}

function auditEvidence(phase) {
  const file = path.join(evidenceDir, phase, "audit.jsonl");
  const records = readEvents(file);
  const expectedCount = tasks.reduce((sum, task) => sum + task.steps.length, 0);
  assert.equal(records.length, expectedCount, `${phase} audit record count`);
  const byPlan = {};
  for (const task of tasks) {
    const planRecords = records.filter((record) => record.plan_id === task.id);
    assert.deepEqual(
      planRecords.map((record) => record.primitive),
      task.steps,
      `${phase}/${task.id} audited primitive sequence`,
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
    byPlan[task.id] = {
      steps: planRecords.length,
      verified_records: planRecords.filter((record) => record.verified).length,
      verification_events: planRecords.reduce((sum, record) => sum + record.events.length, 0),
    };
  }
  return byPlan;
}

const manifest = {
  issue: 707,
  client: "@link-assistant/agent",
  task_count: tasks.length,
  record: {},
  replay: {},
};
for (const task of tasks) {
  manifest.record[task.id] = taskEvidence("record", task);
  manifest.replay[task.id] = taskEvidence("replay", task);
  assert.deepEqual(
    manifest.record[task.id].tools,
    manifest.replay[task.id].tools,
    `${task.id} record/replay sequence`,
  );
  assert.notEqual(
    manifest.record[task.id].session_id,
    manifest.replay[task.id].session_id,
    `${task.id} replay must be a fresh Agent CLI session`,
  );
}
manifest.record_audit = auditEvidence("record");
manifest.replay_audit = auditEvidence("replay");
fs.writeFileSync(
  path.join(evidenceDir, "manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
