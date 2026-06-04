//! Markdown README ingestion for the summarization pipeline.
//!
//! These helpers normalize GitHub README content into plain prose so that
//! [`super::formalize`] can run unmodified, and expose a [`describe_readme`]
//! shortcut that runs the full formalize → summarize → deformalize pipeline
//! over the cleaned text.

use super::{
    deformalize, formalize, summarize, to_topic, Statement, SummarizationConfig, SummarizationMode,
};

/// Strip the most common GitHub README noise from a block of Markdown text:
///
/// - HTML comments (`<!-- ... -->`),
/// - badge / shield image lines (`[![…]](…)`),
/// - heading markers (`#`, `##`, …) — the heading text survives,
/// - fenced code blocks (``` … ```),
/// - inline code fences (`` `…` `` → `…`),
/// - blockquote markers (`> `),
/// - HTML tags (kept content, dropped angle-bracket markup).
///
/// The output is plain prose suitable for [`formalize`]. The function is
/// deterministic and contains no regex engine — every transformation is a
/// linear scan of the input.
#[must_use]
pub fn strip_markdown_noise(markdown: &str) -> String {
    let no_comments = strip_html_comments(markdown);
    let mut out = String::with_capacity(no_comments.len());
    let mut in_code_block = false;
    for line in no_comments.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if is_badge_only_line(trimmed) {
            continue;
        }
        let mut without_heading = trimmed;
        while let Some(rest) = without_heading.strip_prefix('#') {
            without_heading = rest;
        }
        let without_heading = without_heading
            .strip_prefix('>')
            .unwrap_or(without_heading)
            .trim_start_matches([' ', '-', '*', '+'])
            .trim();
        if without_heading.is_empty() {
            out.push('\n');
            continue;
        }
        let unescaped = strip_inline_code_and_html(without_heading);
        out.push_str(unescaped.trim());
        out.push('\n');
    }
    out
}

fn strip_html_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' && chars.peek() == Some(&'!') && text[out.len()..].starts_with("<!--") {
            // Consume the rest of `!--`
            for _ in 0..3 {
                chars.next();
            }
            // Skip until the closing `-->`.
            let mut window: [char; 3] = [' '; 3];
            for inner in chars.by_ref() {
                window[0] = window[1];
                window[1] = window[2];
                window[2] = inner;
                if window == ['-', '-', '>'] {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn is_badge_only_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    // A badge line is a chain of one or more whitespace-separated link or
    // image segments:
    //
    //   ![alt](src)             — image
    //   [text](href)            — link (typically wrapping an image)
    //   [![alt](src)](href)     — link to an image (the canonical README badge)
    //
    // We walk segment-by-segment without a regex engine. Every segment must
    // consume a balanced `[...](...)` or `![...](...)` chunk. Anything else
    // means the line carries non-badge prose and should survive.
    let mut chars = trimmed.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if !consume_link_segment(&mut chars) {
            return false;
        }
    }
    true
}

fn consume_link_segment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    if chars.peek() == Some(&'!') {
        chars.next();
    }
    if chars.next() != Some('[') {
        return false;
    }
    // Consume the bracketed payload, which may itself contain `[...]`,
    // `(...)`, or `![...](...)` constructs.
    if !consume_until_unbalanced(chars, '[', ']') {
        return false;
    }
    if chars.next() != Some('(') {
        return false;
    }
    consume_until_unbalanced(chars, '(', ')')
}

fn consume_until_unbalanced(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    open: char,
    close: char,
) -> bool {
    let mut depth: i32 = 1;
    for ch in chars.by_ref() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return true;
            }
        }
    }
    false
}

fn strip_inline_code_and_html(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_html_tag = false;
    let mut in_inline_code = false;
    for ch in line.chars() {
        if in_html_tag {
            if ch == '>' {
                in_html_tag = false;
            }
            continue;
        }
        match ch {
            '<' => in_html_tag = true,
            '`' => in_inline_code = !in_inline_code,
            _ => out.push(ch),
        }
        let _ = in_inline_code;
    }
    out
}

/// Convert a Markdown README into [`Statement`]s.
///
/// Equivalent to `formalize(&strip_markdown_noise(markdown))` but exposed as
/// a named helper for callers (web-fetch handler, README ingestion).
#[must_use]
pub fn formalize_markdown(markdown: &str) -> Vec<Statement> {
    formalize(&strip_markdown_noise(markdown))
}

/// Describe a README block via the full pipeline.
///
/// The Markdown is normalized through [`strip_markdown_noise`] before
/// formalization, so badges and code blocks never reach the summary.
/// `repo_slug` (e.g. `link-assistant/hive-mind`) is used as the topic label
/// when the caller requests `SummarizationMode::Topic`; otherwise the
/// summarized prose is returned.
#[must_use]
pub fn describe_readme(repo_slug: &str, markdown: &str, config: &SummarizationConfig) -> String {
    let statements = formalize_markdown(markdown);
    if config.mode == SummarizationMode::Topic {
        return to_topic(repo_slug, &statements);
    }
    if statements.is_empty() {
        return String::new();
    }
    let summarized = summarize(&statements, config);
    deformalize(&summarized)
}
