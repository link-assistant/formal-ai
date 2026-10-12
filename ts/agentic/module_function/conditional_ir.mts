import { isKeyword } from '../crate/es_tokenizer.mjs';
const validBinding = (name) =>
  typeof name === 'string' &&
  /^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(name) &&
  !isKeyword(name) &&
  ![
    'eval',
    'arguments',
    'yield',
    'implements',
    'interface',
    'package',
    'private',
    'protected',
    'public',
  ].includes(name);
const txt = (t) => t?.text;
const fail = (reason) => {
  throw Object.assign(new Error(reason), { reason });
};
const literal = (value) => ({ op: 'literal', value });
function parseExpression(trees) {
  if (!trees.length) fail('MissingExpression');
  const q = trees.findIndex((t) => txt(t) === '?');
  if (q >= 0) {
    let depth = 0,
      c = -1;
    for (let i = q + 1; i < trees.length; i++) {
      if (txt(trees[i]) === '?') depth++;
      if (txt(trees[i]) === ':' && depth-- === 0) {
        c = i;
        break;
      }
    }
    if (c < 0) fail('Conditional');
    return {
      op: 'conditional',
      condition: parseExpression(trees.slice(0, q)),
      yes: parseExpression(trees.slice(q + 1, c)),
      no: parseExpression(trees.slice(c + 1)),
    };
  }
  if (
    trees.filter((tree) => ['===', '!=='].includes(txt(tree))).length > 1 ||
    trees.filter((tree) => txt(tree) === '>').length > 1
  )
    fail('UnsupportedComparisonChain');
  for (const operator of ['??', '===', '!==', '>']) {
    const i = trees.findIndex((t) => txt(t) === operator);
    if (i >= 0)
      return {
        op: operator,
        left: parseExpression(trees.slice(0, i)),
        right: parseExpression(trees.slice(i + 1)),
      };
  }
  let node,
    at = 1,
    first = trees[0];
  if (first.$ === 'template')
    node = {
      op: 'template',
      parts: first.parts.map((p) =>
        p.$ === 'chunk'
          ? /\\/u.test(p.text)
            ? fail('EscapedTemplate')
            : literal(p.text)
          : parseExpression(p.trees)
      ),
    };
  else if (first.$ === 'group' && first.delim === 'paren')
    node = parseExpression(first.trees);
  else if (first.kind === 'numeric' && /^(0|[1-9][0-9]*)$/u.test(first.text))
    node = literal(
      Number.isSafeInteger(Number(first.text))
        ? Number(first.text)
        : fail('UnsafeNumericLiteral')
    );
  else if (first.kind === 'string' && !/[\\\r\n]/u.test(first.text))
    node = literal(first.text.slice(1, -1));
  else if (first.text === 'null') node = literal(null);
  else if (['true', 'false'].includes(first.text))
    node = literal(first.text === 'true');
  else if (first.kind === 'identifier')
    node = { op: 'binding', name: first.text };
  else fail('UnsupportedExpression');
  while (at < trees.length) {
    if (txt(trees[at]) === '.' && trees[at + 1]?.kind === 'identifier') {
      const name = txt(trees[at + 1]);
      at += 2;
      if (trees[at]?.$ === 'group' && trees[at].delim === 'paren') {
        const argumentTrees = trees[at++].trees;
        if (name === 'map') {
          if (
            argumentTrees.length < 3 ||
            argumentTrees[0].$ !== 'group' ||
            argumentTrees[0].delim !== 'paren' ||
            argumentTrees[0].trees.length !== 1 ||
            txt(argumentTrees[1]) !== '=>' ||
            !validBinding(txt(argumentTrees[0].trees[0]))
          )
            fail('UnsupportedMapper');
          node = {
            op: 'map',
            receiver: node,
            parameter: txt(argumentTrees[0].trees[0]),
            body: parseExpression(argumentTrees.slice(2)),
          };
        } else if (['join', 'push'].includes(name))
          node = { op: name, receiver: node, argument: parseExpression(argumentTrees) };
        else fail('UnknownMethod');
      } else node = { op: 'member', receiver: node, name };
    } else if (trees[at].$ === 'group' && trees[at].delim === 'bracket')
      node = {
        op: 'index',
        receiver: node,
        index: parseExpression(trees[at++].trees),
      };
    else fail('UnsupportedTail');
  }
  return node;
}
function parseStatements(trees, source) {
  const statements = [];
  let at = 0;
  const statement = () => {
    const start = at;
    while (at < trees.length && txt(trees[at]) !== ';') at++;
    const result = trees.slice(start, at);
    if (at < trees.length) at++;
    return result;
  };
  while (at < trees.length) {
    if (txt(trees[at]) === 'const') {
      at++;
      const name = txt(trees[at++]);
      if (!validBinding(name)) fail('UnsupportedLocalBinding');
      if (txt(trees[at++]) !== '=') fail('LocalDeclaration');
      statements.push({
        op: 'const',
        name,
        value: parseExpression(statement()),
      });
    } else if (txt(trees[at]) === 'if') {
      at++;
      const group = trees[at++];
      if (group?.delim !== 'paren') fail('IfCondition');
      statements.push({
        op: 'if',
        condition: parseExpression(group.trees),
        body: { op: 'effect', value: parseExpression(statement()) },
      });
    } else if (txt(trees[at]) === 'return') {
      const token = trees[at++],
        next = trees[at];
      if (
        next &&
        /[\r\n\u2028\u2029]/u.test(
          new TextDecoder().decode(
            new TextEncoder()
              .encode(source)
              .subarray(token.span.end, next.span.start)
          )
        )
      )
        fail('ReturnLineTerminator');
      statements.push({ op: 'return', value: parseExpression(statement()) });
      if (at < trees.length) fail('AfterReturn');
    } else fail('UnsupportedStatement');
  }
  if (statements.at(-1)?.op !== 'return') fail('MissingReturn');
  return statements;
}

/** Mirrors parse_conditional_body: source syntax is a conditional IR, not an effect certificate. */
export function parseConditionalBody(parameters, body, source) {
  try {
    if (parameters.length !== 1 || !validBinding(parameters[0]))
      fail('UnsupportedArity');
    return {
      status: 'parsed',
      parameter: parameters[0],
      ir: parseStatements(body, source),
      sourceEffects: 'unknown',
      preconditions: [
        'SourceOwnedPlainDataSchema',
        'FiniteScalarCoercions',
        'IntrinsicOwnership',
        'FreshLocalMutationOnly',
      ],
    };
  } catch (error) {
    return {
      status: 'unknown',
      reason: error.reason ?? 'UnsupportedSyntax',
      sourceEffects: 'unknown',
    };
  }
}
