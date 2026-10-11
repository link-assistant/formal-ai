// Text a request names by its value (PR #1188 dogfooding): a configuration
// setting given a new value -- the line assigning a key, the value it holds,
// and the value a request states it holds now -- and a line replaced whole.
// The JavaScript twin of rust/src/agentic_coding/workspace_setting.rs.

import { cleanCueToken, tokens } from './write_request.mjs';
import { usesPostpositions } from './crate/language.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { meaningsWithRole } from './crate/seed_meanings.mjs';
import { wordScopedMatches } from './crate/workspace_change_learning.mjs';
import { matchIndices, splitWhitespace, trim } from './write_str.mjs';

/**
 * Mirrors `fn stated_old_value`: `{key, old}` when the old clause states the
 * value it changes (`K from A`, `K с A`, `K को A की जगह`) -- the seeded
 * `file_edit_old_lead_cue` outside quotes, the value on the side its
 * language's adposition governs -- or null.
 */
export function statedOldValue(clause) {
  const segments = quotedSegmentSpans(clause);
  const toks = tokens(clause);
  const quoted = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  let found = null;
  for (const meaning of meaningsWithRole('file_edit_old_lead_cue')) {
    for (const lexeme of meaning.lexemes) {
      const postpositional = usesPostpositions(lexeme.language);
      for (const word of lexeme.words) {
        const parts = splitWhitespace(word.text.toLowerCase());
        if (parts.length === 0) continue;
        for (let index = 0; index + parts.length <= toks.length; index += 1) {
          const run = toks.slice(index, index + parts.length);
          if (run.every((token, offset) => !quoted(token) && cleanCueToken(token.text) === parts[offset])
            && (found === null || run[0].start > found.start)) {
            found = { start: run[0].start, end: run[run.length - 1].end, postpositional };
          }
        }
      }
    }
  }
  if (found === null) return null;
  let key = clause.slice(0, found.start);
  let old = clause.slice(found.end);
  if (found.postpositional) {
    const words = splitWhitespace(key);
    if (words.length < 2) return null;
    old = words[words.length - 1];
    key = words.slice(0, -1).join(' ');
  }
  old = literalValue(old);
  key = trim(key);
  return key === '' || old === '' ? null : { key, old };
}

/** Mirrors `fn literal_value`: a span that is one quoted literal is its text. */
function literalValue(span) {
  const text = trim(span);
  const segments = quotedSegmentSpans(text);
  return segments.length === 1 && segments[0].start === 0 && segments[0].end === text.length ? segments[0].text : text;
}

/** A value every config format writes bare: a boolean, null or a number. */
const isBareLiteral = (value) => /^(true|false|null|-?\d+(\.\d+)?)$/u.test(value);

/** Mirrors `fn unquoted_value`: a value without the one pair of quotes around it. */
function unquotedValue(value) {
  const first = value[0];
  return value.length >= 2 && (first === '"' || first === "'") && value.endsWith(first) ? value.slice(1, -1) : value;
}

/**
 * Mirrors `fn assigned_setting`: the one line assigning `key` (holding
 * `held`, when the request stated it) with `value`, in the file's own
 * quoting. A key assigned nowhere changes the stated value where it occurs
 * once as a word; otherwise null.
 */
