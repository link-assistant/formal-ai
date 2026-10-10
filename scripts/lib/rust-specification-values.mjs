import {isDeepStrictEqual as exactRecordEquality} from 'node:util';
const sourceOwnedRecordValues=new WeakMap();
// Typed expressions for the supported native specification subset. Each
// operation calls its actual JavaScript twin; unknown types stay unsupported.
import { isDeepStrictEqual } from 'node:util';
const sourceOwnedEnumValues=new WeakMap();
const sourceOwnedOptionValues=new WeakMap();
function ascii(value){if(typeof value!=='string'||/[^\x00-\x7f]/u.test(value))throw Error('unknown non-ASCII native string input');return value;}
function option(value,binding){
 const result=Object.freeze(Object.create(null));
 sourceOwnedOptionValues.set(result,{value,enumPath:binding.enumPath,sourceWitnesses:binding.sourceWitnesses});return result;
}
function optionBinding(context,enumPath){
 const matching=[...context.calls.values()].filter(binding=>binding.nativeOptionQualified&&binding.kind==='nativeOptionStringParse'&&binding.enumPath===enumPath);
 if(!matching.length)throw Error('unknown Option payload source');return matching[0];
}
export function contextualOptions(left,right,context){
 for(const [value,other] of [[left,right],[right,left]])if(value.type==='option:empty'){
  if(!other.type.startsWith('option:enum:'))throw Error('unknown native None payload type');
  value.binding=optionBinding(context,other.type.slice('option:enum:'.length));value.type=other.type;
 }
}
export function nativeEquality(left,right,type){
 if(type?.includes('option:')&&!type.startsWith('option:enum:'))throw Error('unknown native Option collection equality');
 if(!type?.startsWith('option:enum:'))return isDeepStrictEqual(left,right);
 const first=sourceOwnedOptionValues.get(left),second=sourceOwnedOptionValues.get(right),enumPath=type.slice('option:enum:'.length);
 if(!first||!second||first.enumPath!==enumPath||second.enumPath!==enumPath||!isDeepStrictEqual(first.sourceWitnesses,second.sourceWitnesses))throw Error('unowned/foreign native Option equality');
 if(first.value===null||second.value===null)return first.value===second.value;
 const firstEnum=sourceOwnedEnumValues.get(first.value),secondEnum=sourceOwnedEnumValues.get(second.value);
 if(!firstEnum||!secondEnum||firstEnum.enumPath!==enumPath||secondEnum.enumPath!==enumPath)throw Error('unowned native Option payload');
 return firstEnum.variant===secondEnum.variant;
}
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
function qualifyRecordInitializer(type, value, owner) {
  if (type.kind === 'integer') {
    const literal = value.kind === 'literal' && value.type === 'number';
    const constant = value.kind === 'nativeValue' && value.binding.kind === 'nativeConstant'
      && value.binding.nativeSourceQualified && value.binding.nativeType === type.native;
    const integer = literal ? value.value : constant ? value.binding.value : undefined;
    if (!Number.isSafeInteger(integer) || Object.is(integer, -0) || integer < 0
      || integer > (type.native === 'u8' ? 255 : 65535)) throw Error('unknown native integer field provenance or bounds');
    return;
  }
  if (type.kind === 'boolean') {
    if (value.kind === 'literal' && value.type === 'boolean' && typeof value.value === 'boolean') return;
    if (value.kind === 'nativeValue' && value.binding.kind === 'nativeConstant'
      && value.binding.nativeSourceQualified && value.binding.nativeType === 'bool') return;
    throw Error('unknown native boolean field provenance');
  }
  if (type.kind === 'string') {
    if (value.kind !== 'method' || !['to_owned', 'to_string'].includes(value.name)
      || value.args.length || value.target.kind !== 'literal' || value.target.type !== 'string') {
      throw Error('unknown owned String initializer or native move');
    }
    return;
  }
  if (type.kind === 'tuple') {
    if (value.kind !== 'tuple' || value.values.length !== type.elements.length) throw Error('unknown native tuple initializer');
    value.values.forEach((element, index) => qualifyRecordInitializer(type.elements[index], element, owner));
    return;
  }
  if (type.kind === 'record') {
    const fresh = value.kind === 'record' || (value.kind === 'call' && value.binding.kind === 'nativeRecordDefault');
    if (!fresh || !value.binding?.nativeRecordQualified || value.type !== type.type
      || !isDeepStrictEqual(value.binding.sourceWitnesses, owner.sourceWitnesses)) throw Error('unknown nested record initializer ownership');
    return;
  }
  throw Error('unknown native field initializer type');
}

