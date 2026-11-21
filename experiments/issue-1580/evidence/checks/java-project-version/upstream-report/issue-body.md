The release helper can successfully commit and push `v1.2.4` while the tagged project's POM still declares `1.2.3`. A parent version or an XML comment before the project version triggers the defect.

Source checked at the current template head, `dd803a2d9fa5b3a992b435acce73162274fb9e8b`: [`scripts/version-and-commit.mjs`](https://github.com/link-foundation/java-ai-driven-development-pipeline-template/blob/dd803a2d9fa5b3a992b435acce73162274fb9e8b/scripts/version-and-commit.mjs#L171). `getCurrentVersion()` removes comments and the parent block before locating the project version. `updatePomVersion()` instead replaces only the first unfiltered version match, and does not fail when the replacement changes nothing. The two operations can therefore address different XML elements.

A bounded offline reproduction executes the unchanged complete release CLI with `--mode instant --bump-type patch` in four disposable Git repositories, each with a local bare remote and Java/project version `1.2.3`:

| POM layout | CLI exit / output | Tagged project version | Other observed change |
| --- | --- | --- | --- |
| Plain project | 0 / `released=true` | `1.2.4` | Correct control |
| Parent version `9.8.7` first | 0 / `released=true` | `1.2.3` | Java version becomes `1.2.4`; POM replacement makes no change |
| Parent version `1.2.3` first | 0 / `released=true` | `1.2.3` | Parent becomes `1.2.4` |
| Leading comment containing a version element | 0 / `released=true` | `1.2.3` | Comment changes instead of the project version |

The regression reads the actual tagged POM and Java source. A separate Python XML ElementTree check confirms every project's and parent's resulting version. No Git failure is injected: the release commit and local tag push succeed, so this is distinct from the failed-commit handling in #31.

Executable evidence is published in [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581):

- [Pinned-source CLI probe and disposable candidate](https://github.com/link-assistant/formal-ai/blob/ebbf5678fedc4b53c88fe12a549e138d7bcc39e2/experiments/issue-1580/probe-java-project-version.mjs).
- [Regression tests](https://github.com/link-assistant/formal-ai/blob/ebbf5678fedc4b53c88fe12a549e138d7bcc39e2/experiments/issue-1580/template-java-project-version.test.mjs).
- [Complete results, source/log hashes and independent XML observations](https://github.com/link-assistant/formal-ai/blob/ebbf5678fedc4b53c88fe12a549e138d7bcc39e2/experiments/issue-1580/evidence/checks/java-project-version/validation.json).

Run `node --test --test-concurrency=1 experiments/issue-1580/template-java-project-version.test.mjs` in that checkout. Both focused tests pass. Disabling the candidate with `ISSUE_1580_JAVA_DISABLE_PROJECT_VERSION_CANDIDATE=1` produces the expected preservation-test failure. The maintained template entrypoint passes all 54 cases without skips.

The disposable candidate masks comments and the parent block when locating the project version, checks the old project value, and changes only its value bytes. All four controls preserve the rest of the complete POM exactly, including parent/comment contents. This verifies those concrete cases; it is not a general XML-parser correctness claim. A production repair should make reader and writer select the same actual project element and verify the committed version before reporting publication success.

The probe verifies the copied template's Git blob and SHA-256 identities and retains its license. It uses no Maven build, external network, hosted workflow or GitHub publication. No upstream repair is claimed.
