import test from 'node:test';
import assert from 'node:assert/strict';
import {duplicationCoverage,duplicationReminder,reportDuplicationCoverage} from '../../../scripts/report-duplication-coverage.mjs';

test('filename reminder distinguishes configured roots, unmeasured languages and recorded debt without scanning source',()=>{
 const paths=['js/a.mjs','ts/a.mts','rust/src/a.rs','scripts/check.mjs',
  'rust/tests/a.rs','tools/x.py','scripts/run.sh','data/x.lino','docs/a.md','js/a.mjs'];
 const rows=duplicationCoverage(paths);
 assert.deepEqual(rows.find(row=>row.language==='JavaScript/TypeScript'),
  {language:'JavaScript/TypeScript',paths:3,configured:2,outside:1});
 assert.deepEqual(rows.find(row=>row.language==='Rust'),
  {language:'Rust',paths:2,configured:1,outside:1});
 assert.equal(rows.find(row=>row.language==='Python').configured,0);
 assert.equal(duplicationCoverage(['rust/src/a.RS'])[0].configured,0);
 assert.match(duplicationReminder(['file.swift']),/Unmeasured .swift/u);
 const absent=duplicationReminder(paths);
 assert.match(absent,/Measured debt\/gaps: unavailable/u);
 assert.match(absent,/no source bodies were read or scanners run/u);
 assert.match(absent,/no mirror exemption/u);
 const result={schema:'source-scanner-hosted-qualification/v1',phase:'js',
  qualificationComplete:true,strictZeroAccepted:false,operations:[
   {name:'javascript-inventory',files:3,cloneGroups:7,coverageGaps:2,complete:true},
  ]};
 const measured=duplicationReminder(paths,result);
 assert.match(measured,/independent clone groups 7, gaps 2/u);
 assert.match(measured,/recorded strict-zero accepted: false/u);
 assert.match(measured,/not revalidated against current working-tree bytes/u);
 assert.match(duplicationReminder(paths,{...result,operations:[]}),/missing or ambiguous/u);
 assert.match(duplicationReminder(paths,{}),/unsupported shape/u);
 let calls=0;
 const report=reportDuplicationCoverage({summaryPath:null,run:(command,args,options)=>{
  calls++;assert.equal(command,'git');
  assert.deepEqual(args,['ls-files','--cached','--others','--exclude-standard','-z']);
  assert.equal(options.timeout,10000);
  return paths.join('\0')+'\0';
 }});
 assert.equal(calls,1);
 assert.equal(report,absent);
 assert.ok(Buffer.byteLength(report)<=8192);
});
