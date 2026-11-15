// Drives the planner on a regression-test work item (no LLM) and prints the
// transcript, record and the written test, then runs the test independently.
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {planChatStep} from '../../js/agentic/planner.mjs';
import {drive} from '../js_dogfood/drive.mjs';
import {workItemRecordOf} from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const directory=mkdtempSync(join(tmpdir(),'formal-ai-regression-'));
const source=process.env.BROKEN?`export function accepts(value,minimum,maximum) {\n  return value >= minimum && value < maximum;\n}\n`:`export function accepts(value,minimum,maximum) {\n  return value >= minimum && value <= maximum;\n}\n`;
mkdirSync(join(directory,'src'));mkdirSync(join(directory,'test'));writeFileSync(join(directory,'src/ceiling.mjs'),source);
const task=process.argv[2]??'Add a regression test in test/ceiling.test.mjs for the upper bound of function accepts in src/ceiling.mjs: the maximum is inclusive.';
const result=await drive(planChatStep,directory,task,{steps:12,tools:['read','edit','write','bash']});
console.log(result.stop, result.transcript.map(row=>row.tool+' '+JSON.stringify(row.arguments??'').slice(0,120)));
console.log(String(result.answer).slice(0,1500));
console.log(JSON.stringify(result.record??workItemRecordOf(result),null,1)?.slice(0,3000));
const test=join(directory,'test/ceiling.test.mjs');
if(existsSync(test)){console.log(readFileSync(test,'utf8'));const run=spawnSync('node',['--test',test],{encoding:'utf8'});console.log('independent run exit',run.status);}
