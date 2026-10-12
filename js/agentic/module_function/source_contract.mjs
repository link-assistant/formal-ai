// A bounded return-expression contract. Unsupported constructs retain exact gaps.
import { parseConditionalBody } from './conditional_ir.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
const text = (tree) => (tree?.$ === 'leaf' ? tree.text : '');
const variable = (name) => ({ kind: 'parameter', name });
const optional = (value) =>
  value.kind === 'optional' ? value : { kind: 'optional', value };
const same = (left, right) => JSON.stringify(left) === JSON.stringify(right);
function merge(left, right) {
  if (same(left, right)) return left;
  if (left.kind === 'null') return optional(right);
  if (right.kind === 'null') return optional(left);
  throw { reason: 'IncompatibleBranches' };
}
function expression(trees, parameters) {
  if (trees.length === 0) throw { reason: 'MissingReturnExpression' };
  const question = trees.findIndex((tree) => text(tree) === '?');
  if (question >= 0) {
    let depth = 0,
      colon = -1;
    for (let at = question + 1; at < trees.length; at += 1) {
      if (text(trees[at]) === '?') depth += 1;
      if (text(trees[at]) === ':' && depth-- === 0) {
        colon = at;
        break;
      }
    }
    if (colon < 0) throw { reason: 'UnclosedConditional' };
    const condition = expression(trees.slice(0, question), parameters);
    if (condition.kind !== 'boolean') throw { reason: 'MissingConditionType' };
    return merge(
      expression(trees.slice(question + 1, colon), parameters),
      expression(trees.slice(colon + 1), parameters)
    );
  }
  const coalesce = trees.findIndex((tree) => text(tree) === '??');
  if (coalesce >= 0) {
    const left = expression(trees.slice(0, coalesce), parameters),
      right = expression(trees.slice(coalesce + 1), parameters);
    if (left.kind !== 'optional') throw { reason: 'MissingOptionalOperand' };
    return right.kind === 'null' ? left : merge(left.value, right);
  }
  const compare = trees.findIndex((tree) =>
    ['===', '!=='].includes(text(tree))
  );
  if (compare >= 0) {
    expression(trees.slice(0, compare), parameters);
    expression(trees.slice(compare + 1), parameters);
    return { kind: 'boolean' };
  }
  if (
    trees.some(
      (tree) =>
        ['.', '?.'].includes(text(tree)) ||
        (tree.$ === 'group' && tree.delim === 'bracket')
    )
  ) {
    throw { reason: 'MissingStructuralSchema' };
  }
  if (trees.length === 1) {
    const tree = trees[0];
    if (tree.$ === 'group' && tree.delim === 'paren')
      return expression(tree.trees, parameters);
    if (parameters.includes(text(tree))) return variable(text(tree));
    if (text(tree) === 'null') return { kind: 'null' };
    if (['true', 'false'].includes(text(tree))) return { kind: 'boolean' };
    if (
      tree.kind === 'numeric' &&
      /^(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?$/u.test(text(tree))
    )
      return { kind: 'number' };
    if (tree.kind === 'string' && !/[\\\r\n]/u.test(text(tree)))
      return { kind: 'text' };
  }
  throw {
    reason: trees.some((tree) => tree.$ === 'group' && tree.delim === 'paren')
      ? 'UnobservedCallEffect'
      : 'UnsupportedExpression',
  };
}

/** Mirrors `fn source_evidence`: bytes, coordinates and identity of an observed region. */
export function sourceEvidence(source, start, end) {
  const bytes = new TextEncoder().encode(source),
    decode = (value) =>
      new TextDecoder('utf-8', { ignoreBOM: true }).decode(value);
  const content = decode(bytes.subarray(start, end));
  return {
    source: content,
    contentId: sha256Hex(content),
    span: {
      byteStart: start,
      byteEnd: end,
      start: decode(bytes.subarray(0, start)).length,
      end: decode(bytes.subarray(0, end)).length,
    },
  };
}
function unsafeAccessScope(trees) {
  return trees.some(
    (tree) =>
      [
        '=',
        '+=',
        '-=',
        '*=',
        '/=',
        '%=',
        '**=',
        '&&=',
        '||=',
        '??=',
        '&=',
        '|=',
        '^=',
        '<<=',
        '>>=',
        '>>>=',
        '++',
        '--',
        '=>',
        'function',
        'class',
        'delete',
        'yield',
        'await',
      ].includes(text(tree)) ||
      (tree.$ === 'group' &&
        (tree.delim === 'brace' || unsafeAccessScope(tree.trees)))
  );
}
// These are unmet schema requirements, never proof that a property/getter is pure.
function structuralRequirements(parameters, body, source) {
  const result = [];
  if (text(body[0]) !== 'return' || unsafeAccessScope(body)) return result;
  const visit = (trees) => {
    for (let at = 0; at < trees.length; at += 1) {
      const tree = trees[at];
      if (parameters.includes(text(tree))) {
        const selectors = [];
        let cursor = at + 1;
        while (cursor < trees.length) {
          if (
            ['.', '?.'].includes(text(trees[cursor])) &&
            trees[cursor + 1]?.kind === 'identifier'
          ) {
            selectors.push({ property: text(trees[cursor + 1]) });
            cursor += 2;
          } else if (
            trees[cursor].$ === 'group' &&
            trees[cursor].delim === 'bracket'
          ) {
            const index = trees[cursor];
            selectors.push({
              index: sourceEvidence(
                source,
                index.span.start + 1,
                index.span.end - 1
              ).source,
            });
            cursor += 1;
          } else break;
        }
        if (selectors.length > 0) {
          result.push({
            root: text(tree),
            selectors,
            status: 'unproved',
            ...sourceEvidence(
              source,
              tree.span.start,
              trees[cursor - 1].span.end
            ),
          });
          for (const term of trees.slice(at + 1, cursor))
            if (term.$ === 'group') visit(term.trees);
          at = cursor - 1;
          continue;
        }
      }
      if (tree.$ === 'group') visit(tree.trees);
    }
  };
  visit(body);
  return result;
}

/** Mirrors `fn infer_return_contract` in rust/src/agentic_coding/module_function/source_contract.rs: only complete supported bodies establish constraints. */
export function inferReturnContract(parameters, body, source) {
  const base = {
    inputs: parameters.map((name) => ({ name, type: variable(name) })),
    result: null,
    callEffects: 'unknown',
    preconditions: [],
    status: 'unknown',
    gap: null,
    structuralRequirements: structuralRequirements(parameters, body, source),
    conditionalIR: parseConditionalBody(parameters, body, source),
  };
  try {
    if (new Set(parameters).size !== parameters.length)
      throw { reason: 'DuplicateParameter' };
    if (text(body[0]) !== 'return') throw { reason: 'UnsupportedStatement' };
    let terms = body.slice(1);
    if (text(terms.at(-1)) === ';') terms = terms.slice(0, -1);
    if (terms.some((term) => text(term) === ';'))
      throw { reason: 'UnsupportedStatement' };
    const between = new TextDecoder('utf-8', { ignoreBOM: true }).decode(
      new TextEncoder()
        .encode(source)
        .subarray(body[0].span.end, terms[0]?.span.start ?? body[0].span.end)
    );
    if (/[\r\n\u2028\u2029]/u.test(between))
      throw { reason: 'ReturnLineTerminator' };
    return {
      ...base,
      result: expression(terms, parameters),
      callEffects: 'none',
      status: 'supported',
    };
  } catch (gap) {
    return {
      ...base,
      gap: {
        reason: gap.reason ?? 'UnsupportedExpression',
        span: body[0]?.span ?? null,
      },
    };
  }
}

/** Mirrors `fn guarded_call_graph` in rust/src/agentic_coding/module_function/source_contract.rs: unknown contracts never become a callable operand. */
export function guardedCallGraph(first, second) {
  if (
    [first, second].some(
      (entry) =>
        entry.contract.status !== 'supported' ||
        entry.contract.callEffects !== 'none' ||
        entry.moduleEffects !== 'none'
    )
  )
    return { kind: 'gap', reason: 'MissingContract' };
  if (first.parameters.length !== 1 || second.parameters.length !== 1)
    return { kind: 'gap', reason: 'UnsupportedArity' };
  if (first.contract.result.kind !== 'optional')
    return { kind: 'gap', reason: 'MissingOptionalResult' };
  const input = second.contract.inputs[0].type;
  if (input.kind !== 'parameter')
    return { kind: 'gap', reason: 'IncompatibleOperand' };
  const instantiate = (type) =>
    type.kind === 'parameter'
      ? first.contract.result.value
      : type.kind === 'optional'
        ? optional(instantiate(type.value))
        : type;
  const result = instantiate(second.contract.result);
  return {
    kind: 'guarded-call-graph',
    calls: [first.identity, second.identity],
    bindings: [first.binding, second.binding],
    guard: { kind: 'nonnull', call: 0 },
    result: optional(result),
    effects: 'none',
    authored: false,
  };
}
