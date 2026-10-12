import {createHash} from 'node:crypto';
import {lex,
 topLevelItems} from './lexer.mjs';
import {parseJavaScriptItem} from './frontends.mjs';
import {snakeFromCamel} from './ir.mjs';

const PROFILE = 'owned-string-array-v1';
const keywords = new Set('function return const let var class new this super async await yield throw import export default true false null undefined eval arguments'.split(' '));
const digest = source => createHash('sha256').update(Buffer.from(source)).digest('hex');
const fail = reason => { throw new Error(reason);
 };
function freeze(value) {
  if (value && typeof value === 'object') {
    for (const item of Object.values(value)) freeze(item);
    Object.freeze(value);
  }
  return value;
}

function compile(source) {
  if (typeof source !== 'string' || Buffer.byteLength(source) > 65536 || Buffer.from(source).toString('utf8') !== source) fail('MalformedOrBoundedSource');
  const tokens = lex(source,
 'JavaScript');
  if (topLevelItems(tokens).filter(item => !item.comment).length !== 1) fail('ExactlyOneOwnedFunctionRequired');
  const list = tokens.filter(token => token.type !== 'comment');
  let index = 0,
 returned = false;
  const peek = () => list[index];
  const next = () => list[index++] ?? fail('IncompleteSyntax');
  const expect = text => { const token = next();
 if (token.text !== text) fail('UnsupportedSyntax:' + token.text);
 return token;
 };
  const byte = offset => Buffer.byteLength(source.slice(0,
 offset));
  const span = (start,
 end) => ({start: byte(start),
 end: byte(end)});
  const identifier = () => {
    const token = next();
    if (token.type !== 'ident' || !/^[$A-Z_a-z][$0-9A-Z_a-z]*$/u.test(token.text) || keywords.has(token.text)) fail('UnsupportedBinding');
    snakeFromCamel(token.text);
    return token;
  };
  expect('function');
  const name = identifier();
  expect('(');
 expect(')');
 expect('{');
  const sourceSha256 = digest(source),
 bindings = new Map(),
 operations = [];
  const byId = id => [...bindings.values()].find(binding => binding.id === id);

  function string() {
    const token = next();
    if (token.type !== 'string' || /[\r\n]/u.test(token.text) || /\\0[0-9]/u.test(token.text)) fail('UnsupportedStringSyntax');
    const expression = parseJavaScriptItem(lex('const STRING_VALUE = ' + token.text + ';',
 'JavaScript'),
 '');
    if (expression.kind !== 'constant' || expression.value.kind !== 'string') fail('UnsupportedStringExpression');
    const value = expression.value.value;
    if (value.length > 16384) fail('StringProfileBound');
    return {kind: 'string',
 type: 'String',
 value,
 span: span(token.start,
 token.end)};
  }
  function stringExpression() {
    if (peek()?.type === 'string') return string();
    const token = identifier(),
 binding = bindings.get(token.text);
    if (!binding || binding.type !== 'String') fail('ExternalOrNonStringArgument');
    return {kind: 'binding',
 type: 'String',
 id: binding.id,
 span: span(token.start,
 token.end)};
  }
  const knownString = argument => argument.kind === 'string' ? argument.value : byId(argument.id).value;
  function method() {
    const token = identifier(),
 binding = bindings.get(token.text);
    if (!binding || binding.type !== 'OwnedDenseStringArray') fail('ExternalOrSharedReceiver');
    expect('.');
    const property = identifier();
    if (property.text === 'length') return {kind: 'length',
 type: 'Natural',
 receiver: binding.id,
 span: span(token.start,
 property.end)};
    if (!['push',
 'join'].includes(property.text)) fail('UnsupportedArrayOperation');
    expect('(');
    const args = [];
    if (peek()?.text !== ')') while (true) {
      args.push(stringExpression());
      if (peek()?.text !== ',') break;
      next();
      if (peek()?.text === ')') break;
    }
    const end = expect(')');
    if (property.text === 'join' && args.length !== 1) fail('ExplicitStringSeparatorRequired');
    if (property.text === 'push') {
      binding.count += args.length;
      binding.characters += args.reduce((sum,
 argument) => sum + knownString(argument).length,
 0);
      if (binding.count > 1024 || binding.characters > 16384) fail('ArrayProfileBound');
    } else if (binding.characters + Math.max(0,
 binding.count - 1) * knownString(args[0]).length > 16384) fail('JoinProfileBound');
    return {kind: property.text,
 type: property.text === 'push' ? 'Natural' : 'String',
 receiver: binding.id,
 args,
 span: span(token.start,
 end.end)};
  }
  while (peek()?.text !== '}') {
    if (returned) fail('StatementsAfterReturnUnsupported');
    const start = peek() ?? fail('IncompleteFunction');
    if (start.text === 'const') {
      next();
      const token = identifier();
      if (bindings.has(token.text) || token.text === name.text) fail('DuplicateOrShadowBinding');
      expect('=');
      const id = sourceSha256 + ':binding:' + byte(token.start),
 values = [];
      let binding;
      if (peek()?.text === '[') {
        next();
        if (peek()?.text !== ']') while (true) {
          values.push(string());
          if (peek()?.text !== ',') break;
          next();
          if (peek()?.text === ']') break;
        }
        expect(']');
        const characters = values.reduce((sum,
 value) => sum + value.value.length,
 0);
        if (values.length > 1024 || characters > 16384) fail('ArrayProfileBound');
        binding = {id,
 name: token.text,
 type: 'OwnedDenseStringArray',
 count: values.length,
 characters,
 span: span(token.start,
 token.end)};
        operations.push({kind: 'declareArray',
 id,
 type: binding.type,
 values,
 span: span(start.start,
 expect(';').end)});
      } else {
        const value = string();
        binding = {id,
 name: token.text,
 type: 'String',
 value: value.value,
 span: span(token.start,
 token.end)};
        operations.push({kind: 'declareString',
 id,
 type: 'String',
 value,
 span: span(start.start,
 expect(';').end)});
      }
      bindings.set(token.text,
 binding);
    } else if (start.text === 'return') {
      next();
      if (!peek() || /[\r\n\u2028\u2029]/u.test(source.slice(start.end,
 peek().start))) fail('AmbiguousReturnBoundary');
      const value = peek().type === 'string' ? string() : peek().type === 'ident' && bindings.get(peek().text)?.type === 'String' ? stringExpression() : method();
      operations.push({kind: 'return',
 value,
 type: value.type,
 span: span(start.start,
 expect(';').end)});
      returned = true;
    } else {
      const value = method();
      if (value.kind !== 'push') fail('OnlyOwnedPushStatementSupported');
      operations.push({kind: 'effect',
 value,
 type: 'OwnedLocalMutation',
 span: span(start.start,
 expect(';').end)});
    }
  }
  expect('}');
  if (index !== list.length || !returned) fail('IncompleteOrExtraProgram');
  return freeze({schema: 'owned-string-array-ir/v1',
 profile: PROFILE,
 sourceSha256,
 sourceBytes: Buffer.byteLength(source),
 functionName: name.text,
    bindings: [...bindings.values()].map(({count,
 characters,
 value,
 ...binding}) => binding),
 operations,
    grammar: 'All significant source tokens consumed; guarded sequential owned operations',
 effects: 'OwnedLocalOnly',
 returns: operations.at(-1).type});
}

