#!/usr/bin/env node
// Run the CI gates that work without a local Rust build, one at a time.
//
// Coding agents and Formal AI share this runner, so neither keeps its own gate
// list. The gates come from the two places CI keeps them:
//
// - `data/meta/ci-gates/*.lino`, the registry that `scripts/run-ci-gates.rs`
//   runs in the `lint` matrix;
// - single-line `run:` steps of the standalone workflows in
//   `.github/workflows/` that call a checker script under `scripts/`.
//
// A gate whose command needs cargo or a built binary is skipped: CI builds Rust
// and the local machine does not. A workflow step that reads CI context (an
// expression, a variable, an artifact file) is skipped for the same reason.
// The machine does not have it. The runner never runs gates in parallel, so a
// shared machine stays responsive. Each gate's output goes to
// `sandboxes/gates/<name>.log` (git-ignored).
//
// No gate builds the crate here. A gate's `rust-script --test` half and the
// suite that tests every script are left to CI, and `--js-only` also skips
// the gates that compile a Rust script at all, for a machine short of disk.
//
// Usage:
//   node experiments/formal_ai_subagent/local-gates.mjs            # run all
//   node experiments/formal_ai_subagent/local-gates.mjs --js-only  # no Rust compile at all
//   node experiments/formal_ai_subagent/local-gates.mjs --list     # print them
//   node experiments/formal_ai_subagent/local-gates.mjs --only check_file_size,web-ui-boundary
//   node experiments/formal_ai_subagent/local-gates.mjs --match js  # names containing "js"
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const REGISTRY = join(ROOT, 'data/meta/ci-gates');
const WORKFLOWS = join(ROOT, '.github/workflows');
// The branch pull requests merge into, as CI's `github.base_ref` names it.
const BASE_BRANCH = process.env.FORMAL_AI_BASE_BRANCH ?? 'main';
const LOGS = join(ROOT, 'experiments/formal_ai_subagent/sandboxes/gates');
// Commands that need a Rust build, which only CI performs.
const NEEDS_BUILD = /\bcargo\b|\btarget\/(debug|release)\b|\bwasm-pack\b|\bnpm run vscode:test\b/;
// A script that launches cargo itself: a Rust `Command::new("cargo")`, a
// `"cargo",` program argument, or a shell line that runs cargo. The command
// text of `check_capability_routing` names only rust-script, but the script
// runs `cargo run --example …`, a full crate build.
const LAUNCHES_CARGO = /Command::new\("cargo"\)|"cargo",|^\s*(?:exec\s+)?cargo\s+(?:build|test|run|package|check|clippy|metadata)\b/m;

/** Whether a gate's command, or a script under scripts/ it runs, builds Rust. */
function needsBuild(run) {
  if (NEEDS_BUILD.test(run)) return true;
  return (run.match(/scripts\/[\w./-]+\.(?:rs|sh|mjs|py)/g) ?? []).some((script) => {
    try {
      return LAUNCHES_CARGO.test(readFileSync(join(ROOT, script), 'utf8'));
    } catch {
      return false;
    }
  });
}
// A step whose output a later step of its job reads: a build, a test or
// coverage run, or a downloaded artifact. Checker steps after it read CI
// context (the coverage ratchet reads the report the job just generated).
const PRODUCES_OUTPUT = /\bcargo\b|coverage[:-]|--coverage|\bc8\b|llvm-cov|download-artifact|node --test|playwright test/;
// A standalone workflow step counts as a gate when it runs a checker script.
const CHECKER_STEP = /^(python3|node|rust-script|bash)\s+(--test\s+)?scripts\/check[-_]/;

function registryGates() {
  return readdirSync(REGISTRY)
    .filter((file) => file.endsWith('.lino'))
    .map((file) => {
      const text = readFileSync(join(REGISTRY, file), 'utf8');
      const name = text.match(/^ci_gate\s+(\S+)/m)?.[1];
      const run = text.match(/^\s+run\s+"((?:[^"\\]|\\.)*)"/m)?.[1]?.replace(/\\"/g, '"');
      return name && run ? { name, run, source: `data/meta/ci-gates/${file}` } : null;
    })
    .filter(Boolean);
}

// The steps of a workflow file: each `- ` item with its single-line `run:`
// and whether it reads CI context (an expression, a variable or a file that
// only an earlier job produces).
function workflowSteps(text) {
  const steps = [];
  let current = null;
  // Whether an earlier step of the current job produced output a later step
  // reads: a build, a test or coverage run, or a downloaded artifact.
  let produced = false;
  for (const line of text.split('\n')) {
    const indent = line.search(/\S/);
    if (/^ {2}[A-Za-z0-9_-]+:\s*$/.test(line)) {
      produced = false;
      current = null;
      continue;
    }
    if (/^\s*-\s/.test(line)) {
      if (current && PRODUCES_OUTPUT.test(current.text)) produced = true;
      current = { run: null, context: produced, env: {}, indent, text: '' };
      steps.push(current);
    } else if (current && indent >= 0 && indent <= current.indent) {
      // A line at the dash's indentation or left of it belongs to the job.
      if (PRODUCES_OUTPUT.test(current.text)) produced = true;
      current = null;
    }
    if (!current) continue;
    current.text += `${line}\n`;
    const entry = line.match(/^\s+([A-Z][A-Z0-9_]*):\s*(.+?)\s*$/);
    const expressions = line.match(/\$\{\{[^}]*\}\}/g) ?? [];
    // A pull request's base ref is the one piece of CI context a local run
    // has: the branch the PR merges into.
    if (entry && expressions.length && expressions.every((e) => e.includes('github.base_ref'))) {
      current.env[entry[1]] = BASE_BRANCH;
    } else if (expressions.length && !/^\s*(-\s*)?(if|name|timeout-minutes):/.test(line)) {
      current.context = true;
    }
    const run = line.match(/^\s*-?\s*run:\s*(.+?)\s*$/)?.[1];
    if (run && run !== '|' && run !== '>') current.run = run;
  }
  return steps;
}

