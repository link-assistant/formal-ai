import assert from "node:assert/strict";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const host = new WorkerHost();
const fence = String.fromCharCode(96).repeat(3);

test("unchanged native JavaScript request is refused before execution in chat mode", async () => {
  const answer = await host.solve(["Please execute this javascript:", fence + "js", "console.log(1 + 2);", fence].join("\n"));
  assert.equal(answer.intent, "tool_call_refused");
  assert.ok(answer.content.includes("Execution status: refused"));
  assert.ok(answer.content.includes("tool:javascript_execution"));
  assert.ok(answer.content.includes("console.log(1 + 2);"));
  assert.ok(answer.evidence.includes("policy:agent_mode_required_for_tools:tool:javascript_execution"));
});

test("refused user source causes no observable worker-global effect", async () => {
  const prompt = "Run this JavaScript: globalThis.__permissionTripwire = 7;";
  const result = await host.run("globalThis.__permissionTripwire = 0; const refusal = tryNaturalLanguageToolRequest(__permissionPrompt, {}); ({ intent: refusal.intent, value: globalThis.__permissionTripwire })", { __permissionPrompt: prompt });
  assert.equal(result.intent, "tool_call_refused");
  assert.equal(result.value, 0);
});

test("explicit server agent mode uses the real existing worker executor", async () => {
  const answer = await host.solve("Run this JavaScript: console.log(6 * 7);", [], { agentMode: true });
  assert.equal(answer.intent, "javascript_execution");
  assert.ok(answer.content.includes("42"));
  assert.ok(answer.evidence.includes("execution_status:javascript:ran"));
});

test("explicit server chat mode overrides a retained agent preference", async () => {
  const result = await host.run("solve(__permissionPrompt, [], { agentMode: true }, {}, [], { agentMode: false })", { __permissionPrompt: "Execute this JavaScript: console.log(5);" });
  assert.equal(result.intent, "tool_call_refused");
  assert.ok(result.evidence.includes("policy:agent_mode_required_for_tools:tool:javascript_execution"));
});
