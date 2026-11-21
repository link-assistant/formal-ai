The pull-request version guard can report success without a valid comparison.

Reproduced in [scripts/check-version-modification.php](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/b61e334703d93a15e08bf1a278e0df0a9d6e98a8/scripts/check-version-modification.php)
at b61e334703d93a15e08bf1a278e0df0a9d6e98a8. Its blob is unchanged from the
pinned 3a034b27a56067971ca74dd4c74c7707056b215d fixture.

`versionField` converts invalid JSON, missing versions, null and numeric
versions to the same null. The final comparison only rejects a difference when
both values are non-null. In addition, failed `git fetch`/`git show` can be
treated as a newly introduced manifest, rather than unavailable base evidence.

Finite local Git controls execute the actual CLI, without Composer installation
or remote network calls. Removing the head version, invalid head/base JSON,
null/numeric head versions, adding a version to an existing manifest and a missing
base repository all return zero on the original. The unavailable-base case even
has a changed head version. Unchanged and genuinely new manifests return zero;
an ordinary changed version returns one, as expected. A stale remote-tracking
ref also lets a failed fetch be ignored.

The [maintained original/candidate tests](https://github.com/link-assistant/formal-ai/blob/ac9cd6a55e3ea7d8738e6122f623d7676666630d/experiments/issue-1580/template-php-input-integrity.test.mjs) include all those controls:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Run from Formal AI commit ac9cd6a55e3ea7d8738e6122f623d7676666630d with PHP and
Node installed. Each local child has a five-second boundary, bounded output and
a 32 MiB PHP allowance. The test deliberately asserts the original bad result,
then verifies a disposable candidate rejects it. No candidate is applied to this
template by that test.

The candidate distinguishes an object with an absent version from malformed or
wrong-type input, requires a successful fetch and resolved commit, and proves
base-file absence with checked ls-tree before permitting bootstrap. It compares
the two nullable versions strictly, so removal/addition cannot bypass admission.
Positive unchanged/new-file/changed-version behavior is preserved. Please retain
those observation-failure and bootstrap distinctions in the upstream repair.

Found during the full CI/source comparison for
https://github.com/link-assistant/formal-ai/pull/1581.
