title:	E52: Multi-CLI agentic end-to-end matrix in CI (codex, opencode, gemini, qwen, claude, grok, aider)
state:	OPEN
author:	konard (Konstantin Diachenko)
labels:	
comments:	0
assignees:	
projects:	
milestone:	
issue-type:	
parent:	link-assistant/formal-ai#651
sub-issues:	
sub-issues-completed:	
blocked-by:	
blocking:	
number:	671
--
Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E52 in `proposed-issues.md` there; added by PR #652).


**Problem**

`docs/testing/agentic-cli-tools.md` (from issues #625/#628) prescribes a CI
sequence for verifying real third-party CLI clients against `formal-ai serve
--agent-mode` — but it shipped as prose only: `.github/workflows/release.yml`
contains a single `test-agent-cli-e2e` job for our own Agent CLI and no
codex/opencode/gemini/qwen job at all. The cost is already visible: PR #648
closed #647 with claude "intentionally not run" and grok/aider "inferred from
the shared adapters", and hands-on testing immediately produced issue #650
with four defects. Regressions of the #620/#624/#626/#627 fixes have no
guard.

**Approach**

1. Turn the guide's "CI Shape" section into an actual workflow job matrix:
   one leg per CLI (codex, opencode, gemini, qwen, claude, grok, aider, plus
   our Agent CLI as the reference leg), each running the guide's smoke
   sequence against a local `formal-ai serve --agent-mode` with the recorded
   proxy from PR #631 so no leg needs vendor credentials.
2. Encode known upstream constraints as *expected-behavior assertions*, not
   skips: gemini headless `-p` advertises no functionDeclarations (chat-only
   — the #620 comment that was never filed as an issue), codex/gemini/qwen
   lack a headless approval handshake (#511/PR #512). When an upstream
   release lifts a constraint, the assertion fails loudly and we upgrade.
3. Cover the #650 defect surface explicitly: `/responses` instructions
   handling, empty-message interactive behavior per CLI, conversation
   summarization requests, `--globally` alias — each as a regression case
   that fails before the #650 fix and passes after.
4. Keep the matrix fast: legs run in parallel, pinned CLI versions installed
   from a lockfile, recorded transcripts committed so replays are offline
   and deterministic.

**Existing components**

- `docs/testing/agentic-cli-tools.md` — the prescribed sequence.
- The recording proxy from PR #631; `test-agent-cli-e2e` job as template.
- `docs/case-studies/issue-647/` transcripts as seed recordings.

**Acceptance criteria**

- `release.yml` (or a dedicated workflow) runs the full matrix on every PR
  touching server/protocol code; all legs green on the branch.
- Each of the four #650 defects has a failing-then-passing regression case
  in the matrix.
- Upstream-constraint assertions documented inline with links to the
  upstream issues.
- claude, grok, and aider — the never-actually-run integrations from
  PR #648 — each have at least one recorded, replayable session.



