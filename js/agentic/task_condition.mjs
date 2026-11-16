// The output half of independent task acceptance (issue #1166): exact output
// literals and the supported subset of output conditions, read straight from
// the task text without the reader that bound the program's stdout.
// Native twin: rust/src/agentic_coding/task_condition.rs.
//
// Supported condition subset (anything else is an unresolved gap):
//   statement := effect MARKER condition [SEP OTHERWISE effect]
//              | MARKER condition SEP effect [SEP OTHERWISE effect]
//   condition := atom (CONNECTIVE atom)*   one connective kind per condition
//   atom      := operand [NEGATION] relation operand | operand symbol operand
//   operand   := identifier | integer      after function words are dropped
// MARKER, OTHERWISE, relations, connectives and fillers are seed roles; the
// program's inputs are the condition identifiers in code-point order, given as
// command-line arguments, and the truth table enumerates small integers.
import { cached } from './host.mjs';
import { explicitCount } from './crate/intent_formalization_obligations.mjs';
import { detect } from './crate/language.mjs';
import { containsCjk, wordsForRole } from './crate/seed_meanings.mjs';
import { operatorSymbols, structurePhrases } from './crate/proposition_structure.mjs';
import { functionWords, negationCues } from './crate/summarization_vocabulary.mjs';

/** Literal delimiters: opening and closing characters, read verbatim between them. */
const LITERAL_PAIRS = Object.freeze([['`', '`'], ['"', '"'], ['«', '»'], ['“', '”'], ['「', '」']]);
const FENCE = /^[ \t]*(?:```|~~~)/u;
const IDENTIFIER = /^[\p{L}_][\p{L}\p{N}_]*$/u;
const INTEGER = /^-?[0-9]{1,6}$/u;
const SMALL_VALUES = Object.freeze([0, 1, 2]);
const MAXIMUM_ROWS = 81;
const MAXIMUM_INPUTS = 3;
const SEPARATORS = new Set([',', ';', ':', '，', '；', '：', '、']);
/** Relation roles and the comparison each one writes, `left REL right`. */
const RELATION_ROLES = Object.freeze([
  ['number_constraint_lower_strict', 'gt'],
  ['number_constraint_lower_inclusive', 'ge'],
  ['number_constraint_upper_strict', 'lt'],
  ['number_constraint_upper_inclusive', 'le'],
  ['comparison-equal', 'eq'],
  ['comparison-not-equal', 'ne'],
]);
/** Seed operator relations (`symbolic-operator` rule) as comparisons. */
const SYMBOL_RELATIONS = Object.freeze({ less: ['lt', 'gt'], 'less-or-equal': ['le', 'ge'], equal: ['eq', 'eq'], 'not-equal': ['ne', 'ne'] });
const NEGATED = Object.freeze({ lt: 'ge', le: 'gt', gt: 'le', ge: 'lt', eq: 'ne', ne: 'eq' });

const isWordCharacter = (character) => character !== undefined && /[\p{L}\p{N}_]/u.test(character);

/**
 * Mirrors `fn task_literals`: the quoted and code-spanned literals of `text`,
 * each `{text, start, end, escaped}` in UTF-16 offsets (the Rust twin uses
 * byte offsets; only values leave either reader). A double-quoted literal
 * whose closing quote is escaped is marked `escaped`: its exact value is not
 * read.
 */
export function taskLiterals(text) {
  const out = [];
  let index = 0;
  while (index < text.length) {
    const pair = LITERAL_PAIRS.find(([open]) => text.startsWith(open, index));
    if (!pair) {
      index += 1;
      continue;
    }
    const [open, close] = pair;
    if (open === '`' && text.startsWith('```', index)) {
      const end = text.indexOf('```', index + 3);
      index = end < 0 ? text.length : end + 3;
      continue;
    }
    let cursor = index + open.length,
      escaped = false;
    while (cursor < text.length && !text.startsWith(close, cursor) && text[cursor] !== '\n') cursor += 1;
    while (open === '"' && cursor < text.length && text[cursor - 1] === '\\') {
      escaped = true;
      cursor += 1;
      while (cursor < text.length && !text.startsWith(close, cursor) && text[cursor] !== '\n') cursor += 1;
    }
    if (cursor >= text.length || text[cursor] === '\n') {
      index += open.length;
      continue;
    }
    const value = text.slice(index + open.length, cursor);
    if (value !== '') out.push({ text: value, start: index, end: cursor + close.length, escaped });
    index = cursor + close.length;
  }
  return out;
}

/** Mirrors `fn fence_lines`: the lines inside the first fenced block of `text`, or null. */
export function fenceLines(text) {
  const lines = text.split('\n');
  const open = lines.findIndex((line) => FENCE.test(line));
  if (open < 0) return null;
  const close = lines.findIndex((line, index) => index > open && FENCE.test(line));
  if (close < 0) return null;
  return lines.slice(open + 1, close);
}

