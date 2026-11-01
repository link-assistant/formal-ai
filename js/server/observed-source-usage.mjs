// Intended owner: js/server/observed-source-usage.mjs.
// Installed host parser providers are capabilities. Request data supplies only
// observed owner/source/language and cannot install a frontend.
import { readFileSync } from 'node:fs';
import { babelSourceFrontend } from '../../scripts/lib/source-cst-babel.mjs';
import { readSyntaxRules } from '../meta/readable-source.js';
import { observedSourceUsage } from '../agentic/observed_source_usage.mjs';
let parser = null,
  rules = null;
try {
  parser = await import('@babel/parser');
} catch {/* Named missing-grammar gap at use. */}
try {
  rules = readSyntaxRules(readFileSync(new URL('../../data/meta/source-model/syntax-rules.lino', import.meta.url), 'utf8'));
} catch {/* No lexical fallback. */}

/** An optional Rust provider must be installed by the host itself. It can use
 * the existing real Rust source frontend in production; local JS tests provide
 * only Babel and never initialize product WASM. */
export function makeObservedSourceUsageOperation(installedFrontendForSource = null) {
  if (installedFrontendForSource !== null && typeof installedFrontendForSource !== 'function') throw Error('SourceUsageInstalledFrontendRequired');
  return request => {
    if (!request || Object.keys(request).sort().join('|') !== 'language|owner|source') throw Error('SourceUsageRequestContract');
    let frontend = installedFrontendForSource?.(request.owner, request.language) ?? null;
    if (!frontend && ['JavaScript', 'TypeScript'].includes(request.language) && parser && rules) {
      frontend = {
        parse: babelSourceFrontend(parser, request.owner),
        rules
      };
    }
    if (!frontend) throw Error(request.language === 'Rust' ? 'SourceUsageRustFrontendUnavailable' : 'SourceUsageGrammarUnavailable');
    return observedSourceUsage({
      ...request,
      frontend
    });
  };
}
export const observedSourceUsageOperation = makeObservedSourceUsageOperation();
