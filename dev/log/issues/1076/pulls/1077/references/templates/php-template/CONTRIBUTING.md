# Contributing to php-ai-driven-development-pipeline-template

Thank you for your interest in contributing! This document covers the local
workflow, the quality gates, and how releases work so your changes sail through
CI.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/php-ai-driven-development-pipeline-template.git
   cd php-ai-driven-development-pipeline-template
   ```

2. **Install dependencies** (PHP 8.1+ and [Composer](https://getcomposer.org/)
   required)

   ```bash
   composer install
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Follow the existing code style (PSR-12, enforced by PHP-CS-Fixer).
   - Add or update tests for any behaviour change.
   - Keep every file under **1000 lines** — split logic into small classes under
     `scripts/src/` or `src/` instead of growing one file.
   - Put CI/CD logic in a tested class under `scripts/src/` and keep the
     `scripts/*.php` entry points thin. **No languages other than PHP** in the
     pipeline.

3. **Run the quality checks**

   ```bash
   composer lint       # PHP-CS-Fixer (dry-run + diff)
   composer analyse    # PHPStan level 8
   composer check:file-size
   composer test       # PHPUnit
   # …or all at once:
   composer check
   ```

   Auto-fix style issues with `composer lint:fix`.

4. **Add a changelog fragment**

   Every code change needs a fragment so the release pipeline can compute the
   next version. Do **not** edit `CHANGELOG.md` or the `version` field in
   `composer.json` by hand — CI will reject a PR that does.

   ```bash
   composer changeset -- --bump=patch --message="Fix the thing"
   ```

   `--bump` is one of `major`, `minor`, `patch`. See
   [`changelog.d/README.md`](changelog.d/README.md) for the fragment format.

5. **Commit and open a pull request**

   Write a clear commit message and describe what changed and why. CI runs the
   full matrix (PHP 8.1–8.4) plus changeset validation on your PR.

## Testing Guidelines

- Package behaviour lives in `tests/Unit/`; pipeline-logic tests live in
  `tests/Pipeline/`. They are exposed as the `package` and `pipeline` PHPUnit
  test suites.
- Pipeline classes take injectable collaborators (HTTP fetcher, process runner,
  filesystem root) so tests never touch the network or the real git repo.
- When you fix a bug, add a test that fails before the fix and passes after it.

## How Releases Happen

Releases are fully automated and you should never tag or bump versions manually:

1. Merging a PR with changelog fragments to `main` triggers the release job.
2. The pipeline computes the next semantic version from the fragments, updates
   `composer.json` and `CHANGELOG.md`, commits, and pushes a `v<version>` tag.
3. Packagist imports the tag via its GitHub webhook; the pipeline waits for it,
   then publishes the matching GitHub Release.

The whole flow is idempotent — re-running it after a partial failure is safe.

## Code of Conduct

Be respectful and constructive. This project is released into the public domain
under the [Unlicense](https://unlicense.org/); contributions are accepted on the
same terms.
