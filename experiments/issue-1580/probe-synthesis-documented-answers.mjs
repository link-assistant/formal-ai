// Supplied investigation for issue #1580; zero independent authorship credit.
import {readFileSync} from 'node:fs';
import {createWorkerContext,evaluate,plain} from '../../rust/tests/web/support/browser-runtime.mjs';
const worker=createWorkerContext();
await evaluate(worker,'loadSeed()');
const cases=JSON.parse(readFileSync('data/parity/cross-runtime-synthesis.json','utf8'));
const expected=JSON.parse(readFileSync('experiments/issue-1580/evidence/checks/native-summary-expectation/rust-documented-answers.json','utf8'));
let mismatches=0;
for(const entry of cases){
 const response=plain(await worker.solve(entry.prompt,[],{},{},[],{}));
 const exact=Object.hasOwn(expected,entry.id)?response.content===expected[entry.id]:null;
 if(exact===false)mismatches++;
 process.stdout.write(JSON.stringify({id:entry.id,expected:expected[entry.id]??null,exact,response})+'\n');
}
process.stdout.write(JSON.stringify({cases:cases.length,exactCases:Object.keys(expected).length,mismatches,independentAuthorshipCredit:0})+'\n');
process.exitCode=mismatches?1:0;
