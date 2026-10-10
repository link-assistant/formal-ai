// A conservative typed program reader for native constructor and dialog tests.
// Unsupported syntax rejects the complete case; every parsed assertion runs.
import { GENERATED_NATIVE_PROGRAMS } from './generated-native-programs.mjs';
import { readFileSync, realpathSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { tokenize } from './rust-specification-cases.mjs';
import { close, split, expression, evaluate, contextualOptions, nativeEquality } from './rust-specification-values.mjs';
import { iterationBindings, bindIteration } from './rust-specification-tuples.mjs';
import { nativeFunctions, pureHelperBinding, historySourceContract, usesHistoryProducer, generatedSourceContract, verifyScalarBindings } from './rust-specification-bindings.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');
const assertionNames = new Set(['assert', 'assert_eq', 'assert_ne']);
const canonicalCalls = new Map([
  ...GENERATED_NATIVE_PROGRAMS.map(binding => [binding.path, binding]),
  ['formal_ai::compile_natural_language_skill', { kind: 'compiler' }],
  ['formal_ai::UniversalSolver::default', { kind: 'solver' }],
  ['formal_ai::solve_with_history', { kind: 'historySolver', signature: ['string', 'vec:turn'] }],
  ['formal_ai::ConversationTurn::user', { kind: 'user' }],
  ['formal_ai::ConversationTurn::assistant', { kind: 'assistant' }],
  ['lino_objects_codec::format::parse_indented', { kind: 'codec' }],
]);

function registerImport(tokens, prefix, calls) {
  const group = tokens.findIndex(token => token.text === '{');
  if (group >= 0) {
    const base = prefix + tokens.slice(0, group).map(token => token.text).join('');
    for (const part of split(tokens.slice(group + 1, close(tokens, group)))) registerImport(part, base, calls);
    return;
  }
  const aliasAt = tokens.findIndex(token => token.text === 'as');
  const pathTokens = aliasAt < 0 ? tokens : tokens.slice(0, aliasAt);
  const path = prefix + pathTokens.map(token => token.text).join('');
  const alias = aliasAt < 0 ? path.split('::').at(-1) : tokens[aliasAt + 1]?.text;
  if (!calls.importedAliases) calls.importedAliases = new Set();
  if (calls.importedAliases.has(alias)) throw new Error('ambiguous native import alias');
  calls.importedAliases.add(alias);
  for (const name of [...calls.keys()]) if (name === alias || name.startsWith(alias + '::')) calls.delete(name);
  for (const [canonical, binding] of canonicalCalls) {
    if (path === canonical) calls.set(alias, binding);
    if (canonical.startsWith(path + '::')) calls.set(alias + canonical.slice(path.length), binding);
  }
}

function importedCalls(tokens) {
  const calls = new Map(canonicalCalls);
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index].text === '{') { index = close(tokens, index); continue; }
    if (tokens[index].text !== 'use') continue;
    let end = index + 1;
    while (end < tokens.length && tokens[end].text !== ';') end += 1;
    registerImport(tokens.slice(index + 1, end), '', calls);
    index = end;
  }
  return calls;
}


function declarationNames(tokens) {
  const names = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token.kind === 'punct' && ['{', '[', '('].includes(token.text)) {
      index = close(tokens, index);
      continue;
    }
    const name = tokens[index + 1];
    if (token.kind === 'word' && name?.kind === 'word'
      && ['enum', 'struct', 'type', 'mod', 'fn', 'const', 'static'].includes(token.text)) names.push(name.text);
  }
  return names;
}

