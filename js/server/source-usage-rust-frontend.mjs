// Production-owned preload: vendored syntax grammar only, no source inventory or SDK.
// The parser is shared for process lifetime; each synchronous parse deletes its tree.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { sourceCstFrontends } from '../../scripts/lib/source-cst-frontends.mjs';
const vendorPins = Object.freeze({
  'web-tree-sitter.mjs': '7c49e3c1d87e24e0bb4c2def909d17154dfde281f5f8280225450090bb4b8110',
  'web-tree-sitter.wasm': 'c03bccdc3b448a32848f5ae327e209c982bbb0840d43eec8bc2d5759544a1ed3',
  'tree-sitter-rust.wasm': '24c89bd9252255e4aebbcbd7d2d308bd92c86dd95a130fdc80efa49577b8d738'
});
let installedFrontend = null;

/** Called only by an explicit host boot capability, never by request content. */
export function installSourceUsageRustFrontend() {
  installedFrontend ??= (async () => {
    for (const [filename, expected] of Object.entries(vendorPins)) {
      const bytes = readFileSync(new URL('../vendor/tree-sitter/' + filename, import.meta.url));
      if (createHash('sha256').update(bytes).digest('hex') !== expected) {
        throw Error('SourceUsageRustGrammarIntegrityMismatch');
      }
    }
    // Rust-only selection never resolves Babel or an external source-model SDK.
    const runtime = await sourceCstFrontends(fileURLToPath(new URL('../../', import.meta.url)), ['Rust']);
    return (owner, language) => language === 'Rust' ? runtime.forSource(owner, language) : null;
  })();
  return installedFrontend;
}
