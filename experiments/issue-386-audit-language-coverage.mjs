// Issue #386 — audit every canonical meaning for full supported-language
// coverage (en/ru/hi/zh). The Rust invariant
// `seed::meanings::tests::every_meaning_covers_all_supported_languages`
// (src/seed/meanings.rs) fails fast on the FIRST meaning missing any language,
// so it only ever names one violator at a time. This standalone parser walks
// the same MEANING_FILES (in declaration order) and reports EVERY meaning that
// is missing one or more of the four supported languages, so a backfill can be
// planned in one pass instead of discovered one panic at a time.
//
// Run with: node experiments/issue-386-audit-language-coverage.mjs

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { MEANING_FILES } from "./issue-386-meaning-files.mjs";

const SUPPORTED = ["en", "ru", "hi", "zh"];
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const meaningRe = /^  meaning "([^"]+)"\s*$/;
const lexemeRe = /^    lexeme "([^"]+)"\s*$/;
const wordRe = /^      word "/;

let total = 0;
const violators = [];

for (const rel of MEANING_FILES) {
  const text = readFileSync(path.join(repoRoot, rel), "utf8");
  const lines = text.split("\n");
  let cur = null; // { slug, file, langs: Map<lang, wordCount> }
  const flush = () => {
    if (!cur) return;
    total += 1;
    const missing = SUPPORTED.filter((l) => !cur.langs.has(l) || cur.langs.get(l) === 0);
    if (missing.length) violators.push({ ...cur, missing });
  };
  for (const line of lines) {
    const m = meaningRe.exec(line);
    if (m) {
      flush();
      cur = { slug: m[1], file: rel, langs: new Map() };
      continue;
    }
    const lx = lexemeRe.exec(line);
    if (lx && cur) {
      if (!cur.langs.has(lx[1])) cur.langs.set(lx[1], 0);
      continue;
    }
    if (wordRe.test(line) && cur) {
      // attribute the word to the most-recently-seen lexeme
      const langs = [...cur.langs.keys()];
      const last = langs[langs.length - 1];
      if (last) cur.langs.set(last, cur.langs.get(last) + 1);
    }
  }
  flush();
}

console.log(`audited ${total} meanings across ${MEANING_FILES.length} files`);
if (!violators.length) {
  console.log("OK — every meaning covers all supported languages:", SUPPORTED.join("/"));
  process.exit(0);
}
console.log(`\n${violators.length} meaning(s) missing one or more languages:\n`);
for (const v of violators) {
  const have = SUPPORTED.filter((l) => v.langs.has(l) && v.langs.get(l) > 0);
  console.log(`  ${v.slug}  (${v.file})`);
  console.log(`      has: ${have.join(", ") || "(none)"}    MISSING: ${v.missing.join(", ")}`);
}
process.exit(1);
