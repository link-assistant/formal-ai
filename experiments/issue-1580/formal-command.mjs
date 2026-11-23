#!/usr/bin/env node
// Externally supplied commands receive zero autonomous-authorship credit.
import {spawnSync} from 'node:child_process';
import {mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
// Tool result lines are indented by drive.mjs; only its top-level boundary
// ends the transcript. Source bytes may contain the same marker or exit text.
export function reviewCommandResult(result, shellCommand) {
  const stdout = result.stdout ?? '';
  const boundary = /^== answer ==\r?$/mu.exec(stdout);
  const trace = boundary ? stdout.slice(0, boundary.index) : stdout;
  const headers = [...trace.matchAll(/^>> (\S+) ([^\r\n]*)\r?$/gmu)];
  let arguments_, receipt;
  if (headers.length === 1 && trace.slice(0, headers[0].index) === '') {
    try { arguments_ = JSON.parse(headers[0][2]); } catch { /* Invalid arguments earn no execution credit. */ }
    const body = trace.slice(headers[0].index + headers[0][0].length);
    // shell-capture.mjs appends the actual status after all stdout/stderr.
    // A preview, signal or JSON error capture has no such complete footer.
    if (body.startsWith('\n   Output: ')) receipt = /\n   Exit Code: (\d{1,3})\r?\n?$/u.exec(body);
  }
  const exact = headers.length === 1 && headers[0][1] === 'bash'
    && arguments_ !== null && typeof arguments_ === 'object' && !Array.isArray(arguments_)
    && Object.keys(arguments_).length === 1 && arguments_.command === shellCommand;
  const code = receipt ? Number(receipt[1]) : null;
  const driverSucceeded = result.status === 0 && !result.signal && !result.error;
  const qualified = boundary !== null && exact && code !== null && code <= 255;
  const exitCode = !driverSucceeded
    ? (Number.isInteger(result.status) && result.status > 0 ? result.status : 1)
    : qualified ? code : 1;
  return {trace, exact, exitCode};
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
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
// The full response is retained above; show the tool trace once.
const reviewed = reviewCommandResult(result, shellCommand);
process.stdout.write(reviewed.trace);
process.stderr.write(result.stderr ?? '');
if (!reviewed.exact) process.stderr.write('Requested command was not executed verbatim; no verification credit.\n');
process.exit(reviewed.exitCode);
}
