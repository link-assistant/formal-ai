// Issue #1550: the census must fail when a new early return appears.
import { readFileSync } from 'node:fs';
import { jsEntryCensus, rustEntryCensus } from '../../rust/tests/web/helpers/issue-1550-procedure-census.mjs';
const rust = readFileSync('rust/src/solver.rs', 'utf8');
const js = readFileSync('js/worker/formal_ai_worker_solve.js', 'utf8');
console.log('rust clean', rustEntryCensus(rust));
console.log('rust exit before', rustEntryCensus(rust.replace('let mut answer = (|| {', 'if prompt.is_empty() { return SymbolicAnswer::default(); }\n        let mut answer = (|| {')));
console.log('rust exit after', rustEntryCensus(rust.replace('self.run_meta_procedure(', 'if prompt.is_empty() { return answer; }\n        self.run_meta_procedure(')));
console.log('rust new cascade return', rustEntryCensus(rust.replace('let mut answer = (|| {', 'let mut answer = (|| {\n            if prompt.is_empty() { return SymbolicAnswer::default(); }')).cascadeReturns);
console.log('js exit', jsEntryCensus(js.replace('async function solve(prompt', 'async function solve(prompt').replace(/(async function solve\([^)]*\) \{)/u, '$1\n  if (!prompt) return shortcut(prompt);')).exits);