/** Mirrors `fn role_phrases`: the role's surfaces, lowercased, longest first. */
function rolePhrases(role) {
  return cached(`task-condition-role-${role}`, () => [...new Set(wordsForRole(role).map((word) => word.toLowerCase()))]
    .sort((left, right) => Array.from(right).length - Array.from(left).length || (left < right ? -1 : left > right ? 1 : 0)));
}

/**
 * Mirrors `fn phrase_matches`: every `[start, end]` where `phrase` occurs in
 * `lowered` on word boundaries (CJK phrases need none).
 */
function phraseMatches(lowered, phrase) {
  const out = [];
  if (phrase === '') return out;
  const bounded = !containsCjk(phrase);
  let from = 0;
  for (;;) {
    const start = lowered.indexOf(phrase, from);
    if (start < 0) return out;
    const end = start + phrase.length;
    if (!bounded || (!isWordCharacter(lowered[start - 1]) && !isWordCharacter(lowered[end]))) out.push([start, end]);
    from = start + 1;
  }
}

/** Mirrors `fn first_role_match`: the earliest (then longest) match of a role phrase, or null. */
export function firstRoleMatch(lowered, role) {
  let best = null;
  for (const phrase of rolePhrases(role)) {
    for (const match of phraseMatches(lowered, phrase)) {
      if (best === null || match[0] < best[0] || (match[0] === best[0] && match[1] > best[1])) best = match;
    }
  }
  return best;
}

/** Mirrors `fn masked`: `text` lowercased with every literal blanked, so a marker inside a literal is not read. */
function masked(text, literals) {
  let out = '',
    last = 0;
  for (const literal of literals) {
    out += text.slice(last, literal.start).toLowerCase() + '\u0000'.repeat(literal.end - literal.start);
    last = literal.end;
  }
  return out + text.slice(last).toLowerCase();
}

/** Mirrors `fn branch_lines`: the literal lines of `literals` with their explicit counts. */
function branchLines(text, literals, end) {
  const out = [];
  literals.forEach((literal, index) => {
    const next = literals[index + 1]?.start ?? end;
    const count = explicitCount(text.slice(literal.end, Math.max(literal.end, next)));
    for (let time = 0; time < count; time += 1) out.push(literal.text);
  });
  return out;
}

/** Mirrors `fn condition_end`: where the condition text that starts at `from` stops. */
function conditionEnd(lowered, from, limit) {
  for (let index = from; index < limit; index += 1) {
    const character = lowered[index];
    if (character === '\u0000' || SEPARATORS.has(character)) return index;
    if ((character === '.' || character === '。') && !/[0-9]/u.test(lowered[index + 1] ?? '')) return index;
  }
  return limit;
}

