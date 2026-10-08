// `crate::summarization::dialog` (rust/src/summarization/dialog.rs):
// dialog-aware helpers for the summarization pipeline. A `DialogTurn` is
// `{role, text}`; the role is informational except for the weight bias that
// lets user turns dominate a short summary or a chat title.

import {
  SummarizationMode, defaultConfig, deformalize, formalize, isLabelOnly, labelForMode, maxByWeight,
  summarize, toTopic, withLanguage, withMode,
} from './summarization.mjs';
import { stripMarkdownNoise } from './summarization_markdown.mjs';

const TRIM_BOTH = /^\p{White_Space}+|\p{White_Space}+$/gu;
const WORDS = /\p{White_Space}+/u;
const SENTENCE_ENDS = new Set(['.', '!', '?', '。', '！', '？']);
const WORD_TAIL = new Set(['.', ',', ';', ':', '!', '?', '。', '！', '？']);

/** Mirrors `DialogTurn::new` in rust/src/summarization/dialog.rs. */
export function dialogTurn(role, text) {
  return { role, text };
}

/** Mirrors `DialogTurn::user` in rust/src/summarization/dialog.rs. */
export function userTurn(text) {
  return dialogTurn('user', text);
}

/** Mirrors `DialogTurn::assistant` in rust/src/summarization/dialog.rs. */
export function assistantTurn(text) {
  return dialogTurn('assistant', text);
}

/**
 * Mirrors `fn formalize_dialog` in rust/src/summarization/dialog.rs: every
 * turn formalized on its own, user weights raised by 20 and assistant weights
 * lowered by 10, clamped to 0..=100.
 * @param {Array<{role: string, text: string}>} turns
 */
export function formalizeDialog(turns) {
  const out = [];
  for (const turn of turns) {
    let bias = 0;
    if (turn.role === 'user') bias = 20;
    else if (turn.role === 'assistant') bias = -10;
    for (const candidate of formalize(turn.text)) {
      candidate.weight = Math.min(100, Math.max(0, candidate.weight + bias));
      out.push(candidate);
    }
  }
  return out;
}

/**
 * Mirrors `fn summarize_dialog` in rust/src/summarization/dialog.rs.
 * @param {Array<{role: string, text: string}>} turns
 * @param {object} config a `SummarizationConfig`
 */
export function summarizeDialog(turns, config) {
  const statements = formalizeDialog(turns);
  if (isLabelOnly(config.mode)) {
    const highest = maxByWeight(statements);
    return highest === null ? '' : labelForMode(config.mode, toTopic('', [highest]));
  }
  if (statements.length === 0) return '';
  return deformalize(summarize(statements, config));
}

/** ASCII-case-insensitive equality (`str::eq_ignore_ascii_case`). */
function eqIgnoreAsciiCase(left, right) {
  const fold = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  return fold(left) === fold(right);
}

/**
 * Mirrors `fn summarize_dialog_plain` in rust/src/summarization/dialog.rs: the
 * latest user turn's first sentence plus the following assistant turn's first
 * sentence, Markdown removed, bounded to `maxWords` and `maxSentences`.
 * @param {Array<{role: string, text: string}>} turns
 * @param {number} maxWords
 * @param {number} maxSentences
 */
export function summarizeDialogPlain(turns, maxWords, maxSentences) {
  if (maxWords === 0 || maxSentences === 0) return '';
  let userIndex = -1;
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    if (eqIgnoreAsciiCase(turns[index].role, 'user')) {
      userIndex = index;
      break;
    }
  }
  if (userIndex < 0) return '';
  const sentences = [];
  const goal = plainFirstSentence(turns[userIndex].text);
  if (goal !== null) sentences.push(goal);
  if (maxSentences > 1) {
    let status = null;
    for (let index = turns.length - 1; index > userIndex; index -= 1) {
      if (eqIgnoreAsciiCase(turns[index].role, 'assistant')) {
        status = plainFirstSentence(turns[index].text);
        break;
      }
    }
    if (status !== null && (sentences.length === 0 || !eqIgnoreAsciiCase(sentences[0], status))) {
      sentences.push(status);
    }
  }
  return boundPlainWords(sentences.join(' '), maxWords);
}

/** Mirrors `fn plain_first_sentence` in rust/src/summarization/dialog.rs. */
function plainFirstSentence(text) {
  const cleaned = stripMarkdownNoise(text);
  const withoutMarkers = Array.from(cleaned).filter((character) => !'#`*'.includes(character)).join('');
  let plain = withoutMarkers.split(WORDS).filter(Boolean).join(' ');
  for (const separator of [' — ', ' – ']) {
    const at = plain.indexOf(separator);
    if (at >= 0) plain = plain.slice(0, at);
  }
  const chars = Array.from(plain);
  let boundary = null;
  for (let index = 0; index < chars.length; index += 1) {
    const character = chars[index];
    const next = chars[index + 1];
    const nextIsBoundary = next === undefined || WORDS.test(next);
    if (['。', '！', '？'].includes(character) || (['.', '!', '?'].includes(character) && nextIsBoundary)) {
      boundary = index + 1;
      break;
    }
  }
  if (boundary !== null) plain = chars.slice(0, boundary).join('');
  plain = plain.replace(TRIM_BOTH, '');
  if (plain === '') return null;
  return SENTENCE_ENDS.has(Array.from(plain).pop()) ? plain : `${plain}.`;
}

/** Mirrors `fn bound_plain_words` in rust/src/summarization/dialog.rs. */
function boundPlainWords(text, maxWords) {
  const wordsOfText = text.split(WORDS).filter(Boolean);
  if (wordsOfText.length <= maxWords) return text;
  const bounded = Array.from(wordsOfText.slice(0, maxWords).join(' '));
  while (bounded.length > 0 && WORD_TAIL.has(bounded[bounded.length - 1])) bounded.pop();
  return `${bounded.join('')}.`;
}

/**
 * Mirrors `fn generate_chat_title` in rust/src/summarization/dialog.rs: the
 * dialog summarized in `Topic` mode (at most five words).
 * @param {Array<{role: string, text: string}>} turns
 * @param {string} language
 */
export function generateChatTitle(turns, language) {
  return summarizeDialog(turns, withLanguage(withMode(defaultConfig(), SummarizationMode.Topic), language));
}
