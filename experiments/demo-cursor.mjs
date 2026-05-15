// Verify createDemoTurns advances through Example prompts.
import fs from "node:fs";
const src = fs.readFileSync("src/web/app.js", "utf8");
// Pull EXAMPLE_PROMPTS, helpers, cursors, createDemoTurns by string extraction.
const slice = (start, end) => {
  const s = src.indexOf(start);
  const e = src.indexOf(end, s);
  if (s < 0 || e < 0) throw new Error(`slice ${start}`);
  return src.slice(s, e);
};
const code =
  slice("const EXAMPLE_PROMPTS", "const DEMO_GREETING_LABELS") +
  slice("const DEMO_GREETING_LABELS", "function createDemoTurns") +
  slice("function createDemoTurns", "function appendCodeBlock") +
  "return createDemoTurns;";
const fn = new Function(code)();
for (let i = 0; i < 8; i++) {
  console.log(i, fn().map((t) => t.label).join(" + "));
}
