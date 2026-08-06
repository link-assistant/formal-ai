# Issue 894 Case Study

Issue [#894](https://github.com/link-assistant/formal-ai/issues/894) (child of
[#710](https://github.com/link-assistant/formal-ai/issues/710)) — *Complete CI
template upstream filings from the four-template audit*.

## 1. Collected Data

The audit report
[`docs/case-studies/issue-479/template-comparison/REPORT.md`](../issue-479/template-comparison/REPORT.md)
(dated 2026-06-14) compared `link-assistant/formal-ai`'s CI/CD surface against the
four `link-foundation` AI-driven-development-pipeline templates. Its closing
section, *Recommended upstream issues to file*, listed three ready-to-file
recommendations covering eight template/finding pairs. None carried an upstream
issue URL, so nothing could be verified as actually reported, and the report gave
no way to tell a still-valid recommendation from one the templates had since
fixed.

Data collected for this issue (all under `raw-data/`):

| File | Contents |
|---|---|
| `template-heads.json` | Default branch, `pushed_at`, and HEAD commit SHA/date for each of the four templates as of 2026-08-05 |
| `js-tree.txt`, `rust-tree.txt`, `python-tree.txt`, `csharp-tree.txt` | Full recursive path listings at those HEADs |
| `revalidation-greps.txt` | F1 (security scanning), F2 (`links.yml`), F3 (`workflow_run` / `head_sha` / desktop paths) |
| `revalidation-greps-2.txt` | F4 (electron-builder / macOS signing), the JS `desktop-package` job, current Rust-template line numbers, file line counts |
| `sec-*.md`, `links-*.md`, `desktop-rust.md` | The verbatim bodies of the eight issues filed upstream |
| `filed-upstream-issues.json` | Number, title, URL, state, author and creation time of each filed issue |

## 2. Requirements

See [`requirements.md`](requirements.md) (R894-1 … R894-4), mirrored in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md).

## 3. Root Cause

The 2026-06-14 audit was scoped as read-only research: it produced *drafts* of
upstream issues ("Title:", "Body:", "Check:") but stopped short of filing them,
and the report had no field in which a filing URL could live. A recommendation
with no URL and no status is indistinguishable from an unreported gap, and — since
the templates keep moving — indistinguishable from a gap that no longer exists.
Two months later four of the report's eight working-repo recommendations had in
fact been closed downstream (API-docs deploy, published-crate smoke test,
resilient buildx, main-safe concurrency) with no record of it in the report.

Nothing in CI could catch that, because no test read the report.

## 4. Implemented Design

1. **Revalidation.** Every finding was re-checked against the current template
   default branches (`js` `7b70923`, `rust` `c867f78`, `python` `98d6dca`,
   `csharp` `6806bd9`, all fetched 2026-08-05) and against the working repo's
   current `release.yml`. Commands and outputs are preserved verbatim in
   `raw-data/revalidation-greps*.txt`.
2. **Filing.** Each confirmed upstream gap was filed in its owning repository with
   a reproduction, a workaround, and a concrete suggested fix — eight issues in
   total (four security-scanning, three link-checker, one desktop-release).
3. **Ledger.** The report's *Recommended upstream issues to file* section was
   replaced by *Upstream filing status (revalidated 2026-08-05)*: a table keyed by
   finding ID with an explicit status (`confirmed` / `obsolete` /
   `not-applicable` / `local`) and, for every `confirmed` row, the filing URL. A
   second table records the working-repo findings, marking the four that closed
   since the snapshot as `obsolete`.
4. **Documentation check.** `tests/unit/docs_requirements_issue_894.rs` parses the
   ledger and fails if any `confirmed` row lacks a `link-foundation` issue URL, if
   any status word outside the vocabulary appears, if the old un-filed
   recommendation section returns, or if the preserved evidence goes missing.

## 5. Prior Art And Existing Components

- `docs/case-studies/issue-442/reported-issues.md` — the same "report the shared
  defect upstream, one row per template" table shape, from the change-detection
  audit (R3).
- `docs/case-studies/issue-711/template-audit.md` — per-template affected/not-affected
  verdict table with upstream issue and PR links.
- `REQUIREMENTS.md` R367 — the precedent that a defect reproduced in a template
  must be reported upstream *and linked from the case study*.
- The `tests/unit/docs_requirements_issue_*.rs` family — this repository's
  established mechanism for pinning a documentation obligation in CI.

## 6. Verification

```bash
cargo test --test unit docs_requirements_issue_894
```

The check is deliberately falsifiable: deleting any of the eight URLs from the
ledger, or flipping a `confirmed` row's status without providing a filing, fails
the test.

### Filed upstream issues

| ID | Repository | Issue |
|---|---|---|
| U1-js | `js-ai-driven-development-pipeline-template` | [#122](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/122) |
| U1-rust | `rust-ai-driven-development-pipeline-template` | [#115](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/115) |
| U1-python | `python-ai-driven-development-pipeline-template` | [#48](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/48) |
| U1-csharp | `csharp-ai-driven-development-pipeline-template` | [#43](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/43) |
| U2-rust | `rust-ai-driven-development-pipeline-template` | [#116](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/116) |
| U2-python | `python-ai-driven-development-pipeline-template` | [#49](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/49) |
| U2-csharp | `csharp-ai-driven-development-pipeline-template` | [#44](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/44) |
| U3-rust | `rust-ai-driven-development-pipeline-template` | [#117](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/117) |

### Findings closed without a filing

| ID | Finding | Why no filing |
|---|---|---|
| U4 | #479-analogous `workflow_run.head_sha` defect | Not present: 0 `workflow_run` / `head_sha` matches across all four templates |
| U5 | electron-builder 26 macOS signing-skip | Not present: no template uses electron-builder or signs macOS bundles |
| U6 | Action SHA-pinning | Shared, deliberate tag-pinning convention fleet-wide; no divergence to report |
| L1, L3, L4, L7 | API-docs deploy, published-crate smoke test, resilient buildx, main-safe concurrency | Closed downstream since 2026-06-14; marked `obsolete` in the ledger |
| L2, L5, L6, L8 | cargo-lock guard, link checker, multi-OS matrix, security scanning | Owned by `link-assistant/formal-ai` itself — working-repo backlog, not upstream gaps |
