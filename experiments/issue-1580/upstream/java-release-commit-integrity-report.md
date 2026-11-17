The release helper suppresses a failed `git commit`, creates a new version tag on the previous commit, pushes that tag and reports `released=true`. This affects both changeset and instant release modes.

Source reviewed: `dd803a2d9fa5b3a992b435acce73162274fb9e8b`, `scripts/version-and-commit.mjs` (Git blob `0f6812d46591f3f6615f52d834f8c42f965662fc`). The source uses `git commit -m "${commitMessage}" || echo "Nothing to commit"`, so any commit failure is converted to shell success before unconditional tag/push steps. Related merged PR #13 adds interrupted-release recovery and successful-release controls; latest #25/#26 update dependencies and do not remove this fallback. Existing #12 concerns recovery after a successful tag push, a different boundary.

Bounded offline reproduction:

1. Initialize a temporary checkout with POM and Java source version `1.2.3`, a valid patch changeset, and a disposable local bare remote. Push the baseline locally.
2. Install a local pre-commit hook that exits 23, then run the unchanged helper with `--mode changeset`, or with `--mode instant --bump-type patch`.
3. Both invocations exit 0 and write `released=true`. The working POM changes to `1.2.4`, but no release commit exists. The new `v1.2.4` tag is pushed to the local bare remote and its POM still says `1.2.3`.

Removing only `|| echo "Nothing to commit"` in a disposable source copy makes both modes exit 1 without creating or publishing the tag. Successful commits in both original and candidate sources still push an updated `1.2.4` tag. These four positive controls distinguish the error-propagation repair from disabling releases.

The executable reproduction is `experiments/issue-1580/template-java-release-integrity.test.mjs` in link-assistant/formal-ai PR #1581. It preserves the original license and verifies copied Git blob/SHA256 identities. Run `node --test --test-concurrency=1 experiments/issue-1580/template-java-release-integrity.test.mjs`: five cases pass. The maintained fast-PR entrypoint passes sixteen cases without skips. Full logs/hashes are in `experiments/issue-1580/evidence/checks/java-release-integrity.json`.

Every Git push in this reproduction targets a disposable local bare repository. No Maven build, external network, hosted workflow, GitHub API call or remote template modification is performed. The candidate repair is tested only in disposable copies; no upstream fix is claimed. Immutable executable evidence: https://github.com/link-assistant/formal-ai/blob/ff5d883b15aeda06c07e509a73a0a436f10799b7/experiments/issue-1580/template-java-release-integrity.test.mjs.
