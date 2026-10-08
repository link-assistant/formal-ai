// The names in links notation we own (PR #1188, R1188-U6 and R1188-U7).
//
// A name is an unquoted token of a `.lino` line that reads as an identifier:
// a link name, a key, a role, an intent id or a slug, such as `defined-by`,
// `role file_read_action_cue` or `ci_gate check_notation`. Quoted text is
// human text or an external spelling and is never a name; a token with `/`
// or `.` is a path. scripts/measure-notation.mjs counts names and
// experiments/formal_ai_subagent/apply-notation-rules.mjs rewrites them, so
// both read the same tokens through this module.

/**
 * An identifier-shaped token: lowercase letters and digits joined by `-` or
 * `_`. Capitalized tokens (`WebSearch`, `FORMAL_AI_DATA_DIR`, `Q42`) are
 * external spellings: tool names, environment variables, Wikidata ids.
 */
export const NAME = /^[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*$/u;

/** The rules file, data/meta/notation-rules.lino. */
export const RULES_FILE = 'data/meta/notation-rules.lino';

/**
 * The spans of a line that are not names: quoted text and the comment.
 * Returns the line with every quoted span and the comment blanked by spaces,
 * so token offsets still index the original line.
 * @param {string} line
 * @returns {string}
 */
export function blankQuotedAndComment(line) {
  let output = '';
  let quote = null;
  let previousWasSpace = true;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quote !== null) {
      if (character === '\\') {
        output += '  ';
        index += 1;
        continue;
      }
      if (character === quote && line[index + 1] === quote) {
        output += '  ';
        index += 1;
        continue;
      }
      if (character === quote) {
        quote = null;
      }
      output += ' ';
      continue;
    }
    if (character === '"' || character === '`' || (character === "'" && previousWasSpace)) {
      quote = character;
      output += ' ';
      previousWasSpace = false;
      continue;
    }
    if (character === '#' && previousWasSpace) {
      return output + ' '.repeat(line.length - index);
    }
    output += character;
    previousWasSpace = /[\s(]/u.test(character);
  }
  return output;
}

/**
 * The names of one line with their offsets, in order.
 * @param {string} line
 * @returns {Array<{name: string, start: number}>}
 */
export function namesOfLine(line) {
  const names = [];
  const blanked = blankQuotedAndComment(line);
  for (const match of blanked.matchAll(/[^\s()]+/gu)) {
    const token = match[0].endsWith(':') ? match[0].slice(0, -1) : match[0];
    if (NAME.test(token)) {
      names.push({ name: token, start: match.index });
    }
  }
  return names;
}

/**
 * The words of a name, split at `-` and `_`.
 * @param {string} name
 * @returns {Array<string>}
 */
export function wordsOf(name) {
  return name.split(/[-_]/u).filter(Boolean);
}

/**
 * An indented `.lino` document as a tree of `{line, depth, children}`;
 * blank and comment-only lines are left out.
 * @param {string} text
 * @returns {Array<{line: string, depth: number, children: Array<object>}>}
 */
export function parseTree(text) {
  const root = { line: '', depth: -1, children: [] };
  const stack = [root];
  for (const raw of text.split('\n')) {
    const content = blankQuotedAndComment(raw).trim() === '' ? '' : raw.trimEnd();
    if (content === '') {
      continue;
    }
    const depth = content.length - content.trimStart().length;
    while (stack.length > 1 && stack[stack.length - 1].depth >= depth) {
      stack.pop();
    }
    const node = { line: content.trimStart(), depth, children: [] };
    stack[stack.length - 1].children.push(node);
    stack.push(node);
  }
  return root.children;
}

/**
 * The child block of a node as text indented from zero, and its line count.
 * @param {{children: Array<object>}} node
 * @returns {{text: string, lines: number}}
 */
export function childBlockOf(node) {
  const lines = [];
  const visit = (children, depth) => {
    for (const child of children) {
      lines.push(`${'  '.repeat(depth)}${child.line}`);
      visit(child.children, depth + 1);
    }
  };
  visit(node.children, 0);
  return { text: lines.join('\n'), lines: lines.length };
}

/**
 * The rules of data/meta/notation-rules.lino.
 * @param {string} text
 * @returns {{scopes: Array<string>, excluded: Array<{path: string, reason: string}>,
 *   abbreviations: Map<string, string>, properTerms: Set<string>, minimumBlockLines: number,
 *   families: Array<object>}}
 */
export function readRules(text) {
  const rules = {
    scopes: [],
    excluded: [],
    abbreviations: new Map(),
    properTerms: new Set(),
    minimumBlockLines: 3,
    families: [],
  };
  const top = parseTree(text).find((node) => node.line === 'notation-rules');
  if (!top) {
    throw new Error(`${RULES_FILE} has no notation-rules link`);
  }
  for (const node of top.children) {
    const [key, ...rest] = node.line.split(/\s+/u);
    const value = rest.join(' ');
    if (key === 'scope') {
      rules.scopes.push(value);
    } else if (key === 'excluded') {
      rules.excluded.push({ path: value, reason: quotedChild(node, 'reason') });
    } else if (key === 'duplicate-block-minimum-lines') {
      rules.minimumBlockLines = Number(value);
    } else if (key === 'abbreviations') {
      for (const entry of node.children) {
        const [short, full] = entry.line.split(/\s+/u);
        rules.abbreviations.set(short, full);
      }
    } else if (key === 'proper-terms') {
      for (const entry of node.children) {
        rules.properTerms.add(entry.line.split(/\s+/u)[0]);
      }
    } else if (key === 'family') {
      rules.families.push(familyOf(value, node));
    }
  }
  return rules;
}

function quotedChild(node, key) {
  const child = node.children.find((entry) => entry.line.startsWith(`${key} `));
  if (!child) {
    return '';
  }
  const match = /^\S+\s+"((?:[^"\\]|\\.)*)"/u.exec(child.line);
  return match ? match[1] : child.line.slice(key.length + 1);
}

function familyOf(name, node) {
  const family = { name, rule: 'dash-names', files: [], except: new Map(), readers: [], keepNames: new Map(), applied: false };
  for (const child of node.children) {
    const [key, ...rest] = child.line.split(/\s+/u);
    if (key === 'files') {
      family.files.push(rest.join(' '));
    } else if (key === 'rule') {
      family.rule = rest[0];
    } else if (key === 'except') {
      family.except.set(rest[0], quotedChild(child, 'reason'));
    } else if (key === 'readers') {
      family.readers.push(...rest);
    } else if (key === 'keep') {
      family.keepNames.set(rest[0], quotedChild(child, 'reason'));
    } else if (key === 'applied') {
      family.applied = rest[0] === 'true';
    }
  }
  return family;
}
