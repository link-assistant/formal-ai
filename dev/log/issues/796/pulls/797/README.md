# Issue #796 / PR #797 — collected evidence

| Path | Contents |
| --- | --- |
| `timeline-and-root-cause.md` | Reconstructed timeline, root cause per failure, requirement checklist, solution plans |
| `template-gap-analysis.md` | Comparison against the three link-foundation pipeline templates and CI-CD-BEST-PRACTICES.md |
| `online-research.md` | Cited external research: npm deprecation handling, glob deprecation, git trailer semantics |
| `ci-logs/run-*.log` | Full `gh run view --log` output for all four runs named in the issue |
| `raw/run-*.json` | Run/job/step metadata for the same runs |
| `raw/issue-796.json`, `raw/pull-797.json` | Issue and PR payloads |
| `raw/main-runs.json` | Recent default-branch run history |
| `raw/commit-59650f2b-message.txt` | The commit message whose trailer layout broke Auto Release |
| `raw/glob-dependency-chain.txt` | Lockfile trace showing where `glob@10.5.0` comes from |

## Summary

Two independent root causes, both false positives, hit the same commit `12a4b34e`:

- **A.** `scripts/install-node-dependencies.sh` allowlisted npm deprecation
  warnings by exact `name@version`. `glob` floated `7.2.3` -> `10.5.0`, stopped
  matching, and failed the `.vsix` job plus every desktop build.
- **C.** `scripts/self-hosting-metric.rs` read trailers via git's
  `%(trailers)` placeholder, which only parses the last paragraph of a message.
  A blank line between `Formal-AI-Session` and `Formal-AI-Evidence` hid the
  session trailer and failed Auto Release.

The two `skipped` Desktop Release runs listed in the issue are benign.
