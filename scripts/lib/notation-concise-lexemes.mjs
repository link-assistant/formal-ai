// The concise lexeme rule of the links notation passes (PR #1188, R1188-U7;
// docs/links-notation-style.md).
//
// A lexeme whose surfaces each hold one word and the same fields is written on
// one line, its words after the language, and the shared fields once below it:
//
//   lexeme en                          lexeme en actor
//     surface                    ==>     part_of_speech noun
//       text actor                       grammatical_number singular
//       part_of_speech noun
//       grammatical_number singular
//
// A long word list continues on `words` lines. The seed parsers of both roots
// (`expandConciseLexemes` in js/seed_loader.js, `expand_concise_lexemes` in
// rust/src/seed/parser.rs) expand it back to the long form before they parse,
// so every reader sees the same tree; experiments/formal_ai_subagent/
// apply-notation-rules.mjs only writes a conversion that parses to the same
// tree as the original.

/** A concise lexeme line longer than this continues on `words` lines. */
export const LINE_LIMIT = 120;

const leading = (line) => /^ */u.exec(line)[0].length;

/** The line without its comment, by the parser's own rule. */
export function withoutComment(line) {
  let quote = null;
  let previousWasSpace = true;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quote !== null) {
      if ((quote === '"' || quote === '`') && character === '\\') {
        index += 1;
      } else if (quote === "'" && character === "'" && line[index + 1] === "'") {
        index += 1;
      } else if (character === quote) {
        quote = null;
      }
      continue;
    }
    if (character === '"' || character === "'" || character === '`') {
      quote = character;
      previousWasSpace = false;
      continue;
    }
    if (character === '#' && previousWasSpace) {
      return line.slice(0, index);
    }
    previousWasSpace = /\s/u.test(character);
  }
  return line;
}

const isBlank = (line) => withoutComment(line).trim() === '';

/** Whether a raw value is exactly one word token: one quoted span or one bare word. */
export function isOneWord(raw) {
  if (/^"(?:[^"\\]|\\.|"")*"$/u.test(raw) || /^'(?:[^'\\]|\\.|'')*'$/u.test(raw) || /^`(?:[^`\\]|\\.|``)*`$/u.test(raw)) {
    return true;
  }
  // A bare word; the raw-reference prefixes decode several tokens and stay long.
  return /^[^\s"'`#]+$/u.test(raw) && raw !== 'codepoints' && raw !== 'unformalized-raw';
}

/** The direct child blocks of `lines[start..end)`: `{depth, lines}` each. */
function childBlocks(lines, start, end) {
  const children = [];
  let childIndent = -1;
  for (let index = start; index < end; index += 1) {
    if (isBlank(lines[index])) {
      continue;
    }
    const depth = leading(lines[index]);
    if (childIndent === -1) {
      childIndent = depth;
    }
    if (depth <= childIndent || children.length === 0) {
      children.push({ depth, lines: [lines[index]] });
    } else {
      children[children.length - 1].lines.push(lines[index]);
    }
  }
  return children;
}

/**
 * The concise form of one long lexeme block, or null when it has none: every
 * child a `surface` with one single-word `text` and the same field lines, no
 * deeper structure, and no comment on a `text` line but the only one.
 * @param {string} indent the lexeme line's indentation
 * @param {string} language
 * @param {Array<{depth: number, lines: Array<string>}>} surfaces
 * @returns {Array<string> | null}
 */
export function conciseLexeme(indent, language, surfaces) {
  if (surfaces.length === 0) {
    return null;
  }
  const words = [];
  const comments = [];
  let fields = null;
  for (const surface of surfaces) {
    if (withoutComment(surface.lines[0]).trim() !== 'surface' || surface.lines[0].includes('#')) {
      return null;
    }
    const inner = childBlocks(surface.lines, 1, surface.lines.length);
    if (inner.some((child) => child.lines.length !== 1)) {
      return null;
    }
    const texts = inner.filter((child) => /^text(?:\s|$)/u.test(child.lines[0].trim()));
    if (texts.length !== 1 || inner.indexOf(texts[0]) !== 0) {
      return null;
    }
    const textLine = texts[0].lines[0];
    const raw = withoutComment(textLine).trim().slice('text'.length).trim();
    if (!isOneWord(raw)) {
      return null;
    }
    words.push(raw);
    const comment = textLine.slice(withoutComment(textLine).length).trim();
    if (comment !== '') {
      comments.push(comment);
    }
    const own = inner.slice(1).map((child) => child.lines[0].trim());
    if (fields !== null && own.join('\n') !== fields.join('\n')) {
      return null;
    }
    fields = own;
  }
  if (comments.length > 1 || (comments.length === 1 && words.length > 1)) {
    return null;
  }
  const step = surfaces[0].depth - indent.length;
  const childIndent = indent + ' '.repeat(step);
  const head = `${indent}lexeme ${language}`;
  const lines = [];
  const single = `${head} ${words.join(' ')}${comments.length === 1 ? ` ${comments[0]}` : ''}`;
  if (single.length <= LINE_LIMIT) {
    lines.push(single);
  } else {
    lines.push(head);
    let current = `${childIndent}words`;
    for (const word of words) {
      if (current.length + 1 + word.length > LINE_LIMIT && current !== `${childIndent}words`) {
        lines.push(current);
        current = `${childIndent}words`;
      }
      current += ` ${word}`;
    }
    lines.push(current);
  }
  for (const field of fields) {
    lines.push(`${childIndent}${field}`);
  }
  return lines;
}

/**
 * Rewrite every long lexeme block of a document that has a concise form.
 * @param {string} text
 * @returns {{text: string, converted: number}}
 */
export function conciseLexemes(text) {
  const lines = text.split('\n');
  const output = [];
  let converted = 0;
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    const head = /^( *)lexeme[ \t]+([^\s#]+)\s*$/u.exec(line);
    if (!head) {
      output.push(line);
      index += 1;
      continue;
    }
    const indent = head[1].length;
    let end = index + 1;
    while (end < lines.length && (isBlank(lines[end]) || leading(lines[end]) > indent)) {
      end += 1;
    }
    while (end > index + 1 && isBlank(lines[end - 1])) {
      end -= 1;
    }
    const block = lines.slice(index + 1, end);
    const children = childBlocks(lines, index + 1, end);
    const hasCommentLines = block.some((entry) => entry.trim().startsWith('#') || entry.trim() === '');
    const concise = hasCommentLines ? null : conciseLexeme(head[1], head[2], children);
    if (concise === null) {
      output.push(line);
      index += 1;
      continue;
    }
    output.push(...concise);
    converted += 1;
    index = end;
  }
  return { text: output.join('\n'), converted };
}
