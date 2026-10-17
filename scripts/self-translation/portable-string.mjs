// A strict string-only sibling of portable-pure-v1 using the same Links IR.
// No source fragment is carried: unsupported constructs stop generation.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { toLino, fromLino, snakeFromCamel, canonicalRustName } from './ir.mjs';
const require = createRequire(import.meta.url);
const {
  parse
} = require('@babel/parser');
export const PROFILE = 'portable-string-v1';
const fail = reason => {
  throw new Error(PROFILE + ': ' + reason);
};
function scalarText(value) {
  for (const character of value) {
    const code = character.codePointAt(0);
    if (code >= 0xd800 && code <= 0xdfff) fail('UnicodeScalarTextRequired');
  }
  return value;
}
export function stringProgram(source) {
  let tree;
  try {
    tree = parse(source, {
      sourceType: 'module'
    });
  } catch (error) {
    return fail('InvalidJavaScriptSyntax:' + error.reasonCode);
  }
  const programs = [];
  for (const declaration of tree.program.body) {
    const node = declaration.declaration;
    if (declaration.type !== 'ExportNamedDeclaration' || node?.type !== 'FunctionDeclaration' || node.async || node.generator) fail('PureExportedFunctionRequired');
    const params = node.params.map(parameter => {
      if (parameter.type !== 'Identifier') fail('StringParameterRequired');
      const doc = (declaration.leadingComments ?? []).map(comment => {
        return comment.value;
      }).join('\n');
      if (!doc.includes('@param {string} ' + parameter.name)) fail('ExplicitStringParameterRequired');
      return {
        name: snakeFromCamel(parameter.name),
        type: 'string'
      };
    });
    if (new Set(params.map(parameter => {
      return parameter.name;
    })).size !== params.length) fail('UniqueParametersRequired');
    if (node.body.body.length !== 1 || node.body.body[0].type !== 'ReturnStatement') fail('SingleExpressionReturnRequired');
    const names = new Set(params.map(parameter => {
      return parameter.name;
    }));
    function expression(value) {
      if (value?.type === 'StringLiteral') return {
        kind: 'string',
        value: scalarText(value.value)
      };
      if (value?.type === 'Identifier') {
        const name = snakeFromCamel(value.name);
        if (!names.has(name)) fail('UnboundParameter');
        return {
          kind: 'variable',
          name
        };
      }
      if (value?.type === 'BinaryExpression' && ['+', '==='].includes(value.operator)) return {
        kind: 'binary',
        op: value.operator === '+' ? 'add' : 'equal',
        left: expression(value.left),
        right: expression(value.right)
      };
      if (value?.type === 'ConditionalExpression') return {
        kind: 'choose',
        cond: expression(value.test),
        then: expression(value.consequent),
        else: expression(value.alternate)
      };
      if (value?.type === 'CallExpression' && !value.optional && value.callee.type === 'MemberExpression' && !value.callee.computed && value.callee.property.name === 'replaceAll') {
        return {
          kind: 'method',
          name: 'literal-replace',
          receiver: expression(value.callee.object),
          args: value.arguments.map(expression)
        };
      }
      return fail('UnsupportedStringExpression:' + value?.type);
    }
    programs.push({
      kind: 'function',
      name: snakeFromCamel(node.id.name),
      exported: true,
      params,
      returns: 'string',
      body: [{
        kind: 'return',
        value: expression(node.body.body[0].argument)
      }]
    });
  }
  validateStringProgram(programs);
  return programs;
}
export function validateStringProgram(programs) {
  if (!Array.isArray(programs) || programs.length === 0 || programs.length > 64) fail('BoundedFunctionsRequired');
  const functions = new Set();
  for (const program of programs) {
    if (program.kind !== 'function' || !program.exported || program.returns !== 'string' || functions.has(program.name)) fail('StringFunctionRequired');
    try {
      canonicalRustName(program.name);
      program.params.forEach(parameter => canonicalRustName(parameter.name));
    } catch {
      return fail('PortableNameRequired');
    }
    functions.add(program.name);
    if (program.params.length > 16 || program.params.some(parameter => {
      return parameter.type !== 'string';
    })) fail('StringParametersRequired');
    const names = new Set(program.params.map(parameter => {
      return parameter.name;
    }));
    function type(value) {
      if (value.kind === 'string') {
        scalarText(value.value);
        return 'string';
      }
      if (value.kind === 'variable') {
        if (!names.has(value.name)) fail('UnboundParameter');
        return 'string';
      }
      if (value.kind === 'binary' && ['add', 'equal'].includes(value.op)) {
        if (type(value.left) !== 'string' || type(value.right) !== 'string') fail('StringOperandsRequired');
        return value.op === 'add' ? 'string' : 'boolean';
      }
      if (value.kind === 'choose') {
        if (type(value.cond) !== 'boolean' || type(value.then) !== 'string' || type(value.else) !== 'string') fail('StringConditionalRequired');
        return 'string';
      }
      if (value.kind === 'method' && value.name === 'literal-replace') {
        if (type(value.receiver) !== 'string' || value.args.length !== 2 || value.args.some(argument => {
          return argument.kind !== 'string';
        })) fail('LiteralReplacementRequired');
        value.args.forEach(argument => {
          return scalarText(argument.value);
        });
        if (value.args[0].value === '') fail('NonemptyLiteralNeedleRequired');
        if (value.args[1].value.includes('$')) fail('ReplacementTokenSemanticsNotPortable');
        return 'string';
      }
      return fail('UnsupportedStringIR:' + value.kind);
    }
    if (program.body.length !== 1 || program.body[0].kind !== 'return' || type(program.body[0].value) !== 'string') fail('StringReturnRequired');
  }
  return programs;
}
export function stringProgramToMeta(source) {
  return '# ' + PROFILE + '; JavaScript source is authoritative\n' + stringProgram(source).map(toLino).join('\n') + '\n';
}
const rustText = value => {
  return '"' + value.replace(/[\\"\n\r\t\0]/gu, character => {
    return {
      '\\': '\\\\',
      '"': '\\"',
      '\n': '\\n',
      '\r': '\\r',
      '\t': '\\t',
      '\0': '\\0'
    }[character];
  }) + '"';
};
function rustExpression(value) {
  if (value.kind === 'string') return rustText(value.value) + '.to_owned()';
  if (value.kind === 'variable') return value.name + '.to_owned()';
  if (value.kind === 'binary' && value.op === 'add') {
    const parts = [];
    function append(node) {
      if (node.kind === 'binary' && node.op === 'add') {
        append(node.left);
        append(node.right);
      } else parts.push(rustExpression(node));
    }
    append(value);
    return '[' + parts.join(', ') + '].concat()';
  }
  if (value.kind === 'binary' && value.op === 'equal') {
    if (value.right.kind === 'string' && value.right.value === '') {
      if (value.left.kind === 'variable') return value.left.name + '.is_empty()';
      return '(' + rustExpression(value.left) + ').is_empty()';
    }
    return rustExpression(value.left) + ' == ' + rustExpression(value.right);
  }
  if (value.kind === 'choose') return 'if ' + rustExpression(value.cond) + ' { ' + rustExpression(value.then) + ' } else { ' + rustExpression(value.else) + ' }';
  if (value.kind === 'method') {
    let needle = rustText(value.args[0].value);
    if (Array.from(value.args[0].value).length === 1) needle = "'" + value.args[0].value.replace(/[\\'\n\r\t\0]/gu, character => {
      return {
        '\\': '\\\\',
        "'": "\\'",
        '\n': '\\n',
        '\r': '\\r',
        '\t': '\\t',
        '\0': '\\0'
      }[character];
    }) + "'";
    const receiver = value.receiver.kind === 'variable' ? value.receiver.name : '(' + rustExpression(value.receiver) + ')';
    return receiver + '.replace(' + needle + ', ' + rustText(value.args[1].value) + ')';
  }
  return fail('UnsupportedStringEmission');
}
function rustReturn(value, depth) {
  const pad = '    '.repeat(depth);
  if (value.kind === 'choose') {
    const thenLines = [pad + 'if ' + rustExpression(value.cond) + ' {', ...rustReturn(value.then, depth + 1)];
    if (value.else.kind === 'choose') {
      const rest = rustReturn(value.else, depth);
      return [...thenLines, pad + '} else ' + rest[0].trimStart(), ...rest.slice(1)];
    }
    return [...thenLines, pad + '} else {', ...rustReturn(value.else, depth + 1), pad + '}'];
  }
  return [pad + rustExpression(value)];
}
export function renderStringRust(meta) {
  const programs = assertCompleteStringIR(meta);
  return programs.map(program => {
    return ['#[must_use]', 'pub fn ' + program.name + '(' + program.params.map(parameter => {
      return parameter.name + ': &str';
    }).join(', ') + ') -> String {', ...rustReturn(program.body[0].value, 1), '}'].join('\n');
  }).join('\n\n') + '\n';
}
export function assertCompleteStringIR(meta) {
  const links = meta.split('\n').filter(line => {
    return line.trim() && !line.startsWith('#');
  });
  const programs = validateStringProgram(fromLino(meta));
  assert.equal(links.length, programs.length, 'unrecognized or carried meta records');
  return programs;
}
