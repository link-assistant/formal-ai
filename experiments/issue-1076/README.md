# Experiments for issue #1076

Reproductions backing the three upstream reports in
`dev/log/issues/1076/pulls/1077/upstream-reports/`. Each script is
self-contained and needs only bash.

| script | what it shows |
|---|---|
| `repro-budget-poll-interval.sh` | The rust template's budget wrapper stops enforcing entirely when `BUDGET_POLL_SECONDS` is fractional: `elapsed=$(( elapsed + POLL_SECONDS ))` is an arithmetic error, `elapsed` stays 0, and a `sleep 10` under a 2s budget exits 0 after the full 10s. |
| `cases2.sh` | Runs all three templates' wrappers against a child that ignores SIGTERM. The js wrapper returns after 3s and leaves the child running; rust and python wait out the grace period and escalate to SIGKILL. |

The vendored copies (`template-run-with-budget-warning.sh` = rust,
`budget-js.sh`, `budget-python.sh`) are the upstream files as of 2026-09-05, so
the reproductions stay runnable after upstream fixes land. Refresh with:

```sh
gh api repos/link-foundation/<lang>-ai-driven-development-pipeline-template/contents/scripts/run-with-budget-warning.sh \
  -H "Accept: application/vnd.github.raw"
```

`cases2.sh` expects those copies in its working directory:

```sh
cd experiments/issue-1076 && cp template-run-with-budget-warning.sh run-with-budget-warning.sh && bash cases2.sh
```

Expected output (survivor counts are the finding):

```
budget-js.sh                 poll=1    exit=124  wall= 3s survivors=1
budget-js.sh                 poll=0.5  exit=143  wall= 2s survivors=1
run-with-budget-warning.sh   poll=1    exit=124  wall= 5s survivors=0
budget-python.sh             poll=1    exit=124  wall= 6s survivors=0
```

This repository's own `scripts/run-with-budget-warning.sh` is not affected by
either defect; `tests/unit/ci-cd/issue_1017.rs` pins the behaviour.
