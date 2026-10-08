// The link-cli substitution-query dialect for string rewrites
// (rust/src/links_substitution_query/text.rs and mod.rs). The doublet-link
// dialect (links.rs) is not ported. Parse errors are reported as `null`; the
// agentic caller discards the Rust error text (`.ok()`).

import { isAlphanumeric, isWhitespace } from '../write_str.mjs';
import { rewriteProgram, rewriteRule } from './normal_markov.mjs';

/** Mirrors `fn substitution_effect`: the `CrudEvent::as_str` slug. */
export function substitutionEffect(rule) {
  if (rule.pattern === rule.replacement) return 'read';
  if (rule.pattern === '') return 'create';
  if (rule.replacement === '') return 'delete';
  return 'update';
}

/**
 * Mirrors `fn escape` in rust/src/links_substitution_query/mod.rs.
 * @param {string} text
 * @returns {string}
 */
function escape(text) {
  let out = '';
  for (const character of text) {
    if (character === '\\') out += '\\\\';
    else if (character === '"') out += '\\"';
    else if (character === '\n') out += '\\n';
    else if (character === '\r') out += '\\r';
    else if (character === '\t') out += '\\t';
    else out += character;
  }
  return out;
}

/** Mirrors `fn render_substitution_query`. */
export function renderSubstitutionQuery(program) {
  const rules = program.rules;
  const patternsEmpty = rules.every((rule) => rule.pattern === '');
  const replacementsEmpty = rules.every((rule) => rule.replacement === '');
  const elideMatching = rules.length > 0 && patternsEmpty && !replacementsEmpty;
  const elideSubstituting = rules.length > 0 && replacementsEmpty && !patternsEmpty;
  const nameOnMatching = elideSubstituting;
  const matching = elideMatching ? '()' : renderSide(rules.map((rule) => [rule.pattern, rule.terminal && nameOnMatching]));
  const substituting = elideSubstituting ? '()' : renderSide(rules.map((rule) => [rule.replacement, rule.terminal && !nameOnMatching]));
  return `${matching} ${substituting}`;
}

function renderSide(operands) {
  return `(${operands.map(([text, terminal]) => `(${terminal ? 'terminal: ' : ''}"${escape(text)}")`).join(' ')})`;
}

class Failure extends Error {}

class Parser {
  constructor(input) {
    this.chars = Array.from(input);
    this.cursor = 0;
  }

  peek() { return this.chars[this.cursor]; }

  bump() {
    const character = this.chars[this.cursor];
    if (character !== undefined) this.cursor += 1;
    return character;
  }

  eat(expected) {
    if (this.peek() !== expected) return false;
    this.cursor += 1;
    return true;
  }

  skipWhitespace() {
    while (this.peek() !== undefined && isWhitespace(this.peek())) this.cursor += 1;
  }

  atEnd() { return this.cursor >= this.chars.length; }

  parseString() {
    if (!this.eat('"')) throw new Failure();
    let text = '';
    for (;;) {
      const character = this.bump();
      if (character === undefined) throw new Failure();
      if (character === '"') return text;
      if (character === '\\') {
        const escaped = { '\\': '\\', '"': '"', n: '\n', r: '\r', t: '\t' }[this.bump()];
        if (escaped === undefined) throw new Failure();
        text += escaped;
      } else text += character;
    }
  }

  parseSide() {
    this.skipWhitespace();
    if (!this.eat('(')) throw new Failure();
    const operands = [];
    for (;;) {
      this.skipWhitespace();
      const next = this.peek();
      if (next === ')') {
        this.bump();
        return operands;
      }
      if (next !== '(') throw new Failure();
      operands.push(this.parseOperand());
    }
  }

  parseOperand() {
    this.bump();
    this.skipWhitespace();
    const next = this.peek();
    if (next === undefined) throw new Failure();
    const terminal = next === '"' ? false : this.parseOperandName();
    const text = this.parseString();
    this.skipWhitespace();
    if (!this.eat(')')) throw new Failure();
    return { text, terminal };
  }

  parseOperandName() {
    const start = this.cursor;
    while (this.peek() !== undefined && (isAlphanumeric(this.peek()) || this.peek() === '_' || this.peek() === '-')) this.cursor += 1;
    const name = this.chars.slice(start, this.cursor).join('');
    this.skipWhitespace();
    if (name === '' || !this.eat(':') || name !== 'terminal') throw new Failure();
    this.skipWhitespace();
    return true;
  }
}

/** Mirrors `fn parse_substitution_query`: a rewrite program, or null on error. */
export function parseSubstitutionQuery(query, maxSteps) {
  try {
    const parser = new Parser(query);
    const matching = parser.parseSide();
    parser.skipWhitespace();
    if (parser.atEnd()) return null;
    const substituting = parser.parseSide();
    parser.skipWhitespace();
    if (!parser.atEnd()) return null;
    const rules = pairOperands(matching, substituting);
    return rules === null ? null : rewriteProgram(rules, maxSteps);
  } catch (error) {
    if (error instanceof Failure) return null;
    throw error;
  }
}

function pairOperands(matching, substituting) {
  if (!matching.length) return substituting.map((operand) => rewriteRule('', operand.text, operand.terminal));
  if (!substituting.length) return matching.map((operand) => rewriteRule(operand.text, '', operand.terminal));
  if (matching.length !== substituting.length) return null;
  return matching.map((old, index) => rewriteRule(old.text, substituting[index].text, old.terminal || substituting[index].terminal));
}
