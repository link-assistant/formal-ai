// `crate::summarization::file` (rust/src/summarization/file.rs): repository-file
// formalization and summarization. This layer adapts the statement summarizer to
// whole files: file metadata, optional meta-language parse evidence and Markdown
// fenced code blocks are kept as separate formalized records before a short
// prose summary is rendered.
//
// Representation: a `RepositoryFileFormalization` is `{path, format,
// line_count, byte_count, statements, embedded_grammars, meta_language}`, an
// embedded grammar is `{language, line_count, statement_count, meta_language}`
// and a `MetaLanguageFormalization` is `{label, syntax_link_count,
// total_link_count, has_error, text_preserved}`.
//
// Meta-language evidence. Rust parses code and data files through the
// `meta-language` links network (tree-sitter grammars compiled into the crate,
// on by default). The JavaScript root has no such parser, so it mirrors the
// crate's `#[cfg(not(feature = "meta-language"))]` build: `meta_language` is
// `null` and the "meta-language parsed it as ..." sentence is absent. A host
// that has a parser installs it with `installMetaLanguageParser` (a function
// `(label, source) => MetaLanguageFormalization`); the same
// `MAX_META_LANGUAGE_PARSE_BYTES` bound then applies as in Rust.
// summarization_meta_language.mjs builds such a parser over web-tree-sitter for
// the grammars a host loads (the vendored tree-sitter-rust), and states
// exactly which of meta-language's numbers it reproduces.

import { flattenLinoValue } from './links_format.mjs';
import {
  StatementKind, deformalize, formalize, statement, summarize,
} from './summarization.mjs';
import { formalizeMarkdown, strLines } from './summarization_markdown.mjs';

/** Mirrors `MAX_META_LANGUAGE_PARSE_BYTES` in rust/src/summarization/file.rs. */
export const MAX_META_LANGUAGE_PARSE_BYTES = 32 * 1024;
/** Mirrors `MAX_PLAIN_TEXT_FORMALIZATION_BYTES` in rust/src/summarization/file.rs. */
export const MAX_PLAIN_TEXT_FORMALIZATION_BYTES = 32 * 1024;
/** Mirrors `MAX_PLAIN_TEXT_STATEMENTS` in rust/src/summarization/file.rs. */
export const MAX_PLAIN_TEXT_STATEMENTS = 256;

const UTF8 = new TextEncoder();
const UTF8_DECODE = new TextDecoder();
const WHITE_SPACE = /^\p{White_Space}$/u;
const TRIM_BOTH = /^\p{White_Space}+|\p{White_Space}+$/gu;
const TRIM_START = /^\p{White_Space}+/u;
const TRIM_END = /\p{White_Space}+$/u;

const META = { parser: null };

/**
 * Install the host's meta-language parser, or `null` to remove it. Mirrors the
 * crate's `meta-language` cargo feature: with no parser installed the file
 * formalizer behaves like the featureless Rust build.
 * @param {((label: string, source: string) => object)|null} parser
 */
export function installMetaLanguageParser(parser) {
  META.parser = parser;
}

/**
 * Mirrors `MetaLanguageFormalization::is_valid` in rust/src/summarization/file.rs.
 * @param {{has_error: boolean, syntax_link_count: number, text_preserved: boolean}} meta
 */
export function metaLanguageIsValid(meta) {
  return !meta.has_error && meta.syntax_link_count > 0 && meta.text_preserved;
}

/** Mirrors `fn parse_with_meta_language_if_bounded` in rust/src/summarization/file.rs. */
function parseWithMetaLanguageIfBounded(label, source) {
  if (META.parser === null) return null;
  return UTF8.encode(source).length <= MAX_META_LANGUAGE_PARSE_BYTES ? META.parser(label, source) : null;
}

/**
 * Mirrors `fn formalize_repository_file` in rust/src/summarization/file.rs: an
 * arbitrary repository file into metadata, statements and embedded grammars.
 * @param {string} path
 * @param {string} content
 */
