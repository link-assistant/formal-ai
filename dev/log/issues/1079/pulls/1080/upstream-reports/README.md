# Upstream reports filed from issue #1079

Issue #1079 asked to find every false positive, false negative, warning and
error in this repository's CI/CD, to compare **all** files against the five
`link-foundation/*-ai-driven-development-pipeline-template` repositories, and
to file an issue upstream wherever the same defect exists in a template. The
task description added that each report must carry a reproducible example, a
workaround and a code-level fix.

Two of the defects found here reproduce in the templates unchanged. Comparing
in the other direction -- template feature by template feature, rather than
only looking for this repository's bugs upstream -- surfaced three more that
this repository does not have, because it never adopted the gap.

Everything below was reproduced locally before filing, at the template commits
snapshotted in `../references/templates/*-template.HEAD`. Transcripts are in
`../analysis/`.

| # | Defect | Filed | Body |
|---|---|---|---|
| 1 | `scraper 0.21` resolves `selectors 0.26`, which pulls in the unmaintained `fxhash` (RUSTSEC-2025-0057, `patched = []`); `scraper 0.25` is the first release that clears it, and it compiles with no source changes | [web-capture#155](https://github.com/link-assistant/web-capture/issues/155) | `web-capture-scraper-fxhash.md` |
| 2 | `.github/zizmor.yml` declares `'*': hash-pin`, but that policy configures `unpinned-uses`, which only sees *action* references; container references belong to `unpinned-images`, which is Pedantic-persona and never runs — so `docker://rhysd/actionlint:1.7.7` is a mutable third-party tag executing with the repository checked out, under a written policy forbidding exactly that | [rust#165](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/165), [js#177](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/177), [python#75](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/75), [php#5](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/5), [csharp#52](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/52) | `templates-unpinned-actionlint-image.md` |
| 3 | `cargo audit --file Cargo.lock` exits 0 on `unmaintained`, `unsound` and `yanked` findings — it prints `warning: N allowed warnings found` — so the dependency-audit gate is green by construction for that entire class, including yanked releases | [rust#164](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/164) | `rust-template-cargo-audit-deny-warnings.md` |
| 4 | The zizmor job leaves `version:` at its default and two templates document reproducing with `zizmor==1.30.0`, which the action cannot install: it resolves versions from a static table shipped inside itself, and v0.6.2's table stops at 1.29.0 (its `latest` row is the same digest as its `1.29.0` row) | [rust#166](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/166), [js#178](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/178), [python#76](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/76), [php#6](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/6) | `templates-zizmor-version-mismatch.md` |
| 5 | The csharp template is the only one of five with no zizmor job and no `.github/zizmor.yml`; running the other four templates' invocation on it reports 4 high-severity `template-injection` findings in `release.yml` (`workflow_dispatch` inputs interpolated into `run:` blocks in jobs holding `GITHUB_TOKEN` and `NUGET_API_KEY`) and 2 high-severity workflow-level `excessive-permissions` in `docs.yml` | [csharp#53](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/53) | `csharp-template-missing-zizmor.md` |
| 6 | The php template has no `.github/workflows/security.yml` at all — no CodeQL, no dependency audit, no Dependency Review, no weekly schedule — and nothing elsewhere covers it | [php#7](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/7) | `php-template-missing-security-workflow.md` |

## Which templates each defect affects

Checked rather than assumed. The measurement for defect 2 is the narrow
pedantic pass (`--persona pedantic --min-severity high --min-confidence high`);
the transcript is `../analysis/zizmor-narrow-pedantic-templates.log`.

| | rust | js | python | php | csharp |
|---|---|---|---|---|---|
| ships `.github/workflows/security.yml` | yes | yes | yes | **no (defect 6)** | yes |
| CodeQL, source language | `rust` | `javascript-typescript` | `python` | — | `csharp` |
| CodeQL, `actions` | yes | yes | yes | — | yes |
| dependency audit | `cargo audit` (**defect 3**) | `npm audit --audit-level=high` | `scripts/audit_dependencies.py` | — | none |
| Dependency Review | yes | yes | yes | — | yes |
| ships `.github/zizmor.yml` + zizmor job | yes | yes | yes | yes | **no (defect 5)** |
| zizmor `version:` pinned | **no (defect 4)** | **no (defect 4)** | **no (defect 4)** | **no (defect 4)** | n/a |
| comment documents an unrunnable version | **1.30.0** | — | **1.30.0** | — | n/a |
| unpinned actionlint image | `workflows.yml:31` | `workflows.yml:46` | `workflows.yml:42` | `workflows.yml:42` | `workflows.yml:42` |
| other unpinned images | — | `example-app.yml:231` `mcr.microsoft.com/playwright:v1.59.1-noble` | — | — | — |
| pedantic findings / surfaced by the narrow pass | 54 / **1** | 59 / **2** | 39 / **1** | 20 / **1** | 73 / **7** |

The last row is the argument for the fix rather than a one-line edit: on four
of five templates the entire cost of enforcing the declared hash-pin policy on
images is one step that reports exactly this defect and nothing else.

## Defects found here that the templates do **not** have

Checked in the same direction, so the absence is a measurement rather than an
assumption. None was filed, because none reproduces upstream.

| Defect here | Checked | Result |
|---|---|---|
| **D1** — the `Auto Release` job blocks because the release cycle contains no merged Formal-AI-authored pull request | whether any template has a self-development release gate | none does; `scripts/self-development-loop.rs` and the `Formal-AI-*` commit trailers are specific to this repository. D1 is a true positive about this repository's own state, not a pipeline defect — see the top-level README |
| **D8** — the pull-request evidence gate reports a finding whose only remedy the ruleset forbids | whether any template ships a commit-trailer evidence gate | none does. `grep -rl 'Formal-AI\|interpret-trailers\|trailer'` over all five snapshotted trees returns nothing: the `Formal-AI-*` trailers, `scripts/self-hosting-metric.rs` and `scripts/self-development-loop.rs` are specific to this repository, so neither the gate nor its deadlock exists upstream |
| **D9** — an indented `Formal-AI-*:` line parsed as a declared trailer | the same grep, plus whether any template parses commit messages at all | same result: no template reads commit trailers, so there is no parser upstream to carry the bug |
| **D2's second half** — a yanked crate in `Cargo.lock` (`chacha20 0.10.1`) | the rust template's `Cargo.lock` at `4d444d97`, audited both ways with cargo-audit 0.22.2 | 45 crates, no findings either way. The *mechanism* (defect 3) is present upstream; the instance is not. That is why the report is framed as latent — the gate will report success the first time it matters, with no signal that anything changed |

## Defects the templates already fixed and this repository had not adopted

The comparison runs both ways, and this direction is the reason issue #1079
asks for it.

| Template practice | State here before this PR | Now |
|---|---|---|
| actionlint via `docker://rhysd/actionlint` (the image bundles ShellCheck, so `run:` blocks are actually linted) | already adopted in #1076 — and this repository added a ShellCheck canary fixture on top, which no template has, because a bare binary silently skips every `run:` check and still exits 0 | kept, and both references to the image now carry the same digest |
| `persist-credentials: false` on `actions/checkout` | 46 of 48 checkout sites do not set it (`artipacked`, Low confidence, below both configured gates) | adopted — the per-site review was done and is recorded in [`../analysis/artipacked-sweep.md`](../analysis/artipacked-sweep.md). 44 of the 48 sites drop the credential; the four jobs that push with it keep it behind a comment naming the push. Measured 46 findings → 4 |

## Cross-references filed on the issues themselves

- csharp#52 ↔ csharp#53: the unpinned-image finding on that template is not
  persona-suppressed, it is never looked for, so the two fixes are worth
  landing together.