export function expression(tokens, context) {
  if (!tokens.length) throw new Error('empty native expression');
  if (tokens[0].text === '&') return expression(tokens.slice(1), context);
  if(tokens[0].kind==='word'&&tokens[0].text==='None'){
    if(tokens.length!==1||context.environment.has('None'))throw Error('unknown/shadowed None');
    return {kind:'nativeOptionNone',type:'option:empty'};
  }
  if(tokens[0].kind==='word'&&tokens[0].text==='Some'){
    if(context.environment.has('Some')||tokens[1]?.text!=='('||close(tokens,1)!==tokens.length-1)throw Error('unknown/shadowed Some');
    const parts=split(tokens.slice(2,-1));if(parts.length!==1)throw Error('unknown Some arity');
    if(parts[0][0]?.text==='&')throw Error('unknown borrowed Option payload');
    const value=expression(parts[0],context);if(!value.type.startsWith('enum:')||value.nativeBorrowed)throw Error('unknown Option payload ownership');
    const binding=optionBinding(context,value.type.slice(5));
    return {kind:'nativeOptionSome',value,binding,type:'option:'+value.type};
  }
  for (const sign of ['||', '&&', '==', '!=', '<=', '>=']) {
    const at = operator(tokens, sign);
    if (at >= 0) {
      const left = expression(tokens.slice(0, at), context);
      const right = expression(tokens.slice(at + 2), context);
      contextualOptions(left,right,context);
      requireType(right.type, left.type);
      if(left.type.startsWith('record:formal_ai::'))throw Error('unknown native record equality');
      if (sign === '||' || sign === '&&') requireType(left.type, 'boolean');
      if (sign === '<=' || sign === '>=') requireType(left.type, 'number');
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
      if(values.some(entry=>entry.type.includes('option:')))throw Error('unknown native Option collection ownership');
      if(values.some(entry=>entry.type.includes('record:formal_ai::')))throw Error('untracked native record collection ownership');
      value = { kind: 'tuple', values, type: 'tuple:' + JSON.stringify(values.map(entry => entry.type)) };
    }
  } else if (first.text === '[') {
    cursor = close(tokens, 0) + 1;
    const values = split(tokens.slice(1, cursor - 1)).map(part => expression(part, context));
    if(values.some(entry=>entry.type.includes('option:')))throw Error('unknown native Option collection ownership');
      if(values.some(entry=>entry.type.includes('record:formal_ai::')))throw Error('untracked native record collection ownership');
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
      if (context.environment.has(name)) throw new Error('native call is shadowed by a value');
      const binding = context.calls.get(name);
      if (!binding) throw new Error('unsupported native call ' + name);
      if(binding.nativeSourceQualified&&context.environment.has(first.text))throw Error('native scalar path head is shadowed');
      if(['nativeConstant','nativeEnumVariant','nativeRecordConstructor'].includes(binding.kind))throw new Error('native scalar value is not callable');
      const args = split(tokens.slice(cursor + 1, end)).map(part => expression(part, context));
      if(binding.kind==='nativeOptionStringParse'){
        if(args.length!==1||!(args[0].kind==='literal'&&args[0].type==='string'||args[0].kind==='call'&&args[0].binding.kind==='nativeEnumStringMatch'))throw Error('unknown borrowed native string argument ownership');
        if(args[0].kind==='literal')ascii(args[0].value);
      }
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
      const recordBinding = context.calls.get(name);
      const entries = split(tokens.slice(cursor + 1, end)).map(part => {
        if (part[0]?.kind !== 'word' || part[1]?.text !== ':') throw new Error('unsupported record field');
        const entry = expression(part.slice(2), context);
        if(!Object.hasOwn(schema,part[0].text))throw Error('undeclared native record field'); requireType(entry.type, schema[part[0].text]);
        if (recordBinding?.nativeRecordQualified) qualifyRecordInitializer(recordBinding.nativeFields[part[0].text], entry, recordBinding);
        return [part[0].text, entry];
      });
      if (new Set(entries.map(([key]) => key)).size !== entries.length
        || entries.length !== Object.keys(schema).length) throw new Error('incomplete native record');
      const binding=context.calls.get(name);
      if(binding?.nativeRecordQualified&&context.environment.has(first.text))throw Error('shadowed native record owner');
      if(binding?.nativeRecordQualified&&entries.some(([,entry])=>entry.kind==='reference'&&entry.type.startsWith('record:')))throw Error('unknown native record field move');
      value = { kind: 'record', entries, ...(binding?.nativeRecordQualified?{binding}:{}), type: binding?.returnType??'record:' + name };
      cursor = end + 1;
    } else {
      const generated=context.calls.get(name);
      if(['nativeConstant','nativeEnumVariant'].includes(generated?.kind)) {
        if(context.environment.has(first.text))throw new Error('native scalar binding is shadowed');
        value={kind:'nativeValue',binding:generated,type:generated.returnType};
      } else {
        const binding = context.environment.get(name);
        if (!binding) throw new Error('unbound native value ' + name);
        value = { kind: 'reference', name, type: binding.type, origin: binding.origin, ...(binding.nativeBorrowed?{nativeBorrowed:true}:{}) };
      }
    }
  } else throw new Error('unsupported native expression ' + first.text);
  while (cursor < tokens.length) {
    if (tokens[cursor]?.text !== '.' || tokens[cursor + 1]?.kind !== 'word') throw new Error('unsupported native expression tail');
    const name = tokens[cursor + 1].text;
    cursor += 2;
    if (tokens[cursor]?.text !== '(') {
      const schema = value.type.startsWith('record:') ? (context.schemas.get(value.type.slice(7))??context.calls.get(value.type.slice(7))?.fields) : fields[value.type];
      const type = schema&&Object.hasOwn(schema,name)?schema[name]:undefined;
      if (!type) throw new Error(`unsupported native field ${value.type}.${name}`);
      if(value.type.startsWith('record:formal_ai::')&&!['number','boolean'].includes(type))throw Error('unknown native record field move');
      value = { kind: 'field', target: value, name, type };
      continue;
    }
    const end = close(tokens, cursor);
    if(value.type.startsWith('record:formal_ai::')){
      const binding=context.calls.get(value.type.slice(7)+'::'+name);
      if(!['nativeRecordBorrow','nativeRecordCopyRead'].includes(binding?.kind)||end!==cursor+1)throw Error('unknown native record borrowed method');
      value={kind:'call',binding,args:[value],type:binding.returnType};cursor=end+1;continue;
    }
    if(value.type.startsWith('enum:')) {
      const binding=context.calls.get(value.type.slice(5)+'::'+name);
      if(!['nativeEnumScalarMatch','nativeEnumStringMatch'].includes(binding?.kind)||end!==cursor+1)throw new Error('unknown native enum scalar method');
      value={kind:'call',binding,args:[value],type:binding.returnType};cursor=end+1;continue;
    }
    if(value.type.startsWith('option:enum:'))throw Error('unknown native Option method contract');
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
      if (name === 'split_whitespace' && value.type === 'string') signature = ['iter:string'];
      if (name === 'count' && value.type === 'iter:string') signature = ['number'];
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
    if(node.binding) return ownedRecord(node.binding,result);
    return result;
  }
  if (node.kind === 'not') return !(await evaluate(node.value, environment, runtime));
  if (node.kind === 'binary') {
    const left = await evaluate(node.left, environment, runtime);
    if (node.sign === '||' && left) return true;
    if (node.sign === '&&' && !left) return false;
    const right = await evaluate(node.right, environment, runtime);
    if (node.sign === '||' || node.sign === '&&') return right;
    if (node.sign === '<=') return left <= right;
    if (node.sign === '>=') return left >= right;
    const equal = nativeEquality(left,right,node.left.type);
    return node.sign === '==' ? equal : !equal;
  }
  if(node.kind==='nativeOptionNone')return option(null,node.binding);
  if(node.kind==='nativeOptionSome'){
    const value=await evaluate(node.value,environment,runtime),owned=sourceOwnedEnumValues.get(value);
    if(!owned||owned.enumPath!==node.binding.enumPath||!isDeepStrictEqual(owned.sourceWitnesses,node.binding.sourceWitnesses))throw Error('unowned/foreign Some payload');
    return option(value,node.binding);
  }
  if(node.kind==='nativeValue') {
    const binding=node.binding;
    if(binding.kind==='nativeConstant') {
      runtime.observations.push({operation:binding.path,result:binding.value});return binding.value;
    }
    if(binding.kind!=='nativeEnumVariant')throw new Error('unknown native scalar value');
    const value=Object.freeze({nativeEnum:binding.enumPath,variant:binding.variant});
    sourceOwnedEnumValues.set(value,{enumPath:binding.enumPath,variant:binding.variant,sourceWitnesses:binding.sourceWitnesses});
    return value;
  }
  if (node.kind === 'call') {
    const args = await argumentsOf(node.args, environment, runtime);
    const kind = node.binding.kind;
    if(kind==='nativeRecordDefault'){
      if(args.length)throw Error('unknown native default arity');
      const result=ownedRecord(node.binding,{...node.binding.defaults});runtime.observations.push({operation:node.binding.path,result});return result;
    }
    if(kind==='nativeRecordBorrow'||kind==='nativeRecordCopyRead'){
      if(args.length!==1)throw Error('unknown native record borrow arity');
      const owner=sourceOwnedRecordValues.get(args[0]);
      if(!owner||owner.recordPath!==node.binding.recordPath||!exactRecordEquality(owner.sourceWitnesses,node.binding.sourceWitnesses))throw Error('unowned native record borrow');
      if(kind==='nativeRecordCopyRead'&&(node.binding.copy!==true||owner.copy!==true))throw Error('unqualified native by-value record Copy capability');
      const result=borrowedPrimitive(node.binding.body,args[0]);runtime.observations.push({operation:node.binding.path,result});return result;
    }
    if(kind==='nativeEnumDefault'){
      if(args.length!==0)throw Error('unknown derived enum default arity');
      const value=Object.freeze({nativeEnum:node.binding.enumPath,variant:node.binding.variant});
      sourceOwnedEnumValues.set(value,{enumPath:node.binding.enumPath,variant:node.binding.variant,sourceWitnesses:node.binding.sourceWitnesses});
      runtime.observations.push({operation:node.binding.path,result:value});return value;
    }
    if(kind==='nativeEnumScalarMatch'||kind==='nativeEnumStringMatch') {
      if(args.length!==1)throw new Error('unknown native enum arity');
      const owned=sourceOwnedEnumValues.get(args[0]);
      if(!owned||Object.keys(args[0]).length!==2||args[0].nativeEnum!==owned.enumPath||args[0].variant!==owned.variant||owned.enumPath!==node.binding.enumPath||!isDeepStrictEqual(owned.sourceWitnesses,
node.binding.sourceWitnesses))throw new Error('unowned or foreign native enum value');

      const match=node.binding.cases.find(entry=>entry.variant===owned.variant);
      if(!match)throw new Error('unknown source enum variant');
      runtime.observations.push({operation:node.binding.path,arguments:args,result:match.value});return match.value;
    }
    if(kind==='nativeOptionStringParse'){
      if(args.length!==1)throw Error('unknown native string parse arity');let input=ascii(args[0]);
      if(node.binding.normalized)input=input.replace(/^[\x09-\x0d\x20]+|[\x09-\x0d\x20]+$/gu,'').replace(/[A-Z]/gu,character=>String.fromCharCode(character.charCodeAt(0)+32));
      const selected=node.binding.cases.find(entry=>entry.input===input);let value=null;
      if(selected){value=Object.freeze({nativeEnum:node.binding.enumPath,variant:selected.variant});sourceOwnedEnumValues.set(value,{enumPath:node.binding.enumPath,variant:selected.variant,sourceWitnesses:node.binding.sourceWitnesses});}
      const result=option(value,node.binding);runtime.observations.push({operation:node.binding.path,arguments:args,result});return result;
    }
    if (kind === 'substitutionProgram') {
      if (args.length !== 2 || args.some(value => typeof value !== 'string')) throw new Error('invalid observed string arguments');
      for (const value of args) for (const character of value) {
        const point = character.codePointAt(0);
        if (point >= 0xd800 && point <= 0xdfff) throw new Error('unqualified native malformed Unicode string');
      }
      const program = node.binding.program;
      const selected = program.alternatives.find(value => !value.fallback && value.selector === args[program.selectorArgument])
        ?? program.alternatives.find(value => value.fallback);
      if (!selected) throw new Error('substitution fallback absent');
      let result = args[program.inputArgument];
      for (const [from, to] of selected.pairs) {
        if (from === '') throw new Error('unknown empty-pattern Unicode replacement');
        result = result.split(from).join(to);
      }
      runtime.observations.push({ operation: node.binding.path, arguments: args, result });
      return result;
    }
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
    if (node.name === 'split_whitespace') {
      if (/[^\x00-\x7f]/u.test(target)) throw new Error('unsupported non-ASCII native whitespace profile');
      return target.split(/[\t\n\v\f\r ]+/u).filter(value => value !== '');
    }
    if (node.name === 'count') return target.length;
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

function ownedRecord(binding,value){
  if(Object.keys(value).length!==Object.keys(binding.nativeFields).length)throw Error('unknown native record schema');
  for(const [field,type]of Object.entries(binding.nativeFields))validateRecordField(type,value[field],binding.sourceWitnesses);
  Object.freeze(value);sourceOwnedRecordValues.set(value,{recordPath:binding.recordPath,sourceWitnesses:binding.sourceWitnesses,copy:binding.copy===true});return value;
}
function validateRecordField(type,value,witnesses){
  if(type.kind==='integer'){if(!Number.isInteger(value)||Object.is(value,-0)||value<0||value>(type.native==='u8'?255:65535))throw Error('unknown native integer field range');return;}
  if(type.kind==='boolean'){if(typeof value!=='boolean')throw Error('unknown native bool field');return;}
  if(type.kind==='string'){
    if(typeof value!=='string')throw Error('unknown native String field');
    for(const character of value){const point=character.codePointAt(0);if(point>=0xd800&&point<=0xdfff)throw Error('malformed native String field');}return;
  }
  if(type.kind==='tuple'){if(!Array.isArray(value)||value.length!==type.elements.length)throw Error('unknown native tuple field');type.elements.forEach((element,index)=>validateRecordField(element,value[index],witnesses));Object.freeze(value);return;}
  if(type.kind==='record'){
    const owner=sourceOwnedRecordValues.get(value);if(!owner||'record:'+owner.recordPath!==type.type||!exactRecordEquality(owner.sourceWitnesses,witnesses)||!type.copy)throw Error('unknown native nested record ownership');return;
  }
  throw Error('unknown native record field capability');
}
function borrowedPrimitive(node,owner){
  if(node.kind==='literal')return node.value;
  if(node.kind==='projection')return node.path.reduce((value,key)=>value[key],owner);
  const left=borrowedPrimitive(node.left,owner);
  if(node.operator==='&&'&&!left)return false;if(node.operator==='||'&&left)return true;
  const right=borrowedPrimitive(node.right,owner);
  if(node.operator==='&&'||node.operator==='||')return right;
  if(node.operator==='>')return left>right;if(node.operator==='<')return left<right;
  if(node.operator==='==')return left===right;if(node.operator==='!=')return left!==right;
  throw Error('unknown borrowed primitive operation');
}
