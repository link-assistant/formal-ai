# Current-source and CodeQL command repair

Latest fully collected published head `bd6fe77b98aa1cb89c53bfc0667107f6afb13122`
has 27 terminal workflows: 23 success, one skip and three failures. All 178
retained original/compressed command streams verify by SHA256, bytes and lines.
All 110 downloaded original producer payloads match the retained archives.
The failed Rust producer has no SARIF or macro observations. An initial
collector incorrectly required two SARIF reports despite that failure; the
preserved capture was recovered with explicit missing-report status.

[Layered CI 44855584149](https://github.com/link-assistant/formal-ai/actions/runs/44855584149)
fails at full-log lines 8391–8411: a retired committed calculation line census
quotes another function's documentation after helper extraction. The maintained
JavaScript regression fails before repair. Native lookup now reuses the
existing current owned-source walker; its JavaScript counterpart consumes the
existing host's freshly observed modules. Tests cover edits, documentation
changes, moves, deletion and an ignored stale historical symbol. Historical
decoding remains diagnostic and compatible in both languages. No persisted
line census was refreshed or promoted to semantic authority.

[Pipeline 44855584441](https://github.com/link-assistant/formal-ai/actions/runs/44855584441)
fails at full-log lines 43869–43890 with E0599: moving record-field rendering
also moved the formatting trait needed by the parent node-address renderer.
The parent now imports that trait under the renderer's existing native feature
configuration. The hosted reasoning target now includes the complete existing
repository-QA module and its new live-source regression. No native compilation
or execution was performed locally; next-head CI must qualify both repairs.

[Security 44855577716](https://github.com/link-assistant/formal-ai/actions/runs/44855577716)
rejects the query command at full-log lines 1932–1961 because positive and
negative spellings of `expect-discarded-cache` are the same CLI option. The
attempted inverse was not an override. Remove only the rejected extra inverse;
keep available cache retention and the 4096 MiB disk cap for opt-in read-only
investigation. The action still supplies its planning expectation. The exact
query, nine output columns, strict SARIF gate, 4 GiB memory policy, two threads
and four-minute step bounds remain unchanged. Cache reuse and actual query
performance remain unqualified until hosted execution succeeds.

Both before/fixed JavaScript regressions are retained. The final six QA,
three CodeQL option/adapter, 22 strict-security and 18 production-profile cases
pass sequentially. The native-group suite separately passes all 50 cases.
Official pinned translation regeneration and currency verification pass;
Rust 2024 formatting and whitespace checks pass without a build. The first
complete gate pass exposed two planner parity gaps in the draft. Restoring the
historical diagnostic twin and citing the actual live-source twin resolves
them without changing the gate. The final complete 141-entry JavaScript replay
has 88 passed and 53 explicit skips, zero failures, with a
256 MiB JavaScript heap. Initial failed helper and harness attempts remain
separate retained observations and receive no verification credit. The post-archive
abbreviation check also rejected four abbreviated evidence filenames; their
full-word renames and original failed check remain retained before retry.

The earlier `bbda05c0` Pipeline passed 573 Rust tests and eight real registry
CLI cases. That evidence does not qualify later edits. Its completed macro
profile and original standard SARIF reports remain historical observations;
all 631 original unexpanded positions still lack per-child semantic security
qualification. Current source navigation is not complete CST reconstruction.

The latest source-size scan retains 92 warnings and zero ordinary or embedded
data violations under unchanged limits. Publish-token uncertainty, two Chrono
consumers and experimental unsafe-libyaml maintenance remain unresolved.
All eight full template comparisons/practices, original budgets and live
delivery/model identity, actual Pages fallback publication and served identity,
complete aggregate review and complete PR review remain unfinished. Rust
literal review reaches 353/373 identities and 51,512 lines. Strict dated original
whole-file acceptance stays 31/53 plus 22 scoped assertion reviews. Independent
Formal AI reasoning remains 0/20; supplied scripts, patches and conclusions
earn zero independent credit. PR #1581 stays draft and unmerged.

Evidence: `experiments/issue-1580/evidence/ci/native-families-bd6fe77b/`,
`experiments/issue-1580/evidence/ci/codeql-producer-bd6fe77b/` and
`experiments/issue-1580/evidence/checks/bd6-ci-root-cause-repair/`.
