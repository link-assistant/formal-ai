---
bump: patch
---

### Changed
- The four-template CI/CD audit report
  (`docs/case-studies/issue-479/template-comparison/REPORT.md`) no longer ends with
  drafted-but-unfiled recommendations. Every finding was revalidated against the
  current template default branches (2026-08-05) and the *Recommended upstream
  issues to file* section is replaced by an *Upstream filing status* ledger that
  carries a status (`confirmed` / `obsolete` / `not-applicable` / `local`) and, for
  every confirmed row, the upstream issue URL. Eight issues were filed upstream —
  CI security scanning in all four templates, the `links.yml` broken-link checker in
  the Rust/Python/C# templates, and an optional desktop-release workflow in the Rust
  template — each with a reproduction, a workaround and a suggested fix. Findings
  that closed since the June snapshot (API-docs deploy, published-crate smoke test,
  resilient buildx, main-safe concurrency) are marked obsolete with the evidence
  that closed them (issue #894).

### Added
- `tests/unit/docs_requirements_issue_894.rs` parses the filing ledger and fails
  when a `confirmed` finding carries no upstream issue URL, when a status outside
  the documented vocabulary appears, when the pre-filing recommendation section
  returns, or when the preserved revalidation evidence goes missing (R894-4).