export function assignedSetting(source, key, value, target, held = null) {
  const lines = source.split('\n');
  const assigned = lines.map((line, index) => [index, assignment(line, key)]).filter(([, found]) => found !== null);
  const matches = assigned.filter(([, [, old]]) => held === null || unquotedValue(old) === held);
  if (matches.length !== 1) {
    if (held === null || assigned.length > 0) return null;
    const at = wordScopedMatches(source, held);
    return at.length === 1 ? `${source.slice(0, at[0])}${value}${source.slice(at[0] + held.length)}` : null;
  }
  const [index, [head, old, tail]] = matches[0];
  let written = value;
  const quote = /^["']/u.exec(old)?.[0];
  if (quote && !isBareLiteral(value)) written = `${quote}${value}${quote}`;
  else if (!quote && /\.json$/iu.test(target) && !isBareLiteral(value) && !/^["'[{]/u.test(value)) written = JSON.stringify(value);
  if (written === old) return null;
  lines[index] = `${head}${written}${tail}`;
  return lines.join('\n');
}

/**
 * Mirrors `fn assignment`: `[head, value, tail]` when `line` assigns `key`:
 * indentation, declaration words (`const`, `let mut`, `pub(crate) static`),
 * the key bare or quoted, a declared type (`: u32`) when words led, `:`, `=`
 * or `:=`, the value, and an optional trailing `,` or `;`.
 */
function assignment(line, key) {
  let at = line.length - line.replace(/^\s+/u, '').length;
  let declared = false;
  for (;;) {
    const found = keyedValue(line, at, key, declared);
    if (found !== null) return found;
    const word = /^[A-Za-z_][A-Za-z0-9_()]*[ \t]+/u.exec(line.slice(at));
    if (word === null) return null;
    at += word[0].length;
    declared = true;
  }
}

/** Mirrors `fn keyed_value`: `assignment` with the key at `at`. */
function keyedValue(line, at, key, declared) {
  let rest = line.slice(at);
  const quote = rest[0] === '"' || rest[0] === "'" ? rest[0] : null;
  if (quote !== null) rest = rest.slice(1);
  if (key === '' || !rest.startsWith(key)) return null;
  rest = rest.slice(key.length);
  if (quote !== null) {
    if (!rest.startsWith(quote)) return null;
    rest = rest.slice(1);
  }
  rest = rest.replace(/^\s+/u, '');
  let after;
  if (rest.startsWith(':=')) after = rest.slice(2);
  else if (rest.startsWith(':') && !declared) after = rest.slice(1);
  else {
    // `= v`, or a declared type before it (`: u32 = v`); never `==` or `=>`.
    const equals = rest.startsWith('=') ? 0 : rest.startsWith(':') ? rest.indexOf('=') : -1;
    if (equals < 0 || rest[equals + 1] === '=' || rest[equals + 1] === '>') return null;
    after = rest.slice(equals + 1);
  }
  const valueStart = line.length - after.replace(/^\s+/u, '').length;
  const body = line.slice(valueStart).replace(/\s+$/u, '');
  const value = (body.endsWith(',') || body.endsWith(';') ? body.slice(0, -1) : body).replace(/\s+$/u, '');
  return [line.slice(0, valueStart), value, line.slice(valueStart + value.length)];
}

/**
 * Mirrors `fn replaced_lines`: every line that is exactly `old` becomes `next`;
 * with none, every line that is `old` but for its indentation becomes `next`
 * at that indentation (unless `next` brings its own); with none, the one
 * occurrence of `old` in the file. After a `context` (found once), only the
 * first such line below it, and never a bare occurrence. Otherwise null.
 * Several lines (`old` holds a line break) are one consecutive block, found
 * once but for each line's indentation (PR #1188 G80).
 */
export function replacedLines(source, old, next, context = null) {
  const lines = source.split('\n');
  let from = 0;
  if (context !== null) {
    const at = matchIndices(source, context);
    if (at.length !== 1) return null;
    from = source.slice(0, at[0] + context.length).split('\n').length;
  }
  const bare = (line) => (line.endsWith('\r') ? line.slice(0, -1) : line);
  const indentOf = (line) => /^[ \t]*/u.exec(line)[0];
  const stripped = (text) => text.replace(/^[ \t]+|[ \t]+$/gu, '');
  const starts = old.includes('\n') ? lineBlockStarts(source, old, from) : [];
  if (starts.length > 0) {
    if (context === null && starts.length > 1) return null;
    const indent = /^[ \t]/u.test(next) ? '' : indentOf(lines[starts[0]]);
    const placed = next.split('\n').map((line) => (line === '' ? '' : `${indent}${line}`));
    return [...lines.slice(0, starts[0]), ...placed, ...lines.slice(starts[0] + old.split('\n').length)].join('\n');
  }
  const exact = lines.map((line, index) => index >= from && bare(line) === old);
  const loose = lines.map((line, index) => index >= from && stripped(old) !== '' && stripped(bare(line)) === stripped(old));
  let chosen = exact.includes(true) ? exact : loose.includes(true) ? loose : null;
  if (chosen === null) {
    return context === null && matchIndices(source, old).length === 1 ? source.replace(old, () => next) : null;
  }
  const indented = chosen === loose && !/^[ \t]/u.test(next);
  if (context !== null) {
    const first = chosen.indexOf(true);
    chosen = chosen.map((_, index) => index === first);
  }
  return lines.map((line, index) => {
    if (!chosen[index]) return line;
    return `${indented ? indentOf(line) : ''}${next}${line.slice(bare(line).length)}`;
  }).join('\n');
}

/**
 * Mirrors `fn line_block_starts`: the indices of the lines, from line `from`
 * on, where the lines of `block` stand one after another, each but for its
 * indentation (PR #1188 G80).
 */
export function lineBlockStarts(source, block, from = 0) {
  const stripped = (text) => (text.endsWith('\r') ? text.slice(0, -1) : text).replace(/^[ \t]+|[ \t]+$/gu, '');
  const lines = source.split('\n').map(stripped);
  const wanted = block.split('\n').map(stripped);
  const starts = [];
  for (let at = from; at + wanted.length <= lines.length; at += 1) {
    if (wanted.every((line, offset) => lines[at + offset] === line)) starts.push(at);
  }
  return starts;
}
