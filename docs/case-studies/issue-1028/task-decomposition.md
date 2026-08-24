# Issue #1028 — binary recursive coding decomposition

The working rule for this task is binary recursion: every non-leaf is split into exactly two children. A complete layer therefore has `2^n` leaves. This document records the reviewed 32-leaf (`2^5`) layer that drives the Agent-CLI ladder.

The leaves are deliberately phrased as independently checkable work. A leaf may inspect, implement, test, document, or verify one concrete property, but it must not be a vague mental action such as “understand the codebase”.

| Path | Leaf | Completion criterion |
|---|---|---|
| 1.1.1.1.1 | L01 Read #1028 and extract the four concrete requirements into `.agent-ladder/L01-requirements.md`. | File exists and names all four issue requirements. |
| 1.1.1.1.2 | L02 Inspect `scripts/apt-install-with-retry.sh` and record its current retry inputs and defaults. | Evidence file records the actual variable names/defaults. |
| 1.1.1.2.1 | L03 Inspect `tests/unit/ci-cd/issue_1021.rs` and identify the reusable retry harness. | Evidence names the helper and test entry points. |
| 1.1.1.2.2 | L04 Inspect the Agentic CLI workflow Xvfb step and record its retry budget variables. | Evidence matches the workflow values. |
| 1.1.1.3.1 | L05 Calculate the old worst-case 3×90s + 2×5s schedule. | Evidence states 280 seconds before the enclosing 300s cap. |
| 1.1.1.3.2 | L06 Calculate a 1:2:4 geometric allocation for the remaining budget. | Evidence shows progressively larger deadlines and their sum. |
| 1.1.1.4.1 | L07 Verify that callers without `TEST_BUDGET_SECONDS` retain fixed per-attempt deadlines. | Evidence points to the no-budget branch and its test. |
| 1.1.1.4.2 | L08 Specify the invalid-budget guard: a retry schedule must never exceed its enclosing step budget. | Evidence includes the arithmetic rule and expected failure mode. |
| 1.1.2.1.1 | L09 Add a focused test for geometric deadline allocation. | Targeted issue-1028 test passes. |
| 1.1.2.1.2 | L10 Add a slow-mirror stand-in that succeeds only when the later attempt gets more time. | Regression fixture is deterministic and network-free. |
| 1.1.2.2.1 | L11 Prove the old flat 3-second-per-attempt shape fails the slow-mirror fixture. | Test demonstrates the pre-fix failure shape. |
| 1.1.2.2.2 | L12 Prove the budget-aware geometric schedule succeeds on the same fixture. | Test demonstrates recovery on the same input. |
| 1.1.2.3.1 | L13 Test that retry delays are reserved before attempt deadlines are allocated. | Deadline sum plus delays never exceeds the budget. |
| 1.1.2.3.2 | L14 Test that the final attempt receives the largest share. | Test asserts strictly increasing attempt deadlines. |
| 1.1.2.4.1 | L15 Test that the first attempt receives the smallest non-zero share. | Test asserts a positive first deadline smaller than the final deadline. |
| 1.1.2.4.2 | L16 Test that the per-attempt diagnostic reports the deadline actually used for that attempt. | Failure output names the computed deadline. |
| 1.1.3.1.1 | L17 Test that a non-timeout apt failure preserves apt’s own exit status after all retries. | Test returns the stand-in’s non-124 status. |
| 1.1.3.1.2 | L18 Test that timeout status 124 is identified as the deadline path. | Test distinguishes timeout from apt failure. |
| 1.1.3.2.1 | L19 Update the wrapper comments so the geometric schedule is explained without issue-specific magic numbers. | Comments describe the general algorithm. |
| 1.1.3.2.2 | L20 Update the CI workflow comments/env contract so the enclosing budget remains explicit. | Workflow and wrapper agree on the same inputs. |
| 1.1.3.3.1 | L21 Add the issue-1028 case-study with real requirements and test evidence paths. | Case-study file exists and references concrete artifacts. |
| 1.1.3.3.2 | L22 Add the changelog fragment for the retry scheduling fix. | Changelog fragment exists with a `Fixed` entry. |
| 1.1.3.4.1 | L23 Run shell syntax validation on the changed retry wrapper. | `bash -n` succeeds. |
| 1.1.3.4.2 | L24 Run the focused issue-1028 unit test suite. | All issue-1028 tests pass. |
| 1.1.4.1.1 | L25 Review the diff for unrelated production changes. | Evidence lists only issue-1028-relevant files. |
| 1.1.4.1.2 | L26 Produce a PR summary that names the generalization rather than only the incident. | Summary explains budget-aware retry as reusable behavior. |
| 1.1.4.2.1 | L27 Add requirement-traceability evidence for the four issue requirements. | Each requirement has delivery and test evidence. |
| 1.1.4.2.2 | L28 Validate this decomposition has exactly 32 leaf rows and no duplicate leaf IDs. | Deterministic checker reports 32 unique leaves. |
| 1.1.4.3.1 | L29 Validate the committed task decomposition artifact round-trips through the repository's Links Notation contract. | Round-trip comparison is byte-stable. |
| 1.1.4.3.2 | L30 Inspect any failed Agent-CLI leaf logs and classify the failure from observable evidence. | Failure report names the layer: routing, tool, protocol, test, or environment. |
| 1.1.4.4.1 | L31 For any capability gap exposed by the ladder, generalize the fix in Formal AI/Agent tooling rather than adding a prompt-specific branch. | New regression test covers a different phrasing of the same capability. |
| 1.1.4.4.2 | L32 Produce the final self-coding evidence bundle: decomposition, leaf outcomes, test results, and session IDs. | Bundle is reproducible from the temporary-copy run. |

## Recursive shape

```text
Level 0: solve #1028
└─ 2 children
   ├─ 1: understand/specify + diagnose current behavior
   └─ 2: implement + verify + evidence

Each branch is split again until Level 5.
Level 5: 32 independently checkable leaves (L01–L32 above).
```

## Agent-CLI execution rule

Each leaf is executed in a **fresh temporary copy** of this repository, using the real `@link-assistant/agent` CLI against `formal-ai serve --agent-mode`. The prompt must contain a distinct wording from neighboring leaves. The server may use web tools for factual verification; the leaf still has to leave observable evidence in the temporary copy.

The harness is `experiments/issue_1028_agent_cli_ladder/run.sh` and is intended to run with `--attach-logs --verbose` semantics inherited from the repository's Agent-CLI policy.
