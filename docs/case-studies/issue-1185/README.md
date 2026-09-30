# Issue #1185 Case Study: The Executor That Stopped at the First Error

Issue [#1185](https://github.com/link-assistant/formal-ai/issues/1185) (E149,
part of the #1183 umbrella). Drafted on the `qa-reasoning-coding-bulk-fixes`
branch under the fleet's no-build constraint: the code, seeds and probes are
written and formatted; nothing was compiled or run.

## What a user saw

Ask the executor to build or run the program it just emitted and one failed
step ends the whole attempt:

```
rustc --edition 2021 src/main.rs
error[E0308]: mismatched types
 --> src/main.rs:6:33
```

The failure branch of `plan_symbolic_command_reroute`
(`rust/src/agentic_coding/command_reroute.rs`, the `if let Some(failure)` arm
that returns `AgenticPlan::Final(failure.report(...))`) renders the output
back to the user and stops. The error names its file, line and machine code;
a search for that code returns pages that state the fix; the executor reads
none of them. Every capability the loop needs — search, fetch, write, run —
is already in the planner's `Capability` set and the client's tool list. The
issue's word for this: the executor "stops at the first failed step" when the
diagnostic itself is the most addressed text in programming.

## Root cause

Not a missing capability but a missing consumer of one. `Progress::scan`
already resolves tool transcripts into search output, fetched pages,
attempted fetches, successful writes and successful runs per command;
`web_research.rs` already runs a bounded state machine (search → fetch →
answer) over exactly that state for the research intent. The failure branch
is the one place in the planner that holds the richest datum in the whole
transcript — the raw failed output — and it treats it as prose to quote
rather than a diagnostic to formalize. There is no shape table anywhere in
the seed that says where a language's error code and file/line sit, so even
a consumer would have had to hardcode fourteen grammars in Rust.

## The fix

Three data files and one module, all additive:

1. **`data/seed/diagnostic-code-shapes.lino`** (mirror:
   `rust/embedded/data/seed/diagnostic-code-shapes.lino`) — one `language`
   record per emitted language, each carrying `pattern` rows as `{slot}`
   templates (`error[{code}]: {message}` for rustc, `{file}:{line}:{column}:
   error: {message}` for kotlinc/gcc-style, `{head}: {message}` plus fourteen
   head words for python, `PHP Parse error:` for php) and `location` rows for
   lines that only bind position. Recognition aid only — no fix lives here.

2. **`data/seed/multilingual-responses-repair.lino`** (byte-identical mirror)
   — the loop's two honest stops as `localized_response` templates:
   `repair_ladder_exhausted` ("climbed rung {rung} of at most {max_rungs} …
   stopped here instead of retrying without a bound") and
   `repair_unresolved_need` ("({query}) found no fetched source that
   addresses this exact error … reported as such rather than guessed").

3. **`rust/src/agentic_coding/repair_loop.rs`** — the loop itself:
   `formalize_diagnostic` runs one generic template matcher over the seed
   (slots validated by shape — a `{file}` must carry `.` or `/`, a `{line}`
   must be numeric, a `{code}` letter-prefixed and digit-carrying — so no
   template can read prose as a path; location lines bind per the language's
   print order; a python traceback's innermost frame wins); `search_query`
   composes language + code (≤14 tokens); `repair_step` is the rung state
   machine — search, fetch each unseen source URL, retain a fragment only
   when its page addresses the exact code/message, write the `repair_edit`
   Links Notation record to `<stem>.repair.lino`, re-run the failed command
   — with four honest stops (`NoDiagnostic`, `NoTools`, `Exhausted` at
   `MAX_REPAIR_RUNGS = 3`, `NoMatch`); `attempts_from`/`evidence_document`
   reconstruct the attempt chain as a `repair_attempts` record for #1184's
   derivation. `FailedStep` is the plain-data mirror of `command_reroute`'s
   private `StepFailure` plus the language, failed command and artifact path
   the loop needs.

4. **`rust/tests/unit/issue_1185_error_repair_loop.rs`** — twelve probes:
   rustc/kotlinc/python-traceback formalization (innermost frame, no
   fabricated code), no-shape refusal, search-before-final, the no-match
   stop, the bounded ladder with its reported limit, the non-diagnostic
   decline, record-then-retry (write `.repair.lino`, then re-run `rustc`),
   the record is Links Notation not a patch, the evidence chain, the bounded
   query, and planner-contract call shapes. The matched-page fixtures ride
   a `{"content": …, "exit_code": 0}` harness envelope because a page that
   discusses an error is source text, not a failed fetch (issues #905/#908).

## What was reused, not rewritten

- `Progress::scan` for all transcript state; the loop holds no state of its
  own between rungs, so a rung is a pure function of the transcript.
- `web_research.rs`'s bounded-state-machine idiom (`MAX_RESEARCH_ROUNDS` →
  `MAX_REPAIR_RUNGS`) and its per-source fetch bookkeeping.
- `plan_one`/`tool_for`/`fetch_arguments`/`write_arguments` from `planner.rs`
  so the loop's calls are indistinguishable from any other planner step.
- `localized_response` for every user-facing sentence — the module contains
  no language name, message shape or stop prose as a string literal.
- `repair_strategy.rs`'s Links Notation rendering conventions for the
  `repair_edit` record shape.

## Wiring the main session owes

The loop ships inert by design — an unregistered seed is the same decline a
missing cue table produces, so today's behavior is unchanged until:

1. `pub mod repair_loop;` lands in `rust/src/agentic_coding/modules.rs`
   (alphabetical, between `rebuild_plan` and `repair_strategy`);
2. `mod issue_1185_error_repair_loop;` lands in `rust/tests/unit/mod.rs`;
3. `data/meta/seed-registry.lino` gains `seed diagnostic-code-shapes /
   bundle true` and `seed multilingual-responses-repair / bundle true /
   lexicon response`, with the embedded registry refreshed;
4. the failure branch of `command_reroute.rs` consults the loop before its
   `Final`, threading a `repair_rung: u8` through `RecipeProgress`:

   ```rust
   if let Some(failure) = &progress.failure {
       let step = repair_loop::FailedStep::new(&recipe.language, &failure.reported)
           .with_exit_code(failure.exit_code)
           .with_failed_command(command.clone())
           .with_artifact_path(&artifact_path);
       if let Some(plan) = repair_loop::plan_repair(
           messages, tool_names, &step, progress.repair_rung,
           repair_loop::MAX_REPAIR_RUNGS,
       ) {
           return Some(plan);
       }
       // … today's honest failure report follows unchanged
   }
   ```

   A retried command that fails again re-enters with `repair_rung + 1`; the
   derivation record attaches `repair_loop::evidence_document(
   &repair_loop::attempts_from(messages, &step))`.

## Honest boundaries

- **Seed filename.** The issue's design sketch names the table
  `diagnostic_code_shapes`; the fleet plan suggested `error-repair-
  patterns.lino`. The issue wins — the seed is
  `data/seed/diagnostic-code-shapes.lino`.
- **R8 (three roots).** Not delivered under no-build; the Rust root is the
  reference and the js/ts roots need the translator built. Recorded as open
  in the requirement shard.
- **Apply half of R3.** Rendering the retained `repair_edit` record into the
  target language and editing the source is the #1167/E132 renderer's seam;
  the loop re-runs the command once the record is written, which is the
  executor-side half.
- **Live-fetch proof.** The offline fixtures pin the full decision chain;
  the live E0502-from-Stack-Exchange run is gated on the umbrella's live
  harness, not on this branch.
- **Tests run only after wiring.** Until steps 1–3 above land,
  `formalize_diagnostic` returns no diagnostics (the seed is unregistered)
  and the suite cannot see the loop at all — the module is exported nowhere
  yet, which is the compile-time face of the same inertness.
