// Command head and free-text payload in the browser worker.
//
// The twin of the payload readers of rust/src/solver_handlers/text_rewrite.rs
// (`command_colon`, `free_text_payload`, `command_head`), moved out of
// formal_ai_worker_text_transform.js so the summarizer, the text transforms
// and the handler-rule `transform` value source
// (formal_ai_worker_handler_rule_values.js) read one payload. A payload is the
// text after a command colon, after the first line break, or inside the first
// double or guillemet quotes, of at least three words; a CJK character counts
// as a word, since CJK text has no spaces between words (R1188-U20).

/** The colons that end a command head: ASCII and the CJK full-width one. */
const TEXT_PAYLOAD_COLONS = /[:：]/gu;

/**
 * The words of a payload: its whitespace-separated tokens, where a token
 * written in a CJK script counts each of its CJK characters (Rust
 * `payload_word_count`).
 * @param {string} text
 * @returns {number}
 */
function textTransformPayloadWordCount(text) {
  let count = 0;
  for (const token of textTransformWords(text)) {
    const characters = Array.from(token).filter((character) => containsCjk(character)).length;
    count += Math.max(1, characters);
  }
  return count;
}


/**
 * The colon separating a command head (at most twelve words) from a payload
 * (at least three words); -1 when none. Time, ratio and URL colons are skipped.
 * The full-width colon of CJK text separates them the same way (R1188-U20).
 * @param {string} prompt
 * @returns {number}
 */
function textTransformCommandColon(prompt) {
  for (const colon of prompt.matchAll(TEXT_PAYLOAD_COLONS)) {
    const index = colon.index;
    const before = prompt.slice(0, index);
    const beforeIsDigit = /[0-9]$/.test(before);
    const afterIsDigit = /^[0-9]/.test(prompt.slice(index + 1));
    if (beforeIsDigit && afterIsDigit) continue;
    if (prompt.slice(index + 1, index + 3) === "//") continue;
    if (textTransformWords(before).length > 12) continue;
    if (textTransformPayloadWordCount(prompt.slice(index + 1).trim()) < 3) continue;
    return index;
  }
  return -1;
}

/**
 * Strip one matched pair of surrounding quotes, any kind.
 * @param {string} text
 * @returns {string}
 */
function textTransformStripOuterQuotes(text) {
  const pairs = [['"', '"'], ["«", "»"], ["'", "'"]];
  for (const pair of pairs) {
    if (text.length >= 2 && text.startsWith(pair[0]) && text.endsWith(pair[1])) {
      const body = text.slice(1, -1);
      if (body.length > 0) return body;
    }
  }
  return text;
}

/**
 * The first double-quoted or guillemet-quoted span, or null.
 * @param {string} text
 * @returns {string|null}
 */
function textTransformDoubleQuoted(text) {
  const pairs = [['"', '"'], ["«", "»"]];
  for (const pair of pairs) {
    const start = text.indexOf(pair[0]);
    if (start === -1) continue;
    const rest = text.slice(start + 1);
    const end = rest.indexOf(pair[1]);
    if (end > 0) return rest.slice(0, end);
  }
  return null;
}

/**
 * Mirrors `free_text_payload`: the text after a command colon, after the
 * first newline, or inside the first double quotes — at least three words.
 * @param {string} prompt
 * @returns {string|null}
 */
function textTransformFreeTextPayload(prompt) {
  const colon = textTransformCommandColon(prompt);
  if (colon !== -1) {
    const tail = textTransformStripOuterQuotes(prompt.slice(colon + 1).trim());
    if (textTransformPayloadWordCount(tail) >= 3) return tail;
  }
  const newline = prompt.indexOf("\n");
  if (newline !== -1) {
    const rest = textTransformStripOuterQuotes(prompt.slice(newline + 1).trim());
    if (textTransformPayloadWordCount(rest) >= 3) return rest;
  }
  const quoted = textTransformDoubleQuoted(prompt);
  if (quoted !== null && textTransformPayloadWordCount(quoted) >= 3) return quoted;
  return null;
}

/**
 * The command head: the text before the command colon, the first newline, or
 * the first quote — whichever comes first.
 * @param {string} prompt
 * @returns {string}
 */
function textTransformCommandHead(prompt) {
  let cut = textTransformCommandColon(prompt);
  const markers = [prompt.indexOf("\n"), prompt.indexOf('"'), prompt.indexOf("«")];
  for (const marker of markers) {
    if (marker > 0) cut = cut === -1 ? marker : Math.min(cut, marker);
  }
  return cut === -1 ? prompt : prompt.slice(0, cut);
}
