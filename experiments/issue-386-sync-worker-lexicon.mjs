// Issue #386 — keep src/web/formal_ai_worker.js in lockstep with the canonical
// meaning lexicon and route the "write a <program>" gate through semantic roles.
//
// This supersedes the one-shot issue-386-apply-meanings-worker.mjs: it is
// idempotent and re-runnable. Whenever any data/seed/meanings*.lino file
// changes, run:
//   node experiments/issue-386-sync-worker-lexicon.mjs
// It (1) regenerates the inline MEANINGS_LINO array byte-identically from the
// canonical seed (the PROGRAM_PLAN_RULES_LINO convention), (2) ensures the
// program_kind / program_request role constants exist, (3) removes the legacy
// hardcoded PROGRAM_NOUNS / PROGRAM_VERBS arrays, and (4) rewrites the
// writeProgramParameters gate to ask the lexicon which meanings a prompt
// evidences. Parity with the Rust solver is guarded by
// experiments/issue-386-js-cancel-sort.mjs.

import fs from "node:fs";

const root = new URL("..", import.meta.url);
// The canonical lexicon is split across several files so no single .lino file
// breaches the seed file-size guard. Concatenate them in the SAME order as
// MEANING_FILES in src/seed.rs so the Rust loader and this JS mirror parse the
// same input; each file wraps its records under a top-level `meanings` node and
// meaningLexicon() in the worker walks every container.
const MEANING_FILES = ["data/seed/meanings.lino", "data/seed/meanings-units.lino"];
const workerPath = new URL("src/web/formal_ai_worker.js", root);

const lino = MEANING_FILES.map((rel) =>
  fs.readFileSync(new URL(rel, root), "utf8").replace(/\n+$/, ""),
).join("\n");
let worker = fs.readFileSync(workerPath, "utf8");

// --- 1: regenerate the inline MEANINGS_LINO array (byte-identical to seed) ----
// House quote style (matches PROGRAM_PLAN_RULES_LINO): a line containing a
// double-quote and neither a single-quote nor a backslash is wrapped in single
// quotes; otherwise it is JSON.stringify'd.
function jsString(line) {
  if (line.includes('"') && !line.includes("'") && !line.includes("\\")) {
    return "'" + line + "'";
  }
  return JSON.stringify(line);
}
const lines = lino.split("\n");
while (lines.length && lines[lines.length - 1] === "") lines.pop();
const arrayBody = lines.map((l) => "  " + jsString(l)).join(",\n");
const newBlock = `const MEANINGS_LINO = [\n${arrayBody},\n].join("\\n");`;

const blockRe = /const MEANINGS_LINO = \[\n[\s\S]*?\n\]\.join\("\\n"\);/;
if (!blockRe.test(worker)) throw new Error("anchor not found: MEANINGS_LINO inline block");
worker = worker.replace(blockRe, newBlock);

// --- 2: ensure the two new role constants exist ------------------------------
const modConst = `const ROLE_PROGRAM_MODIFICATION = "program_modification";`;
if (!worker.includes(modConst)) throw new Error("anchor not found: ROLE_PROGRAM_MODIFICATION");
const newRoleConsts = `${modConst}
// Semantic role: a kind of program artifact a user can ask to be authored
// (a program, a script, code, a function) — the noun side of "write a <kind>".
const ROLE_PROGRAM_KIND = "program_kind";
// Semantic role: a verb that requests a program artifact be produced (write,
// create, show, generate, make, build) — the verb side of "write a <kind>".
const ROLE_PROGRAM_REQUEST = "program_request";`;
if (!worker.includes("ROLE_PROGRAM_KIND")) {
  worker = worker.replace(modConst, newRoleConsts);
}

// --- 3: remove the legacy hardcoded PROGRAM_NOUNS / PROGRAM_VERBS arrays ------
const arraysRe =
  /\/\/ Issue #312: the Russian reporter wrote[\s\S]*?const PROGRAM_VERBS = \[[\s\S]*?\n\];\n\n/;
if (arraysRe.test(worker)) {
  worker = worker.replace(arraysRe, "");
} else if (worker.includes("const PROGRAM_NOUNS = [")) {
  throw new Error("PROGRAM_NOUNS array present but did not match removal anchor");
}

// --- 4: route the write-program gate through semantic roles ------------------
const gateOld = `  const asksForProgram =
    PROGRAM_NOUNS.some((noun) => containsProgramToken(normalized, noun)) &&
    PROGRAM_VERBS.some((verb) => containsProgramToken(normalized, verb));`;
const gateNew = `  // Issue #386: recognise "write a <program>" by *meaning*, not a hardcoded
  // per-language word list — a program_kind artifact (program / script / code /
  // function) requested by a program_request verb (write / create / … / build).
  // The surface words live once in data/seed/meanings.lino; this code knows the
  // concepts. Mirrors write_program_parameters in src/intent_formalization.rs.
  const asksForProgram =
    lexiconMentionsRole(ROLE_PROGRAM_KIND, normalized) &&
    lexiconMentionsRole(ROLE_PROGRAM_REQUEST, normalized);`;
if (worker.includes(gateOld)) {
  worker = worker.replace(gateOld, gateNew);
} else if (!worker.includes("lexiconMentionsRole(ROLE_PROGRAM_KIND, normalized)")) {
  throw new Error("anchor not found: asksForProgram gate");
}

// --- safety: the legacy identifiers must be fully gone -----------------------
for (const dead of ["PROGRAM_NOUNS", "PROGRAM_VERBS"]) {
  if (worker.includes(dead)) throw new Error(`leftover reference to ${dead}`);
}

fs.writeFileSync(workerPath, worker);
console.log(`synced formal_ai_worker.js to ${MEANING_FILES.join(" + ")}`);
console.log(`MEANINGS_LINO inline array: ${lines.length} lines`);
