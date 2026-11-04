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
