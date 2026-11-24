# Historical recipe fixture source-cache isolation

The complete Layered CI run [44992280972](https://github.com/link-assistant/formal-ai/actions/runs/44992280972) at `754fd314fd3c08152b19f9f5c51a87ea23fa6940` failed only the historical Agent full-API recipe case. Log lines 18123 and 19685 identify the failing case and final prepared-recipe refusal. They do not retain the workflow write arguments, so they cannot establish a specific metadata transition as the hosted root cause.

The fixture supplied an offline client command sandbox but its server host resolved versions from the real process environment. It now uses the existing `offlineSourceCache()` helper to own an empty offline cache before installing the host. A maintained fixture assertion verifies that server generation observes baseline pins without network captures. Removing that installation and supplying `FORMAL_AI_OFFLINE=0` makes the assertion fail; restoring it passes. The original uncontrolled negative attempt aborted without assertion detail and is retained as an unsuccessful experiment.

The full maintained file passed six tests in 225,208 ms, including two-pass Claude, Codex and Agent replays. The final cache assertion was then strengthened and checked separately. The earlier unisolated full replay terminated in V8 with `Check failed: !is_null()`; that native failure has not been attributed to the hosted assertion, memory pressure or a timeout. No native Rust compilation was performed locally.

Failure diagnostics now retain call IDs, file paths and source hashes. Full tool arguments and observations can be captured by explicitly setting `FORMAL_AI_RECIPE_FIXTURE_TRACE` to a JSONL path; tracing is off by default.

`experiments/issue-1580/probe-recipe-version-epoch.mjs` reproduces a separate production limitation without network or compilation: the same executable Scala workflow rendered once from live pins and again from cached pins has different provenance comments. Reconstructing progress against the second source invalidates the first exact workflow write. This is a finite mechanism probe, **not a regression test proving a production fix**. Source ownership checks remain exact; the general metadata-epoch limitation is unfinished.

All intermediate outputs, trace archives and source identities are retained under `experiments/issue-1580/evidence/checks/recipe-source-cache-isolation/`. These supplied scripts and edits earn zero independent Formal AI authorship credit. Hosted replay of the changed fixture is still required. Passing this fixture does not complete issue #1580.
