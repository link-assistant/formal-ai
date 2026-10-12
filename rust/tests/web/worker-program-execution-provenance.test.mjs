import assert from "node:assert/strict";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const host = new WorkerHost();

test("unchanged native TypeScript request exposes actual documentation verification instead of a run", async () => {
  const answer = await host.solve("hello world in TypeScript");
  assert.ok(answer.content.includes("Execution status: not run; this program was rediscovered from"));
  assert.ok(answer.programExecution.rediscoveredFrom);
  assert.ok(answer.content.includes(answer.programExecution.rediscoveredFrom));
  assert.ok(answer.content.includes("checked by decomposition, not by executing it"));
  const verification = answer.rawSolverEvents.find((event) => event.kind === "program_verification");
  assert.ok(verification.payload.includes("verification=decomposition"));
  assert.ok(verification.payload.includes(answer.programExecution.rediscoveredFrom));
  assert.ok(answer.evidence.some((link) => link.startsWith("program_verification:")));
});

test("rediscovered Python source also names its actual page and stays not run", async () => {
  const answer = await host.solve("hello world in Python");
  assert.ok(answer.programExecution.rediscoveredFrom);
  assert.ok(answer.content.includes(answer.programExecution.rediscoveredFrom));
  assert.ok(answer.content.includes("checked by decomposition, not by executing it"));
});

test("an actually executed browser JavaScript template retains observed execution evidence", async () => {
  const answer = await host.solve("hello world in JavaScript");
  assert.ok(answer.evidence.includes("execution_status:javascript:ran"));
  assert.ok(answer.content.includes("Hello, world!"));
  assert.ok(!answer.content.includes("Execution status: not run; this program was rediscovered"));
});
