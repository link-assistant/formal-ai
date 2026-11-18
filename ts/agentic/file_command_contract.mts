import {quoteShellWord as quote} from './portable/string_contracts.mjs';
// Generic finite grammar and actual shell observations; no issue/prompt lookup.
import grammar from '../../data/meta/file-command-grammar.json' with { type: 'json' };
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { readText } from './host.mjs';
const expression = value => new RegExp(value.replaceAll('(?P<', '(?<'), 'iu');
const productions = grammar.patterns.map(row => ({
  ...row,
  pattern: expression(row.expression)
}));
const unquote = value => value.length >= 2 && ["'", '"', '\u0060'].includes(value[0]) && value.at(-1) === value[0] ? value.slice(1, -1) : value;
const pathValid = value => typeof value === 'string' && value.length > 0 && value.length <= 4096 && /^[A-Za-z0-9_.][A-Za-z0-9_./ -]*$/u.test(value) && value.split('/').every(part => part !== '' && part !== '..');
function shellControl(text) {
  let quoted = null,
    escaped = false;
  for (let index = 0; index < text.length; index++) {
    const character = text[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (quoted === "'") {
      if (character === "'") quoted = null;
      continue;
    }
    if (character === '\\') {
      escaped = true;
      continue;
    }
    if (character === quoted) {
      quoted = null;
      continue;
    }
    if ((character === "'" || character === '"') && quoted === null) {
      quoted = character;
      continue;
    }
    if (text.startsWith(grammar.substitutionOpening, index) || grammar.controlCharacters.includes(character)) return true;
  }
  return quoted !== null || escaped;
}
function oneEdit(left, right) {
  if (Math.abs(left.length - right.length) > 1 || left === right) return false;
  let first = 0,
    second = 0,
    edits = 0;
  while (first < left.length && second < right.length) {
    if (left[first] === right[second]) {
      first++;
      second++;
      continue;
    }
    if (++edits > 1) return false;
    if (left.length >= right.length) first++;
    if (right.length >= left.length) second++;
  }
  return edits + (left.length - first) + (right.length - second) === 1;
}
export function normalizeListing(task) {
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 8192) return {text:task,normalizations:[]};
  const first = /^\s*(?:please\s+)?([A-Za-z]+)/u.exec(task)?.[1]?.toLowerCase();
  if (!grammar.listingVerbs.includes(first)) return {
    text: task,
    normalizations: []
  };
  const normalizations = [],
    quoted = quotedSegmentSpans(task),
    pattern = new RegExp(grammar.adjectiveExpression.replaceAll('(?P<', '(?<'), 'giu');
  for (const match of task.matchAll(pattern)) {
    const word = match.groups.adjective,
      start = match.index + match[0].indexOf(word),
      end = start + word.length;
    if (quoted.some(span => start < span.end && end > span.start) || grammar.listingAdjectives.includes(word.toLowerCase())) continue;
    const candidates = grammar.listingAdjectives.filter(candidate => oneEdit(word.toLowerCase(), candidate));
    if (candidates.length === 1) normalizations.push({
      span: [start, end],
      observed: word,
      canonical: candidates[0],
      basis: 'unique-one-edit-adjective-before-container'
    });
  }
  let text = task;
  for (const row of [...normalizations].reverse()) text = text.slice(0, row.span[0]) + row.canonical + text.slice(row.span[1]);
  return {
    text,
    normalizations
  };
}
export function fileCommandContract(task) {
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 8192) return null;
  const base = {
    schema: 'file-command-contract/v1',
    task,
    verified: false,
    complete: false
  };
  const raw = unquote(task.trim()),
    first = /^[A-Za-z][A-Za-z0-9_-]*/u.exec(raw)?.[0]?.toLowerCase();
  if (terminalCommandVocabulary().bare_shell_tokens.includes(first) && shellControl(raw)) return {
    ...base,
    operation: 'unsupported',
    reason: 'CompoundShellInputUnsupported'
  };
  const normalized = normalizeListing(task);
  for (const row of productions) {
    const match = row.pattern.exec(normalized.text);
    if (!match) continue;
    const slots = match.groups ?? {},
      path = unquote(slots.path ?? '.');
    const result = {
      ...base,
      operation: row.operation,
      slots: {
        ...slots,
        path
      },
      normalizations: normalized.normalizations
    };
    const unsupported = reason => ({
      ...result,
      operation: 'unsupported',
      reason
    });
    if (row.operation === 'unsupported') return unsupported(row.reason);
    if (!pathValid(path)) return unsupported('FileCommandPathUnsupported');
    let argumentsList;
    if (row.operation === 'listing') argumentsList = ['ls'];else if (row.operation === 'head') {
      const count = Number(slots.count);
      if (!Number.isSafeInteger(count) || count < 1 || count > 1000) return unsupported('FileCommandLineLimit');
      argumentsList = ['head', '-n', String(count), '--', path];
    } else if (row.operation === 'search') {
      const query = unquote(slots.query);
      if (!query.length || new TextEncoder().encode(query).length > 512 || /[\0\r\n]/u.test(query)) return unsupported('FileCommandQueryUnsupported');
      argumentsList = ['grep', '-rFn', '--', query, path];
    } else if (row.operation === 'count') {
      let program;
      try {
        program = readText('data/meta/file-count-program.cjs');
      } catch {
        return unsupported('FileCountProgramUnavailable');
      }
      if (typeof program !== 'string' || program.length > 8192) return unsupported('FileCountProgramUnavailable');
      argumentsList = ['node', '-e', program, '--', path];
    } else return unsupported('FileCommandOperationUnsupported');
    return {
      ...result,
      arguments: argumentsList,
      command: [argumentsList[0], ...argumentsList.slice(1).map(quote)].join(' ')
    };
  }
  return null;
}
