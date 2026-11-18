// Issue #1550 (acceptance criterion c): the census of the answer paths.
//
// One recursive procedure produces every answer: the native entry
// `UniversalSolver::solve_with_history_probability_store_and_intent_cache`
// (rust/src/solver.rs) runs the handler cascade inside one drafting closure
// whose value is the direct-derivation draft; `controlled_solve` (issue #1584)
// calls it for the turn's prompt and the canonical reading of a task
// paraphrase, and its draft goes to `run_meta_procedure`; the
// worker's `solve` (js/worker/formal_ai_worker_solve.js) drafts through
// `solveImpl` and hands its answer to `metaRunProcedure`. A return outside that
// closure (or a new exit of `solve`) would let a turn skip the procedure, so the census below
// reads both entries and reports every exit.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { specificationCases } from '../../../../scripts/lib/rust-specification-cases.mjs';

export const RUST_ENTRY = 'rust/src/solver.rs';
export const JS_ENTRY = 'js/worker/formal_ai_worker_solve.js';
const RUST_FN = 'fn solve_with_history_probability_store_and_intent_cache(';
const CLOSURE_OPEN = 'let mut solve_core = |prompt: &str, log: &mut EventLog| -> SymbolicAnswer {';
const CLOSURE_CLOSE = '};\n        let crate::cnl_output::ControlledSolve {';
const RUST_EXIT = /\breturn\b|\?\s*;/gu;

/** Source without line comments, so a comment that mentions `return` is no exit. */
const withoutLineComments = (source) => source.replace(/^\s*\/\/.*$/gmu, '');

/**
 * The exits of the native entry: before the cascade closure, inside it (the
 * cascade's methods, whose value is the direct-derivation draft) and after it.
 * @param {string} source rust/src/solver.rs
 */
export function rustEntryCensus(source) {
  const start = source.indexOf(RUST_FN);
  if (start < 0) throw new Error(`${RUST_ENTRY} lost ${RUST_FN}`);
  const end = source.indexOf('\nimpl UniversalSolver {', start);
  const body = withoutLineComments(source.slice(start, end < 0 ? undefined : end));
  const open = body.indexOf(CLOSURE_OPEN);
  const close = body.lastIndexOf(CLOSURE_CLOSE);
  if (open < 0 || close < open) throw new Error(`${RUST_ENTRY}: the cascade closure is gone`);
  const before = body.slice(0, open);
  const inside = body.slice(open + CLOSURE_OPEN.length, close);
  const after = body.slice(close + CLOSURE_CLOSE.length);
  const count = (text) => (text.match(RUST_EXIT) ?? []).length;
  return {
    exitsBefore: count(before),
    cascadeReturns: (inside.match(/\breturn\b/gu) ?? []).length,
    exitsAfter: count(after),
    procedureAfter: /\bself\.run_meta_procedure\(/u.test(after),
  };
}

/**
 * The exits of the worker's `solve` and every caller of `solveImpl`.
 * @param {string} source js/worker/formal_ai_worker_solve.js
 */
export function jsEntryCensus(source) {
  const start = source.indexOf('async function solve(');
  const end = source.indexOf('\nasync function solveImpl(', start);
  if (start < 0 || end < 0) throw new Error(`${JS_ENTRY}: solve or solveImpl is gone`);
  const body = withoutLineComments(source.slice(start, end));
  const exits = [...body.matchAll(/\breturn\b([^;]*);/gu)].map((match) => match[1].trim());
  return { exits, solveImplCalls: (body.match(/\bsolveImpl\(/gu) ?? []).length };
}

/** Every `solveImpl(` call in the worker sources outside `solve`. */
export function solveImplCallersOutsideSolve(root, files) {
  const outside = [];
  for (const file of files) {
    let source = withoutLineComments(readFileSync(join(root, file), 'utf8'));
    if (file === JS_ENTRY) {
      const start = source.indexOf('async function solve(');
      const end = source.indexOf('\nasync function solveImpl(', start);
      source = source.slice(0, start) + source.slice(end).replace('async function solveImpl(', '');
    }
    if (/\bsolveImpl\(/u.test(source)) outside.push(file);
  }
  return outside;
}

/** The prompts of every specification test carried as an ask, deduplicated in corpus order. */
export function censusPrompts(root) {
  const prompts = specificationCases(root).cases
    .flatMap((item) => (item.asks ?? []).map((ask) => ask.prompt));
  return [...new Set(prompts)];
}

/** The text of rust/tests/fixtures/procedure-census-prompts.json. */
export function censusFixtureText(root) {
  return JSON.stringify({
    schema: 'procedure-census-prompts/v1',
    source: 'scripts/lib/rust-specification-cases.mjs',
    prompts: censusPrompts(root),
  }, null, 2) + '\n';
}
