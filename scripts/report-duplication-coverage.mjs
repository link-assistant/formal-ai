#!/usr/bin/env node
// Metadata reminder only. Strict source scanners remain the acceptance authority.
import {execFileSync} from 'node:child_process';
import {appendFileSync,lstatSync,readFileSync} from 'node:fs';
import {extname,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const javascript = new Set(['.js','.mjs','.cjs','.jsx','.ts','.mts','.cts','.tsx']);
const names = new Map([
 ['.rs','Rust'],['.py','Python'],['.sh','Shell'],['.bash','Shell'],['.zsh','Shell'],
 ['.lino','Links Notation'],['.c','C/C++'],['.h','C/C++'],['.cpp','C/C++'],
 ['.go','Go'],['.java','Java'],['.rb','Ruby'],['.yml','YAML'],['.yaml','YAML'],
]);
const maximumReportBytes = 8192;

export function duplicationCoverage(paths) {
 const rows = new Map();
 for (const path of new Set(paths.filter(Boolean))) {
  const rawExtension = extname(path);
  const extension = rawExtension.toLowerCase();
  const language = javascript.has(extension) ? 'JavaScript/TypeScript'
   : names.get(extension) ?? ('Unmeasured '+(/^[.a-z0-9_+-]{1,20}$/u.test(extension) ? extension : '(other/no extension)'));
  if (!rows.has(language)) rows.set(language,{language,paths:0,configured:0,outside:0});
  const row = rows.get(language);
  row.paths++;
  const configured = javascript.has(rawExtension) && /^(?:js|ts)\//u.test(path)
   || rawExtension === '.rs' && path.startsWith('rust/src/');
  if (configured) row.configured++; else row.outside++;
 }
 return [...rows.values()].sort((a,b)=>a.language.localeCompare(b.language));
}

export function duplicationReminder(paths, qualification = null) {
 const rows = duplicationCoverage(paths);
 const lines = [
  '## DRY review reminder',
  '',
  'Deduplication is the first generalization step: review repeated logic, identify its shared invariant, and validate unseen variants and compositions before consolidation.',
  '',
  'Filename census only; no source bodies were read or scanners run. Counts describe configured paths, not measured parsing or clone coverage. Non-source/configuration files may appear.',
  '',
  '| Language / metadata | Paths | In strict default roots | Outside those roots |',
  '|---|---:|---:|---:|',
  ...rows.slice(0,40).map(row=>'| '+row.language+' | '+row.paths+' | '+row.configured+' | '+row.outside+' |'),
  '',
  ...(rows.length > 40 ? ['Additional extension groups omitted from this bounded display: '+(rows.length-40)+'. They remain unmeasured.'] : []),
  'Strict defaults: JavaScript/TypeScript under js/ and ts/; Rust .rs under rust/src/. All other paths and languages remain unmeasured by these defaults.',
  'Generated JS/TS and worker paths remain scanned. Only independently regenerated, source-bound canonical proofs can identify shared definitions; this census grants no mirror exemption.',
  'Strict policy: zero independent clones and zero coverage gaps. No baselines, suppressions, size exemptions, or historical inventories authorize acceptance.',
  '',
 ];
 if (qualification === null) {
  lines.push('Measured debt/gaps: unavailable in this invocation. Coverage and strict acceptance remain unverified.');
 } else {
  lines.push('Recorded scanner result below is not revalidated against current working-tree bytes; inventory success is not strict acceptance.');
  if (qualification.schema !== 'source-scanner-hosted-qualification/v1'
   || !['js','native'].includes(qualification.phase) || !Array.isArray(qualification.operations)) {
   lines.push('Recorded scanner result: unsupported shape; no acceptance inferred.');
  } else {
   const expected = qualification.phase === 'js' ? 'javascript-inventory' : 'native-inventory';
   const inventories = qualification.operations.filter(operation=>operation?.name === expected);
   if (inventories.length !== 1) {
    lines.push('Recorded scanner result: missing or ambiguous inventory; no acceptance inferred.');
   } else {
    const result = inventories[0];
    const count = value=>Number.isSafeInteger(value) && value >= 0 ? String(value) : 'unknown';
    lines.push('Recorded '+qualification.phase+' inventory: files '+count(result.files)
     +', independent clone groups '+count(result.cloneGroups)+', gaps '+count(result.coverageGaps)+'.');
    lines.push('Recorded qualification complete: '+(qualification.qualificationComplete === true)
     +'; recorded strict-zero accepted: '+(qualification.strictZeroAccepted === true)+'.');
   }
  }
 }
 lines.push('', 'Next review: inspect reported repetition and uncovered languages/paths; share implementations or maintained dependencies while preserving behavior. Differently written semantic duplication still needs review.');
 const report = lines.join('\n')+'\n';
 if (Buffer.byteLength(report) > maximumReportBytes) throw Error('Duplication reminder exceeds output bound');
 return report;
}

export function reportDuplicationCoverage({cwd=process.cwd(),qualificationPath=null,summaryPath=process.env.GITHUB_STEP_SUMMARY,run=execFileSync}={}) {
 const listed = run('git',['ls-files','--cached','--others','--exclude-standard','-z'],{
  cwd,encoding:'utf8',timeout:10000,maxBuffer:16*1024*1024,
 });
 let qualification = null;
 if (qualificationPath !== null) {
  const metadata = lstatSync(qualificationPath);
  if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size > 1024*1024)
   throw Error('Scanner result must be a bounded regular file');
  qualification = JSON.parse(readFileSync(qualificationPath,'utf8'));
 }
 const report = duplicationReminder(listed.split('\0'),qualification);
 if (summaryPath) appendFileSync(summaryPath,report);
 return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
 const args = process.argv.slice(2);
 if (args.length !== 0 && (args.length !== 2 || args[0] !== '--qualification'))
  throw Error('Usage: report-duplication-coverage.mjs [--qualification qualification.json]');
 process.stdout.write(reportDuplicationCoverage({qualificationPath:args[1] ?? null}));
}
