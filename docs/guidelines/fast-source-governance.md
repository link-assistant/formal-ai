# Fast source governance

For new seed meanings, declare every role with the maintained registry generator and provide genuine precondition, effect, semantic unit (or "not applicable"), example and role metadata.
Keep English, Russian, Hindi, Chinese and Spanish lexemes and response templates complete.
Localize data without silently widening one backend's parser language scope.
Use exact whole-answer goldens for successful QA; preserve explicit unresolved outcomes.
Run `node scripts/generate-role-registry.mjs` locally; CI compares it with the canonical Python writer.
Cheap source governance belongs in the fast PR preflight; native execution remains on main.
Regenerate complete source witnesses after source-owner changes and review their complete structural differences instead of patching hashes or raising ceilings.

Before pushing, run the scoped JavaScript file-size check on only the changed maintained paths.
Use `checkListedFiles(cwd, new Set(changedMaintainedPaths))` from `scripts/check-file-size.mjs`.
Preserve the existing caps: extract cohesive content rather than remove meaning or raise a limit.

Parser-only grammar needs explicit scope. Such lexical forms must remain available to their role-based recognizers without competing as ordinary semantic terms outside an owned frame. Test that ordinary formalization and multilingual round trips retain their meaning; never lower a quality floor to accommodate dictionary leakage.

When routing changes, verify the recorded stages against the actual functions that emit them. Preserve source-call reachability or explicitly support its bindings; do not replace emitter expectations with a guessed upstream function. Keep small offline regression cases in the fast profile and full corpus/native qualification on main.
