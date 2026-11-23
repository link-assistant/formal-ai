# Terminal f7 CI observations

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
Source f7a71cb86a16e78da0e9ca055c13710d62277739 finishes with 31 successful
workflows, one explicit skip and one failed Layered workflow. Pipeline
[45151783306](https://github.com/link-assistant/formal-ai/actions/runs/45151783306)
succeeds, including its producer, registry checks, Clippy profile and all ten
native consumer groups. The complete 78,840-line Pipeline log and 23,495-line
Layered log are retained with terminal jobs, run metadata and check rollup.

Layered [45151782730](https://github.com/link-assistant/formal-ai/actions/runs/45151782730)
fails only while provisioning JavaScript groups 7 and 8. Its actionable errors
and unchanged-budget repair are documented in [the APT case](search-adapter-apt-recovery.md).
Repair 0096a8168cb9f611ac55a611cce602884c888607 has six failures with unchanged
regressions against the prior scripts and zero afterward; 54 focused tests,
eight affected gates and three separate shell syntax checks pass locally.
Its own hosted qualification remains pending.

The producer-rerun observer finishes without requesting a rerun because the
baseline cohort has a failure. The complete-workflow producer rerun criterion
remains unfinished. Previous failed-consumer artifact reuse keeps its separate
historical scope. No native artifact was downloaded locally.

Pipeline log line 67034 explicitly defers style lint debt behind
FORMAL_AI_STRICT_STYLE_LINTS=1. This successful profile does not establish
strict-style acceptance or completion of the full original workflow families,
main-only execution, template practices or externally blocked criteria.

The terminal-f7-ci-20261123 evidence receipt has 278 members. Its gzip archive
SHA-256 is f28fb07ff9d8e956d0192e42bf558e84b46035e519d11a40bbeb8351fbc83d14.
Every archived member was read back and checked against its byte count and
digest. Supplied work receives zero independent authorship credit.
