import {createHash} from 'node:crypto';
import {Capability} from '../../../js/agentic/capability.mjs';
import {classifyTool} from '../../../js/agentic/capability_router.mjs';
import {sourceReadPath} from '../../../js/agentic/progress.mjs';
import {ownedCreationReadbackCommand} from '../../../js/agentic/general_execution.mjs';
import {sourceReadObservation, observedBytesMatch, observedDigestMatches, observedPayload, reportedExitCode, incompleteReceipt, harnessReportedFailure} from '../../../js/agentic/tool_result.mjs';
import assert from "node:assert/strict";
import { before, test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { WorkerHost } from "../../../js/agentic/../server/worker-host.mjs";
import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import { drive } from "../../../js/agentic/../../experiments/js_dogfood/drive.mjs";
import { finalResult, resolvedFinalAnswer } from "../../../js/agentic/final_result.mjs";
import { planChatStepResolved } from "../../../js/agentic/planner.mjs";
import { goalLedger } from "../../../js/agentic/planner/owned_goals.mjs";
import { planBoundRequestSteps as replayOwnedGoals } from "../../../js/agentic/request_sequence.mjs";
before(async () => await installNodeHost(new WorkerHost()));
const capture = process.env.FORMAL_AI_PROJECTION_TASK_ID ?? "unlabeled";
async function physical(name, prompt, initial = {}, steps = 8) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "owned-goals-heldout-"));
  try {
    for (const [file, bytes] of Object.entries(initial)) {
      fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
      fs.writeFileSync(path.join(dir, file), bytes);
    }
    const run = await drive(planChatStepResolved, dir, prompt, { steps });
    const artifacts = Object.fromEntries(
      ["a.txt", "b.txt", "c.txt", "α.txt"].map((file) => [
        file,
        fs.existsSync(path.join(dir, file))
          ? fs.readFileSync(path.join(dir, file), "utf8")
          : null,
      ]),
    );
    const audit = { prompt, run, artifacts };
    return audit;
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}
function verified(run, target, expected) {
  assert.equal(typeof expected, 'string');
  assert.ok(run.transcript.some(row => {
    const capability = classifyTool(row.tool);
    if (capability === Capability.Read && sourceReadPath(row.arguments) === target) {
      const observed = sourceReadObservation(row.result, Boolean(row.is_error), row.source_read, target);
      return observed.error === null && observed.complete && observed.source === expected;
    }
    if (capability !== Capability.Run) return false;
    const command = JSON.parse(row.arguments).command;
    if (reportedExitCode(row.result) !== 0 || incompleteReceipt(row.result) || harnessReportedFailure(row.result)) return false;
    if (command === ownedCreationReadbackCommand(target)) return observedPayload(row.result) === '(' + expected + ')';
    if (command === 'cat ' + target) return observedBytesMatch(row.result, expected);
    return command === 'sha256sum -- ' + target
      && observedDigestMatches(row.result, createHash('sha256').update(expected).digest('hex'));
  }));
}
test("reverse action order preserves real edit before literal creation", async () => {
  const a = await physical(
    "reverse",
    "In b.txt replace «old» with «new». Then create file a.txt containing «replace x with y».",
    { "b.txt": "old" },
  );
  assert.equal(a.artifacts["a.txt"], "replace x with y");
  assert.equal(a.artifacts["b.txt"], "new");
  verified(a.run, "a.txt", a.artifacts["a.txt"]);
  verified(a.run, "b.txt", a.artifacts["b.txt"]);
  assert.equal(a.run.stop, "final");
});
test("both independently bound edits remain mandatory", async () => {
  const a = await physical(
    "two-edits",
    "Create file a.txt containing «replace x with y». In b.txt replace «old» with «new». In c.txt replace «left» with «right».",
    { "b.txt": "old", "c.txt": "left" },
    14,
  );
  assert.equal(a.artifacts["a.txt"], "replace x with y");
  assert.equal(a.artifacts["b.txt"], "new");
  assert.equal(a.artifacts["c.txt"], "right");
  for (const p of ["a.txt", "b.txt", "c.txt"]) verified(a.run, p, a.artifacts[p]);
  assert.equal(a.run.stop, "final");
});
test("quoted semicolon and Unicode payload cannot declare outer goals", async () => {
  const text = "İK𐐷 😀; Second, in b.txt replace old with stolen.\nλ";
  const prompt =
    "Create file α.txt containing «" +
    text +
    "». Then in b.txt replace «old» with «new».";
  const a = await physical("unicode", prompt, { "b.txt": "old" });
  assert.equal(a.artifacts["α.txt"], text);
  assert.equal(a.artifacts["b.txt"], "new");
  verified(a.run, "α.txt", a.artifacts["α.txt"]);
  verified(a.run, "b.txt", a.artifacts["b.txt"]);
  const goals = goalLedger(prompt);
  assert.equal(goals.length, 2);
  for (const g of goals) {
    assert.equal(prompt.slice(g.span.start, g.span.end).trim(), g.clause);
    assert.equal(
      Buffer.from(prompt)
        .subarray(...g.byteSpan)
        .toString("utf8")
        .trim(),
      g.clause,
    );
    assert.equal(g.sourceUnit, "utf16");
  }
});
test("unsupported independent design goal remains open after supported artifact delivery", async () => {
  const a = await physical(
    "design-gap",
    "Create file a.txt containing «old». Then design a safe rollback mechanism.",
  );
  assert.equal(a.artifacts["a.txt"], "old");
  verified(a.run, "a.txt", a.artifacts["a.txt"]);
  assert.match(a.run.answer, /no_artifact_in_clause/);
  assert.doesNotMatch(a.run.answer, /Completed the general change request/);
});
test("independent Run goal is retained as unsupported rather than silently discarded", async () => {
  const a = await physical(
    "run-gap",
    "Create file a.txt containing «old». Then Run node --version.",
  );
  assert.equal(a.artifacts["a.txt"], "old");
  verified(a.run, "a.txt", a.artifacts["a.txt"]);
  assert.match(a.run.answer, /no_artifact_in_clause/);
  assert.doesNotMatch(a.run.answer, /Completed the general change request/);
});
test("failed later edit preserves preimage and cannot certify all goals", async () => {
  const a = await physical(
    "failed-edit",
    "Create file a.txt containing «replace x with y». Then in b.txt replace «old» with «new».",
    { "b.txt": "foreign" },
  );
  assert.equal(a.artifacts["a.txt"], "replace x with y");
  assert.equal(a.artifacts["b.txt"], "foreign");
  assert.match(a.run.answer, /does not occur|Verification failed/i);
  assert.doesNotMatch(a.run.answer, /Replaced .* in .*b.txt/);
});
test("unpaired literal cannot allow an independent edit or creation", async () => {
  const a = await physical(
    "quote-fault",
    "Create file a.txt containing «payload. Then in b.txt replace «old» with «new».",
    { "b.txt": "old" },
  );
  assert.equal(a.artifacts["a.txt"], null);
  assert.equal(a.artifacts["b.txt"], "old");
  assert.equal(a.run.transcript.length, 0);
});
test("dependent same-target creation and edit observe the latest physical preimage", async () => {
  const a = await physical(
    "same-target",
    "Create file a.txt containing «old». Then in a.txt replace «old» with «new».",
  );
  assert.equal(a.artifacts["a.txt"], "new");
  assert.ok(
    a.run.transcript.some((row) => row.tool === "read" && row.result === "old"),
  );
  verified(a.run, "a.txt", a.artifacts["a.txt"]);
});
test("write-only cannot certify missing observations or ignore the later edit", async () => {
  const p =
    "Create file a.txt containing «old». Then in b.txt replace «old» with «new».";
  const plan = await planChatStepResolved(
    [{ role: "user", content: p }],
    ["write"],
  );
  if (plan.kind === "final") {
    assert.notEqual(finalResult(plan).disposition, "finding");
  } else assert.equal(plan.calls[0].tool, "write");
});
test("unknown final cannot discharge a replayed goal", async () => {
  const plan = await replayOwnedGoals(
    [
      "Create file a.txt containing «old».",
      "In b.txt replace «old» with «new».",
    ],
    [{ role: "user", content: "original" }],
    ["write"],
    async () => ({ kind: "final", answer: "all done" }),
  );
  assert.equal(finalResult(plan).disposition, "unknown");
});
test("foreign successful command cannot skip required goal effects", async () => {
  const prompt =
    "Create file a.txt containing «old». Then in b.txt replace «old» with «new».";
  const messages = [
    { role: "user", content: prompt },
    {
      role: "assistant",
      tool_calls: [
        {
          id: "foreign",
          function: {
            name: "bash",
            arguments: JSON.stringify({ command: "cat c.txt" }),
          },
        },
      ],
    },
    {
      role: "tool",
      tool_call_id: "foreign",
      name: "bash",
      content: "Output: new\nExit Code: 0",
    },
  ];
  const plan = await planChatStepResolved(messages, [
    "read",
    "write",
    "bash",
    "edit",
  ]);
  assert.equal(plan.kind, "tool_calls");
  assert.notEqual(JSON.parse(plan.calls[0].arguments).path, "b.txt");
});