/** Mirrors `fn operand`: the identifier or integer one side of an atom names, or null. */
function operand(side, fillers) {
  const tokens = side.split(/[\s\u0000]+/u).map((token) => token.replace(/^[()'"]+|[()'"?!]+$/gu, '')).filter((token) => token !== '');
  let kept = tokens.filter((token) => !fillers.includes(token));
  // A one-letter operand can spell a function word (`a`, `и`): it is kept
  // when nothing else on the side names an operand.
  if (!kept.length) kept = tokens.filter((token) => Array.from(token).length === 1);
  if (kept.length !== 1) return null;
  const [token] = kept;
  if (INTEGER.test(token)) return { integer: Number(token) };
  return IDENTIFIER.test(token) ? { name: token } : null;
}

/** Mirrors `fn relation_matches`: every relation phrase or symbol in `atom`, `{start, end, comparison}`. */
function relationMatches(atom) {
  const out = [];
  for (const [role, comparison] of RELATION_ROLES) {
    for (const phrase of rolePhrases(role)) for (const [start, end] of phraseMatches(atom, phrase)) out.push({ start, end, comparison });
  }
  for (const entry of operatorSymbols()) {
    const pair = SYMBOL_RELATIONS[entry.op];
    if (!pair) continue;
    let from = 0;
    for (;;) {
      const start = atom.indexOf(entry.symbol, from);
      if (start < 0) break;
      out.push({ start, end: start + entry.symbol.length, comparison: pair[entry.converse ? 1 : 0] });
      from = start + 1;
    }
  }
  return out;
}

/** Mirrors `fn read_atom`: one comparison `{comparison, left, right}` or a gap reason string. */
function readAtom(atom, fillers, negations) {
  const matches = relationMatches(atom);
  if (!matches.length) return 'condition_relation_missing';
  const chosen = matches.reduce((best, match) => {
    const length = match.end - match.start,
      bestLength = best.end - best.start;
    return length > bestLength || (length === bestLength && match.start < best.start) ? match : best;
  });
  if (matches.some((match) => match.end <= chosen.start || match.start >= chosen.end)) return 'condition_relation_repeated';
  const sides = [atom.slice(0, chosen.start), atom.slice(chosen.end)];
  let negated = false;
  const cleaned = sides.map((side) => side.split(/\s+/u).filter((token) => {
    if (!negations.includes(token)) return true;
    negated = !negated;
    return false;
  }).join(' '));
  const left = operand(cleaned[0], fillers),
    right = operand(cleaned[1], fillers);
  if (left === null || right === null) return 'condition_operand_unsupported';
  return { comparison: negated ? NEGATED[chosen.comparison] : chosen.comparison, left, right };
}

/** Mirrors `fn read_condition`: `{connective, atoms}` or a gap reason string. */
export function readCondition(text) {
  const lowered = text.toLowerCase();
  const phrases = structurePhrases(detect(text)).filter((entry) => entry.role !== 'repetition');
  const cuts = [];
  for (const entry of phrases) {
    for (const [start, end] of phraseMatches(lowered, entry.phrase)) {
      if (!cuts.some((cut) => start < cut.end && cut.start < end)) cuts.push({ start, end, role: entry.role });
    }
  }
  cuts.sort((left, right) => left.start - right.start);
  const roles = [...new Set(cuts.map((cut) => cut.role))];
  if (roles.length > 1) return 'condition_connectives_mixed';
  const fillers = [...functionWords(), ...rolePhrases('comparison-filler')];
  const negations = negationCues();
  const atoms = [];
  let from = 0;
  for (const cut of [...cuts, { start: lowered.length, end: lowered.length }]) {
    const atom = readAtom(lowered.slice(from, cut.start).trim(), fillers, negations);
    if (typeof atom === 'string') return atom;
    atoms.push(atom);
    from = cut.end;
  }
  return { connective: roles[0] ?? 'conjunction', atoms };
}

/**
 * Mirrors `fn conditional_statement`: the clause's output statement under the
 * supported subset: `null` when the clause states no condition or otherwise
 * branch, else `{condition, negated, then, otherwise, continues}` or
 * `{gap}`. `continues` marks a clause that only states the otherwise branch
 * of the statement before it.
 */
export function conditionalStatement(clause) {
  const literals = taskLiterals(clause).filter((literal) => !literal.escaped);
  const lowered = masked(clause, literals);
  const otherwise = firstRoleMatch(lowered, 'otherwise-marker');
  const mainEnd = otherwise === null ? lowered.length : otherwise[0];
  const main = lowered.slice(0, mainEnd);
  const negatedMarker = firstRoleMatch(main, 'negated-condition-marker');
  const marker = negatedMarker ?? firstRoleMatch(main, 'condition-marker');
  if (marker !== null && firstRoleMatch(main, 'execution-circumstance') !== null) return null;
  if (marker === null && otherwise === null) return null;
  const elseLiterals = otherwise === null ? [] : literals.filter((literal) => literal.start >= otherwise[1]);
  const otherwiseLines = branchLines(clause, elseLiterals, clause.length);
  if (marker === null) return { continues: true, otherwise: otherwiseLines };
  const conditionStop = conditionEnd(lowered, marker[1], mainEnd);
  const condition = readCondition(clause.slice(marker[1], conditionStop));
  if (typeof condition === 'string') return { gap: condition };
  const thenLiterals = literals.filter((literal) => literal.end <= mainEnd && (literal.end <= marker[0] || literal.start >= conditionStop));
  return { condition, negated: negatedMarker !== null, then: branchLines(clause, thenLiterals, mainEnd), otherwise: otherwiseLines, continues: false };
}

/** Mirrors `fn holds`: whether `condition` (with `negated`) holds for `values`. */
function holds(statement, values) {
  const value = (side) => (side.name === undefined ? side.integer : values.get(side.name));
  const compare = ({ comparison, left, right }) => {
    const [a, b] = [value(left), value(right)];
    return { lt: a < b, le: a <= b, gt: a > b, ge: a >= b, eq: a === b, ne: a !== b }[comparison];
  };
  const { connective, atoms } = statement.condition;
  const truth = connective === 'disjunction' ? atoms.some(compare) : atoms.every(compare);
  return statement.negated ? !truth : truth;
}

/**
 * Mirrors `fn condition_table`: the expected stdout of the conditional
 * statements for every row of small inputs, `{inputs, rows: [{args, lines}]}`,
 * or a gap reason string.
 */
export function conditionTable(statements) {
  const inputs = [],
    constants = [];
  for (const statement of statements) {
    for (const atom of statement.condition.atoms) {
      for (const side of [atom.left, atom.right]) {
        if (side.name !== undefined && !inputs.includes(side.name)) inputs.push(side.name);
        if (side.integer !== undefined) constants.push(side.integer);
      }
    }
  }
  inputs.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  if (inputs.length > MAXIMUM_INPUTS) return 'condition_inputs_exceeded';
  const domain = [...new Set([...SMALL_VALUES, ...constants.flatMap((constant) => [constant - 1, constant, constant + 1])])].sort((left, right) => left - right);
  if (domain.length ** inputs.length > MAXIMUM_ROWS) return 'condition_inputs_exceeded';
  let rows = [[]];
  for (let index = 0; index < inputs.length; index += 1) rows = rows.flatMap((row) => domain.map((value) => [...row, value]));
  return {
    inputs,
    rows: rows.map((row) => {
      const values = new Map(inputs.map((name, index) => [name, row[index]]));
      const lines = statements.flatMap((statement) => (holds(statement, values) ? statement.then : statement.otherwise));
      return { args: row.map(String), lines };
    }),
  };
}
