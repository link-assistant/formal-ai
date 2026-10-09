const fail = (reason) => {
  throw { reason };
};
const scalarType = (type) =>
  ['text', 'number', 'index', 'boolean', 'null', 'literal', 'scalar'].includes(
    type.kind
  );
function propertyPath(node) {
  if (node.op === 'binding') return { root: node.name, path: [] };
  if (node.op === 'member') {
    const parent = propertyPath(node.receiver);
    return parent
      ? { root: parent.root, path: [...parent.path, node.name] }
      : null;
  }
  return null;
}
// Source-local constraint propagation. No field, function, or expected string is supplied.
const typeVariable = () => ({ kind: 'variable' });
const dereference = (type) => (type.binding ? dereference(type.binding) : type);
function constrain(left, right) {
  left = dereference(left);
  right = dereference(right);
  if (left === right) return left;
  if (left.kind === 'variable') {
    left.binding = right;
    return right;
  }
  if (right.kind === 'variable') {
    right.binding = left;
    return left;
  }
  if (left.kind === 'scalar' && scalarType(right)) {
    left.binding = right;
    return right;
  }
  if (right.kind === 'scalar' && scalarType(left)) {
    right.binding = left;
    return left;
  }
  if (left.kind === 'sequence' && ['array', 'text'].includes(right.kind)) {
    left.binding = right;
    return right;
  }
  if (right.kind === 'sequence' && ['array', 'text'].includes(left.kind)) {
    right.binding = left;
    return left;
  }
  if (left.kind === 'number' && right.kind === 'index') {
    left.binding = right;
    return right;
  }
  if (right.kind === 'number' && left.kind === 'index') {
    right.binding = left;
    return left;
  }
  if (left.kind !== right.kind) fail('IncompatibleSourceConstraints');
  if (left.kind === 'record')
    for (const [name, value] of Object.entries(right.fields)) {
      if (Object.hasOwn(left.fields, name)) constrain(left.fields[name], value);
      else left.fields[name] = value;
    }
  else if (left.kind === 'array') constrain(left.item, right.item);
  else if (left.kind === 'optional') constrain(left.value, right.value);
  else if (
    ['literal', 'exceptLiteral'].includes(left.kind) &&
    left.value !== right.value
  )
    fail('IncompatibleDiscriminants');
  return left;
}
function projected(type, name) {
  const record = constrain(type, {
    kind: 'record',
    fields: Object.create(null),
  });
  if (!Object.hasOwn(record.fields, name)) record.fields[name] = typeVariable();
  return record.fields[name];
}
function inferredSourceExpression(node, env) {
  const infer = (value) => inferredSourceExpression(value, env);
  switch (node.op) {
    case 'literal':
      return {
        type: {
          kind:
            node.value === null
              ? 'null'
              : typeof node.value === 'string'
                ? 'text'
                : typeof node.value,
        },
        fresh: false,
      };
    case 'binding':
      if (!env.has(node.name)) fail('UnknownBinding');
      return env.get(node.name);
    case 'member': {
      const receiver = infer(node.receiver);
      if (node.name === 'length') {
        constrain(receiver.type, { kind: 'sequence' });
        return { type: { kind: 'number' }, fresh: false };
      }
      return { type: projected(receiver.type, node.name), fresh: false };
    }
    case 'index': {
      const receiver = constrain(infer(node.receiver).type, {
        kind: 'array',
        item: typeVariable(),
      });
      constrain(infer(node.index).type, { kind: 'index' });
      return { type: { kind: 'optional', value: receiver.item }, fresh: false };
    }
    case '===':
    case '!==':
      infer(node.left);
      infer(node.right);
      return { type: { kind: 'boolean' }, fresh: false };
    case '>':
      constrain(infer(node.left).type, { kind: 'number' });
      constrain(infer(node.right).type, { kind: 'number' });
      return { type: { kind: 'boolean' }, fresh: false };
    case '??': {
      const left = constrain(infer(node.left).type, {
          kind: 'optional',
          value: typeVariable(),
        }),
        right = dereference(infer(node.right).type);
      if (right.kind === 'null') return { type: left, fresh: false };
      constrain(left.value, right);
      return { type: left.value, fresh: false };
    }
    case 'conditional': {
      constrain(infer(node.condition).type, { kind: 'boolean' });
      const yes = infer(node.yes),
        no = infer(node.no),
        yt = dereference(yes.type),
        nt = dereference(no.type);
      if (yt.kind === 'null')
        return {
          type:
            nt.kind === 'optional' ? nt : { kind: 'optional', value: no.type },
          fresh: false,
        };
      if (nt.kind === 'null')
        return {
          type:
            yt.kind === 'optional' ? yt : { kind: 'optional', value: yes.type },
          fresh: false,
        };
      return { type: constrain(yes.type, no.type), fresh: false };
    }
    case 'template':
      for (const part of node.parts)
        constrain(infer(part).type, { kind: 'scalar' });
      return { type: { kind: 'text' }, fresh: false };
    case 'map': {
      const receiver = constrain(infer(node.receiver).type, {
          kind: 'array',
          item: typeVariable(),
        }),
        child = new Map(env);
      if (child.has(node.parameter)) fail('ShadowedMapper');
      child.set(node.parameter, { type: receiver.item, fresh: false });
      const value = inferredSourceExpression(node.body, child);
      constrain(value.type, { kind: 'scalar' });
      return { type: { kind: 'array', item: value.type }, fresh: true };
    }
    case 'push': {
      const receiver = infer(node.receiver);
      if (!receiver.fresh) fail('CallerMutation');
      const array = constrain(receiver.type, {
          kind: 'array',
          item: typeVariable(),
        }),
        argument = infer(node.argument);
      constrain(argument.type, { kind: 'scalar' });
      constrain(array.item, argument.type);
      return { type: { kind: 'number' }, fresh: false };
    }
    case 'join': {
      const receiver = constrain(infer(node.receiver).type, {
        kind: 'array',
        item: typeVariable(),
      });
      constrain(receiver.item, { kind: 'scalar' });
      constrain(infer(node.argument).type, { kind: 'text' });
      return { type: { kind: 'text' }, fresh: false };
    }
    default:
      fail('UnsupportedSourceIR');
  }
}
function cloneType(type, memo = new Map()) {
  type = dereference(type);
  if (memo.has(type)) return memo.get(type);
  const copy = { kind: type.kind };
  memo.set(type, copy);
  if (type.kind === 'record') {
    copy.fields = Object.create(null);
    for (const [key, value] of Object.entries(type.fields))
      copy.fields[key] = cloneType(value, memo);
  } else if (type.kind === 'array') copy.item = cloneType(type.item, memo);
  else if (type.kind === 'optional') copy.value = cloneType(type.value, memo);
  else if (Object.hasOwn(type, 'value')) copy.value = type.value;
  return copy;
}
function schemaFromType(type, seen = new Set()) {
  type = dereference(type);
  if (seen.has(type)) fail('RecursiveSchema');
  const next = new Set(seen);
  next.add(type);
  if (type.kind === 'variable') return { kind: 'json' };
  if (type.kind === 'sequence') fail('UnresolvedSequenceIntrinsic');
  if (type.kind === 'record')
    return {
      kind: 'record',
      fields: Object.fromEntries(
        Object.entries(type.fields).map(([key, value]) => [
          key,
          schemaFromType(value, next),
        ])
      ),
    };
  if (type.kind === 'array')
    return { kind: 'array', item: schemaFromType(type.item, next) };
  if (type.kind === 'optional')
    return { kind: 'optional', value: schemaFromType(type.value, next) };
  if (type.kind === 'union')
    return {
      kind: 'union',
      options: type.options.map((option) => schemaFromType(option, next)),
    };
  return Object.hasOwn(type, 'value')
    ? { kind: type.kind, value: type.value }
    : { kind: type.kind };
}
function inferSourceProgram(compiled) {
  if (compiled?.status !== 'parsed') fail('MissingConditionalIR');
  let input = typeVariable(),
    env = new Map([[compiled.parameter, { type: input, fresh: false }]]),
    result;
  for (const statement of compiled.ir) {
    if (statement.op === 'const') {
      if (env.has(statement.name)) fail('ShadowedLocal');
      const value = inferredSourceExpression(statement.value, env);
      if (!value.fresh) fail('NonFreshLocal');
      env.set(statement.name, value);
    } else if (statement.op === 'if') {
      constrain(inferredSourceExpression(statement.condition, env).type, {
        kind: 'boolean',
      });
      inferredSourceExpression(statement.body.value, env);
    } else if (statement.op === 'return') {
      const node = statement.value,
        path =
          node.op === 'conditional' &&
          node.condition.op === '===' &&
          node.condition.right.op === 'literal'
            ? propertyPath(node.condition.left)
            : null;
      if (path?.root === compiled.parameter) {
        inferredSourceExpression(node.condition, env);
        const yesInput = cloneType(input),
          noInput = cloneType(input);
        let yesField = yesInput,
          noField = noInput;
        for (const name of path.path) {
          yesField = projected(yesField, name);
          noField = projected(noField, name);
        }
        constrain(yesField, {
          kind: 'literal',
          value: node.condition.right.value,
        });
        constrain(noField, {
          kind: 'exceptLiteral',
          value: node.condition.right.value,
        });
        const trueBranchEnvironment = new Map(env),
          noEnv = new Map(env);
        trueBranchEnvironment.set(compiled.parameter, { type: yesInput, fresh: false });
        noEnv.set(compiled.parameter, { type: noInput, fresh: false });
        const yes = inferredSourceExpression(node.yes, trueBranchEnvironment).type,
          no = inferredSourceExpression(node.no, noEnv).type;
        if (dereference(no).kind === 'null')
          result =
            dereference(yes).kind === 'optional'
              ? yes
              : { kind: 'optional', value: yes };
        else if (dereference(yes).kind === 'null')
          result =
            dereference(no).kind === 'optional'
              ? no
              : { kind: 'optional', value: no };
        else result = constrain(yes, no);
        input = { kind: 'union', options: [yesInput, noInput] };
      } else result = inferredSourceExpression(node, env).type;
    }
  }
  if (!result) fail('MissingResultType');
  return { compiled, input, result };
}

/** Mirrors derive_conditional_schema_graph: source-derived schemas stay conditional on owned plain data. */
export function deriveConditionalSchemaGraph(first, second) {
  try {
    const producer = inferSourceProgram(first.contract.conditionalIR),
      consumer = inferSourceProgram(second.contract.conditionalIR),
      produced = dereference(producer.result);
    if (produced.kind !== 'optional') fail('MissingOptionalResult');
    constrain(produced.value, consumer.input);
    const schemas = {
      input: schemaFromType(producer.input),
      produced: schemaFromType(producer.result),
      consumed: schemaFromType(consumer.input),
      result: schemaFromType(consumer.result),
    };
    return {
      kind: 'conditional-schema-graph',
      schemas,
      calls: [first.identity, second.identity],
      bindings: [first.binding, second.binding],
      guard: { kind: 'nonnull', call: 0 },
      sourceEffects: 'conditional',
      effects: 'unknown',
      moduleEffects: [first.moduleEffects, second.moduleEffects],
      preconditions: first.contract.conditionalIR.preconditions,
      suppliedPerTaskSchemas: false,
      authored: false,
      goal: 'unbound',
    };
  } catch (error) {
    return { kind: 'gap', reason: error.reason ?? 'SourceConstraintGap' };
  }
}
