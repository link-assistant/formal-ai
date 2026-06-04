// Issue #386 — single source of truth for the JS-side meaning-file mirror.
//
// The canonical lexicon is split across several .lino files so no single file
// breaches the seed file-size guard. The Rust loader concatenates them in the
// order of `MEANING_FILES` in src/seed/embedded.rs; every JS consumer — the
// worker sync (issue-386-sync-worker-lexicon.mjs) and the mirror verifier
// (issue-386-meanings-mirror.mjs) — must use the SAME order and the SAME
// per-line serializer, so both live here and are imported rather than
// re-declared. Keeping one copy is what stops the verifier from silently
// bit-rotting behind the sync (it previously froze at 9 of the 16 files).

// Mirror of `MEANING_FILES` in src/seed/embedded.rs, in declaration order. When
// a new meanings-*.lino is registered in Rust, add it here in the same slot.
export const MEANING_FILES = [
  "data/seed/meanings.lino",
  "data/seed/meanings-units.lino",
  "data/seed/meanings-calendar.lino",
  "data/seed/meanings-calculator.lino",
  "data/seed/meanings-facts.lino",
  "data/seed/meanings-software-project.lino",
  "data/seed/meanings-program-synthesis.lino",
  "data/seed/meanings-intent.lino",
  "data/seed/meanings-how.lino",
  "data/seed/meanings-meta.lino",
  "data/seed/meanings-web-navigation.lino",
  "data/seed/meanings-web-search.lino",
  "data/seed/meanings-web-search-query.lino",
  "data/seed/meanings-web-research.lino",
  "data/seed/meanings-web-followup.lino",
  "data/seed/meanings-translation.lino",
  "data/seed/meanings-ontology.lino",
  "data/seed/meanings-behavior-rules.lino",
  "data/seed/meanings-proof.lino",
  "data/seed/meanings-policy.lino",
  "data/seed/meanings-docs.lino",
  "data/seed/meanings-skill-compiler.lino",
  "data/seed/meanings-finance.lino",
  "data/seed/meanings-definition-merge.lino",
  "data/seed/meanings-tool-access.lino",
  "data/seed/meanings-feature-capability.lino",
  "data/seed/meanings-playwright.lino",
  "data/seed/meanings-research-table.lino",
  "data/seed/meanings-conversation.lino",
  "data/seed/meanings-summary.lino",
  "data/seed/meanings-coding-catalog.lino",
];

// Authoritative per-line JS string serializer (house quote style, matching the
// PROGRAM_PLAN_RULES_LINO convention): a line containing a double-quote and
// neither a single-quote nor a backslash is wrapped in single quotes; otherwise
// it is JSON.stringify'd (which correctly escapes backslashes and quotes). Used
// to (re)build every inline *_LINO array so the worker body is byte-identical to
// the canonical seed.
export function serializeMeaningLine(line) {
  if (line.includes('"') && !line.includes("'") && !line.includes("\\")) {
    return "'" + line + "'";
  }
  return JSON.stringify(line);
}
