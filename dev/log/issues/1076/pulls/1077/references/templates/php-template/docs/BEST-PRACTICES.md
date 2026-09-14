# Best Practices for AI-Driven PHP Development

This template encodes the CI/CD practices that the
[`link-foundation`](https://github.com/link-foundation) JS, Rust, Python and C#
pipeline templates converged on, adapted to the PHP / Composer / Packagist
ecosystem. This document explains each practice and why it matters — especially
when an AI agent is doing most of the editing.

## Why CI/CD Matters for AI-Driven Development

AI agents make many small, fast changes and cannot "feel" when something is off.
A strict, fully automated pipeline is the safety net: it catches regressions,
enforces a consistent style, and makes releasing a non-event. The goal is a
pipeline that is **deterministic, idempotent, and self-healing**, so an agent can
iterate without a human babysitting every merge.

## The Practices

### 1. One Language in the Pipeline

Every CI/CD script is **native PHP** under [`scripts/`](../scripts), backed by
tested classes in [`scripts/src/`](../scripts/src). No Bash glue, no Node, no
Python. This keeps the pipeline reviewable by the same people (and agents) who
write the package, and lets the pipeline logic be unit-tested like any other code.

### 2. File-Size Limits

No source file may exceed **1000 lines** (`composer check:file-size`). Large files
are hard to review and blow past an AI context window. The limit forces logic into
small, single-responsibility classes.

### 3. Automated Code Formatting

[PHP-CS-Fixer](https://cs.symfony.com/) with `@PSR12` + `@PHP81Migration` is the
single source of truth for style. CI runs it in `--dry-run` mode; `composer
lint:fix` applies it. Style is never a review discussion.

### 4. Strict Static Analysis

[PHPStan](https://phpstan.org/) runs at **level 8** (the strictest). Type-safety
bugs — like the `string|false|array` return of `getopt()` — are caught before they
reach runtime. The pipeline classes are written to pass level 8 with no baseline.

### 5. Comprehensive, Layered Testing

[PHPUnit](https://phpunit.de/) runs two suites: `package` (`tests/Unit/`) and
`pipeline` (`tests/Pipeline/`). Pipeline classes take injectable collaborators
(HTTP fetcher, process runner, filesystem root) so the tests never hit the network
or mutate the real repo. Tests run on the **full matrix** of PHP 8.1–8.4.

### 6. Changeset-Based Versioning

Changelog fragments in `changelog.d/` (each with a `bump:` front-matter and a
Keep-a-Changelog heading) decouple "what changed" from "what's the next version".
Like Changesets (JS) or Scriv (Python), this makes per-PR changelog edits
**conflict-free** and lets the pipeline compute the version from the fragments —
the highest bump across all pending fragments wins.

### 7. The Version Is Owned by the Pipeline

Humans never edit the `version` field or `CHANGELOG.md` by hand;
`check-version-modification.php` fails any PR that tries. This removes a whole
class of merge conflicts and "forgot to bump" mistakes.

### 8. Self-Healing, Idempotent Releases

The release flow uses the git tag `v<version>` as the **only** idempotency guard,
and treats **Packagist + GitHub Releases as the source of truth** — not the tag.
Every step (tag, wait-for-Packagist, create-release) checks whether its work is
already done and no-ops if so. A release that fails halfway through simply
finishes on the next run.

Packagist is special: it publishes on a **tag-push webhook**, so there is no
"upload" step (unlike PyPI/npm). The pipeline pushes the tag and then *polls*
Packagist until the version is importable before cutting the GitHub Release.

### 9. Combined CI/CD Workflow with Change Detection

A single workflow handles CI and CD. `detect-code-changes.php` classifies the
changed files so documentation-only or changelog-only changes skip the expensive
jobs. Releases run only after the checks pass on `main`.

### 10. Concurrency That Never Cancels `main`

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```

Superseded PR runs are cancelled to save CI minutes, but a release in progress on
`main` is **never** interrupted — cancelling mid-release is how you get a tag with
no GitHub Release.

### 11. Proper Cancellation Propagation

Dependent jobs use `if: ${{ !cancelled() }}` rather than `always()`. `always()`
would run a job even after you cancel the workflow; `!cancelled()` runs it whenever
the upstream wasn't cancelled (including when a dispatch-only dependency was
skipped), which is what release ordering actually needs.

### 12. A Timeout on Every Job

Every job sets `timeout-minutes`. A hung step (network wait, flaky test) fails fast
instead of burning the 6-hour GitHub default. The
`tests/Pipeline/WorkflowPolicyTest` asserts this — and the other policies above —
as regression tests over the workflow YAML.

### 13. Pinned Action Versions

Third-party actions are pinned to a major version (`actions/checkout@v4`,
`shivammathur/setup-php@v2`, `ramsey/composer-install@v3`, …) for reproducible
builds.

### 14. Link Checking with an Archive Fallback

`links.yml` checks external links with [lychee](https://github.com/lycheeverse/lychee-action);
when a link is broken, `check-web-archive.php` re-checks it against the
[Wayback Machine](https://web.archive.org/) (in native PHP) and only fails the job
if there is no archived snapshot. Bit-rot in third-party docs doesn't break your CI.

## Quality Enforcement Strategy

The layers compound: formatting and the file-size limit keep the diff small and
readable; static analysis and tests catch behavioural regressions; the changeset +
version-ownership rules keep releases honest; and the idempotent release flow makes
shipping safe to retry. Encoding the workflow policies as PHPUnit tests
(`WorkflowPolicyTest`) means the *pipeline itself* can't silently regress.

## References

- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
- [PSR-12 Coding Style](https://www.php-fig.org/psr/psr-12/)
- [PHPStan](https://phpstan.org/) · [PHP-CS-Fixer](https://cs.symfony.com/) · [PHPUnit](https://phpunit.de/)
- [Packagist publishing](https://packagist.org/about) · [Composer](https://getcomposer.org/)
