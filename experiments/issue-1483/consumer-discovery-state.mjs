// Issue #1483: print the recorded state of the consumer-discovery regression so a
// change in `complete` can be traced to the requirement that flipped.
import fs from 'node:fs';
const src=fs.readFileSync('rust/tests/web/repository-consumer-discovery.test.mjs','utf8')
 .replace(/^test\([\s\S]*$/m,'')
 .replaceAll("'../../../","'../../");
const harness='experiments/issue-1483/.consumer-discovery-harness.mjs';fs.writeFileSync(harness,src+`
const ctx={after:f=>process.on('exit',f)};
const r=await run(workspace(ctx));
console.log(Object.keys(r.state).join(","),JSON.stringify(r.state.remainingWork),JSON.stringify(r.state.checks??null).slice(0,800));console.log(JSON.stringify({reason:r.state.reason,complete:r.state.complete,verified:r.state.verified,requirements:r.state.requirements,ledger:r.state.ledger,gaps:r.state.gaps},null,1).slice(0,6000));
console.log(r.answer?.slice?.(0,3000)??r.final?.answer?.slice(0,3000));
`);
try{await import('./.consumer-discovery-harness.mjs');}finally{fs.rmSync(harness,{force:true});}