function contextOf(source, file, root) {
  const tokens = tokenize(source);
  const calls = importedCalls(tokens);
  for (const alias of declarationNames(tokens)) {
    if (!calls.importedAliases?.has(alias) && alias !== 'formal_ai') continue;
    for (const [name, binding] of [...calls]) {
      if (name === alias || name.startsWith(alias + '::')
        || (alias === 'formal_ai' && binding.nativeSourceQualified)) calls.delete(name);
    }
  }
  const environment = new Map();
  const schemas = new Map([...calls].filter(([,binding])=>binding.kind==='nativeRecordConstructor').map(([name,binding])=>[name,binding.fields]));
  const fixtures = [file ? { file, sha256: hash(source) } : { source, sha256: hash(source) }];
  const context = { calls, environment, schemas, schemaWitnesses: [] };
  // Local functions shadow imported names even when their body is unsupported.
  const functions = nativeFunctions(tokens);
  for (const fn of functions) calls.delete(fn.name);
  for (const fn of functions) {
    if (functions.filter(value => value.name === fn.name).length !== 1) continue;
    try { calls.set(fn.name, pureHelperBinding(fn, context)); continue; }
    catch { /* Effectful and unknown helper bodies remain unbound. */ }
    if (fn.parameters.length !== 4 || fn.parameters[1].text !== ':' || fn.parameters[2].text !== '&'
      || fn.parameters[3].text !== 'str' || fn.returns.map(token => token.text).join('') !== '->SymbolicAnswer') continue;
    const argument = fn.parameters[0].text;
    let body = fn.body;
    if (body[0]?.text === 'return') body = body.slice(1);
    if (body.at(-1)?.text === ';') body = body.slice(0, -1);
    const nested = { ...context, environment: new Map([[argument, { type: 'string' }]]) };
    try {
      const value = expression(body, nested);
      if (value.kind === 'method' && value.name === 'solve' && value.target.kind === 'call'
        && value.target.binding.kind === 'solver' && value.args[0].kind === 'reference'
        && value.args[0].name === argument) {
        calls.set(fn.name, { kind: 'helper', sourceIdentity: hash(source), parameter: argument, profile: 'default' });
      }
    } catch { /* Unknown engine/configuration/effect stays unregistered. */ }
  }
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index].text === '{') { index = close(tokens, index); continue; }
    if (tokens[index].text !== 'const') continue;
    let end = index;
    while (end < tokens.length && tokens[end].text !== ';') end += 1;
    const definition = tokens.slice(index, end);
    const equals = definition.findIndex(token => token.text === '=');
    const name = definition[1]?.text;
    const rhs = definition.slice(equals + 1);
    try {
      let value;
      if (rhs[0]?.text === 'include_str' && rhs[1]?.text === '!' && rhs[2]?.text === '(' && rhs[3]?.kind === 'string' && rhs[4]?.text === ')' && rhs.length === 5) {
        const target = realpathSync(resolve(dirname(file), rhs[3].text));
        if (relative(root, target).startsWith('..')) throw new Error('fixture outside source root');
        const content = readFileSync(target, 'utf8');
        fixtures.push({ file: target, sha256: hash(content) });
        value = { kind: 'literal', type: 'string', value: content };
      } else value = expression(rhs, context);
      if (value.kind === 'literal') environment.set(name, value);
    } catch { /* A used unknown constant remains an unbound value. */ }
    index = end;
  }
  return { ...context, fixtures };
}

function recordHeader(tokens, cursor) {
  const name = tokens[cursor + 1];
  if (name?.kind !== 'word') throw new Error('unsupported native record declaration');
  let open = cursor + 2;
  const lifetimes = [];
  if (tokens[open]?.text === '<') {
    const start = ++open;
    while (open < tokens.length && tokens[open].text !== '>') open += 1;
    if (open === tokens.length) throw new Error('unclosed native lifetime declaration');
    const declaration = tokens.slice(start, open);
    if (declaration[0]?.text === ',' || declaration.some((token, index) => token.text === ',' && declaration[index - 1]?.text === ',')) throw new Error('empty native lifetime parameter');
    const parameters = split(declaration);
    if (!parameters.length) throw new Error('empty native lifetime declaration');
    for (const parameter of parameters) {
      if (parameter.length !== 2 || parameter[0].text !== "'" || parameter[1].kind !== 'word'
        || ['static', '_'].includes(parameter[1].text)) throw new Error('unsupported native lifetime parameter');
      if (lifetimes.includes(parameter[1].text)) throw new Error('duplicate native lifetime parameter');
      lifetimes.push(parameter[1].text);
    }
    open += 1;
  }
  if (tokens[open]?.text !== '{') throw new Error('unsupported native record declaration');
  return { name: name.text, open, lifetimes };
}

