import assert from "node:assert/strict";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
const host = new WorkerHost();

test("unchanged native prime interval request validates rejected candidates before selecting17", async () => {
  const answer = await host.solve("Pick a prime number between 14 and 18");
  assert.ok(answer.content.includes("17"));
  assert.ok(answer.evidence.some((link) => link.startsWith("validation:")));
  const candidates = answer.rawSolverEvents.filter((event) => event.kind === "candidate").map((event) => event.payload);
  assert.deepEqual(candidates.slice(-4), ["14", "15", "16", "17"]);
  assert.ok(answer.rawSolverEvents.some((event) => event.kind === "validation" && event.payload === "prime_between_14_and_18"));
});

test("held-out bounds and seeded languages use the same actual predicate", async () => {
  for (const [prompt, expected] of [
    ["Choose a prime between 8 and 12", "11"],
    ["Выбери простое число от 24 до 30", "29"],
    ["Elige un número primo entre 32 y 40", "37"],
    ["选择8到12之间的质数", "11"],
    ["0 और 3 के बीच अभाज्य संख्या चुनो", "2"],
  ]) {
    const result = await host.run("solverPrimeIntervalValidation(__primePrompt)", { __primePrompt: prompt });
    assert.equal(result.answer, expected, prompt);
  }
});

test("no candidate is invented for an empty prime interval", async () => {
  const result = await host.run("solverPrimeIntervalValidation(__primePrompt)", { __primePrompt: "Pick a prime between 14 and 16" });
  assert.equal(result.answer, null);
  assert.deepEqual(result.events.filter((event) => event.kind === "candidate").map((event) => event.payload), ["14", "15", "16"]);
  assert.equal(result.events.at(-1).payload, "no_prime_in_range");
});

test("unsupported bounds and code-authoring requests remain explicit nonmatches", async () => {
  for (const prompt of ["Pick a prime between 12 and 8", "Pick a prime between 9007199254740992 and 9007199254740994", "Pick a primary number between 14 and 18", "Implement a function returning a prime between14and18"]) {
    assert.equal(await host.run("solverPrimeIntervalValidation(__primePrompt)", { __primePrompt: prompt }), null);
  }
});
