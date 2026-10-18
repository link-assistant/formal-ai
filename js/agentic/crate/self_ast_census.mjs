// Workspace navigation resolves paths and declaration spans from actual owned
// source observations. These records are not source CST representations.
// moduleFromDocument accepts only owned recursive source content, never the
// historical committed count documents. Missing host observations fail closed.

import { cached, host, hasHost } from '../host.mjs';
import { trim } from '../write_str.mjs';
import {readSourceLinks, sourceFromLinks, sourceOwner} from '../../meta/readable-source.js';
import {scanSignatureSymbols} from '../../../scripts/lib/signature-census.mjs';

/** Mirrors `const CENSUS_DIR`. */
export const CENSUS_DIR = 'source-content';
/** Mirrors `const FULL_FIDELITY_PREFIX`. */
export const FULL_FIDELITY_PREFIX = 'src/agentic_coding/';

/** Mirrors `fn fidelity_for`: navigation records do not represent full source content. @param {string} modulePath @returns {string} */
export function fidelityFor(modulePath) {
  return 'navigation';
}

/** Mirrors `fn document_path_for`. */
export function documentPathFor(modulePath) {
  const stem = modulePath.endsWith('.rs') ? modulePath.slice(0, -3) : modulePath;
  return `${CENSUS_DIR}/${stem}.lino`;
}

/**
 * Read owned recursive source content into navigation `{path, fidelity, symbols:
 * [{kind, name, start_line, end_line}]}` using the lexical scanner, or null.
 * Mirrors `fn module_from_document` in rust/src/self_ast_census.rs.
 * @param {string} text
 */
export function moduleFromDocument(text) {
  try {
    const document = readSourceLinks(text), path = sourceOwner(document);
    if (path === null) return null;
    const symbols = scanSignatureSymbols(sourceFromLinks(document)).map(symbol => ({
      kind: symbol.kind, name: symbol.name, start_line: symbol.startLine, end_line: symbol.endLine,
    }));
    return {path, fidelity: fidelityFor(path), symbols};
  } catch { return null; }
}

/** Mirrors `WorkspaceCensus::compile` over already-censused modules. */
export function workspaceCensus(modules) {
  return { modules: [...modules].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0)) };
}

/** Mirrors `fn workspace`: navigation from the host's current owned sources. */
export function workspace() {
  return cached('self-ast-census', () => {
    const documents = hasHost() && typeof host().censusDocuments === 'function' ? host().censusDocuments() : [];
    return workspaceCensus(documents.map((document) => document.module ?? moduleFromDocument(document.text)).filter(Boolean));
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
