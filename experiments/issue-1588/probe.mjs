import {readFileSync} from 'node:fs';
import {createWorkerContext,evaluate} from '../../js/server/worker-host.mjs';
const worker=createWorkerContext(); await evaluate(worker,'loadSeed()');
const fixtures=JSON.parse(readFileSync(new URL('../../data/parity/issue-1588-intent-spans.json',import.meta.url),'utf8'));
for(const row of fixtures) {
 const result=await worker.solve(row.prompt,[],{},{},[],{});
 console.log(JSON.stringify({...row,answer:result.content,intent:result.intent,evidence:result.evidence}));
}
