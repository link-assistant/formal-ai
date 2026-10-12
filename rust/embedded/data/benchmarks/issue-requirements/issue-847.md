Splitting a task into smaller tasks is itself a task, and Formal AI cannot currently do it. That makes it a prerequisite for the coding ladder (companion issue) rather than a convenience: a system that cannot decompose cannot drive its own descent from "solve this issue and open a PR" down to a step it can actually complete, and it cannot participate in the Hive Mind loop, where decomposition is how work is distributed.

## Measured today

`main` @ v0.303.0, driven through `formal-ai with agent` (agent CLI 0.25.0) and confirmed against the HTTP API directly.

**Splitting a task — refused:**

```
Split this coding task into at least two smaller independent subtasks and list them
as a numbered list, nothing else: 'Add a paths-ignore filter for experiments to
release.yml and make docs-changed respect excluded_folders.'
```

> I could not determine `Split this coding task into at least two smaller independent subtasks…` from local Links Notation memory, cached public knowledge, or the source cache, and cannot infer a verified answer.

**Judging whether a task is already atomic — refused:**

```
Answer with one word, yes or no: is this coding task already atomic?
'In scripts/detect-code-changes.rs, add dev/log/ to the excluded_folders array.'
```

> I could not determine `Answer with one word, yes or no: is this coding task already atomic?…`

Both land in the unknown-prompt fallback. The second is the **termination condition** for recursive decomposition — without it, a splitter either never stops or stops arbitrarily.

## Why this is not simply "another unknown prompt"

Decomposition is already a first-class internal concept; it is only unavailable *as a request*.

- `GOALS.md` — *"Split hard tasks recursively into smaller tasks that can be tested, executed, or answered."*
- `NON-GOALS.md` — *"Hiding decomposition behind an opaque rule is not acceptable; every sub-impulse must be a first-class event the user can inspect."*
- The solver already emits `sub_impulse:` events and bounds itself with `max_decomposition_depth` in `SolverConfig`.
- `src/meta_frame.rs:79-90` models a `Need` with a `NeedStatus`; `src/recursive_execution.rs:132` has `solve_recursively`, which recurses over a **caller-supplied** task tree.

So the machinery to *represent* a decomposition exists. What is missing is producing one from a natural-language request and exposing it. `solve_recursively` needs a tree; nothing builds that tree from a prompt.

## Requested scope

1. **Decomposition as a recognised intent.** "Split X into subtasks", "разбей задачу", "what are the steps for X" route to a decomposition handler rather than the unknown fallback, in every supported language (the #386 convention: surfaces in seed data, no per-language phrase lists in Rust).

2. **An atomicity judgement.** Given a task, decide whether it can usefully be split further, and say why. This is the recursion's base case and must be answerable on its own.

3. **Recursive splitting to a fixpoint.** Split, test each child for atomicity, recurse into the non-atomic ones. Bounded by `max_decomposition_depth`; deterministic for a given config per `GOALS.md`.

4. **Children that are independently checkable.** A split is only useful if each child has an observable completion criterion. "Understand the codebase" is not a valid child; "add `dev/log/` to the `excluded_folders` array in `scripts/detect-code-changes.rs`" is.

5. **Every sub-task a first-class inspectable event**, per the `NON-GOALS.md` requirement above — reusing `sub_impulse:` rather than inventing a parallel structure.

6. **Coverage of the whole spectrum**, since the same operation must serve very different inputs: a GitHub issue → a plan; a plan step → an edit; an edit → an atomic operation. The companion coding-ladder issue supplies real instances of each, drawn from open issues that have never had a PR.

## Acceptance

- [ ] "Split this task into subtasks" is recognised in all supported languages and never falls through to the unknown-prompt fallback.
- [ ] The result is a list of children, each with an observable completion criterion.
- [ ] "Is this task atomic?" is answerable standalone and used as the recursion's base case.
- [ ] Recursive splitting terminates: every leaf is atomic, or `max_decomposition_depth` is reached and that is reported rather than hidden.
- [ ] Each sub-task appears as an inspectable `sub_impulse:` event.
- [ ] Deterministic: same task + same config → same decomposition.
- [ ] A real issue from the corpus decomposes into children that a human agrees are (a) smaller and (b) jointly sufficient.
- [ ] Regression coverage in the specification suites, in all four languages.

## Relationship to other issues

- **Prerequisite for** the coding-task ladder issue: that ladder's descent is exactly this operation, applied recursively.
- **Required by** [link-assistant/hive-mind#2059](https://github.com/link-assistant/hive-mind/issues/2059) — Hive Mind dispatches `--tool agent --model formal-ai`; decomposition is how it splits work across agents.
- **Related to** #699 (E57, migrate specialized handlers into data-driven meta-methods) — the decomposition handler should land as a meta-method, not another special case.
- **Related to** #840 — the same root cause. `Create a file …` is currently formalized as an **arithmetic** task before falling through to the refusal; decomposition prompts fall through the same way. Both are intent-routing failures, not capability failures.
- **Related to** #702 — a decomposition is a target-state context; children are the diff against current state.

## Reproduction

```bash
cargo build --release
experiments/issue_847_coding_ladder/run_coding_ladder.sh   # nodes meta.L2.decompose, meta.L3.is_atomic
```

Note for anyone extending that harness: judge `expect_answer` against the assistant's answer only. The agent CLI emits verbose JSON logs on the same streams, and an early version of this measurement matched a bare `1.` and `yes` inside log payloads, scoring both decomposition tasks as passing when the model had in fact refused both.

