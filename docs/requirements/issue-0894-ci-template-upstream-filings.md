## Issue #894 CI Template Upstream Filings

Issue [#894](https://github.com/link-assistant/formal-ai/issues/894) (child of
[#710](https://github.com/link-assistant/formal-ai/issues/710)) closes the
four-template CI/CD audit
(`docs/case-studies/issue-479/template-comparison/REPORT.md`), which ended with
*drafted* upstream issues and no field in which a filing URL could live. A
recommendation with no URL is indistinguishable from an unreported gap and, once
the templates move on, from a gap that no longer exists. The revalidation
evidence and the filed issue bodies live in `docs/case-studies/issue-894/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R894-1 | Revalidate every audit finding against the current template default branches before acting on it. | Implemented: all four templates re-fetched 2026-08-05 (`js` `7b70923`, `rust` `c867f78`, `python` `98d6dca`, `csharp` `6806bd9`); commands and verbatim output preserved in `docs/case-studies/issue-894/raw-data/revalidation-greps.txt` and `revalidation-greps-2.txt`; what moved since the snapshot is recorded in the report's *What the 2026-08-05 revalidation changed* subsection. |
| R894-2 | File each confirmed gap in the owning upstream repository with a reproduction, a workaround, and a suggested fix. | Implemented: eight issues filed — security scanning (js#122, rust#115, python#48, csharp#43), `links.yml` port (rust#116, python#49, csharp#44), optional desktop-release workflow (rust#117). Bodies preserved verbatim as `docs/case-studies/issue-894/raw-data/sec-*.md`, `links-*.md`, `desktop-rust.md`; the created issues as `filed-upstream-issues.json`. |
| R894-3 | Link every filing from the report and mark obsolete findings explicitly. | Implemented: the report's *Recommended upstream issues to file* section is replaced by the *Upstream filing status (revalidated 2026-08-05)* ledger — every `confirmed` row carries its filing URL, U4/U5/U6 are `not-applicable` with the reason, and L1/L3/L4/L7 (API-docs deploy, published-crate smoke test, resilient buildx, main-safe concurrency) are `obsolete` with the evidence that closed them. |
| R894-4 | A confirmed finding may never remain ready-to-file without a URL, enforced by a documentation check. | Implemented: `tests/unit/docs_requirements_issue_894.rs` parses the ledger and fails when a `confirmed` row has no `link-foundation` issue URL, when a status outside the documented vocabulary (`confirmed` / `obsolete` / `not-applicable` / `local`) appears, when the pre-filing recommendation section returns, or when the preserved evidence goes missing. |
