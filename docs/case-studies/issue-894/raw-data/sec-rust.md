The pipeline has thorough lint / test / coverage / release automation, but **no security analysis at all**: there is no CodeQL workflow, no `actions/dependency-review-action`, no SBOM step, and no vulnerability scanner anywhere under `.github/`. (CodeQL supports Rust as well as GitHub Actions workflows.)

## Reproduction

Against `main` (verified 2026-08-05):

```bash
gh api repos/link-foundation/rust-ai-driven-development-pipeline-template/git/trees/HEAD?recursive=1 \
  --jq '.tree[].path' | grep -iE 'codeql|dependabot|security'
# -> tests/unit/ci-cd/workflow_security.rs   (a workflow-hygiene unit test, not a scanner)

grep -rniE 'codeql|dependency-review|security-events|sbom|trivy|grype|osv-scanner|cargo audit' .github/
# -> no matches
```

`tests/unit/ci-cd/workflow_security.rs` asserts workflow hygiene (permissions
blocks, no unsafe expression interpolation). It is valuable, but it is not
dependency or code vulnerability scanning and produces no code-scanning alerts.

So a pull request that adds a vulnerable dependency, or a workflow change that
leaks `GITHUB_TOKEN` into an injectable `run:` block, merges with every check
green. This template is copied into new repositories as the CI baseline, so the
blind spot is inherited by every downstream project.

Note that CodeQL also analyses **GitHub Actions workflows** themselves
(`language: actions`), which is directly relevant here: this repository is
mostly workflow code, and past issues in this repo were exactly of that class
(for example expression interpolation into `run:` bodies and a missing
top-level `permissions:` block).

## Workaround

Until the workflow lands, each downstream repository can enable
*Settings → Code security → Code scanning → CodeQL → Default setup* by hand,
and run ``cargo audit` (or `cargo deny check advisories`)` locally. Both are per-repository manual steps that a
template is supposed to remove.

## Suggested fix

Add `.github/workflows/security.yml`:

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
  codeql:
    name: CodeQL (${{ matrix.language }})
    runs-on: ubuntu-latest
    timeout-minutes: 30
    concurrency:
      group: check-${{ github.workflow }}-${{ github.ref }}-codeql-${{ matrix.language }}
      cancel-in-progress: true
    permissions:
      contents: read
      security-events: write
    strategy:
      fail-fast: false
      matrix:
        language: [rust, actions]
    steps:
      - uses: actions/checkout@v6
      - uses: github/codeql-action/init@v3
        with:
          language: ${{ matrix.language }}
      - uses: github/codeql-action/autobuild@v3
      - uses: github/codeql-action/analyze@v3

  dependency-review:
    name: Dependency Review
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    timeout-minutes: 10
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v6
      - uses: actions/dependency-review-action@v4
        with:
          fail-on-severity: high
          comment-summary-in-pr: on-failure
```

Both are free on public repositories. Keeping this in a separate workflow file
(rather than inside `release.yml`) matches the existing separation of
`docs.yml` / `links.yml` and keeps `release.yml` from growing further.

## Cross-template status

Verified absent in **all four** pipeline templates on 2026-08-05, so this is a
fleet-wide gap rather than a single-template omission:

| Template | HEAD checked | CodeQL / dependency-review / SBOM / scanner |
|---|---|---|
| `js-ai-driven-development-pipeline-template` | `7b70923` | none |
| `rust-ai-driven-development-pipeline-template` | `c867f78` | none |
| `python-ai-driven-development-pipeline-template` | `98d6dca` | none |
| `csharp-ai-driven-development-pipeline-template` | `6806bd9` | none |

## Provenance

Found by a four-template CI/CD comparison audit run in downstream
`link-assistant/formal-ai` (report:
<https://github.com/link-assistant/formal-ai/blob/main/docs/case-studies/issue-479/template-comparison/REPORT.md>,
finding 6 / upstream recommendation 1; filed under
<https://github.com/link-assistant/formal-ai/issues/894>). The downstream repo
has the same gap and is fixing it locally as well.
