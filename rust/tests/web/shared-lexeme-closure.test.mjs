import assert from 'node:assert/strict';
import {execFileSync}from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
const root=path.resolve(import.meta.dirname,'../../..');
test('original closure fixtures and independent malicious scoped references retain their obligations',()=>{
 const output=execFileSync('python3',['-B',path.join(root,'scripts/test-shared-lexeme-closure.py'),root],{encoding:'utf8'});
 assert.match(output,/SCOPED_CLOSURE_CONTROLS_PASS count=18 originalNativeFixturesPreserved=true/);
});
