import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {encodeTyped} from '../../js/server/structured-log.mjs';
import {encodeDialogRecord} from '../../js/server/dialog-codec.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const report=fs.readFileSync(root+'/experiments/issue_714_agentic_mode/run_report_e2e.sh','utf8');
const verifier=root+'/experiments/agent_cli_e2e/issue_819_tui/verify-dialog.mjs';
const ladder=fs.readFileSync(root+'/experiments/issue_847_coding_ladder/run_coding_ladder.sh','utf8');
const typed=message=>encodeTyped({schema:'formal-ai-diagnostic-v1',message});
function temporary(run){
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'evidence-reader-'));
 try{return run(dir);}finally{fs.rmSync(dir,{recursive:true,force:true});}
}
function command(program,args){return spawnSync(program,args,{encoding:'utf8',timeout:10000});}
const prompt='Find hive-mind-control center folder on my desktop';
const expected='/observed/hive-mind-control-center';
function records(){
 return [
  {timestamp_unix_ms:1,request_body:prompt,response_body:'tool request',response_tool_calls:[
   {name:'bash',arguments:{command:'find "$FORMAL_AI_DESKTOP_DIR" -type d -name hive-mind-control-center'}},
   {name:'bash',arguments:{command:'find "$FORMAL_AI_DESKTOP_DIR" -type d -iname "*hive*"'}},
  ]},
  {timestamp_unix_ms:2,request_body:JSON.stringify({prompt,role:'tool',content:expected}),response_body:expected},
 ];
}
function verify(dir,entries,format='lino',terminated=true){
 const dialog=path.join(dir,'dialogs');fs.mkdirSync(dialog);
 const encode=format==='lino'?encodeDialogRecord:JSON.stringify;
 fs.writeFileSync(dialog+'/actual.'+format,entries.map(encode).join('\n')+(terminated?'\n':''));
 return command(process.execPath,[verifier,dialog,'control',dir+'/sequence.json',expected]);
}
for(const format of ['lino','jsonl'])test('actual verifier retains complete '+format+' exchange evidence',()=>temporary(dir=>{
 const result=verify(dir,records(),format);assert.equal(result.status,0,result.stderr);
 const output=JSON.parse(fs.readFileSync(dir+'/sequence.json'));assert.equal(output.exchange_count,2);
 assert.equal(output.sequence.at(-1).content,expected);
}));
for(const format of ['lino','jsonl'])test('actual verifier refuses missing returned result in '+format,()=>temporary(dir=>{
 const entries=records();entries[1].request_body=prompt;
 const result=verify(dir,entries,format);assert.notEqual(result.status,0);assert.match(result.stderr,/never returned/);
}));
test('typed malformed and partial dialog evidence fails before any inferred sequence',()=>{
 for(const malformed of ['(formal-ai-log-v1 broken\n',encodeDialogRecord(records()[0])])temporary(dir=>{
  fs.mkdirSync(dir+'/dialogs');fs.writeFileSync(dir+'/dialogs/actual.lino',malformed);
  const result=command(process.execPath,[verifier,dir+'/dialogs','control',dir+'/sequence.json',expected]);
  assert.notEqual(result.status,0);assert.ok(!fs.existsSync(dir+'/sequence.json'));
 });
});
test('typed verifier preserves original command and no-web oracles',()=>{
 for(const mutate of [entries=>entries[0].response_tool_calls[0].arguments.command+='; bad',entries=>entries[0].response_tool_calls[0].name='web'])temporary(dir=>{
  const entries=records();mutate(entries);assert.notEqual(verify(dir,entries).status,0);
 });
});
test('report counts three actual headers, not embedded body decoys, with immutable raw bytes',()=>temporary(dir=>{
 const helper=root+'/experiments/agent_cli_e2e/server-log-evidence.sh';
 const header='[trace] POST /v1/chat/completions (40 byte body)';
 for(const count of [2,3]){
  const raw=Array.from({length:count},()=>typed(header)).join('\n')+'\n'+typed('body\n'+header)+'\n';
  fs.writeFileSync(dir+'/raw',raw);
  const result=command('bash',['-c','set -euo pipefail\nROOT="$1"\nsource "$2"\nposts="$(project_server_log_file "$3" "$4" /v1/chat/completions)"\ntest "$posts" -ge 3','control',root,helper,dir+'/raw',dir+'/semantic']);
  assert.equal(result.status,count===3?0:1,result.stderr);assert.equal(fs.readFileSync(dir+'/raw','utf8'),raw);
 }
}));
test('report preserves all original three-turn report, gh, memory and round assertions',()=>{
 for(const literal of ['run_turn "$REPORT_PROMPT"','run_turn "GitHub issue" --continue --no-fork','run_turn "Both logs" --continue --no-fork','grep -Fxq issue "$CAPTURE"','grep -Fxq create "$CAPTURE"','grep -Fxq link-assistant/formal-ai "$CAPTURE"',"grep -q 'issues/999' \"$AGENT_LOG\"",'test "$posts" -ge 3',"grep -Fq 'kind \"tool_call\"' \"$MEMORY\"",'FORMAL_AI_AGENT_MODE=1'])assert.ok(report.includes(literal),literal);
 assert.equal(report.split('test ! -f "$CAPTURE"').length-1,2);
 assert.match(report,/posts="\$\(project_server_log_file "\$LOG" "\$SEMANTIC_LOG" \/v1\/chat\/completions\)"/);
 assert.doesNotMatch(report,/grep -c[^\n]+"\$LOG"/);
});
function refusal(raw){return temporary(dir=>{
 const log=dir+'/raw';fs.writeFileSync(log,raw);
 const start=ladder.indexOf('    haystack = output');
 const end=ladder.indexOf('    answered =',start);
 assert.ok(start>=0&&end>start);
 const block=ladder.slice(start,end).split('\n').map(line=>line.startsWith('    ')?line.slice(4):line).join('\n');
 const source='import os, subprocess\nroot='+JSON.stringify(root)+'\noutput='+JSON.stringify('server log: '+log)+'\n'+block+'\nprint(refused)\n';
 return command('python3',['-c',source]);
});}
for(const message of ['could not determine the task','не смог определить задачу','I do not have a template for language X','supported languages: Rust']){
 for(const format of ['legacy','typed'])test('ladder observes '+format+' refusal '+message,()=>{
  const result=refusal(format==='typed'?typed(message)+'\n':message+'\n');
  assert.equal(result.status,0,result.stderr);assert.equal(result.stdout.trim(),'True');
 });
}
test('ladder keeps success distinct and refuses invalid typed evidence',()=>{
 const success=refusal(typed('actual successful answer')+'\n');assert.equal(success.status,0,success.stderr);assert.equal(success.stdout.trim(),'False');
 assert.notEqual(refusal('(formal-ai-log-v1 broken\n').status,0);
});

