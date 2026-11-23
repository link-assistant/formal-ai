# Current TypeScript worker projection repair

[Layered CI 45103301043](https://github.com/link-assistant/formal-ai/actions/runs/45103301043)
on `17a3431f187c7dd5a12d71ce1aa6bcd966d62994` passed eight JavaScript groups
and failed group 3. The original failed job reports the genuine-projection
assertion for `formal_ai_worker_how_to_guide.ts` at lines 2602–2627. This was
stale derived source, not a TypeScript runtime fallback or a compiler failure.

The maintained TypeScript runtime test fails locally before regeneration. The
canonical ES translator's independent currency check identifies exactly the
guide and source-walker twins. Running `node scripts/translate-es.mjs --write`
regenerates those two files, preserving the accepted capture URL and the shared
walker's stale-capture warning. No generator or assertion is changed.

Afterward, the same runtime test passes, including actual TS-source worker and
planner execution, the missing-projection refusal and the altered-identity
refusal. All 12 translator tests, canonical currency, file-size and readability
gates pass. No native build or execution occurs locally. Next-head hosted
qualification and the two actual native artifact rerun paths remain pending.

The [evidence manifest](../../../experiments/issue-1580/evidence/checks/typescript-projection-currency-20261123/manifest.json)
retains the complete original hosted job, local before/after streams, supplied
orchestration and before/after source identities. Every archive member is
round-trip checked by exact bytes and SHA-256. Supplied work receives zero
independent credit; the PR remains draft and wider issue acceptance is unfinished.
