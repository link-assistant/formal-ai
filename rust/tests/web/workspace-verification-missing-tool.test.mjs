// Captured natural-language edit: missing verification capability must not fall
// through to an unrelated whole-file refusal after a real edit has occurred.
import test from 'node:test';
import assert from 'node:assert/strict';
import {planDigestVerification} from '../../../js/agentic/workspace_change/digest_verification.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost(),{sourceUsageRustGrammar:false});
test('missing Run returns a terminal explicit unverified gap',()=>{
 const plan=planDigestVerification('replace a with b in module.mjs',[],['read','edit'],{target:'module.mjs',expected:'b',intent:'coding-text-replaced',slots:[]});
 assert.equal(plan.kind,'final');const answer=JSON.parse(plan.answer);
 assert.equal(answer.reason,'VerificationRunToolUnavailable');assert.equal(answer.target,'module.mjs');
 assert.equal(answer.verified,false);assert.equal(answer.complete,false);
});
test('available Run retains the existing digest command',()=>{
 const plan=planDigestVerification('replace a with b in module.mjs',[],['bash'],{target:'module.mjs',expected:'b',intent:'coding-text-replaced',slots:[]});
 assert.equal(plan.kind,'tool_calls');assert.equal(JSON.parse(plan.calls[0].arguments).command,'sha256sum -- module.mjs');
});
