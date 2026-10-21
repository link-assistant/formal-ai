#!/usr/bin/env node
// Supplied integration automation; inventory and source effects never imply autonomous authorship.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {parse} from '@babel/parser';
import traversal from '@babel/traverse';
import {scanJavaScriptImplementations} from './check-source-duplication.mjs';
const traverse = traversal.default ?? traversal;
const hash = value => createHash('sha256').update(value).digest('hex');
const callbackMethods = new Set(['map', 'flatMap', 'filter', 'some', 'every', 'find', 'findIndex', 'replace', 'replaceAll']);
const admittedPlans = new WeakSet();
const freeze = value => {if (value && typeof value === 'object') {for (const child of Object.values(value)) freeze(child); Object.freeze(value);} return value;};
const knownOperations = new Set(['lower', 'trim', 'text', 'value', 'nonempty', 'positiveLength', 'property']);
const parseSource = source => parse(source, {sourceType: 'module', errorRecovery: false});

/** Recognize only exact unary expressions: no captured variables, optional access, coercion or extra calls. */
export function unaryOperation(node) {
  if (node.type !== 'ArrowFunctionExpression' || node.async || node.generator || node.params.length !== 1 || node.params[0].type !== 'Identifier') return null;
  const name = node.params[0].name, body = node.body;
  const member = (value, property) => value?.type === 'MemberExpression' && !value.computed && !value.optional
    && value.object.type === 'Identifier' && value.object.name === name && value.property.type === 'Identifier' && value.property.name === property;
  if (body.type === 'CallExpression' && !body.optional && body.arguments.length === 0) {
    if (member(body.callee, 'toLowerCase')) return 'lower';
    if (member(body.callee, 'trim')) return 'trim';
  }
  if (member(body, 'text')) return 'text';
  if (member(body, 'value')) return 'value';
  if (body.type === 'MemberExpression' && !body.computed && !body.optional
      && body.object.type === 'Identifier' && body.object.name === name
      && body.property.type === 'Identifier') return 'property';
  if (body.type === 'BinaryExpression' && body.operator === '!==' && body.left.type === 'Identifier' && body.left.name === name
      && body.right.type === 'StringLiteral' && body.right.value === '') return 'nonempty';
  if (body.type === 'BinaryExpression' && body.operator === '>' && member(body.left, 'length')
      && body.right.type === 'NumericLiteral' && body.right.value === 0) return 'positiveLength';
  return null;
}
function observedSource(root, relative) {
  if (typeof relative !== 'string' || !relative.startsWith('js/agentic/') || !relative.endsWith('.mjs')
      || relative.split('/').some(part => !part || part === '.' || part === '..') || relative.includes('\\')) throw Error('OwnedAgenticSourceRequired');
  const file = path.join(root, relative), status = fs.lstatSync(file);
  if (!status.isFile() || status.isSymbolicLink() || fs.realpathSync(file) !== file || status.size > 1048576) throw Error('RegularBoundedSourceRequired');
  const bytes = fs.readFileSync(file), source = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(bytes);
  if (bytes.length > 1048576 || !Buffer.from(source).equals(bytes)) throw Error('SourceEncodingOrBudget');
  return {file, source, sha256: hash(bytes), mode: status.mode};
}
function helperContract(root, contract, operation) {
  if (!knownOperations.has(operation) || typeof contract.exportName !== 'string') throw Error('UnaryHelperContractRequired');
  const observed = observedSource(root, contract.module), ast = parseSource(observed.source);
  const declarations = ast.program.body.filter(node => node.type === 'ExportNamedDeclaration'
    && node.declaration?.type === 'FunctionDeclaration' && node.declaration.id.name === contract.exportName);
  if (declarations.length !== 1) throw Error('UniqueExportedUnaryHelperRequired');
  const fn = declarations[0].declaration;
  if (fn.async || fn.generator || fn.params.length !== 1 || fn.params[0].type !== 'Identifier'
      || fn.body.body.length !== 1 || fn.body.body[0].type !== 'ReturnStatement') throw Error('UnaryHelperBodyMismatch');
  const returned = fn.body.body[0].argument;
  if (operation === 'property') {
    const member = returned?.body;
    if (returned?.type !== 'ArrowFunctionExpression' || returned.async || returned.generator
        || returned.params.length !== 1 || returned.params[0].type !== 'Identifier'
        || returned.params[0].name === fn.params[0].name
        || member?.type !== 'MemberExpression' || !member.computed || member.optional
        || member.object.type !== 'Identifier' || member.object.name !== returned.params[0].name
        || member.property.type !== 'Identifier' || member.property.name !== fn.params[0].name) throw Error('UnaryHelperBodyMismatch');
  } else if (unaryOperation({type: 'ArrowFunctionExpression', params: fn.params, body: returned}) !== operation) throw Error('UnaryHelperBodyMismatch');
  return {...contract, sha256: observed.sha256};
}
/** Emit a reviewable preimage-bound plan. General JavaScript reflection is outside this explicit callback-value contract. */
export function planUnaryConsolidation(root, files, contracts) {
  root = fs.realpathSync(root);
  if (!Array.isArray(files) || files.length > 500 || new Set(files).size !== files.length || !contracts || !Object.keys(contracts).length) throw Error('FiniteExplicitSourceSetRequired');
  const helpers = Object.fromEntries(Object.entries(contracts).map(([operation, contract]) => [operation, helperContract(root, contract, operation)]));
  const changes = [], deferred = [];
  for (const relative of files) {
    const observed = observedSource(root, relative), ast = parseSource(observed.source), edits = [], imports = new Map(), identifiers = new Set();
    traverse(ast, {Identifier(p) {identifiers.add(p.node.name);}});
    traverse(ast, {ArrowFunctionExpression(p) {
      const operation = unaryOperation(p.node), helper = helpers[operation];
      if (!helper) {
        if (p.node.params.length === 1 && ['MemberExpression', 'OptionalMemberExpression'].includes(p.node.body.type)) {
          deferred.push({file: relative, start: p.node.start, reason: 'UnresolvedMemberProjectionContract'});
        }
        return;
      }
      if ((ast.comments ?? []).some(comment => comment.start >= p.node.start && comment.end <= p.node.end)) {
        deferred.push({file: relative, start: p.node.start, operation, reason: 'CommentInsideCallbackRequiresOwnedTransfer'}); return;
      }
      const parent = p.parentPath.node, callee = parent.callee;
      const method = callee?.type === 'MemberExpression' && !callee.computed && !callee.optional && callee.property.type === 'Identifier' ? callee.property.name : null;
      if (parent.type !== 'CallExpression' || parent.optional || !parent.arguments.includes(p.node) || !callbackMethods.has(method)) {
        deferred.push({file: relative, start: p.node.start, operation, reason: 'NotDeclaredCallbackValuePosition'}); return;
      }
      // Reflection on callback identity/name/source and arbitrary custom collection methods is not an equivalence claim.
      const alias = helper.module === relative ? helper.exportName : 'canonicalUnary' + operation[0].toUpperCase() + operation.slice(1);
      if (helper.module !== relative && identifiers.has(alias)) throw Error('UnaryImportBindingCollision:' + relative + ':' + alias);
      if (helper.module === relative && p.scope.getBinding(alias)?.scope !== p.scope.getProgramParent()) throw Error('ShadowedCanonicalUnaryHelper');
      if (helper.module !== relative) imports.set(alias, helper);
      edits.push({start: p.node.start, end: p.node.end, before: observed.source.slice(p.node.start, p.node.end), replacement: operation === 'property' ? alias + '(' + JSON.stringify(p.node.body.property.name) + ')' : alias,
        operation, callbackMethod: method, parameter: p.node.params[0].name, captures: [], valueEvaluations: 1,
        receiverPolicy: 'member-call-keeps-original-value-as-this', contract: 'callback-value-behavior-not-function-reflection'});
    }});
    if (!edits.length) continue;
    const insertion = ast.program.body[0]?.start ?? observed.source.length;
    const importsText = [...imports].map(([alias, helper]) => {
      let specifier = path.posix.relative(path.posix.dirname(relative), helper.module);
      if (!specifier.startsWith('.')) specifier = './' + specifier;
      return `import { ${helper.exportName} as ${alias} } from '${specifier}';\n`;
    }).join('');
    const mutations = [...edits, ...(importsText ? [{start: insertion, end: insertion, before: '', replacement: importsText}] : [])];
    let after = observed.source;
    for (const edit of mutations.sort((a, b) => b.start - a.start)) {
      if (after.slice(edit.start, edit.end) !== edit.before) throw Error('OverlappingUnaryPreimage');
      after = after.slice(0, edit.start) + edit.replacement + after.slice(edit.end);
    }
    parseSource(after);
    changes.push({file: relative, beforeSha256: observed.sha256, afterSha256: hash(after), before: observed.source, after, edits});
  }
  const plan = freeze({schema: 'stateless-unary-consolidation/v1', root, helpers, changes, deferred,
    generatedProjectionsRequired: true, completeSemanticEquivalence: false, autonomousAuthorshipCredit: 0});
  admittedPlans.add(plan);
  return plan;
}
/** All files and helper identities are checked before the first write. Run only through the authorized Formal AI executor. */
export function applyUnaryConsolidation(plan) {
  if (!admittedPlans.has(plan) || plan?.schema !== 'stateless-unary-consolidation/v1') throw Error('UnaryPlanRequired');
  for (const helper of Object.values(plan.helpers)) if (observedSource(plan.root, helper.module).sha256 !== helper.sha256) throw Error('UnaryHelperDrift');
  for (const change of plan.changes) {
    if (observedSource(plan.root, change.file).sha256 !== change.beforeSha256 || hash(change.before) !== change.beforeSha256 || hash(change.after) !== change.afterSha256) throw Error('UnarySourceDrift');
    parseSource(change.after);
  }
  for (const change of plan.changes) fs.writeFileSync(path.join(plan.root, change.file), change.after);
  return {files: plan.changes.length, callbacks: plan.changes.reduce((sum, change) => sum + change.edits.length, 0), requiresIndependentConsumerChecks: true};
}
/** Complete compact inventory, streamed as bounded records. No baseline, suppression or truncated-success mode. */
export function writeCompleteUnaryInventory(root, destination, {scan = scanJavaScriptImplementations, maximumBytes = 134217728, maximumRecordBytes = 8388608} = {}) {
  root = fs.realpathSync(root); destination = path.resolve(destination);
  if (destination === root || destination.startsWith(root + path.sep)) throw Error('PrivateInventoryOutsideRepositoryRequired');
  for (const value of [maximumBytes, maximumRecordBytes]) if (!Number.isSafeInteger(value) || value < 1 || value > 134217728) throw Error('FiniteInventoryBudgetRequired');
  fs.mkdirSync(destination, {mode: 0o700});
  const report = scan(root), members = [];
  let total = 0;
  const writeRecords = (name, rows) => {
    const filename = path.join(destination, name), fd = fs.openSync(filename, 'wx', 0o600), checksum = createHash('sha256');
    let bytes = 0, count = 0;
    try {
      for (const row of rows) {
        const record = Buffer.from(JSON.stringify(row) + '\n');
        if (record.length > maximumRecordBytes || total + record.length > maximumBytes) throw Error('CompleteInventoryBudgetExceeded');
        let offset = 0;
        while (offset < record.length) {const written = fs.writeSync(fd, record, offset); if (written <= 0) throw Error('InventoryWriteNoProgress'); offset += written;}
        checksum.update(record); bytes += record.length; total += record.length; count++;
      }
    } finally {fs.closeSync(fd);}
    members.push({path: name, bytes, records: count, sha256: checksum.digest('hex')});
  };
  writeRecords('sources.jsonl', report.sources);
  writeRecords('raw-groups.jsonl', report.clones);
  writeRecords('independent-groups.jsonl', report.independentClones);
  writeRecords('derivation-proofs.jsonl', report.proofs);
  writeRecords('gaps.jsonl', report.gaps);
  const manifest = {schema: 'complete-source-duplication-inventory/v1', complete: true, coverageScope: 'complete serialization of the existing strict JavaScript/TypeScript guard; repository-wide discovery is separate', roots: ['js', 'ts'], files: report.files, callables: report.callables,
    rawCloneGroups: report.rawCloneGroups, independentCloneGroups: report.independentCloneGroups,
    accepted: report.accepted, members, totalBytes: total, maximumBytes, maximumRecordBytes, autonomousAuthorshipCredit: 0};
  fs.writeFileSync(path.join(destination, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n', {flag: 'wx', mode: 0o600});
  return manifest;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv[2] !== '--inventory' || process.argv.length !== 4) throw Error('usage: node scripts/consolidate-stateless-unary.mjs --inventory PRIVATE_DIRECTORY');
    const report = writeCompleteUnaryInventory(path.resolve(fileURLToPath(new URL('..', import.meta.url))), process.argv[3]);
    console.log(JSON.stringify(report));
  } catch (error) {console.error(error.message); process.exitCode = 1;}
}
