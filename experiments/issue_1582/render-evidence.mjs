// Render measured Node behavior for review; this is not an installed Claude TUI.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {createServeLogger} from '../../js/server/structured-log.mjs';
const root='docs/case-studies/issue-1582';
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'issue-1582-render-'));
try {
  const probe=spawnSync(process.execPath,['--input-type=module','-e',`import {createServeLogger} from './js/server/structured-log.mjs';const log=createServeLogger({env:{HOME:${JSON.stringify(directory)}}});log.log('debug','wrapper.started');log.log('debug','wrapper.session');log.close();`],{encoding:'utf8'});
  if(probe.status!==0||probe.stderr!==''||probe.stdout!=='')throw Error('quiet_log_probe_failed');
  const host=new WorkerHost();const prompt='Как включить доступ к интернету?';
  const answer=await host.solve(prompt,[],{forcedResponseLanguage:'ru'});
  const logger=createServeLogger({env:{HOME:directory,FORMAL_AI_LOG_FORMAT:'nested'}});
  logger.log('debug','wrapper.session',{component:'client_integrations',session_file:'example/session.jsonl'});logger.close();
  const nested=fs.readFileSync(logger.file,'utf8');
  const evidence={kind:'Node regression replay; not an installed Claude TUI',prompt,answer:answer.content,diagnosticStdout:probe.stdout,diagnosticStderr:probe.stderr,nested};
  fs.mkdirSync(root+'/raw-data/local',{recursive:true});fs.writeFileSync(root+'/raw-data/local/render-evidence.json',JSON.stringify(evidence,null,2)+'\n');
  const escape=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
  const html=`<!doctype html><meta charset="utf-8"><title>Issue 1582 measured regression replay</title><style>body{max-width:1050px;margin:40px auto;background:#10141c;color:#e5ebf5;font:18px/1.55 system-ui;padding:0 28px}h1{font-size:29px}h2{font-size:21px;color:#a8d5ff}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#1b2433;padding:22px;border-radius:10px;font:16px/1.5 monospace}.note{color:#bac6d8}.pass{color:#80e4af}</style><h1>Issue #1582 · measured Node regression replay</h1><p class="note">Actual worker response and diagnostic-sink probes. This is not an installed Claude TUI or a native-wrapper end-to-end capture.</p><h2>Harness permission question</h2><pre>${escape(prompt+'\n\n'+answer.content)}</pre><h2>Default diagnostic sink</h2><pre class="pass">child exit: ${probe.status}\nstdout: ${JSON.stringify(probe.stdout)}\nstderr: ${JSON.stringify(probe.stderr)}\nRoutine records persisted in the private PID log.</pre><h2>Explicit nested file format</h2><pre>${escape(nested.slice(nested.lastIndexOf('(formal-ai-log-v1')))}</pre>`;
  fs.writeFileSync(root+'/screenshots/after-replay.html',html);
  console.log(JSON.stringify({kind:evidence.kind,stdoutBytes:probe.stdout.length,stderrBytes:probe.stderr.length,intent:answer.intent,html:root+'/screenshots/after-replay.html'}));
} finally {fs.rmSync(directory,{recursive:true,force:true});}