export function formalizeRepositoryFile(path, content) {
  const format = detectRepositoryFileFormat(path);
  const label = metaLanguageLabelForFormat(format);
  const metaLanguage = label === null ? null : parseWithMetaLanguageIfBounded(label, content);
  const embeddedGrammars = format === 'markdown' ? formalizeMarkdownEmbeddedGrammars(content) : [];
  const statements = statementsForFile(path, content, format);
  if (statements.length === 0) {
    statements.push(statement(`${path} is an empty ${displayFileFormat(format)} file`, StatementKind.Identity, 90));
  }
  return {
    path,
    format,
    line_count: lineCount(content),
    byte_count: UTF8.encode(content).length,
    statements,
    embedded_grammars: embeddedGrammars,
    meta_language: metaLanguage,
  };
}

/**
 * Mirrors `fn summarize_repository_file` in rust/src/summarization/file.rs.
 * @param {string} path
 * @param {string} content
 * @param {object} config a `SummarizationConfig`
 */
export function summarizeRepositoryFile(path, content, config) {
  return fileSummary(formalizeRepositoryFile(path, content), config);
}

/**
 * Mirrors `RepositoryFileFormalization::summary` /
 * `fn render_repository_file_summary` in rust/src/summarization/file.rs.
 * @param {object} formalized a `RepositoryFileFormalization`
 * @param {object} config a `SummarizationConfig`
 */
export function fileSummary(formalized, config) {
  const parts = [];
  parts.push(`${formalized.path} is a ${displayFileFormat(formalized.format)} file with ${formalized.line_count} lines and ${formalized.byte_count} bytes.`);
  const meta = formalized.meta_language;
  if (meta !== null && meta !== undefined && metaLanguageIsValid(meta)) {
    parts.push(`meta-language parsed it as ${meta.label} with ${meta.syntax_link_count} syntax links.`);
  }
  if (formalized.embedded_grammars.length > 0) {
    parts.push(`It has embedded grammar blocks: ${embeddedLanguageList(formalized.embedded_grammars)}.`);
  }
  const contentSummary = deformalize(summarize(formalized.statements, config));
  if (contentSummary !== '') parts.push(`Key content: ${contentSummary}`);
  return parts.join(' ');
}

/**
 * Mirrors `RepositoryFileFormalization::links_notation` in
 * rust/src/summarization/file.rs: compact indented Links Notation.
 * @param {object} formalized a `RepositoryFileFormalization`
 */
export function fileLinksNotation(formalized) {
  let out = 'repository_file\n';
  out += pushField(1, 'path', formalized.path);
  out += pushField(1, 'format', formalized.format);
  out += pushField(1, 'line_count', String(formalized.line_count));
  out += pushField(1, 'byte_count', String(formalized.byte_count));
  out += pushField(1, 'statement_count', String(formalized.statements.length));
  if (formalized.meta_language !== null && formalized.meta_language !== undefined) {
    out += pushMetaLanguage(1, formalized.meta_language);
  }
  for (const candidate of formalized.statements) {
    out += '  statement\n';
    out += pushField(2, 'kind', candidate.kind);
    out += pushField(2, 'weight', String(candidate.weight));
    out += pushField(2, 'text', candidate.text);
  }
  for (const embedded of formalized.embedded_grammars) {
    out += '  embedded_grammar\n';
    out += pushField(2, 'language', embedded.language);
    out += pushField(2, 'line_count', String(embedded.line_count));
    out += pushField(2, 'statement_count', String(embedded.statement_count));
    if (embedded.meta_language !== null && embedded.meta_language !== undefined) {
      out += pushMetaLanguage(2, embedded.meta_language);
    }
  }
  return out.replace(TRIM_END, '');
}

/** Mirrors `fn statements_for_file` in rust/src/summarization/file.rs. */
function statementsForFile(path, content, format) {
  if (format === 'markdown') return markdownFileStatements(content);
  if (isCodeFormat(format)) return codeStatements(path, content, format);
  if (isStructuredFormat(format)) return structuredStatements(path, content, format);
  return plainTextStatements(content);
}

