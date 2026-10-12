// `crate::summarization::markdown` (rust/src/summarization/markdown.rs):
// Markdown README ingestion for the summarization pipeline. The helpers
// normalize GitHub README content into plain prose so `formalize` runs
// unmodified, and `describeReadme` runs the whole formalize -> summarize ->
// deformalize pipeline over the cleaned text. Every transformation is a linear
// scan; no regex engine decides what is noise.

import { agenticMessage } from '../messages.mjs';
import {
  deformalize, formalize, isLabelOnly, labelForMode, summarize, toTopic,
} from './summarization.mjs';

const WHITE_SPACE = /^\p{White_Space}$/u;
const TRIM_START = /^\p{White_Space}+/u;
const TRIM_BOTH = /^\p{White_Space}+|\p{White_Space}+$/gu;
const UTF8 = new TextEncoder();

/**
 * Rust `str::lines`: split on `\n`, drop one `\r` before each `\n`, and no empty
 * line after a trailing newline. (A final line without `\n` keeps a trailing
 * `\r`, as `split_inclusive('\n')` + `strip_suffix` does.)
 * Rust built-in `str::lines`.
 * @param {string} text
 * @returns {Array<string>}
 */
export function strLines(text) {
  const parts = text.split('\n');
  const lastIsTerminator = parts[parts.length - 1] === '';
  if (lastIsTerminator) parts.pop();
  const lastIndex = parts.length - 1;
  return parts.map((line, index) => {
    const hadNewline = index < lastIndex || lastIsTerminator;
    return hadNewline && line.endsWith('\r') ? line.slice(0, -1) : line;
  });
}

/**
 * Mirrors `fn strip_markdown_noise` in rust/src/summarization/markdown.rs:
 * HTML comments, badge lines, heading markers, fenced code blocks, inline code
 * fences, blockquote markers and HTML tags removed; the prose survives.
 * @param {string} markdown
 * @returns {string}
 */
export function stripMarkdownNoise(markdown) {
  const noComments = stripHtmlComments(markdown);
  const out = [];
  let inCodeBlock = false;
  for (const line of strLines(noComments)) {
    const trimmed = line.replace(TRIM_START, '');
    if (trimmed.startsWith('```') || trimmed.startsWith('~~~')) {
      inCodeBlock = !inCodeBlock;
      continue;
    }
    if (inCodeBlock) continue;
    if (isBadgeOnlyLine(trimmed)) continue;
    let withoutHeading = trimmed;
    while (withoutHeading.startsWith('#')) withoutHeading = withoutHeading.slice(1);
    if (withoutHeading.startsWith('>')) withoutHeading = withoutHeading.slice(1);
    let start = 0;
    while (start < withoutHeading.length && ' -*+'.includes(withoutHeading[start])) start += 1;
    withoutHeading = withoutHeading.slice(start).replace(TRIM_BOTH, '');
    if (withoutHeading === '') {
      out.push('\n');
      continue;
    }
    out.push(`${stripInlineCodeAndHtml(withoutHeading).replace(TRIM_BOTH, '')}\n`);
  }
  return out.join('');
}

/**
 * Mirrors `fn strip_html_comments` in rust/src/summarization/markdown.rs. The
 * Rust source tests `text[out.len()..].starts_with("<!--")`, i.e. it compares
 * at the BYTE length of the output built so far, which only equals the read
 * position until a comment has been removed; this keeps that offset arithmetic
 * (and the Rust panic when it lands inside a multi-byte character).
 */
function stripHtmlComments(text) {
  const chars = Array.from(text);
  const bytes = UTF8.encode(text);
  const opener = [0x3c, 0x21, 0x2d, 0x2d];
  const out = [];
  let outBytes = 0;
  let index = 0;
  while (index < chars.length) {
    const character = chars[index];
    index += 1;
    if (character === '<' && chars[index] === '!') {
      if (outBytes < bytes.length && (bytes[outBytes] & 0xc0) === 0x80) {
        throw new Error(agenticMessage('summarization_char_boundary', { index: outBytes }));
      }
      if (opener.every((byte, offset) => bytes[outBytes + offset] === byte)) {
        index += 3;
        const window = [' ', ' ', ' '];
        while (index < chars.length) {
          const inner = chars[index];
          index += 1;
          window[0] = window[1];
          window[1] = window[2];
          window[2] = inner;
          if (window[0] === '-' && window[1] === '-' && window[2] === '>') break;
        }
        continue;
      }
    }
    out.push(character);
    outBytes += UTF8.encode(character).length;
  }
  return out.join('');
}

/** Mirrors `fn is_badge_only_line` in rust/src/summarization/markdown.rs. */
function isBadgeOnlyLine(line) {
  const trimmed = line.replace(TRIM_BOTH, '');
  if (trimmed === '') return false;
  const cursor = { chars: Array.from(trimmed), at: 0 };
  while (cursor.at < cursor.chars.length) {
    if (WHITE_SPACE.test(cursor.chars[cursor.at])) {
      cursor.at += 1;
      continue;
    }
    if (!consumeLinkSegment(cursor)) return false;
  }
  return true;
}

/** Mirrors `fn consume_link_segment` in rust/src/summarization/markdown.rs. */
function consumeLinkSegment(cursor) {
  if (cursor.chars[cursor.at] === '!') cursor.at += 1;
  const open = cursor.chars[cursor.at];
  cursor.at += 1;
  if (open !== '[') return false;
  if (!consumeUntilUnbalanced(cursor, '[', ']')) return false;
  const paren = cursor.chars[cursor.at];
  cursor.at += 1;
  if (paren !== '(') return false;
  return consumeUntilUnbalanced(cursor, '(', ')');
}

/** Mirrors `fn consume_until_unbalanced` in rust/src/summarization/markdown.rs. */
function consumeUntilUnbalanced(cursor, open, close) {
  let depth = 1;
  while (cursor.at < cursor.chars.length) {
    const character = cursor.chars[cursor.at];
    cursor.at += 1;
    if (character === open) {
      depth += 1;
    } else if (character === close) {
      depth -= 1;
      if (depth === 0) return true;
    }
  }
  return false;
}

/** Mirrors `fn strip_inline_code_and_html` in rust/src/summarization/markdown.rs. @param {string} line */
export function stripInlineCodeAndHtml(line) {
  const out = [];
  let inHtmlTag = false;
  for (const character of line) {
    if (inHtmlTag) {
      if (character === '>') inHtmlTag = false;
      continue;
    }
    if (character === '<') inHtmlTag = true;
    else if (character !== '`') out.push(character);
  }
  return out.join('');
}

/**
 * Mirrors `fn formalize_markdown` in rust/src/summarization/markdown.rs:
 * `formalize(stripMarkdownNoise(markdown))`.
 * @param {string} markdown
 */
export function formalizeMarkdown(markdown) {
  return formalize(stripMarkdownNoise(markdown));
}

/**
 * Mirrors `fn describe_readme` in rust/src/summarization/markdown.rs: a README
 * through the full pipeline; a label-only mode returns the repo slug (or its
 * identifier).
 * @param {string} repoSlug
 * @param {string} markdown
 * @param {object} config a `SummarizationConfig`
 */
export function describeReadme(repoSlug, markdown, config) {
  const statements = formalizeMarkdown(markdown);
  if (isLabelOnly(config.mode)) return labelForMode(config.mode, toTopic(repoSlug, statements));
  if (statements.length === 0) return '';
  return deformalize(summarize(statements, config));
}