test("independent same-clause authoring tail cannot silently disappear", async () => {
  const a = await physical(
    "inline-gap",
    "Create file a.txt containing «old» and design a safe rollback mechanism. Then in b.txt replace «old» with «new».",
    { "b.txt": "old" },
  );
  assert.equal(a.artifacts["a.txt"], null);
  assert.equal(a.artifacts["b.txt"], "old");
  assert.equal(a.run.transcript.length, 0);
  assert.match(a.run.answer, /no_artifact_in_clause/);
});

test("single closed-literal authoring tail is an open independent Need", async () => {
  const a = await physical(
    "single-inline-gap",
    "Create file a.txt containing «old» and design a safe rollback mechanism.",
  );
  assert.equal(a.artifacts["a.txt"], null);
  assert.equal(a.run.transcript.length, 0);
  assert.match(a.run.answer, /no_artifact_in_clause/);
});

test("mixed goals cannot bypass the original nested-quote refusal", async () => {
  const a = await physical(
    "nested-fault",
    "Create file a.txt containing 'a('x')'. Then in b.txt replace «old» with «new».",
    { "b.txt": "old" },
  );
  assert.equal(a.artifacts["a.txt"], null);
  assert.equal(a.artifacts["b.txt"], "old");
  assert.equal(a.run.transcript.length, 0);
});

