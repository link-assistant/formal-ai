// Typed expressions for the supported native specification subset. Each
// operation calls its actual JavaScript twin; unknown types stay unsupported.
import { isDeepStrictEqual } from 'node:util';
import { compileNaturalLanguageSkill } from '../../js/agentic/crate/skill_compiler.mjs';
import { symbolicFromWorker } from '../../js/server/solve.mjs';
import { parseLino } from '../../js/server/lino.mjs';
import { installTextHost } from './text-capability-measures.mjs';
import { tupleParts } from './rust-specification-tuples.mjs';

export function close(tokens, start) {
  const pairs = { '(': ')', '[': ']', '{': '}' };
  const stack = [];
  for (let index = start; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token.kind !== 'punct') continue;
    if (pairs[token.text]) stack.push(pairs[token.text]);
    else if (token.text === stack.at(-1)) {
      stack.pop();
      if (!stack.length) return index;
    }
  }
  throw new Error('unclosed native delimiter');
}

export function split(tokens, delimiter = ',') {
  const values = [[]];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token.kind === 'punct' && ['(', '[', '{'].includes(token.text)) {
      const end = close(tokens, index);
      values.at(-1).push(...tokens.slice(index, end + 1));
      index = end;
    } else if (token.kind === 'punct' && token.text === delimiter) values.push([]);
    else values.at(-1).push(token);
  }
  return values.filter(value => value.length);
}

const fields = {
  response: { intent: 'string', answer: 'string', links_notation: 'string', evidence_links: 'vec:string' },
  package: Object.fromEntries(['id', 'legacy_behavior_rule_id', 'source_description', 'trigger',
    'normalized_trigger', 'response', 'rule_id', 'handler_id', 'skill_name'].map(name => [name, 'string'])),
  replay: Object.fromEntries(['package_id', 'rule_id', 'handler_id', 'answer', 'cache_hit'].map(name => [name, 'string'])),
  linkRecord: Object.fromEntries(['stable_id', 'schema_version', 'record_type', 'source_id'].map(name => [name, 'string'])),
};
const callTypes = { compiler: 'result:package', solver: 'solver', user: 'turn', assistant: 'turn', codec: 'result:lino', historySolver: 'response' };
const methodTypes = {
  string: { contains: ['boolean', 'string'], starts_with: ['boolean', 'string'], to_lowercase: ['string'] },
  solver: { solve: ['response', 'string'], solve_with_history: ['response', 'string', 'vec:turn'] },
  package: { replay: ['option:replay', 'string'], links_notation: ['string'], link_records: ['vec:linkRecord'] },
};

function requireType(actual, expected) {
  if (actual !== expected && !(actual === 'vec:empty' && expected.startsWith('vec:'))) {
    throw new Error(`native type mismatch ${actual} expected ${expected}`);
  }
}

function operator(tokens, sign) {
  for (let index = 0; index < tokens.length - 1; index += 1) {
    if (tokens[index].kind === 'punct' && ['(', '[', '{'].includes(tokens[index].text)) {
      index = close(tokens, index);
      continue;
    }
    if (tokens[index].text === sign[0] && tokens[index + 1].text === sign[1]) return index;
  }
  return -1;
}

