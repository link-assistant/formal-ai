// Source-qualified projection of a complete owned String value, never arbitrary callback output.
import { readFileSync } from 'node:fs';
import * as runtime from '../js/vendor/tree-sitter/web-tree-sitter.mjs';

function nodes(root) {
  const result = [], pending = [root];
  while (pending.length) {
    const node = pending.pop();
    result.push(node);
    pending.push(...node.namedChildren);
  }
  return result;
}
function ancestor(node, kind) {
  for (let parent = node.parent; parent !== null; parent = parent.parent) {
    if (parent.type === kind) return parent;
  }
  return null;
}
function method(node, name) {
  if (node?.type !== 'call_expression') return null;
  const field = node.childForFieldName('function');
  if (field?.type !== 'field_expression' || field.childForFieldName('field')?.text !== name) return null;
  return field.childForFieldName('value');
}

/** Accept the bounded Vec<(String,String)> full-value lookup/clone shape; refuse unknown origins. */
export async function qualifyOwnedFullReads(source) {
  await runtime.Parser.init();
  const language = await runtime.Language.load(new URL('../js/vendor/tree-sitter/tree-sitter-rust.wasm', import.meta.url));
  const parser = new runtime.Parser();
  parser.setLanguage(language);
  const tree = parser.parse(source);
  try {
    if (tree.rootNode.hasError) throw new Error('Rust source AST is not clean');
    const edits = [], adaptedHelpers = new Set();
    for (const closure of nodes(tree.rootNode).filter(node => node.type === 'closure_expression')) {
      const body = closure.childForFieldName('body');
      const value = method(body, 'clone');
      if (value?.type !== 'identifier') continue;
      const parameters = closure.childForFieldName('parameters');
      const tuple = parameters?.namedChildren[0];
      if (tuple?.type !== 'tuple_pattern' || tuple.namedChildren.length !== 1
        || tuple.namedChildren[0].text !== value.text || !tuple.text.startsWith('(_,')) continue;
      const argumentsNode = closure.parent;
      const lookup = method(argumentsNode.parent, 'map_or_else');
      const reversed = method(lookup, 'find');
      const iterated = method(reversed, 'rev');
      const collection = method(iterated, 'iter');
      if (collection?.type !== 'identifier') continue;
      if (iterated.childForFieldName('arguments')?.namedChildren.length !== 0
        || reversed.childForFieldName('arguments')?.namedChildren.length !== 0) continue;
      const predicate = lookup.childForFieldName('arguments')?.namedChildren[0];
      if (predicate?.type !== 'closure_expression'
        || predicate.childForFieldName('body')?.type !== 'binary_expression'
        || !predicate.childForFieldName('body').text.includes('==')) continue;
      const arm = ancestor(closure, 'match_arm');
      if (arm?.childForFieldName('pattern')?.text !== '"Read"') continue;
      const owner = ancestor(closure, 'function_item');
      if (owner === null) continue;
      const declarations = nodes(owner).filter(node => node.type === 'let_declaration'
        && node.childForFieldName('pattern')?.text === collection.text);
      if (declarations.length !== 1
        || declarations[0].childForFieldName('type')?.text.replace(/\s+/gu, '') !== 'Vec<(String,String)>') continue;
      if (declarations[0].childForFieldName('value')?.type !== 'macro_invocation'
        || !declarations[0].childForFieldName('value').text.startsWith('vec!')) continue;
      const comparison = predicate.childForFieldName('body');
      const key = predicate.childForFieldName('parameters')?.namedChildren[0];
      const left = comparison.childForFieldName('left');
      const path = comparison.childForFieldName('right');
      if (path?.type !== 'identifier' || key?.type !== 'tuple_pattern'
        || key.namedChildren.length !== 1 || left?.type !== 'identifier'
        || key.namedChildren[0].text !== left.text || !key.text.endsWith(', _)')
        || comparison.children.find(node => !node.isNamed)?.text !== '==') continue;
      const sourcePath = nodes(arm).filter(node => node.type === 'let_declaration'
        && node.childForFieldName('pattern')?.text === path.text);
      if (sourcePath.length !== 1) continue;
      const pathValue = sourcePath[0].childForFieldName('value');
      const pathLookup = method(pathValue, 'expect');
      const pathOrigin = method(pathLookup, 'as_str');
      const providerClosure = ancestor(arm, 'closure_expression');
      const providerParameters = providerClosure?.childForFieldName('parameters')?.namedChildren;
      const argumentName = providerParameters?.[1]?.text;
      const shadowsArgument = providerClosure !== null && nodes(providerClosure).some(node => {
        const binding = node.childForFieldName('pattern');
        if (binding !== null && nodes(binding).some(part =>
          part.type === 'identifier' && part.text === argumentName)) return true;
        return node.type === 'closure_parameters'
          && node.id !== providerClosure.childForFieldName('parameters')?.id
          && nodes(node).some(part => part.type === 'identifier' && part.text === argumentName);
      });
      if (shadowsArgument) continue;
      if (pathOrigin?.type !== 'index_expression' || providerParameters?.length !== 2
        || providerParameters.some(node => node.type !== 'identifier')
        || pathOrigin.namedChildren[0]?.text !== providerParameters[1].text
        || pathOrigin.namedChildren[1]?.text !== '"file_path"') continue;
      const errorBody = argumentsNode.namedChildren[0]?.childForFieldName('body');
      if (errorBody === null || errorBody === undefined
        || nodes(errorBody).some(node => node.type === 'return_expression')) continue;
      const matchArms = arm.parent.namedChildren.filter(node => node.type === 'match_arm');
      const opaqueEdits = [];
      let supported = true;
      for (const other of matchArms.filter(node => node.id !== arm.id)) {
        const otherBody = other.childForFieldName('value') ?? other.namedChildren.at(-1);
        if (otherBody?.type === 'macro_invocation' && otherBody.text.startsWith('panic!')) continue;
        const tail = otherBody?.type === 'block' ? otherBody.namedChildren.at(-1) : otherBody;
        if (tail?.type !== 'call_expression' || !tail.text.startsWith('String::from(')) {
          supported = false;
          break;
        }
        opaqueEdits.push({ start: tail.startIndex, end: tail.endIndex, insert: '(' + tail.text + ').into()' });
      }
      if (!supported) continue;
      const callback = ancestor(arm, 'closure_expression');
      const callArguments = ancestor(callback, 'arguments');
      const providerCall = callArguments?.parent;
      const providerName = providerCall?.childForFieldName('function');
      if (providerName?.type !== 'identifier') continue;
      const helper = nodes(tree.rootNode).find(node => node.type === 'function_item'
        && node.childForFieldName('name')?.text === providerName.text);
      if (helper === undefined || helper.childForFieldName('type_parameters') !== null) continue;
      const callbackParameter = helper.childForFieldName('parameters')?.namedChildren.find(node =>
        node.childForFieldName('type')?.text.replace(/\s+/gu, '') === '&mutdynFnMut(&str,&Value)->String');
      const executeName = callbackParameter?.childForFieldName('pattern')?.text;
      if (executeName === undefined) continue;
      const execution = nodes(helper).find(node => node.type === 'let_declaration'
        && node.childForFieldName('value')?.type === 'call_expression'
        && node.childForFieldName('value').childForFieldName('function')?.text === executeName);
      if (execution === undefined) continue;
      const resultName = execution.childForFieldName('pattern')?.text;
      const call = execution.childForFieldName('value');
      const argumentsList = call.childForFieldName('arguments')?.namedChildren;
      if (argumentsList?.length !== 2 || argumentsList[0].type !== 'identifier'
        || argumentsList[1].type !== 'reference_expression') continue;
      const toolName = argumentsList[0].text;
      const toolArguments = argumentsList[1].text.slice(1);
      const marker = '"content": ' + resultName;
      const contentAt = source.indexOf(marker, helper.startIndex);
      if (contentAt < 0 || contentAt >= helper.endIndex) continue;
      if (!adaptedHelpers.has(helper.startIndex)) {
        const helperName = helper.childForFieldName('name');
        edits.push({ start: helperName.endIndex, end: helperName.endIndex,
          insert: '<R: Into<formal_ai::agentic_coding::tool_result::ProviderToolObservation>>' });
        const callbackType = callbackParameter.childForFieldName('type');
        edits.push({ start: callbackType.startIndex, end: callbackType.endIndex,
          insert: callbackType.text.replace(/String$/u, 'R') });
        const replacement = 'let observation: formal_ai::agentic_coding::tool_result::ProviderToolObservation = '
          + call.text + '.into();\n'
          + '            let opaque_path = (formal_ai::agentic_coding::planner::tool_capability(' + toolName
          + ') == Some(formal_ai::agentic_coding::planner::Capability::Read)).then(|| '
          + toolArguments + '["file_path"].as_str().unwrap_or(""));\n'
          + '            let (' + resultName + ', source_read) = observation.into_transport(opaque_path);';
        edits.push({ start: execution.startIndex, end: execution.endIndex, insert: replacement });
        edits.push({ start: contentAt + marker.length, end: contentAt + marker.length,
          insert: ',\n                "source_read": source_read' });
        adaptedHelpers.add(helper.startIndex);
      }
      edits.push(...opaqueEdits);
      edits.push({ start: errorBody.startIndex, end: errorBody.endIndex, insert: '(' + errorBody.text + ').into()' });
      edits.push({ start: body.startIndex, end: body.endIndex,
        insert: 'formal_ai::agentic_coding::tool_result::ProviderToolObservation::complete_owned_read('
          + path.text + ', ' + value.text + ')' });
    }
    if (edits.length === 0) throw new Error('No source-qualified complete owned Read provider');
    let result = source;
    for (const edit of edits.sort((left, right) => right.start - left.start)) {
      result = result.slice(0, edit.start) + edit.insert + result.slice(edit.end);
    }
    const generated = parser.parse(result);
    try { if (generated.rootNode.hasError) throw new Error('Generated Rust AST is not clean'); }
    finally { generated.delete(); }
    return { source: result, edits };
  } finally { tree.delete(); parser.delete(); }
}

