# Issue 1395: useful reactions and relevant practical answers

This requirement belongs to [PR 1437](https://github.com/link-assistant/formal-ai/pull/1437)
and [issue 1395](https://github.com/link-assistant/formal-ai/issues/1395).

The screenshot is a practical request answered with unrelated fragments about a fictional character. Handle informal wording respectfully and interpret the requested action before selecting a lookup strategy. Generalize from semantic roles and source-backed knowledge; do not memorize the complete screenshot prompt or an exact answer in production code.

- Give actionable, relevant help or an honest targeted clarification. Do not turn unrelated ranked search snippets into an answer.
- Keep search rank, algorithm parameters and dependent copies of the same source separate from factual confidence and independent corroboration. User-facing replies should not expose irrelevant ranking machinery.
- Prefer trusted sources for practical or health guidance; maintain provenance for the minimal cached knowledge and support discovery without that cache.
- Preserve explicit search and definition requests, quotation, negation, and unrelated tasks as counterexamples.
- Matching loops must advance or stop. Reject empty match operands consistently in JavaScript and Rust, with a regression that fails within a finite execution timeout.
- Verify browser/JavaScript, TypeScript and Rust parity, including fresh-source, cached-source and unavailable-source behavior. Rust runs only in CI for this work.
