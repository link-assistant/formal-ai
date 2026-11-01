# Main qualification follow-up

PR #1502 merged as 6d370dd9300f68b100478b03e186c3f73b0cef3c after 34 passing PR checks. Main self-development/debt checks passed. Native CI then rejected unused search-observation receipt/error fields, previously consumed by the diagnostic-prefix implementation. [Task Ladder log](https://github.com/link-assistant/formal-ai/actions/runs/41325824871) and [Write-Effect Ladder log](https://github.com/link-assistant/formal-ai/actions/runs/41325824991) identify the same root cause.

The discovery query evidence now retains validated search gaps and transport errors in JavaScript, TypeScript and Rust. This makes the existing fields useful for diagnosing incomplete searches without warning suppression or weakening custody. The bounded symlink regression asserts those observed diagnostics.

The JavaScript edit was made by Formal AI from an English literal-replacement request, first in scratch and then in the maintained source. Its exact diff was independently checked. The final message incorrectly claimed the file was unchanged: the edit-verification route returned null when no Run tool was offered, allowing an unrelated whole-file guard to answer. Both runtimes now return a terminal explicit verification-capability gap instead. Supplied literal replacements and externally authored engine/native bootstrap work receive zero autonomous synthesis credit. No independent task-acceptance claim follows from the edit.

No local Rust compilation/tests, product WASM or Docker builds were run. Merged-main and release qualification remain pending.

## Further failures included in this batch

Existing typed Read and content-search ownership now precede the new file-command fallback. Captured line-slice and quoted-folder search failures keep their original dedicated behavior; shell-only #1501 requests remain covered. The internal coding source-model boundary returns SourceGrammarFeatureUnavailable when the optional parser feature is disabled. The public library and its governed source identities stay unchanged; default builds re-export the real parser through that same internal boundary. No source verification is fabricated. A new manifest error is a machine diagnostic instead of unseeded English prose.

## Temporary style-lint debt

Main Clippy exposed more than 100 style diagnostics across the preceding source-model changes. Per the maintainer's release-first instruction, default CI temporarily defers the pedantic, nursery, style and complexity categories. Compiler warnings plus Clippy correctness, suspicious-code and performance categories remain fatal. No test or original check is deleted: FORMAL_AI_STRICT_STYLE_LINTS=1 restores the full strict command, and examples still receive their original check. Restore strict style by correcting the recorded public diagnostics, running the strict profile in CI and removing the opt-out flags after it passes. [Original diagnostic run](https://github.com/link-assistant/formal-ai/actions/runs/41325824909/job/124905799760). This is disclosed technical debt, not full style qualification.
