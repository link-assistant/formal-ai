// Derive a Links Notation document from an inspected workspace file
// (rust/src/agentic_coding/structured_document.rs).

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { sourceFromReadResult } from './code_artifact.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress } from './progress.mjs';
import { sentences } from './shell_command_policy.mjs';
import {
  cleanPathToken, isStatedWriteTarget, looksLikeFilePath, pathExtension, safeRelativePath, tokens,
} from './write_request.mjs';
import { parseLinoRoot } from './write_lino.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { pushLinoNode } from './crate/links_format.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { meaningEvidencedIn, mentionsRole } from './write_lexicon.mjs';
import { compareStr, splitWhitespace, trim, trimEnd } from './write_str.mjs';

/**
 * Mirrors `fn plan_step`.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planStep(task, messages, toolNames) {
  const specification = recognise(task);
  if (!specification) return null;
  const readTool = toolFor(toolNames, Capability.Read);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (!readTool || !writeTool) return null;
  const progress = Progress.scan(messages);
  const { input, output } = specification;
  if (progress.attemptedWriteFor(output)) {
    if (!progress.successfulWriteFor(output)) return finalAnswer(agenticMessage('structured_document_write_failed', { output }));
    const expected = progress.successfulWriteContentFor(output);
    if (expected === null || expected === undefined) return finalAnswer(agenticMessage('structured_document_no_content', { output }));
    const observedRaw = progress.successfulReadOutputFor(output);
    if (observedRaw !== null && observedRaw !== undefined) {
      const observed = sourceFromReadResult(observedRaw);
      return finalAnswer(trim(observed) === trim(expected)
        ? agenticMessage('structured_document_verified', { output, input })
        : agenticMessage('structured_document_mismatch', { output }));
    }
    return planOne(readTool, readArguments(output));
  }
  const source = progress.successfulReadOutputFor(input);
  if (source === null || source === undefined) return planOne(readTool, readArguments(input));
  return planOne(writeTool, writeArguments(output, render(specification, sourceFromReadResult(source))));
}

const unique = (values) => values.filter((value, index) => values.indexOf(value) === index);
const filePaths = (text) => tokens(text).map((token) => cleanPathToken(token.text))
  .filter((path) => looksLikeFilePath(path) && safeRelativePath(path));

function recognise(task) {
  const normalized = normalizePrompt(task);
  if (!mentionsRole('workspace_inspection_action', normalized)
    || !mentionsRole('document_composition_action', normalized)
    || !mentionsRole('links_notation_format', normalized)) return null;
  const paths = unique(filePaths(task));
  const output = paths.find((path) => isLinoPath(path) && isStatedWriteTarget(task, path)) ?? composedOutputPath(task);
  if (output === null) return null;
  const input = inspectedSourcePath(task, output);
  if (input === null || input === output) return null;
  const [root, ...fields] = machineIdentifiers(task);
  if (root === undefined) return null;
  const at = task.indexOf(root);
  if (at < 0) return null;
  const afterRoot = task.slice(at + root.length);
  const dot = afterRoot.indexOf('.');
  const schemaClause = `${root}${dot < 0 ? afterRoot : afterRoot.slice(0, dot)}`;
  return fields.length ? { input, output, root, fields, schema_clause: schemaClause } : null;
}

function isLinoPath(path) {
  const extension = pathExtension(path);
  return extension !== null && extension.toLowerCase() === 'lino' && /^[A-Za-z]*$/.test(extension);
}

function composedOutputPath(task) {
  for (const sentence of sentences(task)) {
    if (!mentionsRole('document_composition_action', normalizePrompt(sentence.text))) continue;
    const found = tokens(sentence.text).map((token) => cleanPathToken(token.text)).find((path) => isLinoPath(path) && safeRelativePath(path));
    if (found !== undefined) return found;
  }
  return null;
}

function inspectedSourcePath(task, output) {
  for (const sentence of sentences(task)) {
    if (!mentionsRole('workspace_inspection_action', normalizePrompt(sentence.text))) continue;
    const found = filePaths(sentence.text).find((path) => path !== output && !isStatedWriteTarget(sentence.text, path));
    if (found !== undefined) return found;
  }
  return null;
}

function machineIdentifiers(task) {
  return unique(task.split(/[^A-Za-z0-9_]/u).filter((token) => token.includes('_') && /^[A-Za-z]/.test(token)));
}

function render(specification, source) {
  const tree = parseLinoRoot(source);
  let out = '';
  out = pushLinoNode(out, 0, specification.root, null);
  out = pushLinoNode(out, 2, 'source_observation', null);
  out = pushLinoNode(out, 4, 'path', specification.input);
  out = pushLinoNode(out, 4, 'sha256', sha256Hex(source));
  out = pushLinoNode(out, 4, 'root', tree.children[0] ? tree.children[0].name : null);
  const fields = requestedFields(specification, tree);
  for (const scope of collectionScopes(specification, tree)) {
    out = pushLinoNode(out, 2, 'derived_formalization', scope.id ? scope.id : null);
    for (const field of fields) {
      for (const value of bestSourceValues(scope, field)) out = pushLinoNode(out, 4, field, value);
    }
  }
  return trimEnd(out);
}

function collectionScopes(specification, tree) {
  const normalized = normalizePrompt(specification.schema_clause);
  const asksForIndex = specification.root.split('_').includes('index');
  const asksForEvery = meaningEvidencedIn('quantifier_all', normalized);
  if (!asksForIndex && !asksForEvery) return [tree];
  const container = tree.children[0];
  if (!container) return [tree];
  const counts = new Map();
  for (const child of container.children) counts.set(child.name, (counts.get(child.name) ?? 0) + 1);
  const taskTokens = new Set(splitWhitespace(normalized));
  const repeated = [...counts].filter(([, count]) => count > 1).map(([name]) => name)
    .sort((a, b) => (Number(!taskTokens.has(a)) - Number(!taskTokens.has(b))) || compareStr(a, b));
  if (!repeated.length) return [tree];
  return container.children.filter((child) => child.name === repeated[0]);
}

function requestedFields(specification, tree) {
  const fields = [...specification.fields];
  const seen = new Set(fields);
  const at = specification.schema_clause.indexOf(specification.root);
  const afterRoot = at < 0 ? '' : specification.schema_clause.slice(at + specification.root.length);
  for (const candidate of afterRoot.split(/[^A-Za-z0-9_]/u).filter((part) => part.length > 2)) {
    if (!candidate.includes('_') && treeContainsIdentifierToken(tree, candidate) && !seen.has(candidate)) {
      seen.add(candidate);
      fields.push(candidate);
    }
  }
  return fields;
}

function treeContainsIdentifierToken(node, token) {
  return node.children.some((child) => child.name === token
    || (child.name.endsWith(token) && child.name.slice(0, -token.length).endsWith('_'))
    || (child.children.length > 0 && child.name.startsWith(token) && child.name.slice(token.length).startsWith('_'))
    || treeContainsIdentifierToken(child, token));
}

const utf8Length = (text) => new TextEncoder().encode(text).length;

function bestSourceValues(tree, field) {
  const wanted = identifierTokens(field);
  const candidates = [];
  collectCandidates(tree, field, wanted, candidates);
  const exact = unique(candidates.filter((candidate) => candidate.exact).map((candidate) => candidate.value)).sort(compareStr);
  if (exact.length) return exact;
  let best = null;
  for (const candidate of candidates) {
    const order = best === null ? 1 : (candidate.score - best.score) || (utf8Length(best.value) - utf8Length(candidate.value));
    if (order >= 0) best = candidate;
  }
  return best === null ? [] : [best.value];
}

function collectCandidates(node, field, wanted, out) {
  for (const child of node.children) {
    const found = identifierTokens(child.name);
    const overlap = [...wanted].filter((token) => found.has(token)).length;
    const exact = child.name === field;
    const structurallyRelated = overlap === Math.min(wanted.size, found.size);
    if (exact || (overlap > 0 && structurallyRelated)) {
      const leaves = [];
      collectLeafValues(child, leaves);
      if (child.id) leaves.unshift(child.id);
      if (leaves.length) out.push({ score: overlap + (exact ? 100 : 0), exact, value: leaves.join('; ') });
    }
    collectCandidates(child, field, wanted, out);
  }
}

function collectLeafValues(node, out) {
  for (const child of node.children) {
    if (child.id) out.push(child.id);
    collectLeafValues(child, out);
  }
}

function identifierTokens(identifier) {
  return new Set(identifier.split('_').filter((token) => token !== ''));
}

function readArguments(path) {
  return jsonText({ path, filePath: path, file_path: path });
}