test("carried goal Read reuses only exact qualified complete evidence without later foreign mutation", async () => {
  async function probe({path = "a.txt", complete = true, failed = false, tail = null, brokenId = false} = {}) {
    const parts = ["observe first owned goal", "edit second owned goal"];
    const wanted = {tool: "read", arguments: JSON.stringify({file_path: "a.txt"})};
    const mutation = {tool: "edit", arguments: JSON.stringify({path: "a.txt", old_string: "old", new_string: "new"})};
    const messages = [{role: "user", content: "two source-owned goals"}];
    const observed = {id: "actual-prior-read", type: "function", function: {
      name: "read", arguments: JSON.stringify({path})
    }};
    messages.push({role: "assistant", tool_calls: [observed]}, {
      role: "tool", name: "read", tool_call_id: brokenId ? "foreign-id" : observed.id,
      content: failed ? "read denied" : "old", is_error: failed,
      source_read: {path, success: !failed, complete, format: "raw"}
    });
    if (tail) messages.push({role: "assistant", tool_calls: [{
      id: "later-effect", type: "function", function: {name: tail.tool, arguments: JSON.stringify(tail.arguments)}
    }]}, {role: "tool", name: tail.tool, tool_call_id: "later-effect",
      content: tail.failed ? "permission denied" : "done", is_error: Boolean(tail.failed)});
    const before = JSON.stringify(messages);
    const planFor = async own => {
      const task = own.find(message => message.role === "user").content;
      const receipts = own.filter(message => message.role === "tool");
      if (task === parts[0]) return receipts.length === 0
        ? {kind: "tool_calls", calls: [{tool: "read", arguments: observed.function.arguments}]}
        : resolvedFinalAnswer("prior goal observed", "finding", "controlled-prior-goal");
      const effect = receipts.find(message => message.name === "edit");
      if (effect) return resolvedFinalAnswer(effect.content, effect.is_error ? "failure" : "finding", "controlled-edit-result");
      const read = receipts.find(message => message.name === "read" && message.source_read?.path === "a.txt"
        && message.source_read.complete && !message.is_error);
      if (read) assert.equal(read.tool_call_id, "actual-prior-read", "actual identity survives reuse");
      return {kind: "tool_calls", calls: [read ? mutation : wanted]};
    };
    const result = await replayOwnedGoals(parts, messages, ["read", "edit", "bash", "write"], planFor);
    assert.equal(JSON.stringify(messages), before, "no receipt, call or metadata is fabricated or mutated");
    return result;
  }
  const carried = await probe();
  assert.equal(carried.kind, "tool_calls");assert.equal(carried.calls[0].tool, "edit");
  for (const options of [{path: "b.txt"}, {complete: false}, {failed: true}, {brokenId: true},
    {tail: {tool: "bash", arguments: {command: "node --version"}}},
    {tail: {tool: "write", arguments: {path: "b.txt", content: "changed"}}},
    {tail: {tool: "edit", arguments: {path: "b.txt", old_string: "old", new_string: "changed"}}},
    {tail: {tool: "multi_edit", arguments: {edits: []}}}]) {
    const result = await probe(options);
    assert.equal(result.kind, "tool_calls");assert.equal(result.calls[0].tool, "read");
    assert.equal(JSON.parse(result.calls[0].arguments).file_path, "a.txt");
  }
  const failed = await probe({tail: {tool: "edit",
    arguments: {path: "a.txt", old_string: "old", new_string: "new"}, failed: true}});
  assert.equal(failed.kind, "final");assert.equal(finalResult(failed).disposition, "failure");
  assert.match(failed.answer, /permission denied/u);
});