/** A typed expression, never a guessed prompt or synthesized expected value. */
export function expression(tokens, context) {
  if (!tokens.length) throw new Error('empty native expression');
  if (tokens[0].text === '&') return expression(tokens.slice(1), context);
  for (const sign of ['||', '&&', '==', '!=']) {
    const at = operator(tokens, sign);
    if (at >= 0) {
      const left = expression(tokens.slice(0, at), context);
      const right = expression(tokens.slice(at + 2), context);
      requireType(right.type, left.type);
      if (sign === '||' || sign === '&&') requireType(left.type, 'boolean');
      return { kind: 'binary', sign, left, right, type: 'boolean' };
    }
  }
  if (tokens[0].text === '!') {
    const value = expression(tokens.slice(1), context);
    requireType(value.type, 'boolean');
    return { kind: 'not', value, type: 'boolean' };
  }
  let value;
  let cursor = 1;
  const first = tokens[0];
  if (first.kind === 'string') value = { kind: 'literal', value: first.text, type: 'string' };
  else if (first.kind === 'number' && /^\d+$/u.test(first.text)) {
    const integer = Number(first.text);
    if (!Number.isSafeInteger(integer)) throw new Error('native integer exceeds exact JavaScript range');
    value = { kind: 'literal', value: integer, type: 'number' };
  }
  else if (['true', 'false'].includes(first.text)) value = { kind: 'literal', value: first.text === 'true', type: 'boolean' };
  else if (first.text === '(') {
    cursor = close(tokens, 0) + 1;
    const { parts, comma } = tupleParts(tokens.slice(1, cursor - 1));
    if (!comma && parts.length === 1) value = expression(parts[0], context);
    else {
      const values = parts.map(part => expression(part, context));
      value = { kind: 'tuple', values, type: 'tuple:' + JSON.stringify(values.map(entry => entry.type)) };
    }
  } else if (first.text === '[') {
    cursor = close(tokens, 0) + 1;
    const values = split(tokens.slice(1, cursor - 1)).map(part => expression(part, context));
    for (const entry of values) requireType(entry.type, values[0].type);
    value = { kind: 'array', values, type: 'vec:' + (values[0]?.type ?? 'empty') };
  } else if (first.kind === 'word') {
    let name = first.text;
    while (tokens[cursor]?.text === ':' && tokens[cursor + 1]?.text === ':' && tokens[cursor + 2]?.kind === 'word') {
      name += '::' + tokens[cursor + 2].text;
      cursor += 3;
    }
    if (tokens[cursor]?.text === '(') {
      const end = close(tokens, cursor);
      const binding = context.calls.get(name);
      if (!binding) throw new Error('unsupported native call ' + name);
      const args = split(tokens.slice(cursor + 1, end)).map(part => expression(part, context));
      const expected = binding.signature ?? (binding.kind === 'solver' ? [] : ['string']);
      if (args.length !== expected.length) throw new Error('native call arity ' + name);
      args.forEach((arg, index) => requireType(arg.type, expected[index]));
      if (binding.kind === 'codec' && args[0].origin !== 'compiled-package-notation') throw new Error('unsupported native codec input origin');
      value = { kind: 'call', binding, args, type: binding.returnType ?? (binding.kind === 'helper' ? 'response' : callTypes[binding.kind]) };
      cursor = end + 1;
    } else if (tokens[cursor]?.text === '{') {
      const schema = context.schemas.get(name);
      if (!schema) throw new Error('unknown native record ' + name);
      const end = close(tokens, cursor);
      const entries = split(tokens.slice(cursor + 1, end)).map(part => {
        if (part[0]?.kind !== 'word' || part[1]?.text !== ':') throw new Error('unsupported record field');
        const entry = expression(part.slice(2), context);
        requireType(entry.type, schema[part[0].text]);
        return [part[0].text, entry];
      });
      if (new Set(entries.map(([key]) => key)).size !== entries.length
        || entries.length !== Object.keys(schema).length) throw new Error('incomplete native record');
      value = { kind: 'record', entries, type: 'record:' + name };
      cursor = end + 1;
    } else {
      const binding = context.environment.get(name);
      if (!binding) throw new Error('unbound native value ' + name);
      value = { kind: 'reference', name, type: binding.type, origin: binding.origin };
    }
  } else throw new Error('unsupported native expression ' + first.text);
  while (cursor < tokens.length) {
    if (tokens[cursor]?.text !== '.' || tokens[cursor + 1]?.kind !== 'word') throw new Error('unsupported native expression tail');
    const name = tokens[cursor + 1].text;
    cursor += 2;
    if (tokens[cursor]?.text !== '(') {
      const schema = value.type.startsWith('record:') ? context.schemas.get(value.type.slice(7)) : fields[value.type];
      const type = schema?.[name];
      if (!type) throw new Error(`unsupported native field ${value.type}.${name}`);
      value = { kind: 'field', target: value, name, type };
      continue;
    }
    const end = close(tokens, cursor);
    const parts = split(tokens.slice(cursor + 1, end));
    let args;
    let type;
    if (name === 'any' && value.type.startsWith('iter:')) {
      if (parts.length !== 1 || parts[0][0]?.text !== '|' || parts[0][1]?.kind !== 'word' || parts[0][2]?.text !== '|') throw new Error('unsupported native predicate');
      const parameter = parts[0][1].text;
      const nested = { ...context, environment: new Map(context.environment) };
      nested.environment.set(parameter, { type: value.type.slice(5) });
      const body = expression(parts[0].slice(3), nested);
      requireType(body.type, 'boolean');
      args = [{ kind: 'lambda', parameter, body }];
      type = 'boolean';
    } else {
      args = parts.map(part => expression(part, context));
      let signature = methodTypes[value.type]?.[name];
      if (name === 'expect' && /^(result|option):/u.test(value.type)) signature = [value.type.split(':')[1], 'string'];
      if (name === 'iter' && value.type.startsWith('vec:')) signature = ['iter:' + value.type.slice(4)];
      if (name === 'contains' && value.type === 'vec:string') signature = ['boolean', 'string'];
      if (name === 'len' && value.type.startsWith('vec:')) signature = ['number'];
      if (name === 'is_empty' && value.type.startsWith('vec:')) signature = ['boolean'];
      if (['clone', 'to_owned'].includes(name)) signature = [value.type];
      if (name === 'to_string' && value.type === 'string') signature = ['string'];
      if (!signature || signature.length - 1 !== args.length) throw new Error(`unsupported native method ${value.type}.${name}`);
      args.forEach((arg, index) => requireType(arg.type, signature[index + 1]));
      type = signature[0];
    }
    const origin = name === 'links_notation' && value.type === 'package' ? 'compiled-package-notation'
      : ['clone', 'to_owned', 'to_string'].includes(name) ? value.origin : undefined;
    value = { kind: 'method', target: value, name, args, type, origin };
    cursor = end + 1;
  }
  return value;
}

