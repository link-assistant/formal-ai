The changelog collector deletes skipped release fragments while reporting success.

Reviewed source: `dd803a2d9fa5b3a992b435acce73162274fb9e8b`, `scripts/collect-changelog.mjs`, Git blob `659ecc112fe75039abd66af0ac61bdb9849c84ad`.

The unchanged complete CLI matches frontmatter with LF-only separators. Valid CRLF fragments are treated as unparsed; malformed frontmatter and empty descriptions are also skipped. It nevertheless deletes every discovered changeset after updating the changelog. Consequently it exits zero, loses the input file and omits its release note. A mixed valid/malformed batch writes the valid note and deletes the malformed fragment too.

Bounded offline reproduction:

1. Create a temporary directory with the exact pinned collector and `github-actions-log.mjs`, a baseline `CHANGELOG.md`, and `.changeset/README.md`.
2. Add a patch changeset with CRLF separators and a nonempty description. Run `node scripts/collect-changelog.mjs --version 1.2.4`.
3. Observe exit zero, deletion of the fragment and omission of its description from the changelog. Repeat with missing frontmatter, an empty description and a mixed valid/malformed batch.

The executable reproduction in Formal AI PR #1581 is `experiments/issue-1580/template-java-boundary-controls.test.mjs`. Run `node --test --test-concurrency=1 experiments/issue-1580/template-java-boundary-controls.test.mjs`: five cases pass. A disposable candidate normalizes CRLF and rejects missing/empty descriptions before writes. Positive LF and CRLF controls retain the note in the changelog. Every rejected batch preserves the original changelog and all fragment bytes, including the valid sibling. README is preserved throughout.

Setting `ISSUE_1580_JAVA_KEEP_ORIGINAL_FRAGMENT_READER=1` reproduces the candidate regression failure on the valid CRLF case. Actual collector invocations use bounded isolated Node processes with cleanup. No Maven build, Git push, hosted workflow or GitHub publication occurs. The candidate exists only in disposable copies; no upstream repair is claimed.


Immutable reproduction: [five-case executable control](https://github.com/link-assistant/formal-ai/blob/44771e97373f89d4d582bc384c2fd1f8d69cd83d/experiments/issue-1580/template-java-boundary-controls.test.mjs), [exact source fixture](https://github.com/link-assistant/formal-ai/tree/44771e97373f89d4d582bc384c2fd1f8d69cd83d/experiments/issue-1580/fixtures/java-template), [original and candidate replay evidence](https://github.com/link-assistant/formal-ai/blob/44771e97373f89d4d582bc384c2fd1f8d69cd83d/experiments/issue-1580/evidence/checks/java-boundary-controls/manifest.json).
