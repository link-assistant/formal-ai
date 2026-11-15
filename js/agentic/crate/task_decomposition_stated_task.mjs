// Pure port of rust/src/task_decomposition/stated_task.rs; no task execution.
import {isAlphanumeric, trimEndMatches} from '../write_str.mjs';

const SENTENCE_END = new Set(['.', '!', '?', '。', '！', '？', '।', '॥', '…']);
const QUOTE_PAIRS = [['«', '»'], ['“', '”'], ['‘', '’'], ['「', '」'],
  ['『', '』'], ['"', '"'], ["'", "'"]];

export function withoutSentenceEnd(task) {
  return trimEndMatches(task, character => SENTENCE_END.has(character) || /\s/u.test(character));
}

function askingBlocks(prompt, asks) {
  const blocks = prompt.split('\n\n').map(block => block.trim()).filter(Boolean);
  if (blocks.length < 2) return prompt.trim();
  const asking = blocks.filter(asks);
  return asking.length === 0 ? prompt.trim() : asking.join('\n\n');
}

function quotedSpan(prompt) {
  const characters = Array.from(prompt);
  const open = characters.findIndex(character => QUOTE_PAIRS.some(pair => pair[0] === character));
  if (open < 0) return null;
  const closing = QUOTE_PAIRS.find(pair => pair[0] === characters[open])[1];
  const close = characters.lastIndexOf(closing);
  if (close <= open + 1) return null;
  return characters.slice(open + 1, close).join('').trim() || null;
}

function sentenceSpans(prompt) {
  const spans = [];
  let start = 0;
  let offset = 0;
  for (const character of prompt) {
    const end = offset + character.length;
    const point = prompt.codePointAt(end);
    const next = point === undefined ? null : String.fromCodePoint(point);
    if ((SENTENCE_END.has(character) || '\n;；'.includes(character))
      && !(character === '.' && next !== null && isAlphanumeric(next))) {
      if (prompt.slice(start, offset).trim() !== '') spans.push({start, end});
      start = end;
    }
    offset = end;
  }
  if (prompt.slice(start).trim() !== '') spans.push({start, end: prompt.length});
  return spans;
}

function afterIntroducingColon(prompt, asks) {
  const asking = sentenceSpans(prompt).find(span => asks(prompt.slice(span.start, span.end).trim()));
  if (asking === undefined) return null;
  const sentence = prompt.slice(asking.start, asking.end);
  const localColon = Math.max(sentence.lastIndexOf(':'), sentence.lastIndexOf('：'));
  if (localColon < 0) return null;
  return prompt.slice(asking.start + localColon + 1).trim() || null;
}

/** The same recognizer that admitted the request scopes its task operand. */
export function statedTask(prompt, asks) {
  const stated = askingBlocks(prompt, asks);
  return withoutSentenceEnd((quotedSpan(stated)
    ?? afterIntroducingColon(stated, asks) ?? stated).trim());
}