/** Mirrors `fn plain_text_statements` in rust/src/summarization/file.rs. */
function plainTextStatements(content) {
  const bytes = UTF8.encode(content);
  if (bytes.length <= MAX_PLAIN_TEXT_FORMALIZATION_BYTES) return formalize(content);
  const windowBytes = Math.floor(MAX_PLAIN_TEXT_FORMALIZATION_BYTES / 2);
  const head = prefixAtCharBoundary(bytes, windowBytes);
  const tail = suffixAtCharBoundary(bytes, windowBytes);
  const perEnd = Math.floor(MAX_PLAIN_TEXT_STATEMENTS / 2);
  const statements = formalize(head).slice(0, perEnd);
  let tailStatements = formalize(tail);
  if (tailStatements.length > perEnd) tailStatements = tailStatements.slice(tailStatements.length - perEnd);
  return statements.concat(tailStatements);
}

/** `(bytes[end] & 0xC0) != 0x80`: `str::is_char_boundary` over UTF-8 bytes. */
function isCharBoundary(bytes, index) {
  return index === 0 || index >= bytes.length || (bytes[index] & 0xc0) !== 0x80;
}

/** Mirrors `fn prefix_at_char_boundary` in rust/src/summarization/file.rs. */
function prefixAtCharBoundary(bytes, maxBytes) {
  let end = Math.min(maxBytes, bytes.length);
  while (!isCharBoundary(bytes, end)) end -= 1;
  return UTF8_DECODE.decode(bytes.subarray(0, end));
}

/** Mirrors `fn suffix_at_char_boundary` in rust/src/summarization/file.rs. */
function suffixAtCharBoundary(bytes, maxBytes) {
  let start = Math.max(0, bytes.length - maxBytes);
  while (!isCharBoundary(bytes, start)) start += 1;
  return UTF8_DECODE.decode(bytes.subarray(start));
}

/** Mirrors `fn markdown_file_statements` in rust/src/summarization/file.rs. */
function markdownFileStatements(content) {
  const statements = formalizeMarkdown(content);
  for (const candidate of statements) {
    if (looksLikeHeadingFragment(candidate.text)) candidate.weight = Math.max(0, candidate.weight - 15);
  }
  return statements;
}

/** Mirrors `fn code_statements` in rust/src/summarization/file.rs. */
function codeStatements(path, content, format) {
  const statements = [statement(`${path} is a ${displayFileFormat(format).toLowerCase()} source file`, StatementKind.Identity, 90)];
  for (const symbol of extractCodeSymbols(content, format).slice(0, 8)) {
    statements.push(statement(`Defines ${symbol}.`, StatementKind.Feature, 70));
  }
  return statements;
}

/** Mirrors `fn structured_statements` in rust/src/summarization/file.rs. */
function structuredStatements(path, content, format) {
  const statements = [statement(`${path} is a ${displayFileFormat(format)} data file`, StatementKind.Identity, 90)];
  const keys = extractStructuralKeys(content);
  if (keys.length > 0) statements.push(statement(`Top-level keys: ${keys.join(', ')}.`, StatementKind.Feature, 70));
  return statements;
}

/** Mirrors `fn formalize_markdown_embedded_grammars` in rust/src/summarization/file.rs. */
function formalizeMarkdownEmbeddedGrammars(markdown) {
  const blocks = [];
  let active = null;
  for (const line of strLines(markdown)) {
    const trimmed = line.replace(TRIM_START, '');
    if (active !== null) {
      if (isClosingFence(trimmed, active.marker)) {
        blocks.push(formalizeFencedBlock(active));
        active = null;
      } else {
        active.source += `${line}\n`;
      }
      continue;
    }
    const marker = openingFenceMarker(trimmed);
    if (marker !== null) active = { marker, language: fenceLanguage(trimmed), source: '' };
  }
  if (active !== null) blocks.push(formalizeFencedBlock(active));
  return blocks;
}

