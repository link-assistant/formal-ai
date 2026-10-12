TASK (tag TEACH-F): teach Formal AI the open gaps in experiments/formal_ai_subagent/gaps.md. Unsafe ones come first.

1. **G71 (unsafe).** A request whose quotes do not pair must decline. Today it acts on its payload: an Insert whose quoted ledger row said "set the contents of note.txt to hello" wrote note.txt at the workspace root.
   - Find where the quoted segments are read (`js/agentic/crate/normal_markov.mjs` `quotedSegmentSpans` and its Rust twin, plus `composeEditRequest`).
   - Make an unpaired opening quote, or a backslash-escaped quote inside a quoted literal, decline with a seeded answer that names the problem, instead of reading the payload as the instruction.
2. **G70 (parity).** The Rust `normalize_prompt` expands contractions (`can't` → `cannot`) and the JS agentic `normalizePrompt` does not. JS is the source of truth (JS-first doctrine).
   - Decide from the seed which form is right.
   - Make both runtimes normalize alike, keeping the vocabulary in seed data.
   - Fix the contracted seed surfaces that stop matching (CIFIX counted about 80).
   - Pin both runtimes on the same inputs.
3. **G69.** An unquoted addition ("add hello to config.txt") plans nothing in either root. Plan read, edit, check, or answer with a seeded question naming what is missing. Never a whole-file write (`namesAnAddition` stays).
4. **G63.** A backtick-delimited Replace whose replacement holds a sentence break followed by more backtick spans is cut short. A payload ending in a backtick span takes that span as the target file.
5. **G64.** An append whose quoted payload names file paths routes to `read_many` and runs `cat` on the payload's paths (`planRoutedCapabilityStepIn` in `capability_router.mjs`).
6. **G15.** The general change plan log is overwritten, not appended. Do the honest-step-text variant TEACH-C proposed: the step says what it does (`data/meta/agentic-messages.lino` and its Rust twin at `rust/src/agentic_coding/general_planner.rs`). Running Formal AI from the repository root must not leave `.formal-ai/general-change-plan.lino` modified. Make the driver write plan events under a git-ignored path when `--dir` is the repository root, or make the log append-only, whichever keeps the pinned tool sequences.
7. **G25.** Create and run a Python test, discovering the interpreter on PATH.
8. **T181.** After a correct Replace, Formal AI answered "The command completed successfully without output." instead of naming the replacement.

Each fix:
- generic, with vocabulary in data/seed, JS first and then the Rust twin;
- regression tests in both roots;
- a ledger row in T210–T229, written by Formal AI;
- the gap marked FIXED by Formal AI.

`rust/src/agentic_coding/shell_command.rs` is at 998 of 1000 lines, so move code out before adding any.
