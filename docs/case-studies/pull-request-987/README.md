# Pull request #987 — macOS CI parity

Pull request: <https://github.com/link-assistant/formal-ai/pull/987>

Issue: <https://github.com/link-assistant/formal-ai/issues/961>

The technical history, root-cause analysis, requirements, research, and
tests-first proof live in the [issue #961 case study](../issue-961/README.md).
This directory records what happened specifically in the pull request.

## Initial state and discussion audit

PR #987 was opened as a draft on 2026-08-10 from
`issue-961-dc9b3c144e21` into `main`, with a WIP title and generated placeholder
body. Before implementation it contained only the prepared `.gitkeep` commit.

All three GitHub review surfaces were queried separately and preserved under
`raw-data/`:

| Surface | Initial result |
| --- | --- |
| Conversation comments (`issues/987/comments`) | empty |
| Inline review comments (`pulls/987/comments`) | empty |
| Reviews (`pulls/987/reviews`) | empty |

`raw-data/pull-request-initial.json` preserves the initial title, body, branch
SHAs, and draft state.

## Decisions made in this pull request

1. Keep all four issue findings in one PR, as explicitly required, but give
   each a separate regression test and root-cause account.
2. Reproduce canonicalization with a symlink alias so R961-2 is behavioral on
   Linux, not merely a macOS path-string special case.
3. Centralize the two `script(1)` dialects in test infrastructure instead of
   scattering `cfg` blocks across call sites or adding a new PTY dependency.
4. Pin CI to the existing `macos-15-intel` repository convention and keep the
   Linux timeout unchanged.
5. Treat the changelog fragment as the patch-release trigger; do not manually
   bump `Cargo.toml`, because the release workflow consumes fragments after
   merge.
6. Preserve honest self-authorship attribution: only the two artifacts emitted
   by live Formal AI / Agent CLI sessions receive authorship commit trailers.

## Review checklist

- The PR uses `Fixes https://github.com/link-assistant/formal-ai/issues/961`.
- No product behavior was weakened to make a test pass.
- Both original PTY callers remain covered and no functionality was removed.
- The macOS CI leg runs the same full suite and doc tests as Linux.
- The task decomposition, generated artifacts, and captured logs are committed.
- The issue and PR case studies are separate and cross-linked.
- No UI changed, so before/after screenshots are not applicable.
- No external source material was copied; manual and runner documentation are
  referenced by link only.

## Verification and CI

Local focused and repository-wide results are recorded in the issue case study.
The first fresh CI run for implementation SHA `324fc3fe` created the required
`Test (macos-15-intel)` job, but its pre-test `free-runner-disk.sh` step failed:

```text
df: unrecognized option `--output=avail'
Process completed with exit code 64.
```

The complete 1,100-line job log is preserved locally as
`ci-logs/test-macos-15-intel-31387639397.log`; the failure is at lines
1,064–1,066. Two tests reproduced the BSD parsing failure before the helper was
changed to POSIX `df -Pk`. Fresh final-head results are added here once both
platform legs complete.
