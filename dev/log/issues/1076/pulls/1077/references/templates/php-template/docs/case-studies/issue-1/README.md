# Case Study — Issue #1: Symfony + Composer PHP AI-Driven Development Pipeline Template

> Source issue: [link-foundation/php-ai-driven-development-pipeline-template#1](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/1)
> Labels: `documentation`, `enhancement` · Opened 2026-06-09 by @konard

This case study collects the data behind this template, analyses it, lists every
requirement from the issue, and proposes (and records) a solution plan for each.
The raw source data — the issue payload and a full snapshot of the four sibling
templates — lives in [`data/`](data).

---

## 1. The Issue, Verbatim Requirements

The issue body (see [`data/issue-1.json`](data/issue-1.json)) asks us to build a
PHP equivalent of the existing AI-driven-development pipeline templates. Broken
down into discrete, checkable requirements:

| # | Requirement | Where addressed |
| --- | --- | --- |
| R1 | Reuse **all the best practices** from the JS, Rust, Python and C# CI/CD templates. | Whole template; §3, §4 |
| R2 | **Compare all files** (every GitHub workflow and CI/CD script) across the four templates so we don't repeat CI/CD mistakes. | §3, [`data/templates/*-analysis.md`](data/templates) |
| R3 | CI/CD uses **PHP scripts in `./scripts`** — everything native to PHP, **no other languages**. | [`scripts/`](../../../scripts), [`scripts/src/`](../../../scripts/src) |
| R4 | **Collect issue-related data** into `./docs/case-studies/issue-1/`. | [`data/`](data) |
| R5 | Do a **deep case-study analysis**, including **searching online** for extra facts/data. | §3–§6 |
| R6 | **List each and every requirement** from the issue. | This table |
| R7 | **Propose solutions / solution plans for each requirement**, checking **known existing components/libraries**. | §4, §5 |
| R8 | Plan and **execute everything in a single PR** (#2), continuing until **every requirement is fully addressed**. | PR #2 |

A target stack is also stated in the issue title: **Symfony + Composer PHP**. We
interpret "Symfony" as the broader Symfony/Composer PHP ecosystem and its tooling
conventions (PSR standards, Symfony's PHP-CS-Fixer, Flex-style configuration),
rather than scaffolding a full Symfony application — the sibling templates are all
*library/package* templates, and a package template is the directly comparable
artefact.

---

## 2. Method

1. **Snapshot the references.** All four sibling templates were cloned and their
   complete file trees captured under [`data/templates/`](data/templates) (see the
   `*-file-tree.txt` files), so the comparison is reproducible and not dependent on
   the upstream repos staying unchanged.
2. **Analyse each template** in depth — its release model, workflows, scripts,
   linting, versioning and "AI-driven" conventions. The write-ups are
   [`js-analysis.md`](data/templates/js-analysis.md),
   [`rust-analysis.md`](data/templates/rust-analysis.md),
   [`python-analysis.md`](data/templates/python-analysis.md) and
   [`csharp-analysis.md`](data/templates/csharp-analysis.md).
3. **Extract the common pattern**, note where the templates *diverge*, and decide
   which variant to adopt for PHP (§3, §4).
4. **Map each concept to the PHP/Composer/Packagist ecosystem**, surveying existing
   libraries before writing anything ourselves (§5).
5. **Implement** the result as this template, encoding the workflow policies as
   tests so they can't regress (§6).

---

## 3. Cross-Template Comparison (R2)

### 3.1 What all four templates share

Despite four different language ecosystems, the templates converge on the same
architecture — this is the "best practice" core we port to PHP:

- **Changeset-style changelog fragments** decouple "what changed" from "what
  version". JS uses [Changesets](https://github.com/changesets/changesets); Python
  uses [Scriv](https://scriv.readthedocs.io/) (`changelog.d/`); Rust and C# hand-roll
  the same idea. The highest bump among pending fragments wins.
- **The registry — not the git tag — is the source of truth** for "is this version
  released?": npm for JS, crates.io for Rust, PyPI for Python, NuGet for C#. The tag
  is only an idempotency guard.
- **Self-healing, idempotent releases**: every release step checks whether its work
  already exists and no-ops if so, so a half-finished release completes on re-run.
- **One combined CI/CD workflow** (`release.yml`, display name "CI/CD Pipeline")
  with change detection gating the expensive jobs.
- **The version field is owned by the pipeline**; a guard script fails any PR that
  edits it by hand.
- **A separate docs workflow** builds API docs and deploys to GitHub Pages on
  `main` (TypeDoc / rustdoc / Sphinx / DocFX).
- **A file-size limit** (1000 lines) enforced in CI.
- **Workflow-policy meta-tests**: the JS and C# templates test their *workflow YAML*
  (every job has a timeout, concurrency doesn't cancel `main`, `!cancelled()` is
  used, etc.).
- **Reasonable timeouts on every job**, pinned action versions, and concurrency that
  never cancels an in-progress `main` release.

### 3.2 Where they differ (and what we chose)

| Concern | JS | Rust | Python | C# | **PHP (this template)** |
| --- | --- | --- | --- | --- | --- |
| Script language | Node `.mjs` | Rust (`rust-script`) | Python | Node `.mjs` (run with Bun) | **Native PHP** (R3) |
| Changelog tool | Changesets | hand-rolled `changelog.d/` | Scriv | `.changeset/` | hand-rolled `changelog.d/` (PHP) |
| Registry publish | npm OIDC **upload** | `cargo publish` **upload** | PyPI **upload** (trusted publishing) | NuGet **upload** | **Packagist webhook** (no upload — poll only) |
| Test of pipeline | yes (vitest) | shell experiments | pytest | node tests | **PHPUnit `pipeline` suite** |
| Pre-commit | Husky + lint-staged | `pre-commit` | `pre-commit` | `.pre-commit-config` | Composer scripts (optional hook) |
| Static analysis | ESLint | clippy | mypy | Roslyn analysers | **PHPStan level 8** |

The single most important PHP-specific divergence is **publishing**. npm, crates.io,
PyPI and NuGet all require an explicit upload step with a token. **Packagist does
not**: it imports a package by reading its Git tags via a GitHub webhook. So the PHP
"publish" step is *push the tag, then poll Packagist's
[p2 metadata API](https://repo.packagist.org/p2/vendor/name.json) until the version
appears* — there is no `publish_to_packagist.php` analogous to
`publish_to_pypi.py`. This is captured in
[`wait-for-packagist.php`](../../../scripts/wait-for-packagist.php) and the
[`Packagist`](../../../scripts/src/Packagist.php) class.

### 3.3 CI/CD mistakes the references already fixed (so we inherit the fixes)

The analyses surfaced specific lessons baked into the upstream templates that we
adopt directly:

- **Never `cancel-in-progress` on `main`** — cancelling a release mid-flight yields
  a tag with no GitHub Release. We use
  `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`.
- **Use `!cancelled()`, not `always()`** for release-ordering dependencies, so a
  skipped dispatch-only job doesn't wedge the chain and a real cancel still
  propagates.
- **Give every job a `timeout-minutes`** so a hung network wait fails fast.
- **Pin action versions** for reproducibility.
- **Treat the registry as truth**, so re-running after a partial failure self-heals
  instead of double-publishing or erroring on an existing tag.

These are encoded as assertions in
[`tests/Pipeline/WorkflowPolicyTest.php`](../../../tests/Pipeline/WorkflowPolicyTest.php).

---

## 4. Per-Requirement Solution Plans (R6, R7)

### R1 / R3 — Native-PHP pipeline reusing the best practices

**Plan:** thin `scripts/*.php` entry points over tested classes in `scripts/src/`
(PSR-4 `LinkFoundation\Template\Pipeline\`). Each upstream `.mjs`/`.py`/`.rs` script
maps to a PHP equivalent:

| Upstream concept | PHP entry point | PHP class(es) |
| --- | --- | --- |
| `detect-code-changes` | `detect-code-changes.php` | `ChangeDetector` |
| `create-changeset` / `npx changeset` | `create-changeset.php` | `ChangelogFragments`, `SemVer` |
| `validate-changeset` | `validate-changeset.php` | `ChangesetValidator` |
| version-modification guard | `check-version-modification.php` | `Process`, `Project` |
| `check-release-needed` | `check-release-needed.php` | `ReleaseDecider`, `ReleaseDecision` |
| `version-and-commit` / `bump-version` | `version-and-commit.php` | `VersionReleaser`, `SemVer`, `Changelog`, `Git` |
| `wait-for-{npm,nuget,crate}` | `wait-for-packagist.php` | `Packagist`, `Http` |
| `create-github-release` | `create-github-release.php` | `GitHub`, `ReleaseNotes`, `Changelog` |
| `check-file-size` | `check-file-size.php` | `FileSizeChecker` |
| link archive fallback | `check-web-archive.php` | `WebArchive`, `Http` |

`getopt()` type-safety (it returns `string|false|array`) is handled by a small
[`Cli`](../../../scripts/src/Cli.php) helper so the whole pipeline passes PHPStan
level 8.

### R2 — File-by-file comparison

**Done** in §3 and the four `*-analysis.md` files, backed by the full file-tree
snapshots in `data/templates/*-file-tree.txt`.

### R4 — Data collection

**Done.** [`data/issue-1.json`](data/issue-1.json) and
[`data/issue-1-comments.json`](data/issue-1-comments.json) hold the issue payload;
[`data/templates/`](data/templates) holds the cloned references and analyses.

### R5 — Deep analysis + online research

**Done** in this document. Online research (R5) is folded into §3 and §5: the
Packagist publishing model, the absence of an upload step, the p2 metadata API, and
the existing-library survey were verified against upstream documentation (linked
inline).

### R8 — Single PR, fully addressed

All work lands in PR #2 on branch `issue-1-c9a284e42800`. The
[Status checklist](#7-status-checklist) tracks completion.

---

## 5. Existing Components / Library Survey (R7)

Before hand-rolling anything we checked what the PHP ecosystem already provides:

| Need | Existing options considered | Decision |
| --- | --- | --- |
| Changelog fragments | [changelog-linker](https://github.com/Symplify/ChangelogLinker), [Conventional Changelog](https://github.com/marcocesarato/php-conventional-changelog), manual `changelog.d/` | **Hand-rolled `changelog.d/`** — matches the Rust/C# approach, keeps the format identical across the template family, zero extra runtime deps, and is fully testable. |
| Semantic version parsing | [composer/semver](https://github.com/composer/semver) | Implemented a tiny `SemVer` class for bump/compare so the pipeline has **no runtime dependency** beyond dev tools; `composer/semver` is a reasonable swap-in if richer constraint logic is ever needed. |
| Static analysis | [PHPStan](https://phpstan.org/), [Psalm](https://psalm.dev/) | **PHPStan level 8** — most widely adopted, matches the "strictest level" choice the other templates make (mypy strict, clippy, Roslyn). |
| Code style | [PHP-CS-Fixer](https://cs.symfony.com/), [PHP_CodeSniffer](https://github.com/squizlabs/PHP_CodeSniffer) | **PHP-CS-Fixer** with `@PSR12` + `@PHP81Migration` — it's the Symfony-native formatter, mirroring "Prettier/ruff format" as the single style authority. |
| Testing | [PHPUnit](https://phpunit.de/), [Pest](https://pestphp.com/) | **PHPUnit** — the de-facto standard, no extra abstraction, two suites (`package`, `pipeline`). |
| API docs | [phpDocumentor](https://phpdoc.org/), [Doctum](https://github.com/code-lts/doctum) | **phpDocumentor** — actively maintained, GitHub Pages friendly, the closest analogue to Sphinx/TypeDoc/DocFX. |
| Registry client | [Packagist API](https://packagist.org/apidoc), `composer/composer` programmatic API | **Direct p2 metadata API via a tiny injectable `Http`** — avoids pulling Composer's internals into the pipeline and keeps the network surface trivially mockable. |
| Link checking | [lychee](https://github.com/lycheeverse/lychee-action) | **lychee-action** in CI (same as JS template) + a **native-PHP Wayback fallback** so we don't depend on a JS/Rust script. |
| GitHub releases | [`gh` CLI](https://cli.github.com/), GitHub REST via Guzzle | **`gh` CLI** — already present on GitHub runners, used by the other templates, no HTTP client dependency needed. |

The guiding principle: **zero runtime dependencies** for the published package
(`require` is just `php: >=8.1`), and reuse the runner's existing tools (`git`,
`gh`) for the pipeline so there's nothing extra to install.

---

## 6. What Was Built

- **Pipeline classes** (`scripts/src/`): `Project`, `Actions`, `Process`,
  `ProcessResult`, `SemVer`, `ChangelogFragments`, `Changelog`, `Http`, `Packagist`,
  `GitHub`, `ReleaseDecision`, `ReleaseDecider`, `ReleaseNotes`, `ChangeDetector`,
  `ChangesetValidator`, `FileSizeChecker`, `Git`, `VersionReleaser`, `Cli`,
  `WebArchive`.
- **Entry points** (`scripts/`): the ten scripts in the §4 table plus
  `bootstrap.php`.
- **Workflows** (`.github/workflows/`): `release.yml` (CI/CD), `docs.yml`
  (phpDocumentor → Pages), `links.yml` (lychee + Wayback fallback).
- **Config**: `composer.json` (scripts + PSR-4), `phpunit.xml.dist` (two suites),
  `phpstan.neon.dist` (level 8), `.php-cs-fixer.dist.php`, `.lycheeignore`,
  `.gitignore`.
- **Changelog system**: `changelog.d/` (README, fragment template, the issue-1
  fragment) and a Keep-a-Changelog `CHANGELOG.md` with an insert marker.
- **Tests** (`tests/`): `Unit/` (package) + `Pipeline/` (pipeline logic, including
  the `WorkflowPolicyTest` meta-test) — 65 tests / 166 assertions.
- **Docs**: this case study, `docs/index.md`, `docs/BEST-PRACTICES.md`,
  `CONTRIBUTING.md`, and an updated `README.md`.

---

## 7. Status Checklist

- [x] **R1** Best practices ported from all four templates.
- [x] **R2** All workflows/scripts compared (§3 + analyses).
- [x] **R3** Pipeline is 100% native PHP in `scripts/`.
- [x] **R4** Issue + template data collected under `data/`.
- [x] **R5** Deep analysis + online research recorded.
- [x] **R6** Every requirement listed (§1).
- [x] **R7** Per-requirement plans + library survey (§4, §5).
- [x] **R8** Delivered in PR #2.
