// Finite reproduction of Node's file-only test results without per-file summaries.
import {run} from 'node:test';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
const directory=mkdtempSync(join(tmpdir(),'file-summary-probe-'));
const files=['assertions-only.mjs','named-case.mjs'].map(file=>join(directory,file));
writeFileSync(files[0],"import assert from 'node:assert/strict'; assert.equal(2+2,4);\n");
writeFileSync(files[1],"import test from 'node:test'; test("+JSON.stringify(files[0])+",()=>{});\n");
try {
  const stream=run({files,concurrency:1});
  for await (const event of stream) {
    if(['test:pass','test:fail','test:summary'].includes(event.type)) console.log(JSON.stringify(event));
  }
} finally {rmSync(directory,{recursive:true,force:true});}
