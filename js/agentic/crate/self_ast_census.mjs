// The workspace self-AST census the planner resolves edit targets against
// (rust/src/self_ast_census.rs: `WorkspaceCensus::{module, module_by_reference,
// modules_declaring, resolve}`, `document_path_for`, `fidelity_for`,
// `workspace`).
//
// native-only: rust/src/self_ast_census.rs `workspace()` censuses the source
// files compiled into the binary (`OWNED_SOURCE_FILES`); a JavaScript host has
// no such manifest. The census here is built from the committed census
// documents (data/meta/self-ast/**.lino) the host lists through an optional
// `censusDocuments()` hook returning `[{path, text}]`; without the hook the
// workspace is empty and every reference resolves to nothing, which is the
// "does not apply" answer of the Rust resolver (fails closed).

import { cached, host, hasHost } from '../host.mjs';
import { lines, splitWhitespace, trim } from '../write_str.mjs';

/** Mirrors `const CENSUS_DIR`. */
export const CENSUS_DIR = 'data/meta/self-ast';
/** Mirrors `const FULL_FIDELITY_PREFIX`. */
export const FULL_FIDELITY_PREFIX = 'src/agentic_coding/';

/** Mirrors `fn fidelity_for`: 'full_ast' | 'signature'. */
export function fidelityFor(modulePath) {
  return modulePath.startsWith(FULL_FIDELITY_PREFIX) ? 'full_ast' : 'signature';
}

/** Mirrors `fn document_path_for`. */
export function documentPathFor(modulePath) {
  const stem = modulePath.endsWith('.rs') ? modulePath.slice(0, -3) : modulePath;
  return `${CENSUS_DIR}/${stem}.lino`;
}

/**
 * Parse one committed census document into `{path, fidelity, symbols:
 * [{kind, name, start_line, end_line}]}` (the inverse of
 * `ModuleCensus::links_notation`), or null.
 * @param {string} text
 */
export function moduleFromDocument(text) {
  let path = null;
  const symbols = [];
  let inSymbols = false;
  for (const line of lines(text)) {
    if (line.startsWith('  target ')) path = trim(line.slice('  target '.length));
    if (line === '  symbols') {
      inSymbols = true;
      continue;
    }
    if (inSymbols && line.startsWith('    ') && !line.startsWith('     ')) {
      const [kind, name, start, end] = splitWhitespace(line);
      if (kind && name) symbols.push({ kind, name, start_line: Number(start), end_line: Number(end) });
      continue;
    }
    if (!line.startsWith('    ')) inSymbols = false;
  }
  return path === null ? null : { path, fidelity: fidelityFor(path), symbols };
}

/** Mirrors `WorkspaceCensus::compile` over already-censused modules. */
export function workspaceCensus(modules) {
  return { modules: [...modules].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0)) };
}

/** Mirrors `fn workspace`: the census of the host's committed documents. */
export function workspace() {
  return cached('self-ast-census', () => {
    const documents = hasHost() && typeof host().censusDocuments === 'function' ? host().censusDocuments() : [];
    return workspaceCensus(documents.map((document) => moduleFromDocument(document.text)).filter(Boolean));
  });
}

/** Mirrors `ModuleCensus::symbol`. */
export function moduleSymbol(module, name) {
  return module.symbols.find((symbol) => symbol.name === name) ?? null;
}

/** Mirrors `WorkspaceCensus::module`. */
export function censusModule(census, modulePath) {
  return census.modules.find((module) => module.path === modulePath) ?? null;
}

/** Mirrors `WorkspaceCensus::module_by_reference`. */
export function moduleByReference(census, reference) {
  const exact = censusModule(census, reference);
  if (exact) return exact;
  const matches = census.modules.filter((module) => module.path.endsWith(`/${reference}`));
  return matches.length === 1 ? matches[0] : null;
}

/** Mirrors `WorkspaceCensus::modules_declaring`. */
export function modulesDeclaring(census, name) {
  return census.modules.filter((module) => moduleSymbol(module, name) !== null);
}

function resolution(module, symbol) {
  return {
    module_path: module.path,
    document_path: documentPathFor(module.path),
    fidelity: module.fidelity,
    symbol,
  };
}

/** Mirrors `WorkspaceCensus::resolve`: a `CensusResolution` or null. */
export function resolveReference(census, rawReference) {
  const reference = trim(rawReference);
  if (!reference) return null;
  const colon = reference.lastIndexOf(':');
  if (colon >= 0) {
    const module = moduleByReference(census, reference.slice(0, colon));
    if (!module) return null;
    const symbol = moduleSymbol(module, reference.slice(colon + 1));
    return symbol ? resolution(module, symbol) : null;
  }
  const module = moduleByReference(census, reference);
  if (module) return resolution(module, null);
  const declaring = modulesDeclaring(census, reference);
  if (declaring.length !== 1) return null;
  const symbol = moduleSymbol(declaring[0], reference);
  return symbol ? resolution(declaring[0], symbol) : null;
}
