Historical first-plan fixtures: exact cases are in pr1188-native-first-plan-fixture-draft.json.

Shared setup should drive only the named auxiliary stream and return the first requested effect unconsumed. Assert a single call each turn, decode its real advertised argument schema, and inspect path/command rather than skipping arbitrary Read calls. Bound auxiliary setup to three transitions for read/write/run_command clients:
1. read PLAN_PATH: answer actual saved prior bytes, or the same explicit File not found error the fake workspace would produce for absence.
2. write PLAN_PATH: assert content is prior + required separator + plan.links_notation(); apply exactly those bytes to the workspace. Do not invent success after a denied write.
3. read PLAN_PATH: answer exact applied bytes and verify the next call leaves PLAN_PATH and targets the original requested file.
Replay with a prior stream and verify preserved prefix, one event identity, and no duplicate write on an already observed event. A corrupted readback must never authorize the target mutation.

For Bash/bare shell clients, the first auxiliary operation is the actual canonical append command, not a plan write. Execute it in a selected scratch workspace (or use one shared real shell adapter). Supply its actual event stdout with status 0; assert byte-preserved history, one event, and no command rewriting. Do not report arbitrary 'wrote the plan' as observed append bytes. Existing general-plan-event-append.test.mjs is the working reference.

Write-only clients have no event read/append path: the primary target write is first and final disposition remains honest auxiliary-event-unavailable Gap. Preserve requested target/content assertions; do not simulate an unadvertised read. Existing failed_auxiliary_plan_write_without_read_still_attempts_the_target currently treats the actual first target write as a failed auxiliary write, then expects a second target.

Keep all original verification-negative result assertions and exact target bytes. Fault injection occurs only after successful auxiliary setup, against the same original target action or verification. Source-target guard reads are NOT auxiliary setup: classify them by actual path and handle with their genuine current precondition fixture, never broadly swallow Read calls. Semantic exec_command-vs-write failures and malformed payload classes are separate capability defects.
