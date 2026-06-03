#!/usr/bin/env node
// Issue #386 — rewrite the browser worker's user-intent recognisers
// (web-search history signal, proof request, who-is question) so they reason
// over the meaning lexicon by role + slot instead of hardcoded per-language
// word lists, mirroring the Rust changes in src/solver_handlers/user_intent.rs.
//
// Edits applied to src/web/formal_ai_worker.js, in order:
//   1. Rename the four generic-but-search-named affix helpers
//      search{Prefix,Suffix,Bare,Source}Literals -> {prefix,suffix,bare,source}Literals
//      (they are universal helpers now that proof/who-is reuse them).
//   2. Insert ROLE_WEB_SEARCH_HISTORY_SIGNAL into the web-search role cluster
//      and a new proof + who-is role cluster.
//   3. Rewrite historyMentionsWebSearch, hasProofRequestShape, isWhoIsPrompt to
//      query the lexicon, and replace extractProofClaim's hardcoded prefix array
//      with prefixLiterals(ROLE_PROOF_CLAIM_SCAFFOLD).
//
// The transform is idempotent-safe to *inspect*: it asserts each anchor exists
// exactly where expected and bails loudly otherwise, so a second run (after the
// rename) would fail fast rather than corrupt the file.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const workerPath = path.resolve(here, "../src/web/formal_ai_worker.js");

let source = fs.readFileSync(workerPath, "utf8");

// --- 1. Rename the four affix helpers (every occurrence is `name(`). --------
const renames = [
  ["searchPrefixLiterals(", "prefixLiterals("],
  ["searchSuffixLiterals(", "suffixLiterals("],
  ["searchBareLiterals(", "bareLiterals("],
  ["searchSourceLiterals(", "sourceLiterals("],
];
for (const [from, to] of renames) {
  const count = source.split(from).length - 1;
  if (count === 0) throw new Error(`rename anchor not found: ${from}`);
  source = source.split(from).join(to);
  console.log(`renamed ${count}x  ${from} -> ${to}`);
}

let lines = source.split("\n");

// Helper: replace the lines [startExact .. next column-0 "}"] (inclusive) with
// `replacement` (array of lines). Anchored on an exact, unique signature line.
function replaceFn(signature, replacement) {
  const start = lines.indexOf(signature);
  if (start === -1) throw new Error(`replaceFn: signature not found: ${signature}`);
  if (lines.indexOf(signature, start + 1) !== -1) {
    throw new Error(`replaceFn: signature not unique: ${signature}`);
  }
  let end = -1;
  for (let i = start + 1; i < lines.length; i += 1) {
    if (lines[i] === "}") {
      end = i;
      break;
    }
  }
  if (end === -1) throw new Error(`replaceFn: closing brace not found for: ${signature}`);
  lines.splice(start, end - start + 1, ...replacement);
  console.log(`rewrote ${signature}  (${end - start + 1} -> ${replacement.length} lines)`);
}

// Helper: replace the inclusive span [firstExact .. lastExact], where lastExact
// is the first occurrence at/after firstExact. Both must be exact line matches.
function replaceSpan(firstExact, lastExact, replacement) {
  const start = lines.indexOf(firstExact);
  if (start === -1) throw new Error(`replaceSpan: first not found: ${firstExact}`);
  const end = lines.indexOf(lastExact, start);
  if (end === -1) throw new Error(`replaceSpan: last not found after first: ${lastExact}`);
  lines.splice(start, end - start + 1, ...replacement);
  console.log(`replaced span ${firstExact} .. ${lastExact}  (-> ${replacement.length} lines)`);
}

// Helper: insert `replacement` lines immediately after the exact `anchor` line.
function insertAfter(anchor, replacement) {
  const at = lines.indexOf(anchor);
  if (at === -1) throw new Error(`insertAfter: anchor not found: ${anchor}`);
  lines.splice(at + 1, 0, ...replacement);
  console.log(`inserted ${replacement.length} line(s) after: ${anchor}`);
}

// --- 2. Insert role consts. --------------------------------------------------
insertAfter('const ROLE_WEB_SEARCH_SOURCE_ONLY = "web_search_source_only";', [
  '// Mention of web search inside a *prior* conversation turn (raw lowercased',
  '// substring of the turn text, not the normalised prompt). Mirrors',
  '// ROLE_WEB_SEARCH_HISTORY_SIGNAL in src/seed/roles.rs.',
  'const ROLE_WEB_SEARCH_HISTORY_SIGNAL = "web_search_history_signal";',
]);