test('legacy complete final JSON object without newline remains accepted',()=>temporary(dir=>{
 const result=verify(dir,records(),'jsonl',false);
 assert.equal(result.status,0,result.stderr);
}));
test('typed invalid UTF8 fails before any sequence can be written',()=>temporary(dir=>{
 fs.mkdirSync(dir+'/dialogs');
 fs.writeFileSync(dir+'/dialogs/actual.lino',Buffer.from([0xff,0x0a]));
 const result=command(process.execPath,[verifier,dir+'/dialogs','control',dir+'/sequence.json',expected]);
 assert.notEqual(result.status,0);
 assert.ok(!fs.existsSync(dir+'/sequence.json'));
}));

const directoryCases = [
  ['run_issue_781.sh', 'run_client', 'client_dir', ['agent', '8783']],
  ['run_issue_819.sh', 'run_client', 'client_dir', ['agent', '8783']],
  ['run_issue_819.sh', 'run_client_tui', 'client_dir', ['opencode', '8783', 'primary', 'lookup']],
  ['run_issue_840.sh', 'run_agent_case', 'case_dir', ['definition', '8783', 'lookup', 'answer', '1', '3']],
];
for (const [name, functionName, parent, args] of directoryCases) {
  test(`${name} ${functionName} creates independent client and dialog custody before redirection`, () => temporary(dir => {
    const script = fs.readFileSync(root + '/experiments/agent_cli_e2e/' + name, 'utf8');
    const start = script.indexOf(functionName + '() {');
    const mkdir = script.indexOf('  mkdir -p ', start);
    const end = script.indexOf('\n', mkdir);
    assert.ok(start >= 0 && mkdir > start && end > mkdir);
    const prefix = script.slice(start, end);
    const header = typed('[trace] POST /v1/chat/completions (40 byte body)');
    const replacement = 'mkdir -p "$' + parent + '" "$dialog_dir" || exit 1';
    assert.ok(prefix.includes(replacement));
    for (const broken of [false, true]) {
      const trial = dir + (broken ? '/broken' : '/fixed');
      fs.mkdirSync(trial);
      const body = broken ? prefix.replace(replacement, 'mkdir -p "$dialog_dir"') : prefix;
      const fixture = [
        'set -euo pipefail',
        'ROOT="$1"; WORKDIR="$2"; ARTIFACT_DIR="$WORKDIR/artifacts"; PORT=8783',
        'HEADER="$3"; shift 3',
        'source "$ROOT/experiments/agent_cli_e2e/server-log-evidence.sh"',
        body,
        '  test -d "$' + parent + '"',
        '  test -d "$dialog_dir"',
        '  test "$dialog_dir" != "$' + parent + '"',
        '  test "$FORMAL_AI_DIALOG_LOG_DIR" = "$dialog_dir"',
        '  : > "$client_log"',
        '  printf "%s\\n" "$HEADER" > "$server_log"',
        '  project_server_log --headers-only > "$WORKDIR/posts"',
        '  test "$(cat "$WORKDIR/posts")" = 1',
        '  retain_server_log_evidence',
        '  cmp "$server_log" "$EVIDENCE_DIR/formal-ai.log"',
        '}',
        functionName + ' "$@"',
      ].join('\n');
      const result = command('bash', ['-c', fixture, 'directory-control', root, trial, header, ...args]);
      if (broken) assert.notEqual(result.status, 0);
      else assert.equal(result.status, 0, result.stderr);
    }
  }));
}
