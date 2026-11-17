import assert from "node:assert/strict";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
const host = new WorkerHost();

test("native prime interval request validates rejected candidates before selecting 17", async () => {
  const answer = await host.solve("Pick a prime number between 14 and 18");
  assert.equal(answer.content, "17");
  assert.equal(answer.intent, "number_property_prime");
  assert.ok(answer.evidence.some((link) => link.startsWith("validation:")));
  const candidates = answer.rawSolverEvents.filter((event) => event.kind === "candidate").map((event) => event.payload);
  assert.deepEqual(candidates.slice(-4), ["14", "15", "16", "17"]);
  // Issue #1550: each executed test is recorded with its verdict and the
  // selection names the chosen candidate and how many of them passed.
  const validations = answer.rawSolverEvents.filter((event) => event.kind === "validation").map((event) => event.payload);
  assert.deepEqual(validations, ["number_property_prime(14)=false", "number_property_prime(15)=false", "number_property_prime(16)=false", "number_property_prime(17)=true"]);
  assert.equal(answer.rawSolverEvents.find((event) => event.kind === "trace:simplification").payload, "17=1/4");
});

test("held-out bounds retain the validated intent across en, ru, hi, zh and es", async () => {
  for (const [language, prompt, expected] of [
    ["en", "Choose a prime between 8 and 12", "11"],
    ["ru", "Выбери простое число от 24 до 30", "29"],
    ["es", "Elige un número primo entre 32 y 40", "37"],
    ["zh", "选择8到12之间的质数", "11"],
    ["hi", "0 और 3 के बीच अभाज्य संख्या चुनो", "2"],
  ]) {
    const result = await host.solve(prompt);
    assert.equal(result.content, expected, language);
    assert.equal(result.intent, "number_property_prime", language);
  }
});

test("no candidate is invented for an empty prime interval", async () => {
  const result = await host.run("solverPrimeIntervalValidation(__primePrompt)", { __primePrompt: "Pick a prime between 14 and 16" });
  assert.equal(result.answer, null);
  assert.deepEqual(result.events.filter((event) => event.kind === "candidate").map((event) => event.payload), ["14", "15", "16"]);
  assert.equal(result.events.at(-1).payload, "number_property_prime(16)=false");
  assert.ok(result.events.filter((event) => event.kind === "validation").every((event) => event.payload.endsWith("=false")));
});

test("unsupported bounds and code-authoring requests remain explicit nonmatches", async () => {
  for (const prompt of ["Pick a prime between 12 and 8", "Pick a prime between 9007199254740992 and 9007199254740994", "Pick a primary number between 14 and 18", "Implement a function returning a prime between14and18"]) {
    assert.equal(await host.run("solverPrimeIntervalValidation(__primePrompt)", { __primePrompt: prompt }), null);
  }
});
