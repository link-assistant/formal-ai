# Upstream report 6 - the php template has no `security.yml` at all

**Target:** `link-foundation/php-ai-driven-development-pipeline-template`

**Severity:** the template is one of five that are otherwise kept in step, and
it is the only one with no static analysis, no dependency audit, and no
dependency review. A repository generated from it inherits a pipeline that
never asks a security question.

---

## Title

`.github/workflows/security.yml` is missing; the other four templates all ship
one with CodeQL and a dependency audit

## Evidence

```console
$ for t in rust js python php csharp; do
    echo "$t: $(ls ${t}-template/.github/workflows/ | tr '\n' ' ')"
  done
rust:   desktop-release.yml links.yml release.yml security.yml workflows.yml
js:     example-app.yml     links.yml release.yml security.yml workflows.yml
python: docs.yml            links.yml release.yml security.yml workflows.yml
php:    docs.yml            links.yml release.yml                workflows.yml
csharp: docs.yml            links.yml release.yml security.yml workflows.yml
```

Nothing elsewhere covers it: `grep -ri 'secur\|audit' php-template.FILES`
returns nothing, and `release.yml` has no audit step.

## What the other four have that php does not

| | rust | js | python | php | csharp |
|---|---|---|---|---|---|
| CodeQL (source language) | `rust` | `javascript-typescript` | `python` | -- | `csharp` |
| CodeQL (`actions`) | yes | yes | yes | -- | yes |
| Dependency audit | `cargo audit` | `npm audit --audit-level=high` | `scripts/audit_dependencies.py` | -- | -- |
| Dependency Review | yes | yes | yes | -- | yes |
| Weekly schedule | yes | yes | yes | -- | yes |
| zizmor (in `workflows.yml`) | yes | yes | yes | yes | -- |

php has the workflow audit and none of the rest. csharp is the mirror image:
everything except the workflow audit and a dependency audit -- reported
separately.

## Why the missing dependency audit is the sharpest edge

PHP has a first-party equivalent that needs no extra tooling and no API key:

```console
$ composer audit
```

> This command is used to audit the packages you have installed against
> defined dependency policies, such as security advisories. It checks for and
> lists security vulnerability advisories using the Packagist.org API by
> default [...] The command also detects abandoned packages and packages
> flagged as malware.
>
> -- <https://getcomposer.org/doc/03-cli.md#audit>

Two flags are worth setting explicitly, and the second one is the interesting
one because Composer already gets it right:

- `--locked` audits `composer.lock` rather than the installed tree, so the job
  does not depend on an install step having run first.
- `--abandoned` takes `ignore`, `report` or `fail`. The docs record that it
  *"Defaults to `fail` since Composer 2.7 (defaulted to `report` in Composer
  2.6 that added the option)"*
  (<https://getcomposer.org/doc/06-config.md#abandoned>).

That second point is worth pausing on, because it is the same question the
companion report on `cargo audit` is about and Composer answered it the other
way. An abandoned package is the PHP equivalent of an unmaintained crate: no
advisory will ever be issued for it, because nobody is looking. `cargo audit`
still treats that as a non-fatal warning by default; Composer made it fatal in
2.7. Passing `--abandoned=fail` explicitly costs nothing and states the intent
where a reader can see it, rather than depending on which Composer the runner
happened to install.

## Suggested fix

Add `.github/workflows/security.yml`, modelled on the python template's, with
the PHP-shaped audit job:

```yaml
name: Security

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 6 * * 1'

permissions:
  contents: read

jobs:
  dependency-audit:
    name: Dependency Audit
    runs-on: ubuntu-latest
    timeout-minutes: 10
    concurrency:
      group: check-${{ github.workflow }}-${{ github.ref }}-dependency-audit
      cancel-in-progress: true
    steps:
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
      - uses: shivammathur/setup-php@<pin to a commit hash>
        with:
          php-version: '8.3'
          tools: composer:v2
      # `--abandoned=report` is Composer's default and only *prints*; an
      # abandoned package is the PHP equivalent of an unmaintained crate, and a
      # gate that prints is not a gate.
      - name: Audit committed composer.lock
        run: composer audit --locked --abandoned=fail

  codeql:
    # ... as in the python template, with language: [actions]
    # (PHP is not a CodeQL-supported language; the `actions` analysis is the
    # part that transfers, and it is worth having on its own.)

  dependency-review:
    # ... as in the python template
```

One template-specific note: CodeQL has no PHP analyser, so the `codeql` job
here can only carry the `actions` language. That is a smaller job than the
other templates', and it is still worth adding -- it is the half that audits
the pipeline, which is the half every one of these templates shares.

`shivammathur/*` is not on the `.github/zizmor.yml` allow-list, so it needs a
hash pin or an entry, or the existing `unpinned-uses` audit will (correctly)
fail the new job.

## Related

- <https://getcomposer.org/doc/03-cli.md#audit> -- `composer audit`, `--locked`,
  `--abandoned`.
- <https://packagist.org/apidoc#list-security-advisories> -- the advisory source.
- The companion report on `cargo audit --deny warnings` in the rust template:
  the same "warnings do not fail the build" shape, in a different language.
