import {isDeepStrictEqual} from 'node:util';
import {generateScalars,scalarTestImportContract} from './native-scalar-registry.mjs';
import {

lex,

topLevelItems}

 from '../self-translation/lexer.mjs';

import {

nativeStringContract}

 from './native-string-contract.mjs';

// Source contracts and pure callable bodies for the native specification reader.
import {

 readFileSync }

 from 'node:fs';

import {

 resolve }

 from 'node:path';

import {

 createHash }

 from 'node:crypto';

import {

 tokenize,

testFunctions }

 from './rust-specification-cases.mjs';

import {

 close,

 split,

 expression }

 from './rust-specification-values.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');

const parameterTypes = new Map([
  ['&str',

 'string'],

 ['String',

 'string'],

 ['bool',

 'boolean'],

  ['&SymbolicAnswer',

 'response'],

 ['SymbolicAnswer',

 'response'],

  ['&[ConversationTurn]',

 'vec:turn'],

 ['&[String]',

 'vec:string'],

]);

/** Read free declarations only; methods inside impl blocks are not free calls. */
export function nativeFunctions(tokens) {

  const functions = [];

  for (let index = 0;

 index < tokens.length;

 index += 1) {

    if (tokens[index].text === '{') {

 index = close(tokens,

 index);

 continue;

 }

    if (tokens[index].text !== 'fn' || tokens[index + 2]?.text !== '(') continue;

    const paramsEnd = close(tokens,

 index + 2);

    let open = paramsEnd + 1;

    while (open < tokens.length && tokens[open].text !== '{') open += 1;

    if (open === tokens.length) break;

    const end = close(tokens,

 open);

    functions.push({

 name: tokens[index + 1].text,

 public: tokens[index - 1]?.text === 'pub',

      parameters: tokens.slice(index + 3,

 paramsEnd),

 returns: tokens.slice(paramsEnd + 1,

 open),

      body: tokens.slice(open + 1,

 end) }

);

    index = end;

  }

  return functions;

}

function parametersOf(tokens) {

  return split(tokens).map(part => {

    if (part[0]?.kind !== 'word' || part[1]?.text !== ':') throw new Error('unsupported callable parameter');

    const nativeType = part.slice(2).map(token => token.text).join('');

    const type = parameterTypes.get(nativeType);

    if (!type) throw new Error('unsupported callable parameter type ' + nativeType);

    return {

 name: part[0].text,

 type }

;

  }

);

}

function returnedBody(tokens) {

  let body = tokens;

  if (body[0]?.text === 'return') body = body.slice(1);

  if (body.at(-1)?.text === ';') body = body.slice(0,

 -1);

  return body;

}

const pureMethods = new Set(['iter',

 'any',

 'contains',

 'starts_with',

 'to_lowercase',

  'len',

 'is_empty',

 'clone',

 'to_owned',

 'to_string']);

function pure(node) {

  if (['literal',

 'reference'].includes(node.kind)) return true;

  if (node.kind === 'field') return pure(node.target);

  if (node.kind === 'not') return pure(node.value);

  if (node.kind === 'binary') return pure(node.left) && pure(node.right);

  if (node.kind === 'lambda') return pure(node.body);

  return node.kind === 'method' && pureMethods.has(node.name)
    && pure(node.target) && node.args.every(pure);

}

/** Bind actual single-expression helpers, with no assumed implementation by name. */
export function pureHelperBinding(fn,

 context) {

  const parameters = parametersOf(fn.parameters);

  if (new Set(parameters.map(value => value.name)).size !== parameters.length) throw new Error('duplicate callable parameter');

  const result = fn.returns.map(token => token.text).join('');

  const returnType = parameterTypes.get(result.slice(2));

  if (!result.startsWith('->') || !returnType) throw new Error('unsupported callable return type');

  const nested = {

 ...context,

 environment: new Map(parameters.map(value => [value.name,

 {

 type: value.type }

])) }

;

  const body = expression(returnedBody(fn.body),

 nested);

  if (body.type !== returnType || !pure(body)) throw new Error('unsupported impure callable body');

  return {

 kind: 'pureHelper',

 parameters,

 signature: parameters.map(value => value.type),

 returnType,

 body }

;

}

function exportedHistory(tokens) {

  for (let index = 0;

 index < tokens.length;

 index += 1) {

    if (tokens[index].text === '{') {

 index = close(tokens,

 index);

 continue;

 }

    if (tokens[index].text !== 'pub' || tokens[index + 1]?.text !== 'use') continue;

    let end = index + 2;

    while (end < tokens.length && tokens[end].text !== ';') end += 1;

    const path = tokens.slice(index + 2,

 end);

    if (path[0]?.text === 'solver' && path[1]?.text === ':' && path[2]?.text === ':') {

      if (path[3]?.text === '{' && close(path,

 3) === path.length - 1) {

        if (split(path.slice(4,

 -1)).some(part => part.length === 1 && part[0].text === 'solve_with_history')) return true;

      }

 else if (path.length === 4 && path[3].text === 'solve_with_history') return true;

    }

    index = end;

  }

  return false;

}

