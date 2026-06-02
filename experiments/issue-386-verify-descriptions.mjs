// Issue #386 — structural guard for the per-word-form description backfill.
//
// Adding a `description` child to every `word` must be PURELY ADDITIVE: the
// ordered (meaning, language, word) skeleton of the file must stay identical to
// a baseline (git HEAD by default), and after the edit every word must carry a
// non-empty description. This catches a backfill that drops, reorders, renames,
// or mangles a surface form — the one risk of editing these dense files in bulk.
//
// Usage:
//   node experiments/issue-386-verify-descriptions.mjs <file.lino> [baselineRef]
//   node experiments/issue-386-verify-descriptions.mjs --all [baselineRef]
// baselineRef defaults to HEAD; pass a commit/ref or "-" to skip the skeleton
// diff (described-check only, for brand-new files with no baseline).

import fs from "node:fs";
import { execSync } from "node:child_process";
import path from "node:path";

const root = new URL("..", import.meta.url).pathname;

const MEANING_FILES = [
  "data/seed/meanings.lino",
  "data/seed/meanings-units.lino",
  "data/seed/meanings-calendar.lino",
  "data/seed/meanings-facts.lino",
  "data/seed/meanings-software-project.lino",
  "data/seed/meanings-program-synthesis.lino",
  "data/seed/meanings-intent.lino",
  "data/seed/meanings-ontology.lino",
];

// Parse a .lino lexicon body into ordered records:
//   { key: "slug\tlang\tword\tordinal", described: bool }
// `ordinal` disambiguates a surface that legitimately repeats inside a lexeme.
function parseRecords(text) {
  const records = [];
  let slug = "";
  let lang = "";
  let pending = null; // the most recent word record awaiting its description
  const counts = new Map();
  const lines = text.split("\n");
  for (const raw of lines) {
    const line = raw.replace(/\s+$/, "");
    let m;
    if ((m = line.match(/^  meaning "(.*)"$/))) {
      slug = m[1];
      lang = "";
      pending = null;
    } else if ((m = line.match(/^    lexeme "(.*)"$/))) {
      lang = m[1];
      pending = null;
    } else if ((m = line.match(/^      word "(.*)"$/))) {
      const word = m[1];
      const ck = `${slug}\t${lang}\t${word}`;
      const ord = counts.get(ck) || 0;
      counts.set(ck, ord + 1);
      pending = { key: `${ck}\t${ord}`, described: false };
      records.push(pending);
    } else if ((m = line.match(/^        description "(.*)"$/))) {
      if (pending) pending.described = m[1].trim().length > 0;
    }
  }
  return records;
}

function skeleton(records) {
  return records.map((r) => r.key);
}

function verify(rel, baselineRef) {
  const abs = path.join(root, rel);
  const text = fs.readFileSync(abs, "utf8");
  const cur = parseRecords(text);
  const problems = [];

  // 1) every word described
  const undescribed = cur.filter((r) => !r.described).map((r) => r.key);
  if (undescribed.length) {
    problems.push(
      `${undescribed.length} word(s) lack a non-empty description, e.g. ${undescribed
        .slice(0, 5)
        .join(" | ")}`,
    );
  }

  // 2) skeleton unchanged vs baseline (unless skipped)
  if (baselineRef && baselineRef !== "-") {
    let baseText = null;
    try {
      baseText = execSync(`git show ${baselineRef}:${rel}`, {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      });
    } catch {
      baseText = null; // no baseline (new file) — skip skeleton diff
    }
    if (baseText !== null) {
      const a = skeleton(parseRecords(baseText));
      const b = skeleton(cur);
      if (a.length !== b.length) {
        problems.push(`word count changed: baseline ${a.length} -> now ${b.length}`);
      }
      const n = Math.min(a.length, b.length);
      for (let i = 0; i < n; i++) {
        if (a[i] !== b[i]) {
          problems.push(`skeleton diverges at #${i}: baseline «${a[i]}» -> now «${b[i]}»`);
          break;
        }
      }
    }
  }

  const ok = problems.length === 0;
  console.log(`${ok ? "PASS" : "FAIL"}: ${rel} (${cur.length} words)`);
  for (const p of problems) console.log(`    - ${p}`);
  return ok;
}

const args = process.argv.slice(2);
let targets;
let baselineRef = "HEAD";
if (args[0] === "--all") {
  targets = MEANING_FILES;
  if (args[1]) baselineRef = args[1];
} else {
  targets = [args[0]];
  if (args[1]) baselineRef = args[1];
}

let allOk = true;
for (const rel of targets) {
  if (!fs.existsSync(path.join(root, rel))) {
    console.log(`SKIP: ${rel} (absent)`);
    continue;
  }
  allOk = verify(rel, baselineRef) && allOk;
}
console.log(allOk ? "\nALL PASS" : "\nFAILED");
process.exit(allOk ? 1 - 1 : 1);
