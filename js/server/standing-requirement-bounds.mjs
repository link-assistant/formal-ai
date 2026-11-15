// Bounds on what dreaming may learn as a standing requirement (issue #1559):
// rust/src/dreaming/bounds.rs.
//
// A learned rule is one statement a user made, never a whole message and never
// a system or developer instruction a client delivered as the first user
// message. The rule is bounded in length, may not name one task (a repository,
// branch, issue or pull-request number, working-directory path), and its topic
// is the subject of the statement — a pronoun or function word of any supported
// language is never a topic. Every list and the length limit are data in
// `data/meta/dreaming-lexicon.lino`.

import {
  isAlphanumeric, lexicon, requirementCues, splitWhitespace, trim, trimMatches,
} from './dreaming-support.mjs';

/** The rule length limit used when the lexicon configures none. */
export const DEFAULT_RULE_MAX_CHARS = 240;

const SENTENCE_END = new Set(['.', '!', '?']);
const HARD_SENTENCE_END = new Set(['\n', '。', '！', '？', '।']);
const LEADING_WRAPPERS = new Set(['(', '[', '{', '<', '"', "'", '`', '*', '_']);
const TRAILING_WRAPPERS = new Set([')', ']', '}', '>', '"', "'", '`', ',', ';', ':', '!', '?', '.', '*', '_']);
const DIGITS = /^[0-9]+$/;
const SLUG_PART = /^[A-Za-z0-9._-]+$/;
const HEX = /^[0-9a-f]+$/;
const WHITESPACE = /\p{White_Space}/u;
const UNSPACED_SCRIPT = /[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]/u;

const charCount = (text) => [...String(text)].length;
const isWhitespace = (character) => WHITESPACE.test(character);

/** Mirrors rust/src/dreaming/bounds.rs `rule_max_chars`. */
export function ruleMaxChars() {
  const raw = lexicon().rule_max_chars.at(-1);
  const parsed = raw !== undefined && DIGITS.test(trim(raw)) ? Number(trim(raw)) : 0;
  return parsed > 0 ? parsed : DEFAULT_RULE_MAX_CHARS;
}