insertAfter('const ROLE_ENUMERATION_CONSTRAINT = "enumeration_constraint";', [
  "",
  "// Issue #386 proof + who-is roles — mirror ROLE_PROOF_* / ROLE_WHO_QUESTION_*",
  "// in src/seed/roles.rs. Surfaces live in data/seed/meanings-proof.lino and the",
  "// who_is_question meaning in data/seed/meanings-intent.lino. The proof_directive",
  "// bare verbs and proof_claim_scaffold prefixes share the `prove` meaning,",
  "// separated by slot. (The worker's proof engine does not branch on the",
  "// Goedel/determinism concepts, so those roles are referenced only by Rust.)",
  'const ROLE_PROOF_DIRECTIVE = "proof_directive";',
  'const ROLE_PROOF_REQUEST_LEAD = "proof_request_lead";',
  'const ROLE_PROOF_MARKER = "proof_marker";',
  'const ROLE_PROOF_CLAIM_SCAFFOLD = "proof_claim_scaffold";',
  'const ROLE_WHO_QUESTION_LEAD = "who_question_lead";',
  'const ROLE_WHO_QUESTION_TAIL = "who_question_tail";',
]);

// --- 3a. historyMentionsWebSearch. ------------------------------------------
replaceFn("function historyMentionsWebSearch(history) {", [
  "function historyMentionsWebSearch(history) {",
  "  if (!Array.isArray(history)) return false;",
  "  return history.some((turn) => {",
  '    const content = String(turn && turn.content ? turn.content : "").toLowerCase();',
  "    return lexiconMentionsRoleSubstring(ROLE_WEB_SEARCH_HISTORY_SIGNAL, content);",
  "  });",
  "}",
]);

// --- 3b. hasProofRequestShape. ----------------------------------------------
replaceFn("function hasProofRequestShape(normalized) {", [
  "function hasProofRequestShape(normalized) {",
  '  const text = String(normalized || "").trim();',
  "  if (!text) return false;",
  "  // A proof request is recognised structurally from the meaning lexicon, not",
  "  // from words baked into this file: a clause-initial bare directive verb",
  "  // (proof_directive, with the verb-boundary check), an English request-frame",
  "  // lead that needs no `that` clause (proof_request_lead), or a mid-prompt proof",
  "  // assertion marker in any language (proof_marker).",
  "  return (",
  "    bareLiterals(ROLE_PROOF_DIRECTIVE).some((verb) => startsWithProofVerb(text, verb)) ||",
  "    prefixLiterals(ROLE_PROOF_REQUEST_LEAD).some((lead) => text.startsWith(lead)) ||",
  "    lexiconMentionsRoleSubstring(ROLE_PROOF_MARKER, text)",
  "  );",
  "}",
]);

// --- 3c. extractProofClaim: replace the hardcoded prefix array only. --------
{
  const fnStart = lines.indexOf("function extractProofClaim(normalized) {");
  if (fnStart === -1) throw new Error("extractProofClaim signature not found");
  const arrStart = lines.indexOf("  const prefixes = [", fnStart);
  if (arrStart === -1) throw new Error("extractProofClaim prefixes array not found");
  const arrEnd = lines.indexOf("  ];", arrStart);
  if (arrEnd === -1) throw new Error("extractProofClaim prefixes array close not found");
  lines.splice(
    arrStart,
    arrEnd - arrStart + 1,
    "  // The claim scaffolds (each ending in the … slot) come from the",
    "  // proof_claim_scaffold role in declaration order, so the first matching",
    "  // prefix wins exactly as before — every that/что/कि variant is listed ahead",
    "  // of its shorter sibling in the lexicon. Comma variants are absent: the",
    "  // normaliser rewrites the comma to a space, making them unreachable here.",
    "  const prefixes = prefixLiterals(ROLE_PROOF_CLAIM_SCAFFOLD);",
  );
  console.log(`replaced extractProofClaim prefix array (${arrEnd - arrStart + 1} lines -> 6)`);
}

// --- 3d. isWhoIsPrompt. ------------------------------------------------------
replaceFn("function isWhoIsPrompt(normalized) {", [
  "function isWhoIsPrompt(normalized) {",
  "  // \"who is …\" detection reasons over the who_question meaning: a language",
  "  // whose marker leads the name occupies the who_question_lead prefix slot",
  "  // (English who is …, Russian кто такой …), while one whose marker trails it",
  "  // occupies the who_question_tail suffix slot (Hindi … कौन है, Chinese …是谁).",
  "  return (",
  "    prefixLiterals(ROLE_WHO_QUESTION_LEAD).some((lead) => normalized.startsWith(lead)) ||",
  "    suffixLiterals(ROLE_WHO_QUESTION_TAIL).some((tail) => normalized.endsWith(tail))",
  "  );",
  "}",
]);

fs.writeFileSync(workerPath, lines.join("\n"), "utf8");
console.log("\nwrote", workerPath);
