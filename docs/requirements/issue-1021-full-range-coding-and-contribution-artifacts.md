## Issue #1021 Full-Range Coding And Contribution Artifacts

Issue [#1021](https://github.com/link-assistant/formal-ai/issues/1021) collects
the prompts Formal AI answered wrongly, the capabilities a contribution needs to
land, and the demand that the whole loop be test-covered in one pull request.
See PR #1027 and `docs/case-studies/issue-1021/`.

The rule the issue states first governs every row below: "solve by
generalization, not specialization -- a per-prompt fix that makes one task pass
is a regression against that instruction, not progress". Rows marked **not
delivered** are reported rather than quietly narrowed, per the issue's own "the
bar is not to be met by lowering it".

| ID | Requirement | Status |
| --- | --- | --- |
| R1021-1 | Deliver every sub-issue in a single pull request rather than as a backlog. | One branch, one pull request, one composed body that closes #1021. |
| R1021-2 | Fix the rule that was wrong, never the prompt that exposed it. | Every routing change lands in a rule with its vocabulary in `data/seed/`, and each is pinned by held-out paraphrases in `tests/unit/issue_1021_behaviour_range.rs`. |
| R1021-3 | #868: a bare `ls` is the request, not a fragment. | `bare_shell_tokens` reads the terminal-command catalog, so an argument-free command routes to `exec_command` (`a_bare_command_is_the_request`). |
| R1021-4 | #866/#867: `Execute ls command` runs `ls`, not `ls command`. | The command-naming noun is declared per language in `data/seed/terminal-commands.lino` and stripped before the passthrough (`a_command_naming_noun_is_not_an_argument`). |
| R1021-5 | #865: `List me files here` lists files; `Hello` still greets. | `src/agentic_coding/directory_listing.rs` composes a listing request from seeded parts in any word order (`a_prose_listing_request_routes_to_ls_in_any_word_order`). |
| R1021-6 | #863: an example request for copy stdin to stdout in Rust is answered with code. | Half delivered. The `cp stdin stdout` misrouting is gone -- an example-of request is not an execution request (`an_example_request_is_not_a_command_to_run`, `a_named_exercise_is_not_a_file_operation`) -- but the request is **not yet answered with code**: it reaches web search, because `copy stdin to stdout` is not a catalogued task and the catalog cannot hold one. See finding 9 in the case study. |
| R1021-7 | #862: a Rosetta Code URL is a task to solve, not a `cp` operand. | Half delivered. The URL is no longer lowered to `cp` (`a_web_address_is_a_resource_not_a_program`), but the task it names is **not solved**: like R1021-6 it reaches web search rather than the coding catalog. |
| R1021-8 | #723: a PHP request is answered with PHP. | PHP joins the coding catalog with the eleven verified task templates every other catalogued language carries. A Laravel *scaffold* is **not delivered**: the answer is plain PHP. |
| R1021-9 | #824: a filesystem move it refuses should be performed. | Operands are filtered by how a path is written rather than by where it points, so an absolute or `~`-relative move is planned (`a_move_between_absolute_paths_is_performed`). |
| R1021-10 | #943 (E91): guard against harness-created issues before any GitHub write authority. | `gh issue create` is on the never-delegated rung of `src/contribution_write_path.rs` and is refused in both opt-in states (`filing_an_issue_is_refused_in_both_states`). |
| R1021-11 | #944 (E92): rungs on the mutating-action ladder. | The publishing rungs are delivered: refused-unless-opted-in, and never-delegated. The sandbox-reset rungs stay with #944. |
| R1021-12 | #946 (E94): versioned recoverable memory. | **Not delivered** -- a capability of an unattended run, which this branch does not perform; reported in the case study rather than simulated. |
| R1021-13 | #947 (E95): bounded autonomy with a stuck-recovery limit. | **Not delivered**, for the same reason as R1021-12. |
| R1021-14 | #924 (E77): one real repository change per release landing as a normal reviewed pull request. | **Not delivered by a `solve` run**; `data/meta/self-hosting-ledger.lino` still reads `0.00% self-authored`. |
| R1021-15 | Produce the changelog fragment a code change needs. | `formal_ai::contribution_artifacts::compose` renders it from `data/seed/contribution-artifacts.lino` into the shape `scripts/check-changelog-fragment.rs` accepts. |
| R1021-16 | Produce a pull-request body that links its issue with a closing keyword. | Same generator; the closing line leads the body, and `scripts/check-pull-request-link.rs` accepts it. |
| R1021-17 | Generated code is R379-clean. | Delivered narrowly: `src/contribution_artifacts.rs` holds no natural-language literal, all wording living in seed data. It is not a claim about arbitrary future generated code. |
| R1021-18 | Pin each reported prompt as a routing test with held-out paraphrases. | `tests/unit/issue_1021_behaviour_range.rs` tests unseen word orders and languages, not the reported strings alone. |
| R1021-19 | Test the process artifacts by driving the generator, not by asserting on a fixture. | `the_committed_process_artifacts_are_generator_output` compares every committed fragment and the body against `compose` output; `examples/issue_1021_write_contribution_artifacts.rs` writes them the same way. |
| R1021-20 | Test the write path in both states, with `issue create` refused in both. | `tests/unit/issue_1021_write_path.rs` exercises both states in one process. |
| R1021-21 | Pin the closed circle as a replayable session. | `closed_circle_session_replays` compares `docs/case-studies/issue-1021/closed-circle-run/session.json` byte for byte against a fresh drive of the public API. |
| R1021-22 | Definition of done: a pull request opened by Formal AI from a real `solve` run, green without a human editing the branch. | **Not achieved.** The artifacts such a run needs are delivered and pinned; the run is the remaining gap. |
| R1021-23 | Do not meet the bar by lowering it. | No gate was relaxed. The two ratchets touched moved in the improving direction: the outside-core ceiling drops to 19,539 lines, and the seed-metadata gap floor rises only by closure-generated records. |
| R1021-24 | Collect the case-study data the standing clauses require. | `docs/case-studies/issue-1021/` holds the timeline, requirement list, per-requirement plans, library survey, online research, raw GitHub data, and probe logs. |
| R1021-25 | Add debug output wherever a root cause is not visible in existing logs. | The `examples/issue_1021_*.rs` probes, with their output preserved under `docs/case-studies/issue-1021/logs/`. |
| R1021-26 | File upstream issues with reproductions for defects that are not ours. | None found: every root cause traced back into this repository. |
| R1021-27 | Give every requirement a traceability row. | The rows above and their entries in `docs/requirements-traceability.md`, honest "not yet confirmed" included. |
