Part of the E127 umbrella (#1183); routing defects seen in the parity probes.

## Evidence (0.347.0)

- "Find the bug: def average(xs): return sum(xs) / len(xs) - 1" → "It looks like you want to run a terminal command".
- "Make a 3-day itinerary for a first visit to Rome." → "It looks like you want to run a terminal command".
- "Write a regular expression that matches a US ZIP code with an optional 4-digit extension." → a TypeScript browser-*extension* project plan (`software_project_request … artifact "extension"`).

Routing still reacts to surface words ("extension") and sentence shapes, not to the formalized request.

## Root cause (verified on `main` d209aac64)

- **Shell commands by first word.** `detect_terminal_command` (`rust/src/solver_terminal.rs:80-106`) returns a command whenever `leading_shell_command` (`:63-76`) finds the prompt's first word in `shell_tokens` of `data/seed/terminal-commands.lino` (lines ~111-160). That list contains ordinary English verbs and nouns: `find`, `make`, `file`, `which`, `head`, `tail`, `touch`, `kill`, `export`, `cat`. "Find the bug: …" starts with `find`; "Make a 3-day itinerary …" starts with `make`. The JS worker mirrors the rule (`tryTerminalCommand`).
- **Artifact kind by a word.** `data/seed/meanings-software-project.lino:369` `artifact_extension` has the surface `extension`, so "… ZIP code with an optional 4-digit extension" becomes a `software_project_request` for a TypeScript browser extension.
- Both decisions are taken from a single surface word before the request is formalized; the capability decision table of plan 10 (`data/seed/capability-routing.lino`, `rust/src/agentic_coding/capability_router.rs`) is not consulted for them.

## Requirements

- **R1** A prompt is a shell command only when its formalization says so: it parses as a command line (the executable resolves to a program, e.g. `command -v` in agent mode or the known-program registry `data/seed/setup-publishers.lino`, and the rest parses as arguments), or it is fenced/backticked with a run verb, or it names a terminal explicitly. A natural-language sentence whose first word happens to be a command name is never a shell command.
- **R2** An artifact kind is selected from the formalized object of the request ("write a regular expression" → object `regular_expression`), never from any occurrence of a surface word elsewhere in the sentence ("4-digit extension" modifies "ZIP code").
- **R3** Every request is routed through the capability decision table (object, act, locus) of `data/seed/capability-routing.lino` after formalization; the ad hoc checks in `solver_terminal.rs` and the software-project detector become rows of that table.
- **R4** A routing probe set of at least 200 held-out requests covering every class of #1171 in en/ru/hi/zh, each with its expected route, runs in CI; the three probes above are in it.
- **R5** Three-roots parity: the browser worker's `tryTerminalCommand` and artifact detection follow the same rule (translated code, parity cases).

## Design

- `rust/src/solver_terminal.rs`: replace `leading_shell_command` with `parse_command_line(prompt) -> Option<CommandLine>` built on the existing shell parser (`command_stream::parse_shell_command`, already exported by the pinned crate, `src/lib.rs:89`), accepted only when the first word resolves to a program and the remainder contains no sentence punctuation outside quotes; keep `bare_shell_tokens` for single-word prompts (`git`, `ls`).
- `data/seed/capability-routing.lino`: add rows `object shell_command act run locus workspace` and `object software_artifact act build …`; `rust/src/solver.rs` calls the router after `formalize_prompt_candidates` and before the handler chain.
- Software-project detection reads the formalized object (the head noun of the verb's object phrase), not `mentions_role(ROLE_SOFTWARE_ARTIFACT_KIND, …)`.
- Probe set `data/benchmarks/routing/{en,ru,hi,zh}.lino` (record: `prompt`, `expected_route`), next to the existing `capability-routing/` suite.

## Tests

- `rust/tests/unit/issue_1175_routing.rs`: "Find the bug: def average(xs): …" → `code_debugging`; "Make a 3-day itinerary for a first visit to Rome." → `planning`; "Write a regular expression that matches a US ZIP code with an optional 4-digit extension." → `regex_synthesis`; "find . -name '*.log' -size +10M" → shell command; "make test" → shell command; plus the 200-prompt set.
- Command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1175_routing`.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1175-routing-by-formalization.md`; `rust-script scripts/assemble-requirements.rs --write`; traceability rows.
- [ ] Case study `docs/case-studies/issue-1175/` with the three misroutes and the route table.
- [ ] Changelog fragment in `changelog.d/`; `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.

## Depends on / blocks

- Depends on nothing to start (R1 and R2 are local fixes).
- Blocks #1174, #1176, #1177, #1178 and #1186, whose handlers must be reached, and the routing column of #1171.

## What to do

Route every request from its formalization (act, object, constraints) through the capability decision table (plan 10), including coding intents; a single word may not select an artifact kind when the formalized object is different ("ZIP code … extension" is a data format property, not a software artifact). Add a routing probe set of 200 held-out requests covering all parity classes in four languages, with the expected class, and run it in CI.

## How to test

The routing probe set passes; none of the three prompts above is misrouted.



