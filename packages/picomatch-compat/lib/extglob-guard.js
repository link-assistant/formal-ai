'use strict';

const constants = require('./constants');
const utils = require('./utils');
const { POSIX_REGEX_SOURCE } = constants;

const splitTopLevel = input => {
  const parts = [];
  let bracket = 0;
  let paren = 0;
  let quote = 0;
  let value = '';
  let escaped = false;

  for (const ch of input) {
    if (escaped === true) {
      value += ch;
      escaped = false;
      continue;
    }

    if (ch === '\\') {
      value += ch;
      escaped = true;
      continue;
    }

    if (ch === '"') {
      quote = quote === 1 ? 0 : 1;
      value += ch;
      continue;
    }

    if (quote === 0) {
      if (ch === '[') {
        bracket++;
      } else if (ch === ']' && bracket > 0) {
        bracket--;
      } else if (bracket === 0) {
        if (ch === '(') {
          paren++;
        } else if (ch === ')' && paren > 0) {
          paren--;
        } else if (ch === '|' && paren === 0) {
          parts.push(value);
          value = '';
          continue;
        }
      }
    }

    value += ch;
  }

  parts.push(value);
  return parts;
};

const isPlainBranch = branch => {
  let escaped = false;

  for (const ch of branch) {
    if (escaped === true) {
      escaped = false;
      continue;
    }

    if (ch === '\\') {
      escaped = true;
      continue;
    }

    if (/[?*+@!()[\]{}]/.test(ch)) {
      return false;
    }
  }

  return true;
};

const normalizeSimpleBranch = branch => {
  let value = branch.trim();
  let changed = true;

  while (changed === true) {
    changed = false;

    if (/^@\([^\\()[\]{}|]+\)$/.test(value)) {
      value = value.slice(2, -1);
      changed = true;
    }
  }

  if (!isPlainBranch(value)) {
    return;
  }

  return value.replace(/\\(.)/g, '$1');
};

const hasRepeatedCharPrefixOverlap = branches => {
  const values = branches.map(normalizeSimpleBranch).filter(Boolean);

  for (let i = 0; i < values.length; i++) {
    for (let j = i + 1; j < values.length; j++) {
      const a = values[i];
      const b = values[j];
      const char = a[0];

      if (!char || a !== char.repeat(a.length) || b !== char.repeat(b.length)) {
        continue;
      }

      if (a === b || a.startsWith(b) || b.startsWith(a)) {
        return true;
      }
    }
  }

  return false;
};

const parseRepeatedExtglob = (pattern, requireEnd = true) => {
  if ((pattern[0] !== '+' && pattern[0] !== '*') || pattern[1] !== '(') {
    return;
  }

  let bracket = 0;
  let paren = 0;
  let quote = 0;
  let escaped = false;

  for (let i = 1; i < pattern.length; i++) {
    const ch = pattern[i];

    if (escaped === true) {
      escaped = false;
      continue;
    }

    if (ch === '\\') {
      escaped = true;
      continue;
    }

    if (ch === '"') {
      quote = quote === 1 ? 0 : 1;
      continue;
    }

    if (quote === 1) {
      continue;
    }

    if (ch === '[') {
      bracket++;
      continue;
    }

    if (ch === ']' && bracket > 0) {
      bracket--;
      continue;
    }

    if (bracket > 0) {
      continue;
    }

    if (ch === '(') {
      paren++;
      continue;
    }

    if (ch === ')') {
      paren--;

      if (paren === 0) {
        if (requireEnd === true && i !== pattern.length - 1) {
          return;
        }

        return {
          type: pattern[0],
          body: pattern.slice(2, i),
          end: i
        };
      }
    }
  }
};

const buildCharClassStar = chars => {
  const source = chars.length === 1
    ? utils.escapeRegex(chars[0])
    : `[${chars.map(ch => utils.escapeRegex(ch)).join('')}]`;

  return `${source}*`;
};

const getStarExtglobSequenceChars = pattern => {
  let index = 0;
  const chars = [];

  while (index < pattern.length) {
    const match = parseRepeatedExtglob(pattern.slice(index), false);

    if (!match || match.type !== '*') {
      return;
    }

    const branches = splitTopLevel(match.body).map(branch => branch.trim());
    if (branches.length !== 1) {
      return;
    }

    const branch = normalizeSimpleBranch(branches[0]);
    if (!branch || branch.length !== 1) {
      return;
    }

    chars.push(branch);
    index += match.end + 1;
  }

  if (chars.length < 1) {
    return;
  }

  return chars;
};

