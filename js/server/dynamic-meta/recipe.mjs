import { serverMessage } from '../messages.mjs';
// A portable, data-only fragment of normal Markov algorithms. Rule order,
// leftmost matching, and terminal rules have the same meaning in every format.
import { parseSubstitutionQuery, renderSubstitutionQuery, substitutionEffect } from '../../agentic/crate/links_substitution_query.mjs';

const SOURCE_LENGTH_LIMIT = 64 * 1024;
const DIRECT_RULE_COUNT = 4;
const LITERAL = '"(?:[^"\\\\\u0000-\u001f]|\\\\["\\\\/bfnrt]|\\\\u[0-9a-fA-F]{4})*"';
const SENTENCE = new RegExp(`^Replace (${LITERAL}) with (${LITERAL})( and stop)?\\.$`, 'u');

function checkedProgram(program) {
  const rules = program?.rules;
  if (!Array.isArray(rules) || !rules.length || rules.length > 128 ||
      rules.some(({ pattern, replacement, terminal }) =>
        typeof pattern !== 'string' || typeof replacement !== 'string' ||
        pattern.length > 1024 || replacement.length > 1024 ||
        !pattern.isWellFormed() || !replacement.isWellFormed() || typeof terminal !== 'boolean') ||
      JSON.stringify(rules).length > 12 * 1024) {
    throw new Error(serverMessage('dynamic-meta-invalid-rules'));
  }
  return { rules: rules.map(({ pattern, replacement, terminal }) => ({ pattern, replacement, terminal })) };
}

const javascript = `export function rewrite(input, maxSteps = 256) {
  if (typeof input !== 'string' || !input.isWellFormed() || input.length > ${SOURCE_LENGTH_LIMIT} ||
      !Number.isSafeInteger(maxSteps) || maxSteps < 1 || maxSteps > 1000) {
    throw new Error(${JSON.stringify(serverMessage('dynamic-meta-recipe-bounds'))});
  }
  for (let step = 0; step < maxSteps; step += 1) {
    const rule = rules.find(({pattern}) => input.includes(pattern));
    if (!rule) break;
    const offset = input.indexOf(rule.pattern);
    if (input.length - rule.pattern.length + rule.replacement.length > ${SOURCE_LENGTH_LIMIT}) break;
    input = input.slice(0, offset) + rule.replacement + input.slice(offset + rule.pattern.length);
    if (rule.terminal) break;
  }
  return input;
}
`;

const python = `
def rewrite(text, max_steps=256):
    def length(value):
        return len(value.encode("utf-16-le")) // 2
    size = length(text)
    if size > ${SOURCE_LENGTH_LIMIT} or type(max_steps) is not int or not 1 <= max_steps <= 1000:
        raise ValueError(${JSON.stringify(serverMessage('dynamic-meta-recipe-bounds'))})
    for _ in range(max_steps):
        for rule in rules:
            offset = text.find(rule["pattern"])
            if offset == -1:
                continue
            size += length(rule["replacement"]) - length(rule["pattern"])
            if size > ${SOURCE_LENGTH_LIMIT}:
                return text
            text = text[:offset] + rule["replacement"] + text[offset + len(rule["pattern"]):]
            if rule["terminal"]:
                return text
            break
        else:
            return text
    return text
`;

export function readRecipe(language, source) {
  if (typeof source !== 'string' || source.length > SOURCE_LENGTH_LIMIT) throw new Error(serverMessage('dynamic-meta-invalid-source'));
  const text = source.trim();
  if (language === 'links') return checkedProgram(parseSubstitutionQuery(text, 0));
  if (language === 'english') {
    const rules = text.split(/\r?\n/u).map((line) => {
      const match = SENTENCE.exec(line.trim());
      if (!match) throw new Error(serverMessage('dynamic-meta-controlled-instruction'));
      return { pattern: JSON.parse(match[1]), replacement: JSON.parse(match[2]), terminal: Boolean(match[3]) };
    });
    return checkedProgram({ rules });
  }
  const header = language === 'javascript' ? /^const rules = ([^\n]+);\n/u :
    language === 'python' ? /^import json\nrules = json\.loads\(([^\n]+)\)\n/u : null;
  const match = header?.exec(text);
  if (!match) throw new Error(serverMessage('dynamic-meta-unsupported-program'));
  const program = checkedProgram({ rules: JSON.parse(language === 'python' ? JSON.parse(match[1]) : match[1]) });
  if (text !== renderRecipe(language, program).trim()) throw new Error(serverMessage('dynamic-meta-template-mismatch'));
  return program;
}

export function renderRecipe(language, program) {
  const { rules } = checkedProgram(program);
  const json = JSON.stringify(rules);
  switch (language) {
    case 'links': return renderSubstitutionQuery({ rules });
    case 'english': return rules.map(({ pattern, replacement, terminal }) =>
      `Replace ${JSON.stringify(pattern)} with ${JSON.stringify(replacement)}${terminal ? ' and stop' : ''}.`).join('\n');
    case 'javascript': return `const rules = ${json};\n${javascript}`;
    case 'python': return `import json\nrules = json.loads(${JSON.stringify(json)})\n${python}`;
    default: throw new Error(serverMessage('dynamic-meta-recipe-language'));
  }
}

