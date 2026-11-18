# Issue 1438: reliable reporting and source-read preservation

This requirement belongs to [PR 1437](https://github.com/link-assistant/formal-ai/pull/1437)
and [issue 1438](https://github.com/link-assistant/formal-ai/issues/1438).
Recording it is not implementation or acceptance.

- Generate an explicit, inspectable report submission command after the user selects the destination and context. Prepare the complete report through the shared renderer; do not put temporary-directory shell setup and GitHub submission into an opaque compound Bash action.
- Respect host approval refusals. Do not disguise a denied action, bypass its policy, claim submission, or retry uncertain issue creation. Keep a recoverable local report when preparation succeeded but submission failed.
- Bound process time, stdout/stderr, and temporary-directory collision attempts. Verify an actual issue URL for the selected repository before reporting success.
- Keep the selected full context and separate context links. Do not read unrelated personal session exports or publish them as regression fixtures; use minimal sanitized observations.
- The same public reproduction shows line-numbered Read output written into Rust source. Recognize provider framing separately from file content and preserve source comments, whitespace and newline identity. Reject partial, failed or ambiguous observations before whole-file replacement.
- Exercise original report choices, the captured approval refusal, successful and uncertain submission, and numbered Read counterexamples. Keep JavaScript/TypeScript and Rust behavior aligned, with all Rust compilation and native execution in CI.

Use the JavaScript Formal AI planner for small edits and larger decomposed work. Record unsupported requests and human-supplied implementation honestly; successful mechanical edits do not establish autonomous implementation authorship.
