# php-ai-driven-development-pipeline-template

A comprehensive template for AI-driven Symfony / Composer **PHP** development with a
full, native-PHP CI/CD pipeline.

[![CI/CD Pipeline](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/workflows/CI/CD%20Pipeline/badge.svg)](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/actions)
[![PHP Version](https://img.shields.io/badge/php-8.1%20--%208.4-777BB4.svg)](https://www.php.net/supported-versions.php)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://unlicense.org/)

This template ports the battle-tested CI/CD practices from the sibling
[`js`](https://github.com/link-foundation/js-ai-driven-development-pipeline-template),
[`rust`](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template),
[`python`](https://github.com/link-foundation/python-ai-driven-development-pipeline-template)
and [`csharp`](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template)
templates to the PHP ecosystem. **Every CI/CD script is native PHP** living in
[`scripts/`](scripts) — no Bash glue, no Node, no Python. The release flow is
self-healing and idempotent, and it publishes to [Packagist](https://packagist.org)
and GitHub Releases.

## Features

- **Multi-version PHP support**: tested on PHP 8.1, 8.2, 8.3 and 8.4
- **Native-PHP pipeline**: all CI/CD logic lives in [`scripts/`](scripts) and the
  tested [`scripts/src/`](scripts/src) classes — no other languages used
- **Comprehensive testing**: [PHPUnit](https://phpunit.de/) with separate package
  and pipeline test suites, plus coverage output
- **Code quality**: [PHP-CS-Fixer](https://cs.symfony.com/) (`@PSR12` +
  `@PHP81Migration`) and [PHPStan](https://phpstan.org/) at the strictest level 8
- **Changeset-style changelog**: conflict-free `changelog.d/` fragments drive
  semantic versioning, like Changesets in JS or Scriv in Python
- **Self-healing releases**: idempotent, re-runnable release flow with the git tag
  as the only idempotency guard; Packagist + GitHub Releases are the source of truth
- **Combined CI/CD workflow**: change detection, lint, static analysis, multi-version
  tests, changeset validation, automatic + manual releases — all in one workflow
- **API documentation**: [phpDocumentor](https://phpdoc.org/) build deployed to
  GitHub Pages on push to `main`
- **Link checking with archive fallback**: broken external links are re-checked
  against the [Wayback Machine](https://web.archive.org/) in native PHP
- **File-size guardrail**: no source file may exceed 1000 lines, keeping logic
  reviewable and AI-friendly

## Quick Start

### Using This Template

1. Click **"Use this template"** on GitHub to create a new repository.
2. Clone your new repository.
3. Update [`composer.json`](composer.json): set `name` to your real
   `vendor/package`, update `description`, `homepage` and `authors`.
4. Rename the `LinkFoundation\Template\` namespace / `src/` classes to match your
   package.
5. Replace the example classes in [`src/`](src) and their tests in
   [`tests/Unit/`](tests/Unit).
6. Register your package on [Packagist](https://packagist.org) and add the GitHub
   webhook so tags publish automatically.

> The template ships with the sentinel package name
> `link-foundation/example-package-name`. While that sentinel is in place the
> release steps run in **dry-run** mode (no tags, no Packagist wait, no GitHub
> release), so you can fork and experiment safely. Renaming the package "arms"
> the pipeline.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/link-foundation/php-ai-driven-development-pipeline-template.git
cd php-ai-driven-development-pipeline-template

# Install dependencies (including dev tools)
composer install
```

### Running Tests

```bash
# Run the full test suite
composer test

# Run only the package tests or only the pipeline tests
vendor/bin/phpunit --testsuite package
vendor/bin/phpunit --testsuite pipeline

# Run with coverage (requires Xdebug or PCOV)
vendor/bin/phpunit --coverage-text
```

### Code Quality Checks

```bash
composer lint            # PHP-CS-Fixer (dry-run + diff)
composer lint:fix        # PHP-CS-Fixer (apply fixes)
composer analyse         # PHPStan level 8
composer check:file-size # enforce the 1000-line limit
composer check           # lint + analyse + file-size + test
```

### Recording a Change

Instead of editing `CHANGELOG.md` or the version by hand, add a **changelog
fragment**:

```bash
# Interactive
composer changeset

# Non-interactive
composer changeset -- --bump=minor --message="Add foo support"
```

This writes a file to `changelog.d/`. On the next release the pipeline collects
all fragments, picks the highest bump, updates `CHANGELOG.md` and `composer.json`,
tags, and publishes. See [`changelog.d/README.md`](changelog.d/README.md).

## Project Structure

```
.
├── .github/
│   └── workflows/
│       ├── release.yml         # CI checks + release automation (Packagist + GitHub)
│       ├── docs.yml            # phpDocumentor build + GitHub Pages deploy
│       └── links.yml           # link checking + Wayback fallback
├── changelog.d/                # Changelog fragments (like .changeset/)
│   ├── README.md               # How to write a fragment
│   └── *.md                    # Individual changelog entries
├── docs/
│   ├── index.md                # Documentation landing page
│   ├── BEST-PRACTICES.md       # The CI/CD principles encoded here
│   └── case-studies/issue-1/   # Deep analysis behind this template
├── scripts/                    # Native-PHP CI/CD entry points
│   ├── bootstrap.php           # Shared autoload bootstrap
│   ├── check-file-size.php     # 1000-line guardrail
│   ├── detect-code-changes.php # change detection for conditional CI
│   ├── create-changeset.php    # `composer changeset`
│   ├── validate-changeset.php  # require a fragment on code PRs
│   ├── check-version-modification.php # block manual version edits
│   ├── check-release-needed.php       # decide whether to release
│   ├── version-and-commit.php  # bump, changelog, commit, tag, push
│   ├── wait-for-packagist.php  # poll Packagist after the tag
│   ├── create-github-release.php      # idempotent GitHub Release
│   └── check-web-archive.php   # Wayback fallback for broken links
│   └── src/                    # Tested pipeline classes (PSR-4)
├── src/                        # Your package code (example classes)
├── tests/
│   ├── Unit/                   # Package tests
│   └── Pipeline/               # Pipeline-logic tests
├── .php-cs-fixer.dist.php      # PHP-CS-Fixer configuration
├── phpstan.neon.dist           # PHPStan configuration (level 8)
├── phpunit.xml.dist            # PHPUnit configuration (two suites)
├── composer.json               # Project configuration and scripts
└── CHANGELOG.md                # Generated changelog
```

## How the Release Pipeline Works

1. **On every PR** the pipeline lints, statically analyses and tests the code
   across PHP 8.1–8.4, validates that code changes ship a changelog fragment, and
   ensures nobody hand-edited the version.
2. **On push to `main`** it re-runs the checks, then decides whether a release is
   needed (are there fragments? is the current version already on Packagist?).
3. If a release is needed it computes the next version from the fragments, updates
   `composer.json` + `CHANGELOG.md`, commits, and pushes a `v<version>` tag.
4. It waits for Packagist to import the new version (Packagist publishes via
   webhook on tag push — there is no upload step), then creates the matching GitHub
   Release.
5. Every step is **idempotent**: re-running a release that already happened is a
   no-op, so transient failures self-heal on re-run.

A manual `workflow_dispatch` lets you cut an explicit `major`/`minor`/`patch`
release without a fragment.

See [`docs/BEST-PRACTICES.md`](docs/BEST-PRACTICES.md) for the full rationale and
[`docs/case-studies/issue-1/README.md`](docs/case-studies/issue-1/README.md) for
the comparison of the four sibling templates that informed this design.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Released into the public domain under the [Unlicense](https://unlicense.org/).
