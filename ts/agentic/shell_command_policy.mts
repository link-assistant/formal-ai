// Sentence scoping and command-policy classification for shell routing: a
// port of rust/src/agentic_coding/shell_command_policy.rs.
//
// A `Sentence` is `{text, span: {start, end}}`; spans are UTF-16 code-unit
// indices into the prompt (so `prompt.slice(start, end)` is the sentence with
// its terminator), the JavaScript twin of the Rust byte range.

import { carriesPolicyLead } from './crate/seed_caller_context.mjs';
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';
import { isAlphanumeric, splitOnce, splitWhitespace, toAsciiLowercase, trim, trimMatches, trimStart } from './crate/rust_str.mjs';

const SHELL_ENDS = new Set(['.', '!', '?', ';', '\n', '。', '！', '？', '；', '।']);
const PROSE_ENDS = new Set(['.', '!', '?', '\n', '。', '！', '？', '।']);

/** Mirrors `fn sentences` (a semicolon ends a sentence). */
export function sentences(prompt) {
  return splitSentences(prompt, (character) => SHELL_ENDS.has(character));
}

/** Mirrors `fn prose_sentences` (a semicolon joins clauses). */
export function proseSentences(prompt) {
  return splitSentences(prompt, (character) => PROSE_ENDS.has(character));
}

/** Mirrors `fn split_sentences`. */
function splitSentences(prompt, endsSentence) {
  const out = [];
  let spanStart = 0;
  let start = 0;
  let index = 0;
  for (const character of prompt) {
    const at = index;
    index += character.length;
    if (!endsSentence(character)) continue;
    if (character === '.') {
      const next = Array.from(prompt.slice(index))[0];
      if (next !== undefined && isAlphanumeric(next)) continue;
    }
    const text = trim(prompt.slice(start, at));
    const end = index;
    if (text) {
      out.push({ text, span: { start: spanStart, end } });
      spanStart = end;
    }
    start = end;
  }
  const tail = trim(prompt.slice(start));
  if (tail) out.push({ text: tail, span: { start: spanStart, end: prompt.length } });
  return out;
}

/** Mirrors `fn sentence_spans`: each sentence's text. */
export function sentenceSpans(prompt) {
  return sentences(prompt).map((sentence) => sentence.text);
}

/** Mirrors `fn states_a_command_policy`. */
export function statesACommandPolicy(sentence) {
  const lower = sentence.toLowerCase();
  if (!carriesPolicyLead(lower)) return false;
  const split = splitOnce(lower, ',');
  return split === null || !ordersANamedCommand(split[1]);
}

/** Mirrors `fn orders_a_named_command`. */
function ordersANamedCommand(clause) {
  const vocab = terminalCommandVocabulary();
  const words = splitWhitespace(clause);
  return words.some((word, index) => vocab.run_verbs.includes(normalizeCommandWord(word))
    && words[index + 1] !== undefined && vocab.shell_tokens.includes(normalizeCommandWord(words[index + 1])));
}

/** Mirrors `fn governs_commands_rather_than_requesting_one`. */
export function governsCommandsRatherThanRequestingOne(prompt) {
  const spans = sentenceSpans(prompt);
  return spans.length > 0 && spans.every(statesACommandPolicy);
}

/**
 * Mirrors `fn command_span`: the command text after a passthrough prefix
 * (`Run`), past an optional colon. A leading code span is the command and the
 * words after it are prose (``Run `node --test x.test.js` and tell me …``,
 * PR #1188 G76); with no leading span, a remainder whose backticks do not pair
 * is no command (the shell would read a command substitution), so null.
 * @param {string} remainder
 */
export function commandSpan(remainder) {
  const text = trimStart(remainder.startsWith(':') ? remainder.slice(1) : remainder);
  const fence = /^`+/u.exec(text)?.[0] ?? '';
  if (fence !== '') {
    const close = text.indexOf(fence, fence.length);
    const inner = close < 0 ? '' : trim(text.slice(fence.length, close));
    if (inner !== '' && !inner.includes('`')) return inner;
  }
  return shellQuotesPaired(text) ? text : null;
}

