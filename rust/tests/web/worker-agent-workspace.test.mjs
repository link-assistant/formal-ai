import assert from "node:assert/strict";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
const host = new WorkerHost(), quote = String.fromCharCode(96);
const literal = (text) => quote + text + quote;
test("unchanged native five-action request performs real isolated actions", async () => {
  const prompt = "[agent] In the isolated workspace, create file report.txt with " + literal("alpha") + ", modify report.txt to " + literal("beta") + ", create file scratch.tmp with " + literal("remove me") + ", delete scratch.tmp, and run command " + literal("cat report.txt");
  const answer = await host.solve(prompt);
  assert.equal(answer.intent, "agent_workspace_task");
  for (const text of ["Workspace isolation:", "created report.txt", "modified report.txt", "deleted scratch.tmp", "Command: " + literal("cat report.txt"), "Output:\n" + quote.repeat(3) + "text\nbeta"]) assert.ok(answer.content.includes(text), text);
  for (const kind of ["create_file", "modify_file", "delete_file", "run_command"]) assert.ok(answer.evidence.some((link) => link.startsWith("action_log:" + kind + ":")), kind);
  assert.equal(answer.rawSolverEvents.filter((event) => event.kind.startsWith("action_log:")).length, 5);
});
test("held-out paths and contents follow actual ordered mutations", async () => {
  const answer = await host.solve("[agent] create file nested/held.txt with " + literal("delta") + ", modify file nested/held.txt to " + literal("epsilon") + ", run terminal command " + literal("cat nested/held.txt"));
  assert.equal(answer.intent, "agent_workspace_task");
  assert.ok(answer.content.includes("text\nepsilon"));
});
test("fresh workspaces cannot read previous-turn or host files", async () => {
  const result = await host.solve("[agent] run command " + literal("cat report.txt"));
  assert.equal(result.intent, "agent_workspace_task_failed");
  assert.ok(result.content.includes("file unavailable in isolated workspace"));
  const escaped = await host.run('tryBrowserAgentWorkspace(__prompt)', { __prompt: "[agent] create file ../outside.txt with " + literal("bad") + ", run command " + literal("cat ../outside.txt") });
  assert.equal(escaped.intent, "agent_workspace_task_failed");
  assert.ok(escaped.content.includes("path escapes"));
  assert.ok(!escaped.content.includes("text\nbad"));
});
test("unavailable processes and mutations publish actual failures", async () => {
  const result = await host.solve("[agent] modify absent.txt to " + literal("unused") + ", run command " + literal("node impossible.js"));
  assert.equal(result.intent, "agent_workspace_task_failed");
  assert.ok(result.content.includes("Exit: 127"));
  assert.ok(result.content.includes("command unavailable"));
  assert.ok(result.rawSolverEvents.some((event) => event.kind === "trace:execution_failure"));
});
test("chat without opt-in cannot create a workspace", async () => {
  assert.equal(await host.run('tryBrowserAgentWorkspace(__prompt)', { __prompt: "create file plain.txt with " + literal("unexecuted") }), null);
});