function recordSchema(tokens, lifetimes) {
  const schema = {};
  const fields = [];
  for (const field of split(tokens)) {
    if (field[0]?.kind !== 'word' || field[1]?.text !== ':') throw new Error('unsupported native record declaration');
    const type = field.slice(2);
    let lifetime = null;
    const borrowed = type[0]?.text === '&';
    if (borrowed && type[1]?.text === "'" && type.length === 4 && type[2].kind === 'word' && type[3].text === 'str') {
      lifetime = type[2].text;
      if (lifetime !== 'static' && !lifetimes.includes(lifetime)) throw new Error('undeclared native field lifetime ' + lifetime);
    } else if (!(type.length === 2 && borrowed && type[1].text === 'str')
      && !(type.length === 1 && type[0].text === 'String')) throw new Error('unsupported native record field type ' + type.map(token => token.text).join(''));
    if (Object.hasOwn(schema, field[0].text)) throw new Error('duplicate native record field');
    schema[field[0].text] = 'string';
    fields.push({ name: field[0].text, type: 'string', borrowed, lifetime });
  }
  for (const lifetime of lifetimes) if (!fields.some(field => field.lifetime === lifetime)) throw new Error('unused native lifetime parameter ' + lifetime);
  return { schema, fields };
}

const diagnosticOnly = value => ['literal', 'reference'].includes(value.kind)
  || (value.kind === 'field' && diagnosticOnly(value.target));

function bodyProgram(tokens, context) {
  const steps = [];
  for (let cursor = 0; cursor < tokens.length;) {
    if (tokens[cursor].text === 'struct') {
      const header = recordHeader(tokens, cursor);
      const end = close(tokens, header.open);
      const record = recordSchema(tokens.slice(header.open + 1, end), header.lifetimes);
      context.schemas.set(header.name, record.schema);
      context.schemaWitnesses.push({ name: header.name, declaredLifetimes: header.lifetimes, fields: record.fields });
      cursor = end + 1;
      if (tokens[cursor]?.text === ';') cursor += 1;
      continue;
    }
    if (tokens[cursor].text === 'for') {
      let inAt = cursor + 1;
      while (inAt < tokens.length && tokens[inAt].text !== 'in') {
        if (tokens[inAt].text === '(') inAt = close(tokens, inAt);
        inAt += 1;
      }
      if (inAt === tokens.length) throw new Error('unsupported native iteration binding');
      const pattern = tokens.slice(cursor + 1, inAt);
      const parameter = pattern.map(token => token.text).join('');
      let open = inAt + 1;
      while (open < tokens.length && tokens[open].text !== '{') {
        if (['(', '['].includes(tokens[open].text)) open = close(tokens, open);
        open += 1;
      }
      if (open === tokens.length) throw new Error('native loop body absent');
      const input = tokens.slice(inAt + 1, open);
      const values = expression(input, context);
      const vector = values.type.startsWith('vec:');
      const iterator = values.type.startsWith('iter:');
      if ((!vector && !iterator) || ['vec:empty', 'iter:empty'].includes(values.type)) throw new Error('native iteration needs known vector element type');
      const borrowed = input[0]?.text === '&' || iterator;
      const bindings = iterationBindings(pattern, values.type.slice(vector ? 4 : 5), borrowed);
      const end = close(tokens, open);
      const nested = { ...context, environment: new Map(context.environment), schemas: new Map(context.schemas) };
      for (const binding of bindings) nested.environment.set(binding.name, { type: binding.type, ...(borrowed?{nativeBorrowed:true}:{}) });
      steps.push({ kind: 'for', parameter, pattern, borrowed, bindings, values, body: bodyProgram(tokens.slice(open + 1, end), nested) });
      cursor = end + 1;
      continue;
    }
    let end = cursor;
    while (end < tokens.length && tokens[end].text !== ';') {
      if (['(', '[', '{'].includes(tokens[end].text)) end = close(tokens, end);
      end += 1;
    }
    const statement = tokens.slice(cursor, end);
    cursor = end + 1;
    if (!statement.length) continue;
    if (statement[0].text === 'let') {
      if (statement[1]?.kind !== 'word' || statement[2]?.text !== '=') throw new Error('unsupported native let binding');
      const value = expression(statement.slice(3), context);
      if(value.kind==='reference'&&value.type.startsWith('record:formal_ai::'))throw Error('untracked native record move');
      const name = statement[1].text;
      context.environment.set(name, { type: value.type, origin: value.origin });
      steps.push({ kind: 'let', name, value });
      continue;
    }
    if (assertionNames.has(statement[0].text) && statement[1]?.text === '!' && statement[2]?.text === '(' && close(statement, 2) === statement.length - 1) {
      const args = split(statement.slice(3, -1));
      const binary = statement[0].text !== 'assert';
      if (args.length < (binary ? 2 : 1)) throw new Error('native assertion arguments absent');
      const left = expression(args[0], context);
      const right = binary ? expression(args[1], context) : null;
      if(binary)contextualOptions(left,right,context);
      if(binary&&left.type.startsWith('record:formal_ai::'))throw Error('unknown record equality capability');
      if (binary && left.type !== right.type) throw new Error('native assertion type mismatch');
      if (!binary && left.type !== 'boolean') throw new Error('native assertion needs boolean');
      // Diagnostic formatting values may read data, but may not add effects.
      for (const diagnostic of args.slice(binary ? 2 : 1)) {
        const value = expression(diagnostic, context);
        if (!diagnosticOnly(value)) throw new Error('unsupported diagnostic side effect');
      }
      steps.push({ kind: 'assertion', name: statement[0].text, left, right });
      continue;
    }
    const value = expression(statement, context);
    if (value.kind !== 'method' || value.name !== 'expect') throw new Error('unsupported native statement effect');
    steps.push({ kind: 'evaluate', value });
  }
  return steps;
}

