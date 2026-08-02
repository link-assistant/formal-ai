# Online research (2026-07-16)

Primary sources were preferred.

- Aider's [blame script](https://github.com/Aider-AI/aider/blob/main/scripts/blame.py)
  classifies Aider-authored commits from commit markers and uses Git blame to
  calculate surviving code ownership. Its
  [history](https://github.com/Aider-AI/aider/blob/main/HISTORY.md) publishes
  release-by-release percentages. This validates explicit attribution plus a
  visible release metric, but Formal AI uses changed lines because issue #657
  explicitly defines the release unit that way.
- The SICA paper, [*Self-Improving Coding Agent*](https://arxiv.org/abs/2504.15228),
  evaluates self-edits against an external coding benchmark and reports an
  improvement from 17% to 53%. The transferable practice is gate changes with
  objective evaluation; E37 owns those gates and E38 measures the resulting
  authored share rather than treating self-editing itself as success.
- Git's official [`git log` format documentation](https://git-scm.com/docs/pretty-formats)
  defines the `trailers` placeholder used to read attribution keys, while
  official [`git show` documentation](https://git-scm.com/docs/git-show)
  documents `--numstat`. These primitives keep the metric local, auditable, and
  independent of GitHub API state.

Inference: explicit commit trailers are Formal AI's analogue to Aider's commit
markers, strengthened by resolving a committed evidence path and exact session
id at the same historical snapshot.
