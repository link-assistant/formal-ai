# Issue 1589 work plan

Scope: mixed calendar months/years; full years/months between dates in English and Russian; exact Gregorian day offsets; currency provenance and cent rounding; derivations in thinking_steps; shared JavaScript/native regressions. Related issues are context, with no required sub-issues.

- [x] Read issue, all comments, PR conversation/inline/reviews and repository guidance.
- [x] Merge main, inspect recent related PR 1596 and reproduce six original probes.
- [ ] Draft seed-driven duration parsing and exact calendar arithmetic, then native twin.
- [ ] Bind currency conversion to captured rate/source/date and round exactly to cents.
- [ ] Add shared per-requirement and fresh-home HTTP regressions, including thinking_steps.
- [ ] Run focused JavaScript tests sequentially, formatting/projection checks and relevant local gates; no local Rust compilation.
- [ ] Collect latest-head CI runs/timestamps/logs; fix all actionable failures before each push.
- [ ] Review complete PR diff; commit all changes, update PR title/body and mark ready after green checks.

Frozen reasoning units (before new outcomes): calendar quantity binding; whole-period anniversary semantics; unsupported/ambiguous input handling; exact currency rounding; rate provenance; cross-runtime/API verification. Attempt Formal AI reasoning on whole-period semantics; independently assess the complete response. Supplied conclusions/procedures receive zero autonomous reasoning credit.

Development boundary: bootstrap discovery and the driver adapter were supplied by Codex. The small source-repair request returned SourceRepairOperatorUnavailable without reading or changing a file. Subsequent supplied recipes are executed through real JavaScript Formal AI tools and earn zero autonomous code-authorship credit. Retain failures, tool receipts and source identities. No subagents are used.
