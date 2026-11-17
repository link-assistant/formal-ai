# Issue #1159: Workspace prerequisite recovery

Run failures are classified with the shared prerequisite classifier. Missing executables with FORMAL_AI_INSTALL_GRANT=workspace fetch their pinned publisher. A public process-free parser accepts only typed setup notation with a postcondition. Harness tools execute scoped setup, postcondition and one original-command retry, with explicit PATH and transcript binding; absolute/parent write scopes and shell/escalation setup programs are refused.

## Requirement status

Partial. Typed publisher recovery is drafted and bounded. Ordinary publisher prose cannot become an executable recipe. Broader source discovery, per-process/network grants and typed recipes for publisher pages that lack them remain open. No local process installation, build or test was executed.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.

## Issue #1162 follow-up (2026-11-16)

**Log excerpt.** The `scalac` failure is from the 2026-09-15 Scala run (backend 0.351.0), as quoted in the issue:

```
Step `scalac Main.scala` for `Main.scala` failed. No exit code was reported…
/bin/sh: 1: scalac: not found
```

The 2026-09-27 Scala/agent log (`docs/case-studies/issue-1162/raw-data/hive-mind-logs/scala-agent-2026-09-27.log.gz`) stops earlier. `gh` was unauthenticated inside the session, and Formal AI ended `planned_not_executed` (line 4211) before any compile step. That defect is #1155.

**Root cause.** The executor finalized on the first failed step. Prerequisite recovery now runs from the executor, but the answers did not say what happened. Without a grant, the stop did not name the procedure a grant would run. With a grant, a successful recovery was not reported at all.

**Fix.** The grant-required answer names the missing program, `FORMAL_AI_INSTALL_GRANT=workspace`, the pinned publisher procedure, the re-probe and the single retry of the original command. A successful transaction is reported as "Recovered the missing prerequisite `<program>` under FORMAL_AI_INSTALL_GRANT=workspace: the pinned `<program>` publisher setup procedure (<url>) ran, `<program>` was re-probed, and `<command>` was retried once and succeeded." A failed transaction is never reported as a recovery. Rust: `rust/src/agentic_coding/prerequisite_recovery.rs`. JS: `js/agentic/prerequisite_recovery.mjs`. Wording: `data/meta/work-item-steps.lino`.

**Tests.** `rust/tests/unit/issue_1159_toolchain_bootstrap.rs` replays the `scalac` 127 failure: the no-grant stop, then the installer write, the publisher, re-probe and retry transaction, and the recovered report. `rust/tests/web/observed-prerequisite-recovery.test.mjs` covers the report and "a failed transaction is not reported as a recovery". `experiments/issue-1159/scala-recovery-report.mjs` reproduces both answers.

**Same toolchain in the workflow (R1159-4).** For Scala this holds and is pinned. The workflow installs `scalac` at the version resolved from `https://api.github.com/repos/scala/scala3/releases/latest`, the release feed the `scalac` publisher record names, and it runs the same verified command (`the_scala_workflow_installs_scalac_from_the_release_feed_of_the_recovery_publisher`). For Kotlin it is only partial. The publisher record names the kotlinlang.org documentation page, while the workflow pin comes from the JetBrains/kotlin release feed. Pointing the record at that feed directly fails, because the release carries more than one `.zip` asset and the installer refuses ambiguous archives.

**Not implemented.** Preferring `scala-cli` / `kotlinc -script` as a recorded fallback procedure (R1159-3). The bounded installer only unpacks `.zip` release assets, and `scala-cli` ships single gzipped binaries. That needs a new, security-reviewed archive path.

The manual end-to-end check on a released version is still **blocked**: it needs a release that contains this change and Hive Mind credentials, and neither is available to the solver. The issue therefore stays open after pull request https://github.com/link-assistant/formal-ai/pull/1579.
