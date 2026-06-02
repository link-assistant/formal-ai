// Issue #386 — prove every meaning across all canonical files reaches the
// "link" ontology root by walking defined_by edges (the C1 reachability claim).
// Run: node experiments/issue-386-reach-link.mjs

import fs from "node:fs";

const root = new URL("..", import.meta.url);
const FILES = [
  "data/seed/meanings.lino",
  "data/seed/meanings-units.lino",
  "data/seed/meanings-calendar.lino",
  "data/seed/meanings-facts.lino",
  "data/seed/meanings-software-project.lino",
  "data/seed/meanings-program-synthesis.lino",
  "data/seed/meanings-intent.lino",
  "data/seed/meanings-how.lino",
  "data/seed/meanings-ontology.lino",
];

const definedBy = new Map(); // slug -> [targets]
const order = [];
let cur = null;
for (const rel of FILES) {
  const text = fs.readFileSync(new URL(rel, root), "utf8");
  for (const raw of text.split("\n")) {
    const line = raw.trimEnd();
    let m = line.match(/^  meaning "(.+)"$/);
    if (m) { cur = m[1]; definedBy.set(cur, []); order.push(cur); continue; }
    m = line.match(/^    defined_by "(.+)"$/);
    if (m && cur) definedBy.get(cur).push(m[1]);
  }
}

// closed-graph sanity: every defined_by target is a defined slug
const slugs = new Set(definedBy.keys());
const dangling = [];
for (const [s, ts] of definedBy) for (const t of ts) if (!slugs.has(t)) dangling.push(`${s}->${t}`);

// reachability to "link"
function reaches(slug) {
  const seen = new Set();
  const stack = [slug];
  while (stack.length) {
    const x = stack.pop();
    if (x === "link") return true;
    if (seen.has(x)) continue;
    seen.add(x);
    for (const t of definedBy.get(x) || []) stack.push(t);
  }
  return false;
}

const unreachable = order.filter((s) => !reaches(s));
const roots = order.filter((s) => (definedBy.get(s) || []).includes(s)); // self-loop

console.log(`meanings: ${order.length}`);
console.log(`self-rooted (defined_by self): ${JSON.stringify(roots)}`);
console.log(`dangling defined_by targets: ${dangling.length}${dangling.length ? " :: " + dangling.join(", ") : ""}`);
console.log(`unreachable from "link": ${unreachable.length}${unreachable.length ? " :: " + unreachable.join(", ") : ""}`);

const ok = dangling.length === 0 && unreachable.length === 0 && roots.length === 1 && roots[0] === "link";
console.log(ok ? "\nALL REACH LINK ✓ (single root: link)" : "\nFAILED");
process.exit(ok ? 0 : 1);
