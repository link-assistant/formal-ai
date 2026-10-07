# Generated tree

`ts/` is regenerated from `js/` by `formal-ai translate --from js --to ts --write`
(plan 16 L2). Hand edits are forbidden here; fix the source or the translator.

`ts/agentic/**/*.mts` mirrors the agentic ES modules `js/agentic/**/*.mjs`
(issues #1180 R11, #1184 R9, #1185 R8). That set is rendered by
`node scripts/translate-es.mjs --write` alone; the native leg owns the
`.js` → `.ts` file set.
