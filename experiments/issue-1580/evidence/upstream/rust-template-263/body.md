The shared changelog scanner accepts non-ASCII whitespace after closing code fences. That can make a release heading inside an example selectable and truncate the surrounding real release section.

Observed source: [`scripts/release-content.rs` at 260ae6f](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/blob/260ae6f826d6bff0cbfe2280098c287a4c09b3f4/scripts/release-content.rs#L26). The closing condition uses `text[length..].trim().is_empty()`. Rust trimming accepts U+00A0/U+2003/U+202F and vertical whitespace; [CommonMark fence closure](https://spec.commonmark.org/0.31.2/#fenced-code-blocks) allows trailing spaces and tabs.

Add this test to the existing `tests` module:

```rust
#[test]
fn non_ascii_whitespace_does_not_close_a_code_fence() {
    for suffix in ["\u{a0}", "\u{2003}", "\u{202f}", "\u{b}", "\u{c}"] {
        let body = format!("```md\n```{suffix}\n## [9.9.9]\n```\n");
        let content = format!("## [1.2.3]\n{body}## [1.2.2]\nold");
        assert!(extract_section(&content, "9.9.9").is_err());
        assert_eq!(extract_section(&content, "1.2.3").unwrap(), body);
    }
}
```

Source-derived expected failure on the cited revision: `.trim()` accepts each suffix, closes the example early, and exposes `## [9.9.9]`. Preserve matching delimiter/minimum length checks and restrict the suffix to ASCII `b' '` or `b'\t'`. Include positive cases with spaces/tabs and both backtick/tilde delimiters.

Validation limits: I read the complete current upstream file and checked for an existing issue. I have not executed this unmodified upstream Rust test in this memory-limited development container. The downstream production parser has an ASCII-only regression registered for hosted native execution in [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581); this report does not claim an upstream runtime pass or independent Formal AI authorship. A related documentation gate has separately reproduced the false-heading acceptance with physical JavaScript fixtures.

Found during the complete pipeline comparison requested by https://github.com/link-assistant/formal-ai/issues/1580.