/** Mirrors `fn named_shell_command_in_sentence`. */
export function namedShellCommandInSentence(prompt, vocab) {
  const lower = toAsciiLowercase(prompt);
  const hasPhrase = vocab.terminal_phrases.some((phrase) => lower.includes(phrase));
  const hasCjkVerb = vocab.cjk_run_verbs.some((verb) => lower.includes(verb));
  const words = splitWhitespace(prompt);
  const isRunVerb = (word) => vocab.run_verbs.includes(normalizeCommandWord(word));
  const isShellToken = (word) => {
    const normalized = normalizeCommandWord(word);
    return normalized !== '' && vocab.shell_tokens.includes(normalized);
  };
  const hasVerb = words.some(isRunVerb) || hasCjkVerb;
  for (let index = 1; index < words.length; index += 1) {
    if (isShellToken(words[index]) && isRunVerb(words[index - 1])) return collectCommand(words.slice(index));
  }
  // Shape 2: a shell token named as a command -- written as code, or right
  // after a seeded command noun (`run the command ls`) -- given run context.
  // A shell word elsewhere in the sentence is prose: `Create a Python test
  // … and run it` names no `python` command (PR #1188 G25).
  const namedAsCommand = (word, index) => isShellToken(word)
    && (word.startsWith('`') || (index > 0 && vocab.command_nouns.includes(toAsciiLowercase(words[index - 1]))));
  if (hasVerb || hasPhrase) {
    const word = words.find(namedAsCommand);
    if (word !== undefined) return normalizeCommandWord(word);
  }
  return null;
}

/** Mirrors `fn collect_command`. */
function collectCommand(words) {
  const parts = [normalizeCommandWord(words[0])];
  for (const word of words.slice(1)) {
    if (isProseWord(word)) break;
    const trimmed = trimMatches(word, (character) => character === '`' || character === ',' || character === '.');
    if (!trimmed) break;
    parts.push(trimmed);
  }
  return parts.join(' ');
}

/** Mirrors `fn normalize_command_word`. */
export function normalizeCommandWord(word) {
  const bare = trimMatches(word, (character) => character === '`');
  const head = /^[0-9A-Za-z_-]*/.exec(bare)[0];
  return toAsciiLowercase(head);
}

const PROSE_WORDS = new Set([
  'command', 'commands', 'to', 'in', 'into', 'on', 'the', 'a', 'an', 'and', 'then', 'please', 'for', 'of', 'that',
  'which', 'so', 'this', 'these', 'those', 'here', 'there', 'me', 'us', 'you', 'it', 'from', 'at', 'with', 'will',
  'would', 'can', 'could', 'should', 'using', 'via', 'inside', 'within', 'output', 'result', 'results', 'contents',
  'content', 'directory', 'folder', 'folders', 'file', 'files', 'currently', 'again', 'also', 'just', 'now',
  'nothing', 'anything', 'everything', 'something', 'all', 'any', 'every', 'each', 'no',
]);

/** Mirrors `fn is_prose_word`. */
export function isProseWord(word) {
  const normalized = toAsciiLowercase(trimMatches(word, (character) => !/^[0-9A-Za-z]$/.test(character)));
  return PROSE_WORDS.has(normalized);
}

/** Mirrors `fn shell_quotes_paired`; the shell remains the syntax authority. */
export function shellQuotesPaired(command) {
  let quote = null;
  let escaped = false;
  for (const character of command) {
    if (quote === "'") {
      if (character === quote) quote = null;
    } else if (escaped) {
      escaped = false;
    } else if (character === '\\') {
      escaped = true;
    } else if (quote !== null) {
      if (character === quote) quote = null;
    } else if (["'", '"', '`'].includes(character)) {
      quote = character;
    }
  }
  return quote === null && !escaped;
}