export function analyzeRecipe({ rules }) {
  let decreases = true;
  const analysis = rules.map((rule, index) => {
    // If an earlier pattern occurs inside this one, it always wins priority.
    const shadowedBy = rules.findIndex(({ pattern }, earlier) => earlier < index && rule.pattern.includes(pattern));
    const lengthChange = rule.replacement.length - rule.pattern.length;
    if (shadowedBy === -1 && !rule.terminal && lengthChange >= 0) decreases = false;
    return { ...rule, index, effect: substitutionEffect(rule), lengthChange, shadowedBy: shadowedBy === -1 ? null : shadowedBy };
  });
  return { rules: analysis, termination: decreases ? 'strictly_decreasing' : 'unproven' };
}

// Compile ordered patterns to a failure-linked trie. Each node stores the
// highest-priority rule ending there, including endings along failure links.
export function compileRecipe({ rules }) {
  const nodes = [{ edges: {}, failure: 0, rule: rules.length }];
  const remaining = rules.slice(DIRECT_RULE_COUNT);
  if (remaining.length && remaining.every(({ pattern }) => pattern.length > 0)) {
    // A native literal search can reject wide misses before walking the trie.
    nodes[0].prefilter = remaining.map(({ pattern }) => pattern.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&')).join('|');
  }
  for (const [index, { pattern }] of rules.entries()) {
    let at = 0;
    // Use UTF-16 code units, matching JavaScript indexOf and slice.
    for (let i = 0; i < pattern.length; i += 1) {
      const unit = pattern[i];
      if (nodes[at].edges[unit] === undefined) {
        nodes[at].edges[unit] = nodes.length;
        nodes.push({ edges: {}, failure: 0, rule: rules.length });
      }
      at = nodes[at].edges[unit];
    }
    nodes[at].rule = Math.min(nodes[at].rule, index);
  }
  const queue = [0];
  for (let i = 0; i < queue.length; i += 1) {
    const parent = queue[i];
    for (const [unit, child] of Object.entries(nodes[parent].edges)) {
      queue.push(child);
      if (parent === 0) continue;
      const failure = transition(nodes, nodes[parent].failure, unit);
      nodes[child].failure = failure;
      nodes[child].rule = Math.min(nodes[child].rule, nodes[failure].rule);
    }
  }
  return nodes;
}

function transition(nodes, at, unit) {
  while (at !== 0 && nodes[at].edges[unit] === undefined) at = nodes[at].failure;
  return nodes[at].edges[unit] ?? 0;
}

function compiledMatch(rules, nodes, prefilter, input) {
  const directCount = Math.min(rules.length, DIRECT_RULE_COUNT);
  const direct = interpretedMatch(rules, input, directCount);
  if (direct || directCount === rules.length) return direct;
  if (prefilter && !prefilter.test(input)) return null;
  let index = nodes[0].rule;
  let offset = index < rules.length ? 0 : -1;
  let at = 0;
  for (let i = 0; i < input.length && index !== directCount; i += 1) {
    at = transition(nodes, at, input[i]);
    const candidate = nodes[at].rule;
    if (candidate < index) {
      index = candidate;
      offset = i + 1 - rules[index].pattern.length;
    }
  }
  return offset === -1 ? null : { index, offset };
}

function interpretedMatch(rules, input, limit = rules.length) {
  for (let index = 0; index < limit; index += 1) {
    const offset = input.indexOf(rules[index].pattern);
    if (offset !== -1) return { index, offset };
  }
  return null;
}

export function runRecipe({ rules }, input, { maxSteps = 256, compiled } = {}) {
  if (typeof input !== 'string' || input.length > SOURCE_LENGTH_LIMIT || !input.isWellFormed() ||
      !Number.isSafeInteger(maxSteps) || maxSteps < 1 || maxSteps > 1000) {
    throw new Error(serverMessage('dynamic-meta-recipe-bounds'));
  }
  let output = input;
  const trace = [];
  const prefilter = compiled?.[0].prefilter === undefined ? null : new RegExp(compiled[0].prefilter, 'u');
  for (let step = 0; step < maxSteps; step += 1) {
    const match = compiled ? compiledMatch(rules, compiled, prefilter, output) : interpretedMatch(rules, output);
    if (!match) return { output, trace, halt: { kind: 'no_applicable_rule' } };
    const { index, offset } = match;
    const rule = rules[index];
    const before = output.slice(0, offset);
    const after = output.slice(offset + rule.pattern.length);
    if (before.length + rule.replacement.length + after.length > SOURCE_LENGTH_LIMIT) {
      return { output, trace, halt: { kind: 'output_limit' } };
    }
    output = before + rule.replacement + after;
    trace.push({ rule_index: index, byte_offset: Buffer.byteLength(before) });
    if (rule.terminal) return { output, trace, halt: { kind: 'terminal_rule', index } };
  }
  return { output, trace, halt: { kind: 'step_limit' } };
}