async function argumentsOf(nodes, environment, runtime) {
  const values = [];
  for (const node of nodes) values.push(await evaluate(node, environment, runtime));
  return values;
}

/** Evaluate only registered typed operations over actually observed values. */
export async function evaluate(node, environment, runtime) {
  if (node.kind === 'literal') return node.value;
  if (node.kind === 'reference') {
    if (!environment.has(node.name)) throw new Error('missing observed native binding ' + node.name);
    return environment.get(node.name);
  }
  if (node.kind === 'field') return (await evaluate(node.target, environment, runtime))[node.name];
  if (node.kind === 'array' || node.kind === 'tuple') {
    const values = [];
    for (const value of node.values) values.push(await evaluate(value, environment, runtime));
    return values;
  }
  if (node.kind === 'record') {
    const result = {};
    for (const [key, value] of node.entries) result[key] = await evaluate(value, environment, runtime);
    return result;
  }
  if (node.kind === 'not') return !(await evaluate(node.value, environment, runtime));
  if (node.kind === 'binary') {
    const left = await evaluate(node.left, environment, runtime);
    if (node.sign === '||' && left) return true;
    if (node.sign === '&&' && !left) return false;
    const right = await evaluate(node.right, environment, runtime);
    if (node.sign === '||' || node.sign === '&&') return right;
    const equal = isDeepStrictEqual(left, right);
    return node.sign === '==' ? equal : !equal;
  }
  if (node.kind === 'call') {
    const args = await argumentsOf(node.args, environment, runtime);
    const kind = node.binding.kind;
    if (kind === 'solver') return runtime.host;
    if (kind === 'historySolver') return solve(runtime.host, args[0], args[1], runtime);
    if (kind === 'pureHelper') {
      const values = new Map(node.binding.parameters.map((parameter, index) => [parameter.name, args[index]]));
      return evaluate(node.binding.body, values, runtime);
    }
    if (kind === 'user' || kind === 'assistant') return { role: kind, content: args[0] };
    if (kind === 'compiler') {
      installTextHost();
      const value = compileNaturalLanguageSkill(args[0]);
      runtime.observations.push({ operation: 'compile_natural_language_skill', description: args[0], status: value.status, packageId: value.id });
      return { ok: value.status === 'compiled', value, error: value.status };
    }
    if (kind === 'codec') {
      try { return { ok: true, value: parseLino(args[0]) }; }
      catch (error) { return { ok: false, error: error.message }; }
    }
    if (kind === 'helper') return solve(runtime.host, args[0], [], runtime);
  }
  if (node.kind === 'method') {
    const target = await evaluate(node.target, environment, runtime);
    if (node.name === 'any') {
      const lambda = node.args[0];
      for (const value of target) {
        const nested = new Map(environment);
        nested.set(lambda.parameter, value);
        if (await evaluate(lambda.body, nested, runtime)) return true;
      }
      return false;
    }
    const args = await argumentsOf(node.args, environment, runtime);
    if (node.name === 'expect') {
      if (!target.ok) throw new Error(`native expectation failed: ${args[0]} (${target.error ?? 'None'})`);
      return target.value;
    }
    if (node.name === 'solve') return solve(target, args[0], [], runtime);
    if (node.name === 'solve_with_history') return solve(target, args[0], args[1], runtime);
    if (node.name === 'replay') {
      const value = target.replay(args[0]);
      return { ok: value !== null, value };
    }
    if (node.name === 'links_notation') return target.linksNotation();
    if (node.name === 'link_records') return target.linkRecords();
    if (['clone', 'to_owned', 'to_string', 'iter'].includes(node.name)) return target;
    if (node.name === 'len') return target.length;
    if (node.name === 'is_empty') return target.length === 0;
    if (node.name === 'contains') return target.includes(args[0]);
    if (node.name === 'starts_with') return target.startsWith(args[0]);
    if (node.name === 'to_lowercase') return target.toLowerCase();
  }
  throw new Error('unsupported typed runtime operation ' + node.kind);
}

async function solve(host, prompt, history, runtime) {
  const observed = await host.solve(prompt, history);
  const answer = symbolicFromWorker(observed, history);
  runtime.observations.push({ operation: 'solve', prompt, history, answer });
  return answer;
}