/** Mirrors `fn formalize_fenced_block` in rust/src/summarization/file.rs. */
function formalizeFencedBlock(block) {
  const language = normalizeLanguageLabel(block.language);
  const symbols = isCodeFormat(language) ? extractCodeSymbols(block.source, language) : [];
  const label = metaLanguageLabelForFormat(language);
  const metaLanguage = label === null ? null : parseWithMetaLanguageIfBounded(label, block.source);
  return {
    language,
    line_count: lineCount(block.source),
    statement_count: symbols.length,
    meta_language: metaLanguage,
  };
}

/** Mirrors `fn opening_fence_marker` in rust/src/summarization/file.rs. */
function openingFenceMarker(trimmedLine) {
  const chars = Array.from(trimmedLine);
  const ch = chars[0];
  if (ch !== '`' && ch !== '~') return null;
  let len = 0;
  while (len < chars.length && chars[len] === ch) len += 1;
  return len >= 3 ? { ch, len } : null;
}

/** Mirrors `fn is_closing_fence` in rust/src/summarization/file.rs. */
function isClosingFence(trimmedLine, opening) {
  const closing = openingFenceMarker(trimmedLine);
  if (closing === null) return false;
  if (closing.ch !== opening.ch || closing.len < opening.len) return false;
  return trimmedLine.slice(closing.len).replace(TRIM_BOTH, '') === '';
}

/** Mirrors `fn fence_language` in rust/src/summarization/file.rs. */
function fenceLanguage(trimmedLine) {
  const marker = openingFenceMarker(trimmedLine);
  if (marker === null) return 'text';
  const infoString = trimmedLine.slice(marker.len).replace(TRIM_BOTH, '');
  if (marker.ch === '`' && infoString.includes('`')) return 'text';
  let end = 0;
  const chars = Array.from(infoString);
  while (end < chars.length && !(WHITE_SPACE.test(chars[end]) || [',', ';', '{'].includes(chars[end]))) end += 1;
  const first = chars.slice(0, end).join('');
  return first === '' ? 'text' : first;
}

/**
 * Mirrors `fn detect_repository_file_format` in rust/src/summarization/file.rs.
 * @param {string} path
 */
export function detectRepositoryFileFormat(path) {
  const lower = path.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  const fileName = pathFileName(lower) ?? '';
  switch (fileName) {
    case 'cargo.toml': case 'pyproject.toml': return 'toml';
    case 'package.json': case 'tsconfig.json': case 'package-lock.json': case 'bun.lock': return 'json';
    case 'dockerfile': return 'dockerfile';
    default: break;
  }
  switch (pathExtension(fileName)) {
    case 'md': case 'markdown': case 'mdown': return 'markdown';
    case 'rs': return 'rust';
    case 'js': case 'mjs': case 'cjs': case 'jsx': return 'javascript';
    case 'ts': case 'tsx': return 'typescript';
    case 'py': return 'python';
    case 'go': return 'go';
    case 'java': return 'java';
    case 'c': case 'h': return 'c';
    case 'cc': case 'cpp': case 'cxx': case 'hpp': case 'hh': return 'cpp';
    case 'cs': return 'csharp';
    case 'rb': return 'ruby';
    case 'json': return 'json';
    case 'yaml': case 'yml': return 'yaml';
    case 'toml': return 'toml';
    case 'html': case 'htm': return 'html';
    case 'css': return 'css';
    case 'xml': case 'svg': return 'xml';
    case 'ini': return 'ini';
    case 'sh': case 'bash': return 'shell';
    case 'lino': return 'links_notation';
    default: return 'text';
  }
}

/**
 * `Path::file_name` on a Unix path: the last component, ignoring empty and `.`
 * components, and `null` for a path ending in `..`.
 * @param {string} path
 */
function pathFileName(path) {
  const components = path.split('/').filter((part) => part !== '' && part !== '.');
  const last = components[components.length - 1];
  return last === undefined || last === '..' ? null : last;
}

/**
 * `Path::extension` of a file name: the text after the last `.`, absent when
 * there is no `.` or the only one starts the name (`.bashrc`).
 * @param {string} fileName
 */
