// Capability questions in every language whose cues the seed carries
// (R1188-U1). The browser worker used to check its own cues for ru, zh and hi
// only and read every other language as English, so a Spanish question fell
// through to the unknown answer. The native twin shares the cases in
// `rust/tests/unit/specification/capabilities.rs`.
import assert from "node:assert/strict";
import { test } from "node:test";

import { WorkerHost } from "./support/browser-runtime.mjs";

const CASES = [
  ["web_search", "¿Puedes buscar en internet?", "búsqueda web"],
  ["javascript_execution", "¿Puedes ejecutar JavaScript?", "«ejecución de JavaScript»"],
  ["translation", "¿Puedes traducir?", "«traducción»"],
  ["agent_mode", "¿Tienes modo agente?", "«modo agente»"],
  ["javascript_execution", "Can you execute JavaScript?", "JavaScript execution"],
  ["javascript_execution", "Ты можешь выполнять JavaScript?", "выполнение JavaScript"],
  ["javascript_execution", "支持脚本执行吗？", "JavaScript 执行"],
];

const host = new WorkerHost();

for (const [feature, prompt, fragment] of CASES) {
  test(`capability question ${JSON.stringify(prompt)} names ${feature}`, async () => {
    const response = await host.solve(prompt);
    assert.equal(response.intent, "capabilities", response.content);
    assert.ok(response.evidence.includes(`feature:question:${feature}`), response.evidence.join(", "));
    assert.ok(response.content.toLowerCase().includes(fragment.toLowerCase()), response.content);
  });
}
