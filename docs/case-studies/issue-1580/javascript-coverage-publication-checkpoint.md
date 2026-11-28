# Coverage batch publication

The earlier published head `6672bfc98bf02cb8db124421a75b118e30abd349` has 16 successful normal
workflows and 1 skipped workflow(s),
with 9 unfinished. All 402 retained
command streams verify by SHA256, byte and line counts. Completed failures are
absent. Pending or running workflows are not treated as passing qualification.
The coordinating agent deliberately supersedes only this older head, as allowed
by AGENTS.md, to publish the complete locally verified batch in one push.

All 156 maintained observations, 145 default-branch policy checks, 19 production
profile tests, 25 CI-speed tests, 13 headroom checks and both new coverage
regressions pass. The original 141-entry local run and every failure remain
retained; all three failing gates have successful affected retries. Touched
documentation/workflow gates and JavaScript/Rust translation currency pass.
Local source warnings are 76 under unchanged limits. No native build ran.
Final-head hosted execution, the remaining warnings and full issue acceptance
remain unfinished. Supplied work receives zero independent credit.

CI evidence: `experiments/issue-1580/evidence/ci/native-families-6672bfc9/`.