function pathExtension(fileName) {
  const at = fileName.lastIndexOf('.');
  if (at <= 0) return '';
  return fileName.slice(at + 1);
}

/** Mirrors `fn normalize_language_label` in rust/src/summarization/file.rs. */
function normalizeLanguageLabel(label) {
  const lower = label.replace(TRIM_BOTH, '').replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  switch (lower) {
    case 'rs': return 'rust';
    case 'js': case 'mjs': case 'cjs': case 'jsx': return 'javascript';
    case 'ts': case 'tsx': return 'typescript';
    case 'py': return 'python';
    case 'c++': return 'cpp';
    case 'c#': case 'cs': return 'csharp';
    case 'md': return 'markdown';
    case '': return 'text';
    default: return lower;
  }
}

/** Mirrors `fn meta_language_label_for_format` in rust/src/summarization/file.rs. */
function metaLanguageLabelForFormat(format) {
  switch (format) {
    case 'rust': case 'javascript': case 'typescript': case 'python': case 'go': case 'java':
    case 'c': case 'cpp': case 'csharp': case 'ruby': case 'json': case 'yaml': case 'toml':
    case 'html': case 'css': case 'xml': case 'ini':
      return format;
    default: return null;
  }
}

/** Mirrors `fn is_code_format` in rust/src/summarization/file.rs. */
function isCodeFormat(format) {
  return ['rust', 'javascript', 'typescript', 'python', 'go', 'java', 'c', 'cpp', 'csharp', 'ruby'].includes(format);
}

/** Mirrors `fn is_structured_format` in rust/src/summarization/file.rs. */
function isStructuredFormat(format) {
  return ['json', 'yaml', 'toml', 'ini', 'links_notation', 'xml', 'html', 'css'].includes(format);
}

/**
 * Mirrors `fn display_file_format` in rust/src/summarization/file.rs.
 * @param {string} format
 */
export function displayFileFormat(format) {
  switch (format) {
    case 'markdown': return 'Markdown';
    case 'rust': return 'Rust';
    case 'javascript': return 'JavaScript';
    case 'typescript': return 'TypeScript';
    case 'python': return 'Python';
    case 'go': return 'Go';
    case 'java': return 'Java';
    case 'c': return 'C';
    case 'cpp': return 'C++';
    case 'csharp': return 'C#';
    case 'ruby': return 'Ruby';
    case 'json': return 'JSON';
    case 'yaml': return 'YAML';
    case 'toml': return 'TOML';
    case 'html': return 'HTML';
    case 'css': return 'CSS';
    case 'xml': return 'XML';
    case 'ini': return 'INI';
    case 'shell': return 'shell';
    case 'links_notation': return 'Links Notation';
    case 'dockerfile': return 'Dockerfile';
    default: return 'text';
  }
}

/** Mirrors `fn extract_code_symbols` in rust/src/summarization/file.rs. */
function extractCodeSymbols(content, format) {
  const symbols = [];
  for (const line of strLines(content)) {
    const trimmed = line.replace(TRIM_START, '');
    if (trimmed.startsWith('//') || trimmed.startsWith('#') || trimmed.startsWith('*')) continue;
    const symbol = symbolFromLine(trimmed, format);
    if (symbol !== null) pushUnique(symbols, symbol);
  }
  return symbols;
}

/** Mirrors `fn symbol_from_line` in rust/src/summarization/file.rs. */
function symbolFromLine(line, format) {
  switch (format) {
    case 'rust': return rustSymbol(line);
    case 'javascript': case 'typescript': return jsSymbol(line);
    case 'python': return labelled(prefixedSymbol(line, 'def ') ?? prefixedSymbol(line, 'class '), 'python symbol');
    case 'go': return labelled(prefixedSymbol(line, 'func '), 'go function');
    case 'java': case 'csharp': return classLikeSymbol(line);
    case 'c': case 'cpp': return cLikeSymbol(line);
    case 'ruby': return labelled(prefixedSymbol(line, 'def ') ?? prefixedSymbol(line, 'class '), 'ruby symbol');
    default: return null;
  }
}

