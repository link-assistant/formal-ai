#!/usr/bin/env node
// The worker's offline response data is a checked projection of canonical seed.
import {readFileSync, writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import vm from 'node:vm';
export const OUTPUT = 'js/seed/worker-bootstrap-responses.js';
export const SOURCE = 'data/seed/multilingual-responses.lino';
export function renderBootstrap(loader, seed) {
  const context = vm.createContext({console});
  context.self = context;
  vm.runInContext(loader, context);
  const api = context.FormalAiSeed;
  const responses = api.extractMultilingualResponses(api.parse(seed));
  return '// Generated response data from '+SOURCE+'; do not edit by hand.\n'
    + '// Regenerate with node scripts/generate-worker-bootstrap.mjs --write.\n'
    + 'self.FORMAL_AI_BOOTSTRAP_RESPONSES = Object.freeze('
    + JSON.stringify(responses, null, 2) + ');\n';
}
export function main(argv, root = path.resolve(import.meta.dirname, '..')) {
  const rendered = renderBootstrap(readFileSync(path.join(root,'js/seed_loader.js'),'utf8'), readFileSync(path.join(root,SOURCE),'utf8'));
  const output = path.join(root,OUTPUT);
  if (argv.includes('--write')) writeFileSync(output,rendered);
  else if (readFileSync(output,'utf8') !== rendered) throw new Error(OUTPUT+' is stale; regenerate the seed response projection');
  console.log('worker bootstrap response projection verified');
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
