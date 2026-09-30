# PR #1188 CI feedback and draft repairs

Evidence checked on 2026-09-30: the latest remote checks still cover
`e4193a615c2a0788a2f7e1067a1a55d4c5d7912d`, before subsequent local draft commits.
No local build, test, package installation, project generator, CI rerun, or push was
performed for this repair pass. Standalone rustfmt formatted source files only.
A committed repair is a draft until CI validates the eventual pushed tree.

## Failures with concrete fixes drafted

| Existing failure | Evidence | Repair |
|---|---|---|
| Five CLI archive jobs cannot find `target/<triple>/release/formal-ai` | Desktop Release [run 36707093631](https://github.com/link-assistant/formal-ai/actions/runs/36707093631) shows successful builds followed by `cp: cannot stat` on every target | CLI build explicitly passes `--target-dir target`, matching archive packaging; no platform path guessing |
| Canonical notation rejects code-review header | Main pipeline [run 36707093714](https://github.com/link-assistant/formal-ai/actions/runs/36707093714), data_files failure at code-review-rules.lino:5 | Preserve the interrupted fix rewriting colon-bearing header prose; source and embedded mirror match |
| Variance cache cannot reconstruct raw JSON | Same run, `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json` | Preserve missing `audio`, `sourceUrls`, and `word` fields from checked-in raw JSON |
| Duplicate statistics operation definition | Same run, `meaning_definitions_are_unique` | Define the statistics operation under the existing statistics concept instead of repeating the expectation definition |
| Duplicate `foot`, `word_problem_change`, `word_problem_total` slugs | Same run, `meaning_slugs_are_globally_unique` | Keep the richer measurement foot record; merge response-template metadata into the existing word-problem marker records |
| Forbidden legacy `meaning` field in meanings seed | Same run, `meaning_seed_uses_id_fact_format` | Rename to `explanation` and update code-explanation reader |
| Role-specific code-task cues are never found | Source inspection of format conversion, shell composition and other code handlers; their CI failures occur in the same run | Read `role` beneath the `intent` wrapper, where the seed stores it, in all nine code-task handlers |
| Code explanation reads only the first construct | `meanings-code-structure-explanations.lino` has multiple construct children under one structure root | Iterate every construct child instead of selecting `.first()` |
| Missing Wikidata records | Same run, semantic_grounding failures | Fetch actual official API records for Q11663, Q218593, Q23925413, Q253276, Q48013, Q482798; retain raw JSON and human-readable notation pairs |
| Attestation count is stale after CLI leg added | Coverage [run 36707093380](https://github.com/link-assistant/formal-ai/actions/runs/36707093380), issue_717 expected 2 but found 3 | Include the third CLI provenance attestation in the workflow contract |
| Desktop budget tests consume CLI matrix too | Same run, issue_730 and issue_896 fail on unquoted CLI labels | Scope the assertion to the desktop build job before the CLI job; preserve all six desktop cap checks |
| Coverage cap assertion predates issue #1149 | Same run, workflow_coverage expects 105; workflow documents measured 135-minute cap and 5400-second budget | Pin the current measured 135-minute contract |
| rustfmt differences | Main run, Lint and Format Check lists 13 files | Format the listed files except centrally owned lib.rs and solver_dispatch.rs; also format newly edited handlers and continuation source |

Wikidata snapshots came from `https://www.wikidata.org/w/api.php` with
`action=wbgetentities`, `props=labels|descriptions|aliases`, and the five supported
languages. Python's local TLS store was missing issuer certificates; OS curl's
validated TLS store fetched the records instead. No certificate check was disabled.

## Exact integration residuals

- **Central formatting:** apply rustfmt to `rust/src/lib.rs` and
  `rust/src/solver_dispatch.rs` after central wiring. This pass deliberately did
  not edit either file.
- **Generated AST census:** coverage reports missing census for `src/cli_env.rs`
  and stale census documents. Regenerate from the final source tree in CI;
  the draft has no locally generated census or fabricated fidelity markers.
- **TS regeneration:** Layered CI [run 36707093399](https://github.com/link-assistant/formal-ai/actions/runs/36707093399)
  changes `ts/worker/formal_ai_worker_10.ts` and `_20.ts`. Reconcile with the UI
  agent's current JS/TS work, then let CI compare the actual translator output.
- **Role registry:** `reference_closure::role_registry_is_in_lockstep_with_usage`
  requires registry reconciliation after every newly added role. Regenerate or
  update the role manifest from final source, preserving the closure gate.
- **Handler/core metadata:** issue_918 and migration/core-ledger failures require
  central registration and final source-size measurements. Add entries for the
  actual registered handlers rather than relaxing the ledger or floor.
- **Remaining behavior:** coverage also reports software-project routing,
  phrase-vocabulary expectations, word-problem arithmetic, inverse unit
  conversion, standard deviation, and direct code-task behavior. The role-loader
  and earlier fact-routing repairs address concrete roots; the full set remains
  unverified and must not be reported green without new CI evidence.
- **Fact routing:** prior committed repair `fbe37c88e` normalizes both fact-match
  operands and prevents seeded fact questions being taken by web-search routing.
  It addresses the ten specification fact-routing failures and related concrete
  thinking failures; final CI validation is outstanding.
- **Documentation issue state:** docs_issue_citations fails against the committed
  issue snapshot. Refresh actual issue status/citations after the final PR table,
  without changing the gate to hide stale references.
- **Self-hosting evidence:** [run 36707093387](https://github.com/link-assistant/formal-ai/actions/runs/36707093387)
  states that a zero-hour-old PR lacks a genuine `Formal-AI-Model: formal-ai`
  authored contribution. Produce a genuine contribution through the supported
  authoring workflow, or use its existing 24-hour relaxation. Never add an
  attribution trailer to work that Formal AI did not author.
- **Proactive failure-report E2E (draft repaired):** [run 36707093271](https://github.com/link-assistant/formal-ai/actions/runs/36707093271)
  contains the failed-command result and opt-in report invitation. The actual
  failure was strict stderr classification rejecting the fixture's structured
  expected exit-127 tool event. A narrow filter now preserves the raw log,
  removes only that exact deliberately missing command event from the policy
  input, and passes every other line to the unchanged strict classifier.
  Trace assertions remain in place. No helper or E2E execution was run locally.
- **Binary-tree agent ladder:** latest check is failed. Retrieval of its old log
  was pending during this pass; no failure cause or repair is claimed.

## Validation record

Source formatting completed. Seed mirrors were copied byte-for-byte from their
edited source documents. Existing failure logs were read; no test definitions were
executed. All untouched failures remain explicit above. The final push belongs to
the coordinating agent after the drafting fleet finishes.

## Follow-up source repair

The prior fact-normalization fix borrowed string slices from a temporary normalized
String. Bind that String before collecting slices so the fix is valid Rust; no
compiler execution was used to identify or check this lifetime correction.
