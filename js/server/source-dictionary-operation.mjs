import { deriveNamedImportEdit } from '../agentic/source_import_binding.mjs';
// Installed complete-grammar capability; observed source is input, never executed.
import { readFileSync } from 'node:fs';
import { babelSourceFrontend } from '../../scripts/lib/source-cst-babel.mjs';
import { readSyntaxRules } from '../meta/readable-source.js';
import { deriveOwnKeyDictionaryRepair } from '../agentic/source_dictionary_binding.mjs';
let parser = null,
  rules = null,
  knowledge = null;
try {
  parser = await import('@babel/parser');
} catch {}
try {
  rules = readSyntaxRules(readFileSync(new URL('../../data/meta/source-model/syntax-rules.lino', import.meta.url), 'utf8'));
} catch {}
try {
  knowledge = JSON.parse(readFileSync(new URL('../../data/meta/ecmascript-own-key-dictionary.json', import.meta.url), 'utf8'));
} catch {}
export function observedSourceRepairOperation(request) {
  if (request?.operation === 'add-named-import') {
    if (Object.keys(request).sort().join('|') !== 'binding|module|operation|owner|source' || !parser || !rules) throw Error('NamedImportRequestContract');
    return deriveNamedImportEdit({
      ...request,
      frontend: {
        parse: babelSourceFrontend(parser, request.owner),
        rules
      }
    });
  }
  if (!request || Object.keys(request).sort().join('|') !== 'operation|owner|source' || request.operation !== 'own-key-storage') throw Error('SourceRepairRequestContract');
  if (!parser || !rules || !knowledge) throw Error('SourceRepairInstalledGrammarUnavailable');
  return deriveOwnKeyDictionaryRepair({
    owner: request.owner,
    source: request.source,
    frontend: {
      parse: babelSourceFrontend(parser, request.owner),
      rules
    },
    knowledge
  });
}
