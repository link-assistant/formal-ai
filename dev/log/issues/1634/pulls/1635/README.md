# PR 1635 CI failure: Artifact transport (macos-15), run 45483073309

- Job 138761089401 was queued 20:39:26Z and cancelled 20:54:33Z with no runner and no steps.
- Annotation: "The job was not acquired by Runner of type hosted even after multiple attempts"
  plus "Due to capacity constraints, jobs targeting macOS arm64 runners may experience longer queue times."
- Root cause: GitHub-hosted macOS arm64 capacity shortage (infrastructure), not repository code.
  The same workflow passes on main and every other check on this head SHA passed.
- Fix: re-run the failed job; no code change needed.
