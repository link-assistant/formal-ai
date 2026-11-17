#!/usr/bin/env node
// Externally supplied commands receive zero autonomous-authorship credit.
import {spawnSync} from 'node:child_process';
import {mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const command=process.argv[2];
if(!command)throw new Error('usage: formal-command.mjs <shell command>');
const directory=resolve('experiments/issue-1580/evidence/commands');
mkdirSync(directory,{recursive:true});
const identity=spawnSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).stdout.trim();
const id=new Date().toISOString().replaceAll(':','-');
const shellCommand="env sh -c '"+command.replaceAll("'", "'\\''")+"'";
const request='Run '+shellCommand;
const result=spawnSync(process.execPath,['--max-old-space-size=256','experiments/js_dogfood/drive.mjs','--dir',process.cwd(),'--steps','4',request],{encoding:'utf8',maxBuffer:8*1024*1024});
writeFileSync(directory+'/'+id+'.json',JSON.stringify({runtime:'formal-ai JavaScript driver',node:process.version,source:identity,request,stdout:result.stdout,stderr:result.stderr,status:result.status,signal:result.signal,autonomousAuthorshipCredit:0},null,2)+'\n');
process.stdout.write(result.stdout??'');
process.stderr.write(result.stderr??'');
const calls=[...(result.stdout??'').matchAll(/^>> bash (\{.*\})$/gmu)].map(match=>JSON.parse(match[1]));
const exact=calls.some(call=>call.command===shellCommand);
const codes=[...(result.stdout??'').matchAll(/Exit Code: (\d+)/gu)].map(match=>Number(match[1]));
if(!exact)process.stderr.write('Requested command was not executed verbatim; no verification credit.\n');
process.exit(result.status||(!exact||!codes.length?1:codes.find(code=>code!==0)??0));
