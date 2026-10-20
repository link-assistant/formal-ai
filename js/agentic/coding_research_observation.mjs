// Lossless lexical/catalog observations; canonical kernels make research decisions.
import { callbackText as canonicalUnaryText } from './callback_values.mjs';
import { validateProgram } from './host.mjs';
import { tokenizeWithSpans } from './crate/es_tokenizer.mjs';
import { observeSourceCallables } from './module_function/callable_catalog.mjs';
function lexicalTokens(trees, output) {
  for (const tree of trees) {
    if (tree.$ === 'leaf') output.push({ ...tree.span, text: tree.text, kind: tree.kind });
    else if (tree.$ === 'group') {
      const pair = { paren: ['(', ')'], brace: ['{', '}'], bracket: ['[', ']'] }[tree.delim];
      output.push({ start: tree.span.start, end: tree.span.start + 1, text: pair[0], kind: 'punctuator' });
      lexicalTokens(tree.trees, output);
      output.push({ start: tree.span.end - 1, end: tree.span.end, text: pair[1], kind: 'punctuator' });
    } else output.push({ ...tree.span, text: '', kind: 'unsupported' });
  }
}
/** Lossless tree flattening is representation normalization, not assertion parsing. */
export function observeCodingResearchAcceptanceTokens(source) {
  const tokens = [];
  try { lexicalTokens(tokenizeWithSpans(source), tokens); }
  catch { return { texts: [], kinds: [], complete: false }; }
  return { texts: tokens.map(canonicalUnaryText), kinds: tokens.map(token => token.kind), complete: true };
}
/** Observe existing grammar/catalog/token spans without selecting or accepting a procedure. */
export function observeCodingResearchRebindingCandidate(source, language, modulePath) {
  const catalog = observeSourceCallables(source, modulePath);
  const declaration = catalog.declarations[0];
  const text = declaration?.source ?? '';
  const pieces = [], pieceBindings = [], spans = [];
  let lexical = true;
  try { lexicalTokens(tokenizeWithSpans(text), spans); } catch { lexical = false; }
  const bytes = new TextEncoder().encode(text);
  const decoder = new TextDecoder('utf-8', { ignoreBOM: true });
  let cursor = 0;
  for (const span of spans) {
    if (span.start < cursor || span.end < span.start || span.end > bytes.length) { lexical = false; break; }
    if (span.start > cursor) { pieces.push(decoder.decode(bytes.slice(cursor, span.start))); pieceBindings.push(''); }
    pieces.push(decoder.decode(bytes.slice(span.start, span.end)));
    pieceBindings.push(span.kind === 'identifier' ? span.text : '');
    cursor = span.end;
  }
  if (cursor < bytes.length) { pieces.push(decoder.decode(bytes.slice(cursor))); pieceBindings.push(''); }
  return { $: 'candidate', language, syntaxValid: lexical && validateProgram(language, source) === true,
    moduleEffectsNone: catalog.moduleEffects === 'none', moduleSyntaxSupported: catalog.moduleSyntax === 'supported',
    importsCount: catalog.imports.length, declarationsCount: catalog.declarations.length,
    contractSupported: declaration?.contract.status === 'supported', callEffectsNone: declaration?.contract.callEffects === 'none',
    name: declaration?.name ?? '', parameters: declaration?.parameters ?? [],
    exported: catalog.exports.some(item => item.local === declaration?.name), pieces, pieceBindings };
}