/** Verify the actual public default-profile producer before accepting its twin. */
export function historySourceContract(root) {

  const fixtures = ['rust/src/lib.rs',

 'rust/src/solver.rs'].map(path => {

    const file = resolve(root,

 path);

    const source = readFileSync(file,

 'utf8');

    return {

 file,

 source,

 sha256: hash(source) }

;

  }

);

  if (!exportedHistory(tokenize(fixtures[0].source))) throw new Error('native history producer is not publicly exported');

  const matches = nativeFunctions(tokenize(fixtures[1].source))
    .filter(fn => fn.name === 'solve_with_history' && fn.public);

  if (matches.length !== 1) throw new Error('ambiguous native history producer');

  const fn = matches[0];

  const parameters = parametersOf(fn.parameters);

  if (parameters.length !== 2 || parameters[0].type !== 'string' || parameters[1].type !== 'vec:turn'
    || fn.returns.map(token => token.text).join('') !== '->SymbolicAnswer') throw new Error('unsupported native history producer signature');

  const body = expression(returnedBody(fn.body),

 {

    environment: new Map(parameters.map(value => [value.name,

 {

 type: value.type }

])),

    calls: new Map([['UniversalSolver::default',

 {

 kind: 'solver' }

]]),

 schemas: new Map(),

  }

);

  if (body.kind !== 'method' || body.name !== 'solve_with_history'
    || body.target.kind !== 'call' || body.target.binding.kind !== 'solver'
    || body.args.length !== 2 || body.args.some((value,

 index) => value.kind !== 'reference'
      || value.name !== parameters[index].name)) throw new Error('unsupported native history producer body');

  return fixtures.map(({

 file,

 sha256 }

) => ({

 file,

 sha256 }

));

}

export function usesHistoryProducer(value) {

  if (!value || typeof value !== 'object') return false;

  if (value.kind === 'call' && value.binding.kind === 'historySolver') return true;

  return Object.values(value).some(child => Array.isArray(child)
    ? child.some(usesHistoryProducer) : usesHistoryProducer(child));

}

/** Qualify generated AST programs against their actual source before execution. */
export function generatedSourceContract(root,

 steps,

 source,

 body) {

  const witnesses = new Map();

  function visit(value) {

    if (!value || typeof value !== 'object') return;

    if ((value.kind === 'call' && value.binding.kind === 'substitutionProgram') || value.binding?.nativeSourceQualified) {

      const matched=topLevelItems(lex(source,

'Rust')).map(item=>source.slice(item.start,

item.end)).filter(raw=>testFunctions(tokenize(raw)).some(fn=>JSON.stringify(fn.body)===JSON.stringify(body)));

      if(matched.length!==1)throw Error('generated test source span is ambiguous');

      nativeStringContract(matched[0]);
      if(value.binding?.nativeSourceQualified) {
        const lexical=lex(matched[0],'Rust').filter(t=>t.type!=='comment'),functionAt=lexical.findIndex(t=>t.text==='fn');
        if(lexical.slice(0,functionAt).map(t=>t.text).join('')!=='#[test]'||lexical[functionAt+2]?.text!=='('||lexical[functionAt+3]?.text!==')')throw new Error('unknown scalar test scope or parameters');
        for(const witness of scalarTestImportContract(source,root,value.binding)){
          const file=resolve(root,witness.path);if(hash(readFileSync(file))!==witness.sha256)throw Error('unqualified scalar import witness');
          witnesses.set(file,{file,sha256:witness.sha256});
        }
      }

      for (const expected of value.binding.sourceWitnesses) {

        const file = resolve(root,

 expected.path),

 source = readFileSync(file,

 'utf8');

        if (hash(source) !== expected.sha256) throw new Error('unqualified generated native source');

        witnesses.set(file,

 {

 file,

 sha256: expected.sha256 }

);

      }

    }

    for (const child of Object.values(value)) if (Array.isArray(child)) child.forEach(visit);

 else visit(child);

  }

  visit(steps);
  verifyScalarBindings({steps,fixtures:[...witnesses.values()]});

  return [...witnesses.values()];

}


/** Rebuild scalar IR from actual source, before any runtime observation. */
export function verifyScalarBindings(program) {
  const cache=new Map();
  function visit(node) {
    if(!node||typeof node!=='object')return;
    if(node.binding?.nativeSourceQualified) {
      const binding=node.binding,expected=binding.sourceWitnesses;
      if(!Array.isArray(expected)||expected.length!==2)throw Error('unknown scalar source witness schema');
      const fixtures=expected.map(witness=>program.fixtures.find(fixture=>fixture.file?.endsWith('/'+witness.path)&&fixture.sha256===witness.sha256));
      if(fixtures.some(fixture=>!fixture))throw Error('scalar source fixture absent');
      const key=fixtures.map(fixture=>fixture.file+':'+fixture.sha256).join('|');
      if(!cache.has(key))cache.set(key,generateScalars(readFileSync(fixtures[0].file,'utf8'),expected[0].path,binding.path.split('::')[1],readFileSync(fixtures[1].file,'utf8')).programs);
      const actual=cache.get(key).find(candidate=>candidate.path===binding.path);
      if(!actual||!isDeepStrictEqual(actual,binding))throw Error('scalar program no longer matches source AST');
    }
    for(const child of Object.values(node))if(Array.isArray(child))child.forEach(visit);else visit(child);
  }
  visit(program.steps);
}
