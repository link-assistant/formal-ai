import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
const require = createRequire(import.meta.url);
const {parse} = require('@babel/parser');
const {parseDocument} = require('yaml');
function literalPath(value) {
  assert.equal(typeof value, 'string', 'Primitive source path required');
  assert.match(value, /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);
  assert.ok(!value.split('/').some(part => ['.', '..', '.git'].includes(part)));
  return value;
}
function frozenOperand(node) {
  assert.equal(node?.type, 'CallExpression');
  assert.equal(node.arguments.length, 1);
  assert.equal(node.callee.type, 'MemberExpression');
  assert.equal(node.callee.computed, false);
  assert.equal(node.callee.object.type, 'Identifier');
  assert.equal(node.callee.object.name, 'Object');
  assert.equal(node.callee.property.name, 'freeze');
  return node.arguments[0];
}
function bindingNames(node) {
  if (!node) return [];
  if (node.type === 'Identifier') return [node.name];
  if (node.type === 'RestElement') return bindingNames(node.argument);
  if (node.type === 'AssignmentPattern') return bindingNames(node.left);
  if (node.type === 'ArrayPattern') return node.elements.flatMap(bindingNames);
  if (node.type === 'ObjectPattern') return node.properties.flatMap(property =>
    bindingNames(property.type === 'RestElement' ? property.argument : property.value));
  return [];
}
function canonicalBindings(ast, declaration) {
  function visit(node, parent = null) {
    if (!node || typeof node !== 'object') return;
    if (Array.isArray(node)) { for (const child of node) visit(child, parent); return; }
    let bindings = [];
    if (node.type === 'VariableDeclarator') bindings = bindingNames(node.id);
    if (['FunctionDeclaration','FunctionExpression','ArrowFunctionExpression'].includes(node.type)) {
      bindings = [...bindingNames(node.id),...node.params.flatMap(bindingNames)];
    }
    if (['ClassDeclaration','ClassExpression','CatchClause'].includes(node.type)) {
      bindings = [...bindings,...bindingNames(node.id ?? node.param)];
    }
    if (['ImportSpecifier','ImportDefaultSpecifier','ImportNamespaceSpecifier'].includes(node.type)) {
      bindings = bindingNames(node.local);
    }
    assert.ok(!bindings.includes('Object'), 'Intrinsic Object binding must be unshadowed');
    if (bindings.includes('RELEASE_PROTOCOL_FILES')) {
      assert.equal(node, declaration, 'One canonical registry binding required');
    }
    if (node.type === 'Identifier' && node.name === 'Object') {
      const propertyKey = parent?.type === 'ObjectProperty' && parent.key === node &&
        !parent.computed && !parent.shorthand;
      const memberKey = parent?.type === 'MemberExpression' && parent.property === node && !parent.computed;
      if (!propertyKey && !memberKey) {
        assert.equal(parent?.type, 'MemberExpression', 'Intrinsic Object must not escape');
        assert.equal(parent.object, node);
        assert.equal(parent.computed, false);
        assert.ok(['freeze','hasOwn'].includes(parent.property.name), 'Unknown intrinsic operation');
      }
    }
    if (node.type === 'MemberExpression' && node.object?.type === 'Identifier' && node.object.name === 'Object') {
      assert.equal(parent?.type, 'CallExpression', 'Intrinsic helper mutation or alias refused');
      assert.equal(parent.callee, node);
      assert.equal(parent.optional ?? false, false);
    }
    if (node.type === 'Identifier' && ['globalThis','global','window','self','eval','Function'].includes(node.name)) {
      const key = parent?.type === 'MemberExpression' && parent.property === node && !parent.computed;
      const property = parent?.type === 'ObjectProperty' && parent.key === node && !parent.computed;
      assert.ok(key || property, 'Dynamic global intrinsic access refused');
    }
    if (['AssignmentExpression','UpdateExpression','UnaryExpression'].includes(node.type)) {
      const operand = node.type === 'AssignmentExpression' ? node.left : node.argument;
      function protectedOperand(value) {
        if (!value || typeof value !== 'object') return false;
        if (Array.isArray(value)) return value.some(protectedOperand);
        if (value.type === 'Identifier' && ['Object','RELEASE_PROTOCOL_FILES'].includes(value.name)) return true;
        return Object.entries(value).some(([key,child]) => !['loc','start','end'].includes(key) && protectedOperand(child));
      }
      if (node.type !== 'UnaryExpression' || node.operator === 'delete') {
        assert.equal(protectedOperand(operand), false, 'Canonical binding mutation refused');
      }
    }
    for (const [key, child] of Object.entries(node)) {
      if (!['loc','start','end','comments','tokens'].includes(key)) visit(child, node);
    }
  }
  visit(ast);
}
export function projectReleaseTriggerPaths(recipeSource, workflowSource, role) {
  assert.equal(typeof recipeSource, 'string');
  assert.equal(typeof workflowSource, 'string');
  assert.equal(typeof role, 'string');
  const ast = parse(recipeSource, {sourceType:'module'});
  const declarations = ast.program.body.filter(node => node.type === 'ExportNamedDeclaration')
    .flatMap(node => (node.declaration?.declarations || []).map(declaration =>
      ({declaration,kind:node.declaration.kind})))
    .filter(entry => entry.declaration.id?.name === 'RELEASE_PROTOCOL_FILES');
  assert.equal(declarations.length, 1, 'One canonical source registry required');
  assert.equal(declarations[0].kind, 'const', 'Immutable canonical registry required');
  canonicalBindings(ast, declarations[0].declaration);
  const registry = frozenOperand(declarations[0].declaration.init);
  assert.equal(registry.type, 'ObjectExpression');
  assert.ok(registry.properties.every(node => node.type === 'ObjectProperty' && !node.computed));
  const roles = registry.properties.map(node => node.key.name ?? node.key.value);
  assert.equal(new Set(roles).size, roles.length, 'Unique immutable role bindings required');
  for (const property of registry.properties) {
    const values = frozenOperand(property.value);
    assert.equal(values.type, 'ArrayExpression');
    for (const value of values.elements) {
      assert.equal(value?.type, 'StringLiteral', 'Every role must have a closed frozen source array');
      literalPath(value.value);
    }
  }
  const selected = registry.properties.filter(node => (node.key.name ?? node.key.value) === role);
  assert.equal(selected.length, 1, 'One declared role required');
  const array = frozenOperand(selected[0].value);
  assert.equal(array.type, 'ArrayExpression');
  const declared = array.elements.map(node => {
    assert.equal(node?.type, 'StringLiteral', 'Static declared source required');
    return literalPath(node.value);
  });
  assert.equal(new Set(declared).size, declared.length);
  const document = parseDocument(workflowSource, {uniqueKeys:true,keepCstNodes:true,keepNodeTypes:true});
  assert.equal(document.errors.length, 0, 'Valid original workflow required');
  const sequence = document.getIn(['on','pull_request','paths'], true);
  assert.equal(sequence?.cstNode?.type, 'SEQ', 'Owned block path sequence required');
  assert.ok(sequence.items.length > 0);
  assert.ok(sequence.items.every(item => item.type !== 'ALIAS' && typeof item.value === 'string'));
  const existing = sequence.items.map(item => literalPath(item.value));
  assert.equal(new Set(existing).size, existing.length);
  const missing = declared.filter(value => !existing.includes(value));
  if (!missing.length) return Object.freeze({text:workflowSource,added:Object.freeze([]),declared:Object.freeze(declared)});
  const last = sequence.items.at(-1);
  const start = workflowSource.lastIndexOf('\n', last.range[0]) + 1;
  const prefix = /^([ ]*)- /u.exec(workflowSource.slice(start));
  assert.ok(prefix, 'Exact existing block item indentation required');
  const lineEnd = workflowSource.indexOf('\n', last.range[1]);
  assert.ok(lineEnd >= 0, 'Existing path line ending required');
  const offset = lineEnd + 1;
  const ending = workflowSource[lineEnd - 1] === '\r' ? '\r\n' : '\n';
  const insertion = missing.map(value => prefix[1] + '- ' + JSON.stringify(value) + ending).join('');
  const text = workflowSource.slice(0, offset) + insertion + workflowSource.slice(offset);
  const result = parseDocument(text, {uniqueKeys:true});
  assert.equal(result.errors.length, 0);
  const before = document.toJSON(), after = result.toJSON();
  assert.deepEqual(after.on.pull_request.paths, [...existing,...missing]);
  after.on.pull_request.paths = before.on.pull_request.paths;
  assert.deepEqual(after, before, 'No other workflow semantics may change');
  assert.equal(text.slice(0,offset) + text.slice(offset+insertion.length),workflowSource);
  return Object.freeze({text,added:Object.freeze(missing),declared:Object.freeze(declared),offset,insertion});
}
const physical = fileURLToPath(import.meta.url);
if (process.argv[1] && resolve(process.argv[1]) === physical) {
  assert.deepEqual(process.argv.slice(2), ['--check']);
  const root = resolve(dirname(physical), '..');
  const recipe = readFileSync(resolve(root,'scripts/release-protocol.mjs'),'utf8');
  const workflow = readFileSync(resolve(root,'.github/workflows/publish-source-networks.yml'),'utf8');
  const result = projectReleaseTriggerPaths(recipe,workflow,'source-networks');
  assert.equal(result.added.length,0,'Declared release source missing from publication triggers');
  console.log('Declared release trigger paths match the source registry');
}