/** `name.map(|name| format!("{label} {name}"))`. */
function labelled(name, label) {
  return name === null ? null : `${label} ${name}`;
}

/** Mirrors `fn rust_symbol` in rust/src/summarization/file.rs. */
function rustSymbol(line) {
  const stripped = stripLeadingWords(line, ['pub', 'async', 'unsafe', 'const']);
  const keywords = [
    ['fn ', 'function'], ['struct ', 'struct'], ['enum ', 'enum'], ['trait ', 'trait'],
    ['mod ', 'module'], ['type ', 'type'], ['const ', 'constant'], ['static ', 'static'],
  ];
  for (const [keyword, label] of keywords) {
    const name = prefixedSymbol(stripped, keyword);
    if (name !== null) return `rust ${label} ${name}`;
  }
  if (!stripped.startsWith('impl ')) return null;
  return labelled(firstIdentifier(stripped.slice('impl '.length)), 'rust impl');
}

/** Mirrors `fn js_symbol` in rust/src/summarization/file.rs. */
function jsSymbol(line) {
  const stripped = stripLeadingWords(line, ['export', 'default', 'async']);
  return labelled(prefixedSymbol(stripped, 'function '), 'javascript function')
    ?? labelled(prefixedSymbol(stripped, 'class '), 'javascript class')
    ?? labelled(prefixedSymbol(stripped, 'const ') ?? prefixedSymbol(stripped, 'let ') ?? prefixedSymbol(stripped, 'var '), 'javascript binding');
}

/** Mirrors `fn class_like_symbol` in rust/src/summarization/file.rs. */
function classLikeSymbol(line) {
  const stripped = stripLeadingWords(line, ['public', 'private', 'protected', 'internal', 'static', 'sealed']);
  return labelled(prefixedSymbol(stripped, 'class '), 'class')
    ?? labelled(prefixedSymbol(stripped, 'interface '), 'interface')
    ?? labelled(prefixedSymbol(stripped, 'enum '), 'enum');
}

/** Mirrors `fn c_like_symbol` in rust/src/summarization/file.rs. */
function cLikeSymbol(line) {
  if (!line.includes('(') || line.endsWith(';')) return null;
  const beforeParen = line.slice(0, line.indexOf('(')).replace(TRIM_END, '');
  const tokens = beforeParen.split(/\p{White_Space}+/u).filter(Boolean);
  const name = tokens[tokens.length - 1];
  return name !== undefined && isIdentifier(name) ? `function ${name}` : null;
}

/** Mirrors `fn prefixed_symbol` in rust/src/summarization/file.rs. */
function prefixedSymbol(line, prefix) {
  return line.startsWith(prefix) ? firstIdentifier(line.slice(prefix.length)) : null;
}

/** Mirrors `fn first_identifier` in rust/src/summarization/file.rs. */
function firstIdentifier(text) {
  const chars = Array.from(text);
  let start = 0;
  while (start < chars.length && !isIdentifierStart(chars[start])) start += 1;
  let end = start;
  while (end < chars.length && (isAsciiAlphanumeric(chars[end]) || chars[end] === '_')) end += 1;
  const candidate = chars.slice(start, end).join('');
  return candidate === '' ? null : candidate;
}

/** Mirrors `fn strip_leading_words` in rust/src/summarization/file.rs. */
function stripLeadingWords(line, wordsToStrip) {
  let rest = line.replace(TRIM_START, '');
  for (;;) {
    let changed = false;
    for (const word of wordsToStrip) {
      if (rest.startsWith(word)) {
        const afterWord = rest.slice(word.length);
        const first = Array.from(afterWord)[0];
        if (first !== undefined && WHITE_SPACE.test(first)) {
          rest = afterWord.replace(TRIM_START, '');
          changed = true;
        }
      }
    }
    if (!changed) return rest;
  }
}

