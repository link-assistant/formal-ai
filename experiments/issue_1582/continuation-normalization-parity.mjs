// Supplied parity review: normalize native seed forms exactly as the JS recognizer.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {parseLedger,sourceFiles,audit} from '../../scripts/check-minimal-core-boundary.mjs';
const path='rust/src/solver_handler_how.rs';
let source=fs.readFileSync(path,'utf8');
const before=`        .any(|surface| {
            normalized
                .strip_prefix(surface.as_str())`;
assert(source.includes(before));
source=source.replace(before,`        .any(|surface| {
            let surface = normalize_prompt(surface);
            !surface.is_empty() && normalized
                .strip_prefix(surface.as_str())`);
source=source.replace('crate::coding::contains_cjk(surface)\n','crate::coding::contains_cjk(&surface)\n');
fs.writeFileSync(path,source);
assert.equal(spawnSync('rustfmt',['--edition','2024',path],{stdio:'inherit'}).status,0);
const file='data/meta/core-boundary-ledger.lino';
let text=fs.readFileSync(file,'utf8');
const ledger=parseLedger(text), files=sourceFiles(process.cwd());
const active=ledger.entries.filter(entry=>['promote','migrate'].includes(entry.disposition));
const changed=active.filter(entry=>entry.baselineLines!==files.get(entry.path));
assert.deepEqual(changed.map(entry=>entry.path),[path]);
const previous=changed[0].baselineLines, measured=files.get(path);
const start=text.indexOf('  source '+path+'\n'),end=text.indexOf('\n  source ',start+3);
const stop=end<0?text.length:end;
text=text.slice(0,start)+text.slice(start,stop).replace('baseline_lines '+previous,'baseline_lines '+measured)+text.slice(stop);
const sum=entries=>entries.reduce((total,entry)=>total+files.get(entry.path),0);
for(const [key,value]of Object.entries({source_lines_max:sum(active),outside_core_lines_max:sum(active.filter(entry=>entry.disposition==='migrate'))}))text=text.replace(new RegExp('(  '+key+' )\\d+'),'$1'+value);
assert.deepEqual(audit(parseLedger(text),files),[]);
fs.writeFileSync(file,text);
console.log(JSON.stringify({path,previous,measured,unchangedDispositions:true}));