function readsCiContext(step) {
  if (step.context || step.run.includes('$')) return true;
  // An argument naming a data file that is not in the tree is an artifact.
  return step.run.split(/\s+/).some((word) => /\.(tsv|json|txt)$/.test(word) && !existsSync(join(ROOT, word)));
}

function workflowGates(known) {
  const gates = [];
  for (const file of readdirSync(WORKFLOWS).filter((f) => /\.ya?ml$/.test(f))) {
    const steps = workflowSteps(readFileSync(join(WORKFLOWS, file), 'utf8'))
      .filter((step) => step.run && CHECKER_STEP.test(step.run));
    // A `--test` step is followed by its run step; keep them as one gate.
    const merged = [];
    for (const step of steps) {
      const previous = merged.at(-1);
      if (previous && previous.run.includes('--test') && previous.run.split(/\s+/).at(-1) === step.run.split(/\s+/).at(1)) {
        merged[merged.length - 1] = {
          run: `${previous.run} && ${step.run}`,
          context: previous.context || readsCiContext(step),
          env: { ...previous.env, ...step.env },
        };
      } else {
        merged.push({ run: step.run, context: readsCiContext(step), env: step.env });
      }
    }
    merged.forEach((step, index) => {
      if (known.has(step.run)) return;
      known.add(step.run);
      const base = file.replace(/\.ya?ml$/, '');
      gates.push({
        name: merged.length > 1 ? `${base}#${index + 1}` : base,
        run: step.run,
        source: `.github/workflows/${file}`,
        context: step.context,
        env: step.env,
      });
    });
  }
  return gates;
}

// `rust-script --test X` compiles a second, debug binary of every gate script
// (about 1.4 GB of cache for the whole set); the script's own unit tests run
// in CI. Locally a gate keeps only the check itself.
const SCRIPT_TESTS = /rust-script --test \S+ && /g;
const RUNS_SCRIPT_TESTS = /\bscripts\/test-scripts\.sh\b/;

function allGates() {
  const registry = registryGates();
  const known = new Set(registry.map((gate) => gate.run));
  const jsOnly = process.argv.includes('--js-only');
  return [...registry, ...workflowGates(known)].map((gate) => ({
    ...gate,
    run: gate.run.replace(SCRIPT_TESTS, ''),
  })).map((gate) => ({
    ...gate,
    skipped: needsBuild(gate.run) ? 'needs a Rust build (CI only)'
      : RUNS_SCRIPT_TESTS.test(gate.run) || /^rust-script --test \S+$/.test(gate.run) ? 'compiles script unit tests (CI only)'
      : jsOnly && /\brust-script\b/.test(gate.run) ? 'compiles a Rust script (--js-only)'
      : gate.context ? 'reads CI context (secrets, variables or artifacts)' : null,
  }));
}

function option(name) {
  const at = process.argv.indexOf(name);
  return at >= 0 ? process.argv[at + 1] ?? '' : null;
}

const gates = allGates();
if (process.argv.includes('--list')) {
  for (const gate of gates) console.log(`${gate.skipped ? "skip" : "run "}  ${gate.name.padEnd(44)} ${gate.run}${gate.skipped ? `  [${gate.skipped}]` : ""}`);
  process.exit(0);
}
const only = option('--only')?.split(',').filter(Boolean);
const match = option('--match');
const chosen = gates.filter((gate) =>
  (!only || only.includes(gate.name)) && (!match || gate.name.includes(match)));
mkdirSync(LOGS, { recursive: true });
const failed = [];
for (const gate of chosen) {
  if (gate.skipped) {
    console.log(`SKIP ${gate.name} (${gate.skipped})`);
    continue;
  }
  const started = Date.now();
  const result = spawnSync('bash', ['-c', gate.run], { cwd: ROOT, env: { ...process.env, ...gate.env }, encoding: 'utf8', maxBuffer: 64 << 20 });
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  const log = join(LOGS, `${gate.name.replace(/[^\w.#-]/g, '_')}.log`);
  writeFileSync(log, `$ ${gate.run}\n${result.stdout ?? ''}${result.stderr ?? ''}`);
  const passed = result.status === 0;
  console.log(`${passed ? 'PASS' : 'FAIL'} ${gate.name} ${seconds}s${passed ? '' : ` -> ${log.slice(ROOT.length + 1)}`}`);
  if (!passed) failed.push(gate.name);
}
console.log(`\n${chosen.length - failed.length}/${chosen.length} passed or skipped; failed: ${failed.join(', ') || 'none'}`);
process.exit(failed.length ? 1 : 0);