const repeatedExtglobRecursion = pattern => {
  let depth = 0;
  let value = pattern.trim();
  let match = parseRepeatedExtglob(value);

  while (match) {
    depth++;
    value = match.body.trim();
    match = parseRepeatedExtglob(value);
  }

  return depth;
};

// A wildcard inside another unbounded repetition can backtrack exponentially,
// including when the extglob has only one branch (GHSA-fc5q-39xc-vjpf).
// Quoted and bracket-class stars remain literals; escaping follows unescape.
const hasUnboundedStar = (pattern, options) => {
  const lastBracket = options.nobracket === true ? -1 : pattern.lastIndexOf(']');
  let bracket = -1;
  let classStar = false;
  let quote = false;
  let escaped = false;
  for (let i = 0; i < pattern.length; i++) {
    const ch = pattern[i];
    if (escaped) {
      if (ch === '*' && options.unescape === true && bracket === -1 && !quote) return true;
      escaped = false;
      continue;
    }
    if (ch === '\\') {
      escaped = true;
      continue;
    }
    if (bracket !== -1) {
      // Only recognized POSIX classes consume their inner closing bracket.
      if (ch === '[' && pattern[i + 1] === ':' && options.posix !== false) {
        const match = /^\[:([a-z]+):\]/.exec(pattern.slice(i, i + 16));
        if (match && POSIX_REGEX_SOURCE[match[1]]) {
          i += match[0].length - 1;
          continue;
        }
      }
      if (ch === '*') classStar = true;
      if (ch === ']' && i > bracket + 1 &&
          !(i === bracket + 2 && (pattern[bracket + 1] === '^' ||
            (options.posix === true && pattern[bracket + 1] === '!')))) {
        bracket = -1;
        classStar = false;
      }
      continue;
    }
    if (ch === '"') {
      quote = !quote;
      continue;
    }
    if (quote) continue;
    // A nested "[" is literal within a class. An unclosed class is escaped by
    // the parser, so its following stars remain active wildcards.
    if (ch === '[' && i < lastBracket) {
      bracket = i;
    } else if (ch === '*') {
      return true;
    }
  }
  return classStar;
};

const analyzeRepeatedExtglob = (body, options) => {
  if (options.maxExtglobRecursion === false) {
    return { risky: false };
  }

  const max =
    typeof options.maxExtglobRecursion === 'number'
      ? options.maxExtglobRecursion
      : constants.DEFAULT_MAX_EXTGLOB_RECURSION;

  const branches = splitTopLevel(body).map(branch => branch.trim());

  if (branches.length > 1) {
    if (
      branches.some(branch => branch === '') ||
      branches.some(branch => /^[*?]+$/.test(branch)) ||
      hasRepeatedCharPrefixOverlap(branches)
    ) {
      return { risky: true };
    }
  }

  // A repeated extglob is "risky" (prone to catastrophic backtracking) when a
  // branch is itself a `*(...)` sequence, since that nests an unbounded quantifier
  // inside the outer `+(...)`/`*(...)`. When *every* branch reduces to single
  // characters we can emit one flat, ReDoS-safe character class that preserves the
  // meaning of ALL branches (e.g. `+(*(a)|*(b))` -> `[ab]*`), rather than dropping
  // every branch but the first.
  const safeChars = [];
  let sawStarSequence = false;
  let combinable = true;

  for (const branch of branches) {
    const chars = getStarExtglobSequenceChars(branch);
    if (chars) {
      sawStarSequence = true;
      safeChars.push(...chars);
      continue;
    }

    const literal = normalizeSimpleBranch(branch);
    if (literal && literal.length === 1) {
      safeChars.push(literal);
      continue;
    }

    combinable = false;

    if (repeatedExtglobRecursion(branch) > max) {
      return { risky: true };
    }
  }

  if (sawStarSequence) {
    return combinable
      ? { risky: true, safeOutput: buildCharClassStar([...new Set(safeChars)]) }
      : { risky: true };
  }

  // Preserve the safe flattening above, then apply the guard to every branch
  // count, including a single wildcard branch or a wildcard under @(…).
  if (branches.some(branch => hasUnboundedStar(branch, options))) {
    return { risky: true };
  }

  return { risky: false };
};

module.exports = analyzeRepeatedExtglob;
