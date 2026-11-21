// Bounded source-level reproduction against the archived upstream function.
// It does not claim to launch an installed Gemini CLI. Supplied recipe.
import fs from 'node:fs';
import assert from 'node:assert/strict';
const source=fs.readFileSync('docs/case-studies/issue-1582/raw-data/research/gemini-tokenLimits.ts','utf8');
const names=source.match(/import \{([\s\S]*?)\} from/u)[1].split(',').map(x=>x.trim()).filter(Boolean);
const fn=source.match(/export function tokenLimit\([\s\S]*?\n\}/u)[0].replace('export ','').replace('model: Model','model').replace('): TokenCount',')');
const tokenLimit=Function(...names,'DEFAULT_TOKEN_LIMIT','GEMMA_4_TOKEN_LIMIT','return '+fn)(...names,1048576,256000);
const advertisedContext=1234567,actual=tokenLimit('formal-ai');
assert.equal(actual,1048576);assert.notEqual(actual,advertisedContext);
console.log(JSON.stringify({kind:'upstream-source-level-reproduction',model:'formal-ai',advertisedContext,geminiTokenLimit:actual,overridableParameter:false}));
