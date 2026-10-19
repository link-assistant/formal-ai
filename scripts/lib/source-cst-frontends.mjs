// Caller-owned grammar providers project complete syntax into one readable meta model.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { babelSourceFrontend } from './source-cst-babel.mjs';
import { rustSourceFrontend } from './source-cst-rust.mjs';
import { readSyntaxRules } from '../../js/meta/readable-source.js';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const digest = file => createHash('sha256').update(fs.readFileSync(file)).digest('hex');
export async function sourceCstFrontends(runtimeDirectory = process.env.FORMAL_AI_CST_RUNTIME || root) {
  const require = createRequire(path.join(path.resolve(runtimeDirectory), 'package.json'));
  const entry = require.resolve('@babel/parser'),
    parser = await import(pathToFileURL(entry).href);
  const metadata = require('@babel/parser/package.json');
  const wasmRuntime = path.join(root, 'js/vendor/tree-sitter/web-tree-sitter.mjs'),
    grammar = path.join(root, 'js/vendor/tree-sitter/tree-sitter-rust.wasm');
  const runtime = await import(pathToFileURL(wasmRuntime).href);
  await runtime.Parser.init();
  const rustParser = new runtime.Parser();
  rustParser.setLanguage(await runtime.Language.load(grammar));
  const rust = rustSourceFrontend(rustParser);
  const rulesFile = path.join(root, 'data/meta/source-model/syntax-rules.lino'),
    rules = readSyntaxRules(fs.readFileSync(rulesFile, 'utf8'));
  const providers = {
    babel: {
      provider: '@babel/parser',
      version: metadata.version,
      moduleSha256: digest(entry)
    },
    rust: {
      provider: 'web-tree-sitter/rust',
      runtimeSha256: digest(wasmRuntime),
      grammarSha256: digest(grammar)
    }
  };
  return {
    forSource(filename, language) {
      if (!['JavaScript', 'TypeScript', 'Rust'].includes(language)) throw Error('UnsupportedSourceCstLanguage');
      return {
        parse: language === 'Rust' ? rust : babelSourceFrontend(parser, filename),
        rules,
        provenance: {
          ...(language === 'Rust' ? providers.rust : providers.babel),
          syntaxRulesSha256: digest(rulesFile)
        }
      };
    },
    close() {
      rustParser.delete();
    }
  };
}
