Parent: #710

Audit verdict: still-broken.

Current evidence
- docs/case-studies/issue-479/template-comparison/REPORT.md preserves a file-by-file comparison against all four pipeline templates.
- Several cross-template and desktop workflow gaps were left as ready-to-file recommendations without upstream issue URLs.

Acceptance
- Revalidate every report finding against current template default branches.
- File each confirmed gap in the owning upstream repository with a reproduction, workaround, and suggested fix.
- Link every filing from the report and mark obsolete findings explicitly.
- Add a documentation check that no confirmed finding remains ready-to-file without a URL.
