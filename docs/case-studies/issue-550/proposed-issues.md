# Upstream / Third-Party Issue Analysis — Issue #550

The issue asks: *"If issue related to any other repository/project, where we can report
issues on GitHub, please do so. Each issue must contain reproducible examples,
workarounds and suggestions for fix the issue in code."* (M6)

This document records the assessment of whether any defect belongs to a
third-party/upstream project.

## Conclusion

**No third-party issues are warranted.** All five defects are in **formal-ai's own
first-party code** — its stylesheet (`src/web/styles.css`) and its thinking naturalizer
(`src/thinking.rs` / `src/web/app.js`). None is caused by a dependency or external
project, so there is no upstream maintainer to report to. The correct trackers are the
product issue **[formal-ai#550][issue]** (and its mirror **[hive-mind#1963][hm-issue]**);
the fix is **[formal-ai#551][pr]**.

## Per-defect attribution

| Defect | Owning code | Third-party involved? |
|---|---|---|
| P1 per-line fade | `src/web/styles.css` — fade attached to the wrong element by formal-ai's own #488 thinking-preview work | No |
| P2 detail clipped | `src/thinking.rs` + `src/web/app.js` — formal-ai's own 120-char cap | No |
| P3 width jump | `src/web/styles.css` — formal-ai's own leftover `width: 116px` typing-indicator clamp | No |
| P4 services theming | `src/web/styles.css` — formal-ai's own missing dark rules for the #549 services/update panel | No |
| P5 partial hover | `src/web/styles.css` — formal-ai's own per-element styling drift | No |

The browser features the fix relies on (`mask-image`, `:has()`, `:focus-visible`, and
CSS custom properties for the follow-up) are stable, widely-supported web-platform
standards (see README §6) — there is no browser bug to report; the defects were in how
formal-ai used (or failed to use) those features.

## If an upstream issue *were* needed

For completeness, the template that would be used (none is filed because none applies): a
minimal reproducible HTML/CSS snippet (the `experiments/issue-1963-harness.html` already
provides this style of repro, importing the **shipped** stylesheet), the observed vs.
expected rendering, a documented workaround, and a concrete code-level fix suggestion —
exactly the structure of the analysis comment posted on [formal-ai#550][issue].

[issue]: https://github.com/link-assistant/formal-ai/issues/550
[hm-issue]: https://github.com/link-assistant/hive-mind/issues/1963
[pr]: https://github.com/link-assistant/formal-ai/pull/551