const assertionCount = steps => steps.reduce((count, step) => count + (step.kind === 'assertion' ? 1 : step.kind === 'for' ? assertionCount(step.body) : 0), 0);

/** Read all assertions, or return one complete-case rejection. */
export function typedProgramOf(body, { source, file = '', root = '' }) {
  try {
    if (file && readFileSync(file, 'utf8') !== source) throw new Error('native source identity mismatch');
    const context = contextOf(source, file, root);
    const initial = [...context.environment];
    const steps = bodyProgram(body, context);
    context.fixtures.push(...generatedSourceContract(root, steps, source, body));
    if (usesHistoryProducer(steps)) context.fixtures.push(...historySourceContract(root));
    const nativeAssertions = body.filter((token, index) => assertionNames.has(token.text) && body[index + 1]?.text === '!').length;
    if (!nativeAssertions || assertionCount(steps) !== nativeAssertions) throw new Error('native assertion conservation failure');
    return { program: { steps, initial, fixtures: context.fixtures, schemaWitnesses: context.schemaWitnesses, nativeAssertions } };
  } catch (error) { return { reason: error.message }; }
}

/** Execute original ordered operations; a failure is reported, never skipped. */
export async function executeTypedProgram(host, program) {
  const runtime = { host, observations: [], assertions: 0, iterations: [] };
  const environment = new Map(program.initial.map(([name, value]) => [name, value.value]));
  async function run(steps, values) {
    for (const step of steps) {
      if (step.kind === 'let') values.set(step.name, await evaluate(step.value, values, runtime));
      else if (step.kind === 'evaluate') await evaluate(step.value, values, runtime);
      else if (step.kind === 'for') {
        const entries = await evaluate(step.values, values, runtime);
        runtime.iterations.push({ parameter: step.parameter, length: entries.length });
        for (const value of entries) {
          const nested = new Map(values);
          for (const [name, observed] of bindIteration(step.pattern, value, step.borrowed)) nested.set(name, observed);
          await run(step.body, nested);
        }
      } else if (step.kind === 'assertion') {
        const left = await evaluate(step.left, values, runtime);
        const right = step.right ? await evaluate(step.right, values, runtime) : true;
        const equal = step.name === 'assert' ? left === true : nativeEquality(left,right,step.left.type);
        runtime.assertions += 1;
        if ((step.name === 'assert_ne' ? !equal : equal) !== true) throw new Error(`native ${step.name} failed: ${JSON.stringify(left).slice(0, 180)} expected ${JSON.stringify(right).slice(0, 180)}`);
      }
    }
  }
  try {
    for (const fixture of program.fixtures) {
      const observed = fixture.file ? readFileSync(fixture.file, 'utf8') : fixture.source;
      if (hash(observed) !== fixture.sha256) throw new Error('stale native fixture identity');
    }
    verifyScalarBindings(program);
    await run(program.steps, environment);
    return { status: 'passed', nativeAssertions: program.nativeAssertions, assertions: runtime.assertions,
      observations: runtime.observations, iterations: runtime.iterations };
  } catch (error) {
    return { status: 'failed', failure: error.message, nativeAssertions: program.nativeAssertions,
      assertions: runtime.assertions, observations: runtime.observations, iterations: runtime.iterations };
  }
}
