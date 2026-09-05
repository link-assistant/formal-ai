# Documentation

Welcome to the documentation for the **php-ai-driven-development-pipeline-template** —
a native-PHP CI/CD pipeline template for Symfony / Composer packages.

## Contents

- **[Best Practices](BEST-PRACTICES.md)** — the CI/CD principles this template
  encodes, and why, distilled from the four sibling templates.
- **[Case study: issue #1](case-studies/issue-1/README.md)** — the deep analysis
  that produced this template: a file-by-file comparison of the JS, Rust, Python
  and C# templates, the full requirement list, and the per-requirement solution
  plans.
- **[Changelog fragments](../changelog.d/README.md)** — how to record changes.
- **[Contributing](../CONTRIBUTING.md)** — local workflow and quality gates.

## API Reference

The class-level API reference is generated from source by
[phpDocumentor](https://phpdoc.org/) and published to GitHub Pages on every push
to `main` (see [`.github/workflows/docs.yml`](../.github/workflows/docs.yml)).

To build the docs locally:

```bash
# Requires the phpDocumentor PHAR (https://phpdoc.org/)
phpdoc -d src,scripts/src -t build/docs
```

## Pipeline at a Glance

| Stage | Script | What it does |
| --- | --- | --- |
| Change detection | `detect-code-changes.php` | classifies changed files to gate CI |
| Lint | `composer lint` | PHP-CS-Fixer `@PSR12` + `@PHP81Migration` |
| Static analysis | `composer analyse` | PHPStan level 8 |
| File size | `check-file-size.php` | enforces the 1000-line limit |
| Tests | `composer test` | PHPUnit across PHP 8.1–8.4 |
| Changeset gate | `validate-changeset.php`, `check-version-modification.php` | require a fragment, forbid manual version edits |
| Release decision | `check-release-needed.php` | fragments present? version already on Packagist? |
| Version + tag | `version-and-commit.php` | bump, changelog, commit, tag, push |
| Wait for registry | `wait-for-packagist.php` | poll Packagist after the tag |
| GitHub Release | `create-github-release.php` | idempotent release with notes |
| Link archive | `check-web-archive.php` | Wayback fallback for broken links |

Each script is a thin entry point over a tested class in
[`scripts/src/`](../scripts/src).
