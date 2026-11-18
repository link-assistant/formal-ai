// Probe the current JS root against the #1166 acceptance additions.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import * as O from '../../js/agentic/crate/intent_formalization_obligations.mjs';
import { explicitStdout, programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import * as CI from '../../js/agentic/ci_workflow.mjs';
await installNodeHost(new WorkerHost());
const body = readFileSync(new URL('./issue-1450-body.md', import.meta.url), 'utf8');
const show = (label, text) => {
  const g = O.formalizeRequest(text);
  console.log('==', label);
  console.log(' nodes', g.nodes.length, 'kinds', O.obligationKinds(g));
  console.log(' literals', JSON.stringify(g.literals), 'bound', JSON.stringify(O.boundOutputLiterals(text)), 'stdout', JSON.stringify(explicitStdout(text)));
  console.log(' gaps', O.obligationGapLines(text).length);
};
show('1450', body);
for (const n of g().nodes) console.log('   ', JSON.stringify(n.clause.slice(0,90)), n.expectation?.kind ?? JSON.stringify(n.expectation).slice(0,60));
function g(){return O.formalizeRequest(body);}
show('twice', 'Write a Python program that prints "Hi" twice.');
show('two lines', 'Write a Python program that prints "Hi" and then prints "Hi" again.');
show('distinct', 'Write a Python program that prints "A" and then prints "B".');
show('unicode', 'Write a Python program that prints exactly: "Привет, 世界! \\ \\"q\\""');
show('backslash', 'Write a Python program that prints "C:\\\\path\\n"');
show('paraphrase', 'make a small Kotlin app that writes Hello, World! to the console; add CI');
console.log('ci requested', CI.requestedIn(body), CI.workflowPath());
const answer = programContractAnswer(body);
console.log(JSON.stringify(answer && {files: answer.files?.map?.(f=>f.path), keys:Object.keys(answer)}, null, 1)?.slice(0,1500));
