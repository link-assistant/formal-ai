// Observed ESM callable identities. Discovery never certifies an unknown contract.
import { createHash } from 'node:crypto';
import { lex, topLevelItems } from './lexer.mjs';

const digest = (text) => createHash('sha256').update(text, 'utf8').digest('hex');
const bytes = (text) => Buffer.byteLength(text, 'utf8');
const meaningful = (item) => item.tokens.filter((token) => token.type !== 'comment');
const identifier = (token) => token?.type === 'ident';
const unknown = () => ({ inputs: null, result: null, callEffects: null, moduleEffects: null, status: 'unknown' });
const span = (source, item) => ({ start: item.start, end: item.end,
  byteStart: bytes(source.slice(0, item.start)), byteEnd: bytes(source.slice(0, item.end)) });

function moduleOrigin(origin, specifier) {
  if (!specifier.startsWith('.')) return null;
  try { return new URL(specifier, origin).href; } catch { return null; }
}

function stringValue(token) {
  if (token?.type !== 'string') return null;
  // Escaped specifiers need a real language decoder; never guess their origin.
  if (token.text.includes('\\')) return null;
  return token.text.slice(1, -1);
}

function namedBindings(tokens) {
  const bindings = [];
  let index = 0;
  while (index < tokens.length) {
    const imported = tokens[index++];
    if (!identifier(imported)) return null;
    let local = imported;
    if (tokens[index]?.text === 'as') {
      index += 1; local = tokens[index++];
      if (!identifier(local)) return null;
    }
    bindings.push({ imported: imported.text, local: local.text });
    if (index === tokens.length) break;
    if (tokens[index++]?.text !== ',') return null;
  }
  return bindings;
}

function importBindings(tokens) {
  const bindings = [];
  let index = 1;
  if (identifier(tokens[index])) {
    bindings.push({ imported: 'default', local: tokens[index++].text });
    if (tokens[index]?.text === ',') index += 1;
  }
  if (tokens[index]?.text === '*') {
    if (tokens[index + 1]?.text !== 'as' || !identifier(tokens[index + 2])) return null;
    bindings.push({ imported: '*', local: tokens[index + 2].text });
    index += 3;
  } else if (tokens[index]?.text === '{') {
    const end = tokens.findIndex((token, at) => at > index && token.text === '}');
    if (end < 0) return null;
    const named = namedBindings(tokens.slice(index + 1, end));
    if (named === null) return null;
    bindings.push(...named); index = end + 1;
  }
  if (tokens[index]?.text !== 'from') return null;
  const specifier = stringValue(tokens[index + 1]);
  return specifier === null ? null : { specifier, bindings };
}

function declaration(tokens) {
  let index = tokens[0]?.text === 'export' ? 1 : 0;
  const exported = index === 1;
  const isDefault = tokens[index]?.text === 'default';
  if (isDefault) index += 1;
  if (tokens[index]?.text === 'async') index += 1;
  if (tokens[index++]?.text !== 'function' || !identifier(tokens[index])) return null;
  const name = tokens[index++].text;
  if (tokens[index++]?.text !== '(') return null;
  const parameters = [];
  while (tokens[index]?.text !== ')') {
    if (!identifier(tokens[index])) return null;
    parameters.push(tokens[index++].text);
    if (tokens[index]?.text === ',') index += 1;
    else if (tokens[index]?.text !== ')') return null;
  }
  index += 1;
  if (tokens[index]?.text !== '{' || tokens.at(-1)?.text !== '}') return null;
  return { name, parameters, exported: exported ? (isDefault ? 'default' : name) : null };
}

