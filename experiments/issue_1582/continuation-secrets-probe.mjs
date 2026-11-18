// Bounded external scanner diagnosis; reports are masked by secretlint by default.
import fs from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
const files=execFileSync('git',['diff','--name-only','--diff-filter=ACMRT','-z','origin/main','HEAD'],{encoding:'utf8'}).split('\0').filter(file=>file&&!/^docs\/case-studies\/.*\.log$/u.test(file));
const mode=process.argv[2];
const cli=process.env.SECRETLINT_CLI;
if(!cli || !fs.statSync(cli).isFile())throw Error('Set SECRETLINT_CLI to the pinned secretlint 13.0.5 CLI');
if(!['literal','glob'].includes(mode))throw Error('Choose literal or glob');
const output=fs.openSync('/tmp/issue-1582-secretlint-'+mode+'.json','w');
const errors=fs.openSync('/tmp/issue-1582-secretlint-'+mode+'.errors.log','w');
const result=spawnSync(process.execPath,['--max-old-space-size=384',cli,...(mode==='literal'?['--no-glob']:[]),'--format','json',...files],{stdio:['ignore',output,errors]});
fs.closeSync(output);fs.closeSync(errors);
console.log(JSON.stringify({mode,files:files.length,status:result.status,signal:result.signal,error:result.error?.message}));
