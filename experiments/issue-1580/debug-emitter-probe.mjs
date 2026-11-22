import { rustLocation } from '../../js/server/debug-stage.mjs';
import { FunctionIndex, readStageSources, stageEmitter } from '../../js/server/debug-stage-sources.mjs';
const root = process.cwd(), sources = readStageSources(root);
const handler = rustLocation('arithmetic', root);
const index = new FunctionIndex(root, sources.runtimes.get('rust'));
const levels = [];
let names = new Set([handler?.symbol]);
for (let depth = 0; depth <= 2; depth++) {
  levels.push({depth, names: [...names].sort()});
  names = new Set([...names].flatMap(name => [...(index.calls.get(name) ?? [])]));
}
console.log(JSON.stringify({handler, levels, emitter: stageEmitter(sources, 'rust', 'calculation:engine', handler?.symbol, root), autonomousAuthorshipCredit: 0}, null, 2));
