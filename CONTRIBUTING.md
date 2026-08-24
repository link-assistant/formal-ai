# Contributing to formal-ai

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## How we develop Formal AI: drive the Agent CLI, never defer

**From issue #538 forward, this is the only way we develop the Formal AI
system.** We do not solve a task by editing code and data by hand and we do not
solve it partway and defer the rest to a roadmap. We solve it by **driving Formal
AI through its own [Agent CLI](https://github.com/link-assistant/agent)** (the
in-repo agentic driver in `src/agentic_coding/`, running against the
OpenAI-compatible `formal-ai serve` server), and we get *every* requirement done
in the same pull request.

This section is the development policy and target workflow, not a claim that
the repository is already autonomously self-coded. When an existing tool gap
forces a manual tool extension, record that boundary plainly, retry a smallest
leaf through Formal AI, and measure only genuinely session-authored lines. A
reviewed task decomposition must name its smallest leaves; the acceptance floor
is at least one real Formal-AI/Agent-CLI-authored leaf out of every five (20%),
with a captured session and paired commit trailers. Raise that measured share
over time; never relabel manual work as self-authored.

Concretely, every change must follow these rules:

1. **The tool authors the change, not you.** Drive the Agent CLI + Formal AI to
   produce the change. Where the output lands in the repo (e.g. seed data), a
   test must assert that the committed artifact is **byte-for-byte** what the
   Agent-CLI-driven recipe produces, so the tool — not a hand-edit — is the
   author and cannot silently regress. See the issue #538 case study
   ([`docs/case-studies/issue-538/`](docs/case-studies/issue-538/)) for the
   pattern and the committed `agent-cli-session*.json` sessions.
2. **No pre-emptive deferral, no refusals, no follow-ups.** "This is large or
   hard" is never a reason to ship a slice and route the rest to a roadmap. Find
   the smallest real, tested, reproducible slice of *each* requirement and
   execute it now, in this PR. Read
   [`docs/case-studies/issue-538/refusal-anti-pattern.md`](docs/case-studies/issue-538/refusal-anti-pattern.md)
   before opening a PR — it is the failed reasoning we do not repeat, and we do
   not teach Formal AI to refuse or defer like that.
3. **When the tool can't do it, extend the tool, then retry.** Falling back to a
   manual edit is allowed only after you have proven the Agent CLI / Formal AI
   cannot yet do it — and then you must immediately improve the Agent CLI /
   Formal AI so it *can* in general, and re-run through the tool.
4. **Prove generality with different words each time.** Use a *different* natural
   language request for each case so a passing run proves the solution is truly
   general, not hardcoded to one phrasing (issue #538 drives tomato and potato
   with two differently-worded requests).
5. **Report faithfully.** State what is done and verified plainly. Honesty means
   reporting results accurately; it is never a license to stop early or to dress
   a refusal as an "honest scope" section.
6. **Real Agent-CLI E2E tests in CI, plus a per-requirement test and a
   whole-task test.** Every change that touches the agentic path must add (or
   update) a real end-to-end test that boots `formal-ai serve` and drives it
   with the actual `@link-assistant/agent` CLI over the OpenAI-compatible
   endpoint — no mocks or in-process shortcuts. Keep the round-trip green in CI
   (see `test-agent-cli-e2e` in `.github/workflows/release.yml` and the driver
   script `experiments/agent_cli_e2e/run_agent_cli.sh`). In addition, ship one
   unit/integration test **per requirement in the issue** and one test that
   exercises the **whole task** end-to-end so a regression on any single
   requirement — or on the composition of all of them — breaks the build.
7. **Hardcoded cases only in tests; production code stays general.** A test may
   hardcode inputs and expected outputs (that is what a test is *for*), but the
   engine, planner, seed loader, and Agent-CLI-driven recipes never branch on a
   specific concept, phrase, or URL. If the only way to make a green case pass
   is a match-on-literal in `src/`, extend the general routing table (`concept
   registry`, `capability classifier`, `plan_chat_step`) so future concepts get
   the same treatment for free.
8. **Real logs in the case study, not synthesized ones.** When a case study
   claims the Agent CLI drove the change, it must ship the real captured log of
   the round-trip (see `docs/case-studies/issue-538/agent-cli-e2e-run.log`), and
   the committed session JSON must be reproducible byte-for-byte by `cargo test`.
9. **Commit in small, atomic steps.** Every commit should be independently
   useful and reviewable — one logical change per commit, buildable in
   isolation. Interrupted work stays preserved in the PR because each commit
   already stands on its own; do not batch a day of unrelated edits into one
   commit.
10. **Forward history only.** Never rewrite a PR branch or any shared branch by
    force-pushing or force-updating its ref. Preserve existing commits and
    resolve conflicts by adding new commits (merge, revert, or corrective
    commits as appropriate). Branch updates for normal development must always
    move a ref forward in history.

### Testing external agentic CLIs
