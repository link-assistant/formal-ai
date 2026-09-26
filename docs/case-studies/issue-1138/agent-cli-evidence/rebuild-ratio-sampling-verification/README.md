# Formal AI rebuild-ratio-sampling verification sessions (issue #1145)

External Agent CLI, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-26/27. The server binary is
`/tmp/formal-ai-target-incr/debug/formal-ai`, rebuilt after the PR #1146
test edit; the change touches only the integration test source, so the
served model behavior is the same build verified for PR #1144. No hosted
model participated.

## Context

Main run 36263664670 failed `Test (ubuntu-latest / full)` on
`issue_1106_projection_reuse::rebuilding_a_projection_costs_what_appending_to_it_costs`
— a 2.27 s rebuild against the 2 s generosity floor, on a runner loaded
by the rest of the suite, while the identical commit had passed the same
test on its branch run hours earlier. PR #1146 generalizes the pin: each
leg (append, rebuild) becomes the minimum of three sampled timings, and
the failure message names the binding budget. The sessions below had the
local Formal AI model read the rewritten test.

## Sessions

1. `ses_f20cd37eeffeRXvqLYBf5RF61h` — "in the test
   `rebuilding_a_projection_costs_what_appending_to_it_costs`, what are
   the two legs that get timed, and what budget must the rebuild leg
   stay under?" The model planned `read`, read
   `tests/integration/issue_1106_projection_reuse.rs` in full, and its
   final turn presented the file contents — including the
   `fastest_completion_cost` sampler and the
   `checked_mul(MAXIMUM_RATIO).max(Duration::from_secs(2))` budget line
   the question asked about.
2. `ses_f20cd2564ffeW1cqSsTGDZthBj` — "what does the function
   `fastest_completion_cost` do, how many samples does it take, and
   which sample does it return?" Same genuine read; the returned
   contents contain the `(0..samples).map(...).min()` body, so the
   three-sample minimum the fix rides on is in the session record.
3. `ses_f20cd1608ffeRaSfxhLxNuLIoA` — "quote the line that computes the
   budget from the append cost and the line that compares the rebuild
   cost against it." Same genuine read of the same file.

All three sessions ended `hasError: false` with the file's contents as
the final assistant turn; none produced a prose synthesis beyond the
presented contents, so the mechanical confirmation is the human-side
verification recorded in issue #1145 and the PR: the rewritten test
passes twice back-to-back locally (6.9 s / 7.3 s) and
`cargo fmt --check` is clean.