function evaluate(ir) {
  const storage = new Map();
  const read = expression => {
    if (expression.kind === 'string') return expression.value;
    if (expression.kind === 'binding') return storage.get(expression.id);
    const array = storage.get(expression.receiver);
    if (expression.kind === 'push') {
      for (const argument of expression.args) array.values[array.length++] = read(argument);
      return array.length;
    }
    if (expression.kind === 'length') return array.length;
    if (expression.kind === 'join') {
      const separator = read(expression.args[0]);
      let text = '';
      for (let index = 0;
 index < array.length;
 index++) text += (index ? separator : '') + array.values[index];
      return text;
    }
    fail('UnsupportedTypedIR');
  };
  for (const operation of ir.operations) {
    if (operation.kind === 'declareArray') {
      const array = {values: Object.create(null),
 length: 0};
      for (const value of operation.values) array.values[array.length++] = value.value;
      storage.set(operation.id,
 array);
    } else if (operation.kind === 'declareString') storage.set(operation.id,
 operation.value.value);
    else if (operation.kind === 'effect') read(operation.value);
    else return read(operation.value);
  }
  fail('MissingReturn');
}

export function interpretOwnedStringArray(source) {
  try {
    const ir = compile(source),
 value = evaluate(ir);
    return {state: 'Evaluated',
 profile: PROFILE,
 ir,
 value,
 evaluated: true,
 effects: 'OwnedLocalOnly',
 externalEffects: 'Unsupported',
 platform: 'Unsupported',
 native: 'Pending',
 admitted: false};
  } catch (error) {
    return {state: 'Unsupported',
 profile: PROFILE,
 reason: error.message,
 ir: null,
 evaluated: false,
 effects: 'Unknown',
 platform: 'Unsupported',
 admitted: false};
  }
}
