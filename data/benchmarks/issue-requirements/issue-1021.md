Umbrella for one deliverable: **Formal AI handles the whole coding range — from the simplest hello-world pull request to opening pull requests against itself — and every step is covered by tests.**

Deliver **all sub-issues in a single pull request** and close them together. They are one capability, not a backlog: a change the harness can author but cannot land is not a capability, and a gate it satisfies by accident is not a test.

## Where we actually are

**The upstream side is done.** The hive-mind series that drove this — [#2158](https://github.com/link-assistant/hive-mind/issues/2158), [#2154](https://github.com/link-assistant/hive-mind/issues/2154), [#2146](https://github.com/link-assistant/hive-mind/issues/2146), [#2119](https://github.com/link-assistant/hive-mind/issues/2119) — is entirely closed, and its standing instruction is the frame for this issue:

> All that prevent Formal AI from coding via Hive Mind — is an issue, and must be fixed.

with the method stated in #2119: **solve by generalization, not specialization, improving self learning and self healing meta algorithm.** A per-prompt fix that makes one task pass is a regression against that instruction, not progress.

**The mechanism exists.** Hive Mind can open pull requests (`solve.auto-pr.lib.mjs`). Formal AI can modify existing files, not only create them (`Capability::Edit`, `Capability::MultiEdit` in `src/agentic_coding/capability_router.rs`). The full circle `solve <issue> --tool agent --model formal-ai` runs in CI as the issue #921 gate.

**The result is still zero.** `data/meta/self-hosting-ledger.lino` records `0.00% self-authored` — no commit has ever carried the paired session and evidence attribution. #924 (E77) asked for exactly this loop, "landing as a normal reviewed pull request", and was closed without it ever landing one.

## The gap, precisely

The #921 gate is deliberately read-only: `experiments/issue_921_hive_mind_full_circle/github-readonly-prepare-wrapper.sh` refuses `pr create`, `pr merge` and `issue create`, and `solve` runs against a throwaway local repository with no push. CI proves *a file change was reached*; it has never proved *a pull request was accepted*.

Between those two lies every gate a pull request must pass — 38 of them in `data/meta/ci-gates/`, including **R379** (no natural-language string outside seed and allowlist), a **changelog fragment** for any code change, a **PR body linking its issue** with a closing keyword, file-size bands, clippy, rustfmt, and en/ru/hi/zh/es parity.

Formal AI produces none of those three process artifacts. Remove the read-only wrapper today and it opens a pull request that fails on the changelog and PR-link gates **before its code is ever judged**.

This is not hypothetical. Pull request #1019 — a four-line CI fix, with a human reading every log — took **six iterations** to go green, each uncovering the next gate: `NaN` timeout → reverting over-broad edits → an OpenCode schema rejection → clippy doc backticks. Two of those failures were created by the fix itself. An unattended harness has nobody reading those logs.

## The range to cover

The simplest end is already tracked as failing behaviour, and these are the tests: **#868** (`ls`), **#866**/**#867** (`Execute ls command`), **#865** (`Hello` + `List me files here`), **#863** (example of copy stdin to stdout in Rust), **#862** (execute a Rosetta Code task in Rust), **#723** (write PHP Laravel code), **#824** (a filesystem move it refuses but should perform).

The far end is a pull request against this repository. Everything between is the same meta algorithm, which is why they belong in one delivery rather than one fix each.

## What is collected here

**Sub-issues — the behaviour range, each already filed and each a test case:**
#868 `ls` · #866 / #867 `Execute ls command` · #865 `Hello` + `List me files here` · #863 copy stdin to stdout in Rust · #862 execute a Rosetta Code task in Rust · #723 write PHP Laravel code · #824 a filesystem move it refuses but should perform.

**Blocked by — the capabilities the range needs.** These are E-series issues already parented to the planning umbrella #651, so GitHub allows only a dependency link, not a second parent:
- **#943 (E91)** — guard against harness-created issues. A precondition for any GitHub write authority.
- **#946 (E94)** — versioned recoverable memory: roll back to the last stable state when a self-authored version fails to compile.
- **#947 (E95)** — bounded autonomy: a stuck-recovery limit, so an unattended run ends instead of looping.
- **#944 (E92)** — mutating-action ladder rungs, since self-coding is by definition mutation of existing files, and the write-effect ladder currently stops at creating one.

**Carried forward from a closed issue:** #924 (E77) asked for exactly this loop — one real repository change per release, "landing as a normal reviewed pull request" — and was closed without one ever landing. The ledger still reads 0.00%. That requirement lives here now rather than in a new issue.

## Safety, which is not optional here

The harness has already created five unwanted issues (#784, #786, #789, #790, #791) against an explicit instruction not to. **#943** is that guard, and it is a precondition for granting any GitHub write authority — not a nice-to-have alongside it.

## Test coverage is the deliverable, not the proof of it

Every step gets an automated test, and the honest kind:

- each prompt above pinned as a routing/behaviour test, **with held-out paraphrases** so a passing run means generalization and not a memorised seed;
- the process artifacts (fragment, linked body, R379-clean generated code) tested by driving the generator, not by asserting on a hand-written fixture;
- the write path tested in both states — refused by default, permitted only under an explicit opt-in, with `issue create` refused in **both**;
- the closed circle pinned as a replayable session, the way `cargo test self_coding_session_replays` already pins the inner loop.

## Definition of done

A pull request opened by Formal AI against `link-assistant/formal-ai`, from a real `solve` run, **all checks green without a human editing the branch**. Merging stays a human decision; being mergeable on its own merit is the bar.

And the bar is not to be met by lowering it: if closing the circle requires relaxing R379, the changelog rule or the PR-link rule, that is a finding to report, not a change to make.

## Standing clauses

- Collect data to `docs/case-studies/issue-1021` — timeline, full requirement list, per-requirement solution plans, existing-library survey, online research.
- Add debug/verbose output wherever a root cause cannot be determined from existing logs.
- File upstream issues with reproductions for defects that are not ours.
- Every requirement gets a `docs/requirements-traceability.md` row: delivered version/date/commit, the automated test reference, and a manual confirmation — with honest "not yet confirmed" where no record exists.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