/** Catalog complete declarations from one observed module, without evaluating it. */
export function observeCallableModule(source, origin) {
  if (typeof source !== 'string') throw new TypeError('source must be observed text');
  const moduleId = new URL(origin).href;
  const catalog = { moduleId, sha256: digest(source), bytes: bytes(source),
    declarations: [], imports: [], exports: [], gaps: [] };
  const locals = new Set();
  const addLocal = (name) => {
    if (locals.has(name)) catalog.gaps.push({ kind: 'duplicate-binding', name });
    locals.add(name);
  };
  for (const item of topLevelItems(lex(source, 'JavaScript'))) {
    if (item.comment) continue;
    const tokens = meaningful(item);
    const callable = declaration(tokens);
    if (callable !== null) {
      addLocal(callable.name);
      const text = source.slice(item.start, item.end);
      catalog.declarations.push({ name: callable.name, parameters: callable.parameters,
        moduleId, moduleSha256: catalog.sha256, declarationSha256: digest(text),
        span: span(source, item), source: text, contract: unknown() });
      if (callable.exported) catalog.exports.push({ exposed: callable.exported, local: callable.name });
    } else if (tokens[0]?.text === 'import') {
      const parsed = importBindings(tokens);
      if (parsed === null) {
        catalog.gaps.push({ kind: 'unsupported-import', span: span(source, item) });
        continue;
      }
      for (const binding of parsed.bindings) {
        addLocal(binding.local);
        catalog.imports.push({ ...binding, specifier: parsed.specifier,
          moduleId: moduleOrigin(moduleId, parsed.specifier), span: span(source, item) });
      }
    } else if (tokens[0]?.text === 'export' && tokens[1]?.text === '{') {
      const end = tokens.findIndex((token, index) => index > 1 && token.text === '}');
      const bindings = end < 0 ? null : namedBindings(tokens.slice(2, end));
      const reexport = tokens[end + 1]?.text === 'from';
      if (bindings === null || reexport) {
        catalog.gaps.push({ kind: reexport ? 'unobserved-reexport' : 'unsupported-export', span: span(source, item) });
        continue;
      }
      catalog.exports.push(...bindings.map(({ imported, local }) => ({ exposed: local, local: imported })));
    } else if (tokens[0]?.text === 'export') {
      catalog.gaps.push({ kind: 'unsupported-export', span: span(source, item) });
    } else if (['const', 'let', 'var', 'class'].includes(tokens[0]?.text) && identifier(tokens[1])) {
      addLocal(tokens[1].text);
    }
  }
  for (const entry of catalog.declarations) {
    entry.shadowedImports = catalog.imports.filter((binding) => entry.parameters.includes(binding.local))
      .map((binding) => binding.local);
  }
  return catalog;
}

/** Reject stale source or declaration evidence, including changes outside the body. */
export function validateObservedCallable(entry, source) {
  return digest(source) === entry.moduleSha256
    && digest(source.slice(entry.span.start, entry.span.end)) === entry.declarationSha256
    && bytes(source.slice(0, entry.span.start)) === entry.span.byteStart
    && bytes(source.slice(0, entry.span.end)) === entry.span.byteEnd;
}

/** Resolve local/imported aliases only through modules supplied as actual observations. */
export function bindObservedExport(catalogs, moduleId, exposed, local, reserved = []) {
  if (!/^[A-Za-z_$][\w$]*$/u.test(local) || reserved.includes(local)) return { kind: 'gap', reason: 'binding-collision' };
  const seen = new Set();
  const follow = (origin, name) => {
    const key = JSON.stringify([origin, name]);
    if (seen.has(key)) return { kind: 'gap', reason: 'binding-cycle' };
    seen.add(key);
    const catalog = catalogs.find((record) => record.moduleId === origin);
    if (!catalog) return { kind: 'gap', reason: 'module-unobserved' };
    if (catalog.gaps.some((gap) => gap.kind === 'duplicate-binding')) return { kind: 'gap', reason: 'ambiguous-binding' };
    const exports = catalog.exports.filter((binding) => binding.exposed === name);
    if (exports.length !== 1) return { kind: 'gap', reason: 'export-unresolved' };
    const binding = exports[0];
    const declaration = catalog.declarations.find((entry) => entry.name === binding.local);
    if (declaration) return { kind: 'observed', declaration, import: { moduleId, exported: exposed, local } };
    const imported = catalog.imports.find((entry) => entry.local === binding.local);
    if (!imported || !imported.moduleId || imported.imported === '*') return { kind: 'gap', reason: 'declaration-unobserved' };
    return follow(imported.moduleId, imported.imported);
  };
  return follow(moduleId, exposed);
}
