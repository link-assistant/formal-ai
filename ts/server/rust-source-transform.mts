import path from 'node:path';
import { createHash } from 'node:crypto';
import * as runtime from '../vendor/tree-sitter/web-tree-sitter.mjs';
import { REPO_ROOT } from './lino.mjs';
import { rustConstantRenameCandidate } from '../agentic/crate/rust_constant_rename.mjs';

let parserPromise = null;
async function parserForRust() {
  if (parserPromise === null) parserPromise = (async () => {
    await runtime.Parser.init();
    const language = await runtime.Language.load(path.join(REPO_ROOT, 'js/vendor/tree-sitter/tree-sitter-rust.wasm'));
    const parser = new runtime.Parser();
    parser.setLanguage(language);
    return parser;
  })();
  return parserPromise;
}

/** Parse actual source and reparse the structural candidate; no filesystem write is authorized. */
export async function prepareRustConstantRename(source, previous, replacement) {
  if (typeof source !== 'string') return null;
  const parser = await parserForRust();
  const tree = parser.parse(source);
  try {
    const candidate = rustConstantRenameCandidate(tree, source, previous, replacement);
    if (candidate === null) return null;
    const changed = parser.parse(candidate.content);
    try {
      if (changed.rootNode.hasError) return null;
      const digest = text => createHash('sha256').update(text).digest('hex');
      return Object.freeze({ ...candidate, candidateSyntax: 'tree-sitter-clean',
        beforeSha256: digest(source), afterSha256: digest(candidate.content), compilerValidity: 'unknown' });
    } finally { changed.delete(); }
  } finally { tree.delete(); }
}