/** Mirrors rust/src/dreaming/bounds.rs `starts_with_opener`. */
function startsWithOpener(lowered, opener) {
  if (!opener || !lowered.startsWith(opener)) return false;
  // A CJK opener carries no word boundary; a spaced one must end on one.
  if (![...opener].some(isWhitespace)) return true;
  const next = [...lowered.slice(opener.length)][0];
  return next === undefined || !isAlphanumeric(next);
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `is_session_instruction`: the text opens
 * the way a system or developer instruction does ("You are an AI issue
 * solver…"), so it configures one session and is never a learned requirement.
 */
export function isSessionInstruction(content) {
  if (content === null || content === undefined) return false;
  const lowered = trim(String(content)).toLowerCase();
  return lexicon().instruction_openers.some((opener) => startsWithOpener(lowered, opener));
}

/** Mirrors rust/src/dreaming/bounds.rs `statement_sentences`. */
export function statementSentences(content) {
  const chars = [...String(content)];
  const sentences = [];
  let current = '';
  const flush = () => {
    const sentence = trim(current);
    if (sentence) sentences.push(sentence);
    current = '';
  };
  chars.forEach((character, index) => {
    if (HARD_SENTENCE_END.has(character)) {
      if (character !== '\n') current += character;
      flush();
      return;
    }
    current += character;
    const next = chars[index + 1];
    if (SENTENCE_END.has(character) && (next === undefined || isWhitespace(next))) flush();
  });
  flush();
  return sentences;
}

function strippedToken(raw) {
  const chars = [...raw];
  let start = 0;
  let end = chars.length;
  while (start < end && LEADING_WRAPPERS.has(chars[start])) start += 1;
  while (end > start && TRAILING_WRAPPERS.has(chars[end - 1])) end -= 1;
  return chars.slice(start, end).join('');
}

function isUrlNamingOneTask(token) {
  const at = token.indexOf('://');
  if (at <= 0) return false;
  return token.slice(at + 3).split('/').filter(Boolean).length >= 3;
}

function isPath(token) {
  if (token.startsWith('~/') || token.startsWith('./') || token.startsWith('../')) return true;
  if (token.startsWith('/') && token.slice(1).includes('/')) return true;
  const chars = [...token];
  return chars.length >= 3 && /^[A-Za-z]$/.test(chars[0]) && chars[1] === ':' && chars[2] === '\\';
}

function isRepositorySlug(token) {
  const parts = token.split('/');
  if (parts.length !== 2) return false;
  return parts.every((part) => SLUG_PART.test(part) && /[A-Za-z]/.test(part) && !part.startsWith('.'))
    && parts.some((part) => /[-_.0-9]/.test(part));
}

function hasCommitLikeSegment(token) {
  return token.split(/[^A-Za-z0-9]/).some((segment) => {
    const lowered = segment.toLowerCase();
    return lowered.length >= 7 && HEX.test(lowered) && /[0-9]/.test(lowered) && /[a-f]/.test(lowered);
  });
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `contains_task_identifier`: the text
 * names one task — a URL into a repository, a working-directory path, an
 * `owner/repo` slug, an issue or pull-request number, or a branch/commit hash.
 */
export function containsTaskIdentifier(text) {
  const references = lexicon().task_reference_words;
  const parts = [];
  for (const raw of splitWhitespace(String(text))) {
    const token = strippedToken(raw);
    if (!token) continue;
    if (isUrlNamingOneTask(token) || isPath(token) || isRepositorySlug(token) || hasCommitLikeSegment(token)) return true;
    if (token.startsWith('#') && DIGITS.test(token.slice(1))) return true;
    for (const part of token.split(/[-#/]/)) if (part) parts.push(part.toLowerCase());
  }
  return parts.some((part, index) => references.includes(part) && DIGITS.test(parts[index + 1] ?? ''));
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `is_bounded_rule`: short enough, not a
 * session instruction, and naming no single task.
 */
export function isBoundedRule(text) {
  const rule = trim(String(text ?? ''));
  return rule !== ''
    && charCount(rule) <= ruleMaxChars()
    && !isSessionInstruction(rule)
    && !containsTaskIdentifier(rule);
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `bounded_statement`: the first sentence
 * of `content` that carries a requirement cue and stays inside the bounds, or
 * null. A message opening like a session instruction yields nothing at all.
 */
export function boundedStatement(content) {
  if (content === null || content === undefined || isSessionInstruction(content)) return null;
  const cues = requirementCues();
  for (const sentence of statementSentences(content)) {
    const lowered = sentence.toLowerCase();
    if (cues.some((cue) => lowered.includes(cue)) && isBoundedRule(sentence)) return sentence;
  }
  return null;
}

/** Mirrors rust/src/dreaming/bounds.rs `is_topic_stopword`. */
export function isTopicStopword(word) {
  return lexicon().topic_stopwords.some((stopword) => stopword === word);
}

/** Mirrors rust/src/dreaming/bounds.rs `is_answer_form_cue`. */
export function isAnswerFormCue(word) {
  return lexicon().answer_form_cues.some((cue) => cue === word);
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `is_valid_topic`: a topic is not empty
 * and not made only of pronouns and function words — an answer-form verb
 * ("answer", "reply") stays a valid topic.
 */
export function isValidTopic(topic) {
  const words = splitWhitespace(String(topic ?? '').toLowerCase());
  return words.length > 0
    && words.some((word) => isAnswerFormCue(word) || !isTopicStopword(word));
}

/** Mirrors rust/src/dreaming/bounds.rs `is_unspaced_script`: a script written without spaces between words. */
function isUnspacedScript(character) {
  return UNSPACED_SCRIPT.test(character);
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `unspaced_topic`: the topic inside one
 * run of an unspaced script ("你必须总是用中文回答") — the earliest answer-form
 * verb it contains, else what is left after its leading pronouns, function
 * words and requirement cues (two or more characters).
 */
function unspacedTopic(token, cues) {
  let best = null;
  let bestAt = Infinity;
  for (const cue of lexicon().answer_form_cues) {
    const at = cue ? token.indexOf(cue) : -1;
    if (at < 0) continue;
    if (at < bestAt || (at === bestAt && cue.length > best.length)) {
      best = cue;
      bestAt = at;
    }
  }
  if (best !== null) return best;
  const prefixes = [...lexicon().topic_stopwords, ...cues].filter(Boolean);
  let rest = token;
  for (;;) {
    const longest = prefixes.reduce((found, prefix) => (rest.startsWith(prefix) && prefix.length > found.length ? prefix : found), '');
    if (!longest) break;
    rest = rest.slice(longest.length);
  }
  return charCount(rest) >= 2 ? rest : null;
}

/**
 * Mirrors rust/src/dreaming/bounds.rs `statement_topic`: the subject of the
 * statement — its first answer-form verb or first significant word (three or
 * more characters, not a number, a stopword or a requirement cue). A run of
 * an unspaced script is read by `unspacedTopic`.
 */
export function statementTopic(statement) {
  const cues = requirementCues();
  for (const raw of splitWhitespace(String(statement ?? ''))) {
    const token = trimMatches(raw, (character) => !isAlphanumeric(character)).toLowerCase();
    if (!token) continue;
    if (isAnswerFormCue(token)) return token;
    if ([...token].some(isUnspacedScript)) {
      const topic = unspacedTopic(token, cues);
      if (topic !== null) return topic;
      continue;
    }
    if (charCount(token) >= 3 && !DIGITS.test(token) && !isTopicStopword(token) && !cues.includes(token)) return token;
  }
  return null;
}
