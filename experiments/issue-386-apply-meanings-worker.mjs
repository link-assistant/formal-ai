// Issue #386 — apply the meaning-lexicon refactor to src/web/formal_ai_worker.js.
//
// The worker must mirror src/seed/meanings.rs: recognition references semantic
// *roles*, not hardcoded per-language word lists. To keep the embedded lexicon
// byte-identical to the canonical seed (the PROGRAM_PLAN_RULES_LINO convention),
// we generate the inline `MEANINGS_LINO` array directly from
// `data/seed/meanings.lino` rather than hand-transcribing it. Run once:
//   node experiments/issue-386-apply-meanings-worker.mjs
// Idempotent-ish: it asserts each anchor matches exactly once, else it throws.

import fs from "node:fs";

const root = new URL("..", import.meta.url);
const linoPath = new URL("data/seed/meanings.lino", root);
const workerPath = new URL("src/web/formal_ai_worker.js", root);

const lino = fs.readFileSync(linoPath, "utf8");
let worker = fs.readFileSync(workerPath, "utf8");

// --- build the inline MEANINGS_LINO array, matching house quote style --------
// Lines with a double-quote and no single-quote are wrapped in single quotes
// (e.g. '  meaning "result"'); lines without are JSON.stringify'd. This is the
// exact convention used by PROGRAM_PLAN_RULES_LINO just above in the worker.
function jsString(line) {
  if (line.includes('"') && !line.includes("'") && !line.includes("\\")) {
    return "'" + line + "'";
  }
  return JSON.stringify(line);
}
const lines = lino.split("\n");
while (lines.length && lines[lines.length - 1] === "") lines.pop();
const arrayBody = lines.map((l) => "  " + jsString(l)).join(",\n");

const lexiconBlock = `// Issue #386: the language-independent *meaning* lexicon — the JS mirror of
// \`data/seed/meanings.lino\` and \`src/seed/meanings.rs\`. Recognition references
// semantic *roles* (which surface words evidence a program artifact / a program
// modification, in any language), never a hardcoded per-language word list. This
// is an inline, byte-identical copy of the canonical seed (the same convention
// as PROGRAM_PLAN_RULES_LINO) so the worker stays self-contained when no seed has
// been fetched; parity is guarded by \`experiments/issue-386-js-cancel-sort.mjs\`.
const MEANINGS_LINO = [
${arrayBody},
].join("\\n");

// Semantic role: a thing a program produces that a later turn can refer back to
// (a result, an output, the program/script/code itself, an ordering).
const ROLE_PROGRAM_ARTIFACT = "program_artifact";
// Semantic role: an operation a follow-up turn can request against the active
// program (sort, reverse, cancel, change, …) — additive or subtractive.
const ROLE_PROGRAM_MODIFICATION = "program_modification";

let cachedMeaningLexicon = null;
// Parse the embedded lexicon once. Each meaning keeps the semantic roles it
// plays and the surface words (across every language) that evidence it.
function meaningLexicon() {
  if (cachedMeaningLexicon) return cachedMeaningLexicon;
  const root = parseLinoTree(MEANINGS_LINO);
  // meanings.lino wraps its records under a single top-level \`meanings\` node,
  // so the records are its children (not the document root's).
  const container = root.children.find((child) => child.name === "meanings") || root;
  const meanings = [];
  for (const node of container.children) {
    if (node.name !== "meaning") continue;
    const roles = [];
    const words = [];
    for (const child of node.children) {
      if (child.name === "role") roles.push(child.value);
      else if (child.name === "lexeme") {
        for (const lexWord of child.children) {
          if (lexWord.name === "word") words.push(lexWord.value);
        }
      }
    }
    meanings.push({ slug: node.value, roles, words });
  }
  cachedMeaningLexicon = meanings;
  return cachedMeaningLexicon;
}

// Does \`normalized\` mention any surface word of any meaning carrying \`role\`?
// Mirrors the CJK-substring vs. whitespace-token contract via containsProgramToken.
function lexiconMentionsRole(role, normalized) {
  return meaningLexicon().some(
    (meaning) =>
      meaning.roles.includes(role) &&
      meaning.words.some((word) => containsProgramToken(normalized, word)),
  );
}

`;

// --- replacement 1: drop the two hardcoded arrays, insert the lexicon --------
const arraysStart = "const PROGRAM_FOLLOW_UP_REFERENTS = [";
const arraysEndAnchor = "\nfunction detectedProgramModifiers(";
const startIdx = worker.indexOf(arraysStart);
if (startIdx === -1) throw new Error("anchor not found: PROGRAM_FOLLOW_UP_REFERENTS array");
if (worker.indexOf(arraysStart, startIdx + 1) !== -1) {
  throw new Error("anchor not unique: PROGRAM_FOLLOW_UP_REFERENTS array");
}
const endIdx = worker.indexOf(arraysEndAnchor, startIdx);
if (endIdx === -1) throw new Error("anchor not found: detectedProgramModifiers");
worker = worker.slice(0, startIdx) + lexiconBlock + worker.slice(endIdx + 1); // +1 drops the leading \n

// --- replacement 2: route looksLikeBare* through roles, drop hasAnyProgramToken
const fnBlockOld = `function hasAnyProgramToken(normalized, tokens) {
  return tokens.some((token) => containsProgramToken(normalized, token));
}

function looksLikeBareProgramArtifactFollowUp(normalized) {
  return (
    hasAnyProgramToken(normalized, PROGRAM_FOLLOW_UP_REFERENTS) &&
    hasAnyProgramToken(normalized, PROGRAM_FOLLOW_UP_ACTIONS)
  );
}`;
const fnBlockNew = `function looksLikeBareProgramArtifactFollowUp(normalized) {
  // Issue #386: a bare follow-up modifies an existing program artifact when the
  // prompt evidences a program_artifact meaning *and* a program_modification
  // meaning. The surface words live once in the seed; this code knows concepts.
  return (
    lexiconMentionsRole(ROLE_PROGRAM_ARTIFACT, normalized) &&
    lexiconMentionsRole(ROLE_PROGRAM_MODIFICATION, normalized)
  );
}`;
if (!worker.includes(fnBlockOld)) throw new Error("anchor not found: hasAnyProgramToken/looksLikeBare block");
if (worker.indexOf(fnBlockOld) !== worker.lastIndexOf(fnBlockOld)) {
  throw new Error("anchor not unique: hasAnyProgramToken/looksLikeBare block");
}
worker = worker.replace(fnBlockOld, fnBlockNew);

// Safety: the old array identifiers must be fully gone.
for (const dead of ["PROGRAM_FOLLOW_UP_REFERENTS", "PROGRAM_FOLLOW_UP_ACTIONS", "hasAnyProgramToken"]) {
  if (worker.includes(dead)) throw new Error(`leftover reference to ${dead}`);
}

fs.writeFileSync(workerPath, worker);
console.log("applied meaning-lexicon refactor to formal_ai_worker.js");
console.log(`MEANINGS_LINO inline array: ${lines.length} lines`);