/** Mirrors `fn extract_structural_keys` in rust/src/summarization/file.rs. */
function extractStructuralKeys(content) {
  const keys = [];
  for (const line of strLines(content)) {
    const trimmed = line.replace(TRIM_BOTH, '');
    const key = quotedKey(trimmed) ?? bareKey(trimmed);
    if (key !== null) pushUnique(keys, key);
    if (keys.length >= 8) break;
  }
  return keys;
}

/** Mirrors `fn quoted_key` in rust/src/summarization/file.rs. */
function quotedKey(line) {
  if (!line.startsWith('"')) return null;
  const rest = line.slice(1);
  const close = rest.indexOf('"');
  if (close < 0) return null;
  return rest.slice(close + 1).replace(TRIM_START, '').startsWith(':') ? rest.slice(0, close) : null;
}

/** Mirrors `fn bare_key` in rust/src/summarization/file.rs. */
function bareKey(line) {
  const colon = line.indexOf(':');
  const equals = line.indexOf('=');
  const at = colon < 0 ? equals : equals < 0 ? colon : Math.min(colon, equals);
  if (at < 0) return null;
  const trimmed = line.slice(0, at).replace(TRIM_BOTH, '');
  const bare = trimmed !== '' && Array.from(trimmed).every((ch) => isAsciiAlphanumeric(ch) || ch === '_' || ch === '-' || ch === '.');
  return bare ? trimmed : null;
}

/** Mirrors `fn embedded_language_list` in rust/src/summarization/file.rs. */
function embeddedLanguageList(blocks) {
  const languages = [];
  for (const block of blocks) pushUnique(languages, block.language);
  return languages.join(', ');
}

/** Mirrors `fn push_unique` in rust/src/summarization/file.rs. */
function pushUnique(values, candidate) {
  if (!values.includes(candidate)) values.push(candidate);
}

/** Mirrors `fn line_count` in rust/src/summarization/file.rs. */
function lineCount(content) {
  return content === '' ? 0 : strLines(content).length;
}

/** Mirrors `fn looks_like_heading_fragment` in rust/src/summarization/file.rs. */
function looksLikeHeadingFragment(text) {
  return text.split(/\p{White_Space}+/u).filter(Boolean).length <= 6 && !hasTerminalPunctuation(text);
}

/** Mirrors `fn has_terminal_punctuation` in rust/src/summarization/file.rs. */
function hasTerminalPunctuation(text) {
  const last = Array.from(text).pop();
  return last !== undefined && ['.', '!', '?', ':', '。', '…'].includes(last);
}

/** Mirrors `fn is_identifier` in rust/src/summarization/file.rs. */
function isIdentifier(text) {
  const chars = Array.from(text);
  return chars.length > 0 && isIdentifierStart(chars[0])
    && chars.slice(1).every((ch) => isAsciiAlphanumeric(ch) || ch === '_');
}

/** Mirrors `fn is_identifier_start` in rust/src/summarization/file.rs. */
function isIdentifierStart(ch) {
  return /^[A-Za-z_]$/.test(ch);
}

/** `char::is_ascii_alphanumeric`. */
function isAsciiAlphanumeric(ch) {
  return /^[0-9A-Za-z]$/.test(ch);
}

/** Mirrors `fn push_meta_language` in rust/src/summarization/file.rs. */
function pushMetaLanguage(indent, meta) {
  let out = `${'  '.repeat(indent)}meta_language\n`;
  out += pushField(indent + 1, 'label', meta.label);
  out += pushField(indent + 1, 'syntax_link_count', String(meta.syntax_link_count));
  out += pushField(indent + 1, 'total_link_count', String(meta.total_link_count));
  out += pushField(indent + 1, 'has_error', String(meta.has_error));
  out += pushField(indent + 1, 'text_preserved', String(meta.text_preserved));
  return out;
}

/** Mirrors `fn push_field` (with `write_indent`) in rust/src/summarization/file.rs. */
export function pushField(indent, name, value) {
  return `${'  '.repeat(indent)}${name} ${flattenLinoValue(value)}\n`;
}
