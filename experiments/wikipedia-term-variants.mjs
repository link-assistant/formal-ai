// Experiment: verify wikipediaTermVariants emits the Russian "Surname,
// Given names" form so `Кто такой Илон Маск?` resolves on ru.wikipedia.org.
//
// Run with: node experiments/wikipedia-term-variants.mjs

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const workerPath = path.resolve(
  __dirname,
  "..",
  "src",
  "web",
  "formal_ai_worker.js",
);

const src = fs.readFileSync(workerPath, "utf8");
// Extract the two helpers we need without booting the full worker.
const slice = (marker) => {
  const start = src.indexOf(`function ${marker}`);
  if (start < 0) throw new Error(`could not find ${marker}`);
  let depth = 0;
  let i = src.indexOf("{", start);
  for (; i < src.length; i++) {
    if (src[i] === "{") depth++;
    else if (src[i] === "}") {
      depth--;
      if (depth === 0) return src.slice(start, i + 1);
    }
  }
  throw new Error(`unterminated ${marker}`);
};

const code = `${slice("capitalizeWords")}\n${slice("wikipediaTermVariants")}\nreturn wikipediaTermVariants(term);`;
const fn = new Function("term", code);

const cases = [
  "Илон Маск",
  "илон маск",
  "Donald Trump",
  "donald trump",
  "javascript",
  "Mahatma Gandhi",
];

for (const c of cases) {
  console.log(JSON.stringify(c), "→", fn(c));
}
