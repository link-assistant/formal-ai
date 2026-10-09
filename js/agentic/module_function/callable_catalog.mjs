// Complete lexical declaration evidence and bounded contracts; no source is emitted.
import { tokenizeWithSpans, isKeyword } from '../crate/es_tokenizer.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
import { inferReturnContract, guardedCallGraph, sourceEvidence } from './source_contract.mjs';
const text = (tree) => tree?.$ === 'leaf' ? tree.text : '';
const group = (tree, delim) => tree?.$ === 'group' && tree.delim === delim;
const identifier = (tree) => tree?.kind === 'identifier'
  && !isKeyword(text(tree)) && !['yield', 'implements', 'interface', 'package', 'private', 'protected', 'public', 'eval', 'arguments'].includes(text(tree));
const safeBinding = (name) => /^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(name);
const decode = (bytes) => new TextDecoder('utf-8', { ignoreBOM: true }).decode(bytes);
const unknown = (reason) => ({ inputs: null, result: null, callEffects: 'unknown', preconditions: [], status: 'unknown', gap: { reason } });
function names(trees) {
  const result = [];
  for (let at = 0; at < trees.length; at += 2) {
    if (!identifier(trees[at]) || !safeBinding(text(trees[at])) || at + 1 < trees.length && text(trees[at + 1]) !== ',') return null;
    result.push(text(trees[at]));
  }
  return new Set(result).size === result.length ? result : null;
}
function aliases(trees) {
  const result = [];
  for (let at = 0; at < trees.length;) {
    if (!identifier(trees[at])) return null;
    const local = text(trees[at++]); let exposed = local;
    if (text(trees[at]) === 'as') {
      if (!identifier(trees[at + 1])) return null;
      exposed = text(trees[at + 1]); at += 2;
    }
    result.push({ local, exposed });
    if (at < trees.length && text(trees[at++]) !== ',') return null;
  }
  return result;
}
function dependency(path, specifier) {
  if (!specifier.startsWith('.') || specifier.includes('\\')) return null;
  const parts = path.split('/').slice(0, -1);
  for (const part of specifier.split('/')) {
    if (part === '' || part === '.') continue;
    if (part === '..') { if (parts.length === 0) return null; parts.pop(); } else parts.push(part);
  }
  return parts.join('/');
}

/** Mirrors `fn observe_source_callables` in rust/src/agentic_coding/module_function/callable_catalog.rs: complete UTF-8/UTF-16 declaration identities. */
export function observeSourceCallables(source, path) {
  const bytes = new TextEncoder().encode(source);
  const catalog = { path, contentId: sha256Hex(bytes), bytes: bytes.length,
    declarations: [], exports: [], imports: [], initialization: [], gaps: [], moduleEffects: 'none', moduleSyntax: 'supported' };
  let trees;
  try { trees = tokenizeWithSpans(source); }
  catch (error) { return { ...catalog, moduleEffects: 'unknown', moduleSyntax: 'unknown', gaps: [{ reason: 'LexicalFailure', start: error.start }] }; }
  for (let at = 0; at < trees.length;) {
    const start = at; let cursor = at;
    const exported = text(trees[cursor]) === 'export'; if (exported) cursor += 1;
    const defaultExport = text(trees[cursor]) === 'default'; if (defaultExport) cursor += 1;
    const asynchronous = text(trees[cursor]) === 'async'; if (asynchronous) cursor += 1;
    if (text(trees[cursor]) === 'function' && identifier(trees[cursor + 1])
      && group(trees[cursor + 2], 'paren') && group(trees[cursor + 3], 'brace')) {
      const name = text(trees[cursor + 1]), parameters = names(trees[cursor + 2].trees);
      const span = { byteStart: trees[start].span.start, byteEnd: trees[cursor + 3].span.end };
      span.start = decode(bytes.subarray(0, span.byteStart)).length;
      span.end = decode(bytes.subarray(0, span.byteEnd)).length;
      const body = decode(bytes.subarray(span.byteStart, span.byteEnd));
      const contract = !safeBinding(name) ? unknown('UnsupportedBinding') : parameters === null ? unknown('UnsupportedParameters') : asynchronous ? unknown('AsyncResult')
        : inferReturnContract(parameters, trees[cursor + 3].trees, source);
      catalog.declarations.push({ name, parameters, source: body, span, contract,
        identity: { path, moduleContentId: catalog.contentId, declarationContentId: sha256Hex(body), span } });
      if (exported) catalog.exports.push({ local: name, exposed: defaultExport ? 'default' : name });
      // Invalid/unproved declaration syntax still blocks initialization certification.
      if (!safeBinding(name) || parameters === null) catalog.moduleEffects = 'unknown';
      // Unsupported bodies lack a syntax proof even though they do not execute here.
      if (contract.status !== 'supported') catalog.moduleSyntax = 'unknown';
      // A deferred body has its own effect contract; declaring it does not execute it.
      at = cursor + 4; if (text(trees[at]) === ';') at += 1; continue;
    }
    if (exported && group(trees[cursor], 'brace') && text(trees[cursor + 1]) !== 'from') {
      const named = aliases(trees[cursor].trees);
      if (named === null) catalog.gaps.push({ reason: 'UnsupportedExport', start: trees[start].span.start });
      else catalog.exports.push(...named);
      at = cursor + 1; if (text(trees[at]) === ';') at += 1; continue;
    }
    if (text(trees[start]) === 'import') {
      let end = start + 1; while (end < trees.length && text(trees[end]) !== ';') end += 1;
      const terms = trees.slice(start + 1, end); let bindings = null, literal = null;
      if (terms.length === 1 && terms[0].kind === 'string') { literal = text(terms[0]); bindings = []; }
      if (terms.length === 3 && group(terms[0], 'brace') && text(terms[1]) === 'from' && terms[2].kind === 'string') {
        const named = aliases(terms[0].trees);
        if (named !== null) { bindings = named.map(({ local, exposed }) => ({ imported: local, local: exposed })); literal = text(terms[2]); }
      }
      const specifier = literal !== null && !literal.includes('\\') ? literal.slice(1, -1) : null;
      if (bindings === null || specifier === null) catalog.gaps.push({ reason: 'UnsupportedImport', start: trees[start].span.start });
      else catalog.imports.push({ specifier, path: dependency(path, specifier), bindings });
      catalog.initialization.push({ kind: 'import', effects: 'unknown',
        ...sourceEvidence(source, trees[start].span.start, trees[Math.min(end, trees.length - 1)].span.end) });
      catalog.moduleEffects = 'unknown'; at = end + 1; continue;
    }
    catalog.moduleEffects = 'unknown';
    catalog.gaps.push({ reason: 'UnprovedModuleStatement', start: trees[start].span.start });
    at += 1; while (at < trees.length && text(trees[at]) !== ';') at += 1;
    catalog.initialization.push({ kind: 'unclassified', effects: 'unknown',
      ...sourceEvidence(source, trees[start].span.start, trees[Math.min(at, trees.length - 1)].span.end) });
    at += 1;
  }
  const locals = [...catalog.declarations.map((entry) => entry.name), ...catalog.imports.flatMap((entry) => entry.bindings.map((binding) => binding.local))];
  if (new Set(locals).size !== locals.length) catalog.gaps.push({ reason: 'DuplicateBinding', start: null });
  if (new Set(catalog.exports.map((entry) => entry.exposed)).size !== catalog.exports.length) catalog.gaps.push({ reason: 'DuplicateExport', start: null });
  for (const exported of catalog.exports) {
    if (!locals.includes(exported.local)) catalog.gaps.push({ reason: 'ExportDeclarationUnobserved', start: null });
  }
  if (catalog.gaps.length > 0) { catalog.moduleEffects = 'unknown'; catalog.moduleSyntax = 'unknown'; }
  return catalog;
}

