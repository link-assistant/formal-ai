import { interpretOwnedStringArray } from './owned-string-array.mjs';
import { snakeFromCamel } from './ir.mjs';

const PROFILE = 'owned-string-array-rust-v1';

function scalarString(value) {
  for (let index = 0;
 index < value.length;
 index++) {
    const unit = value.charCodeAt(index);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = value.charCodeAt(++index);
      if (!(next >= 0xdc00 && next <= 0xdfff)) throw new Error('UnpairedSurrogate');
    } else if (unit >= 0xdc00 && unit <= 0xdfff) throw new Error('UnpairedSurrogate');
  }
  return value;
}

function rustLiteral(value) {
  scalarString(value);
  let text = '"';
  for (const character of value) {
    if (character === '"' || character === '\\') text += '\\' + character;
    else if (character === '\n') text += '\\n';
    else if (character === '\r') text += '\\r';
    else if (character === '\t') text += '\\t';
    else if (character === '\0') text += '\\0';
    else if (character.codePointAt(0) < 32 || character.codePointAt(0) === 127) {
      text += '\\u{' + character.codePointAt(0).toString(16) + '}';
    } else text += character;
  }
  return text + '"';
}

function freeze(value) {
  if (value && typeof value === 'object') {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
}

// Original source is the sole input. Supplied JSON IR, hashes and callbacks
// have no acceptance route. This temporary profile does not alter emitRust.
export function emitOwnedStringArrayRust(source) {
  try {
    if (typeof source !== 'string') throw new Error('OriginalSourceRequired');
    scalarString(source);
    const observation = interpretOwnedStringArray(source);
    if (observation.state !== 'Evaluated') throw new Error(observation.reason);
    const ir = observation.ir;
    const names = new Map(ir.bindings.map(binding => [binding.id,
 snakeFromCamel(binding.name)]));
    if (new Set(names.values()).size !== names.size) throw new Error('LoweredNameCollision');
    const mutated = new Set(ir.operations.filter(operation => operation.value?.kind === 'push')
      .map(operation => operation.value.receiver));
    const scalar = expression => expression.kind === 'string'
      ? rustLiteral(expression.value) : names.get(expression.id) + '.as_str()';
    const owned = expression => expression.kind === 'string'
      ? rustLiteral(expression.value) + '.to_owned()' : names.get(expression.id) + '.clone()';
    const expression = value => {
      if (value.kind === 'string') return rustLiteral(value.value) + '.to_owned()';
      if (value.kind === 'binding') return names.get(value.id);
      const receiver = names.get(value.receiver);
      if (!receiver) throw new Error('UnknownReceiverBinding');
      if (value.kind === 'length') return receiver + '.len() as u32';
      if (value.kind === 'join') return receiver + '.join(' + scalar(value.args[0]) + ')';
      if (value.kind === 'push') {
        const pushes = value.args.map(argument => receiver + '.push(' + owned(argument) + ');');
        return pushes.length === 0 ? '{ ' + receiver + '.len() as u32 }'
          : '{\n' + pushes.concat(receiver + '.len() as u32').map(line => '        ' + line).join('\n') + '\n    }';
      }
      throw new Error('UnsupportedTypedOperation');
    };
    const returnType = ir.returns === 'String' ? 'String' : ir.returns === 'Natural' ? 'u32' : null;
    if (!returnType) throw new Error('UnsupportedReturnType');
    const lines = ['pub fn ' + snakeFromCamel(ir.functionName) + '() -> ' + returnType + ' {'];
    const operationMap = [];
    for (const operation of ir.operations) {
      const startLine = lines.length + 1;
      let text;
      if (operation.kind === 'declareArray') {
        text = 'let ' + (mutated.has(operation.id) ? 'mut ' : '') + names.get(operation.id)
          + ': Vec<String> = vec![' + operation.values.map(owned).join(', ') + '];';
      } else if (operation.kind === 'declareString') {
        text = 'let ' + names.get(operation.id) + ': String = ' + owned(operation.value) + ';';
      } else if (operation.kind === 'effect') text = 'let _ = ' + expression(operation.value) + ';';
      else if (operation.kind === 'return') text = expression(operation.value);
      else throw new Error('UnsupportedTypedStatement');
      lines.push(...('    ' + text).split('\n'));
      operationMap.push({kind: operation.kind,
 sourceSpan: operation.span,
 startLine,
 endLine: lines.length,
 text});
    }
    lines.push('}');
    return freeze({state: 'SourceEmitted',
 profile: PROFILE,
 source,
 ir,
 code: lines.join('\n') + '\n',
      operationMap,
 naturalBound: [0,
 1024],
 encoding: 'UnicodeScalarStrings',
      allocation: 'FreshLocalVec',
 effects: 'OwnedLocalOnly',
 platform: 'Unsupported',
      native: 'Pending',
 admitted: false});
  } catch (error) {
    return freeze({state: 'Unsupported',
 profile: PROFILE,
 reason: error.message,
      code: null,
 ir: null,
 native: 'Pending',
 platform: 'Unsupported',
 admitted: false});
  }
}