/** Compare all non-comment leaf tokens; formatting never changes source operands. */
export async function rustTokenSignature(source) {
  await runtime.Parser.init();
  const language = await runtime.Language.load(new URL('../js/vendor/tree-sitter/tree-sitter-rust.wasm', import.meta.url));
  const parser = new runtime.Parser();
  parser.setLanguage(language);
  const tree = parser.parse(source);
  try {
    if (tree.rootNode.hasError) throw new Error('Rust token source AST is not clean');
    const tokens = [];
    function visit(node) {
      if (node.type === 'line_comment' || node.type === 'block_comment') return;
      if (node.childCount === 0) tokens.push([node.type, node.text]);
      else for (const child of node.children) visit(child);
    }
    visit(tree.rootNode);
    return JSON.stringify(tokens);
  } finally { tree.delete(); parser.delete(); }
}

if (process.argv[2] === '--check') {
  const before = readFileSync(process.argv[3], 'utf8');
  const expected = readFileSync(process.argv[4], 'utf8');
  const generated = await qualifyOwnedFullReads(before);
  if (await rustTokenSignature(generated.source) !== await rustTokenSignature(expected)) {
    throw new Error('Owned-reader projection token drift');
  }
  console.log('Source-qualified owned-reader AST projection matches; no native compilation claimed.');
}