/** Mirrors `fn observed_guarded_graph` in rust/src/agentic_coding/module_function/callable_catalog.rs: recatalog actual preimages before type/effect checks. */
export function observedGuardedGraph(observations, first, second) {
  const catalogs = observations.map(({ path, content }) => observeSourceCallables(content, path));
  const resolve = (binding, seen = new Set()) => {
    const key = binding.path + ':' + binding.exported;
    if (seen.has(key)) return { kind: 'gap', reason: 'BindingCycle' }; seen.add(key);
    const catalog = catalogs.find((entry) => entry.path === binding.path);
    if (catalog === undefined) return { kind: 'gap', reason: 'ModuleUnobserved' };
    if (binding.contentId !== undefined && binding.contentId !== catalog.contentId) return { kind: 'gap', reason: 'SourceChanged' };
    if (catalog.gaps.length > 0) return { kind: 'gap', reason: 'ModuleContractGap' };
    if (catalog.moduleSyntax !== 'supported') return { kind: 'gap', reason: 'ModuleSyntaxUnknown' };
    const names = catalog.exports.filter((entry) => entry.exposed === binding.exported);
    if (names.length !== 1) return { kind: 'gap', reason: 'ExportUnresolved' };
    const declared = catalog.declarations.find((entry) => entry.name === names[0].local);
    if (declared !== undefined) return { ...declared, moduleEffects: catalog.moduleEffects, binding: { path: binding.path, exported: binding.exported } };
    const imported = catalog.imports.flatMap((entry) => entry.bindings.map((item) => ({ ...item, path: entry.path })))
      .find((entry) => entry.local === names[0].local);
    if (imported === undefined || imported.path === null) return { kind: 'gap', reason: 'DeclarationUnobserved' };
    // Import initialization is a separate obligation, even when the export alias resolves.
    const target = resolve({ path: imported.path, exported: imported.imported }, seen);
    return target.kind === 'gap' ? target : { ...target, moduleEffects: 'unknown', binding: { path: binding.path, exported: binding.exported } };
  };
  const producer = resolve(first), consumer = resolve(second);
  if (producer.kind === 'gap') return producer;
  if (consumer.kind === 'gap') return consumer;
  return guardedCallGraph(producer, consumer);
}

/** Mirrors `fn observed_callable_graphs` in rust/src/agentic_coding/module_function/callable_catalog.rs: supported graphs remain unbound to the requested goal. */
export function observedCallableGraphs(observations) {
  const bindings = [];
  for (const observation of observations) {
    const catalog = observeSourceCallables(observation.content, observation.path);
    if (catalog.moduleEffects !== 'none' || catalog.moduleSyntax !== 'supported' || catalog.gaps.length !== 0) continue;
    for (const exported of catalog.exports) {
      const declaration = catalog.declarations.find((entry) => entry.name === exported.local);
      if (declaration?.contract.status !== 'supported' || declaration.parameters.length !== 1) continue;
      bindings.push({ path: catalog.path, exported: exported.exposed, contentId: catalog.contentId,
        optional: declaration.contract.result.kind === 'optional' });
    }
  }
  const graphs = [];
  for (const first of bindings.filter((binding) => binding.optional)) {
    for (const second of bindings) {
      const graph = observedGuardedGraph(observations, first, second);
      if (graph.kind === 'guarded-call-graph') graphs.push(graph);
    }
  }
  return graphs;
}
