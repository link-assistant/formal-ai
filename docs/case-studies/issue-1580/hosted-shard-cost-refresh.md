# Complete hosted shard cost refresh

Layered CI run [45240466225](https://github.com/link-assistant/formal-ai/actions/runs/45240466225)
passed on source e46254ac53c360e4c45652fa7abd63fb0d9ecad7. Its tested merge
commit was 158c4151a2c66ecc5871d0370f11e19ef5e1dd9f. Passing functional
assertions did not satisfy the runtime warning criterion: groups 9 and 7
crossed the unchanged 270-second warning threshold. Their runner durations were
306.819501889 and 273.048657371 seconds.

The previous dialogue split ran successfully: the original, second and third
owners took 47.474293328, 36.605133486 and 124.65305008 seconds. All 432
maintained observation cases also passed in that run. These observations qualify
those source identities; they do not qualify the subsequent scheduling repair.

## Reproduction and repair

The capture retains every queued/completed file from all nine hosted jobs:
922 distinct files, original job identifiers, exact durations and original log
line numbers. The complete log SHA-256 is
a0892fbafe15c724eba662993c4d1db18889731e04b06c7d9030182d97750df1.
The fixture preserves both actual warnings, the run response, tested merge
identity and the complete prior scheduling floor.

Run `node --test --test-concurrency=1 experiments/issue-1580/hosted-javascript-scheduling.test.mjs`.
Before the repair, ten historical cases pass and the new case fails. The prior
plan replays the new group 9 at 323.164620067 seconds including the existing
30-second runner allowance.

The finite reserve experiment evaluates thirteen candidates across all eleven
retained cohorts. Updating weights from every complete observation with the
existing 65% reserve fits every cohort; no previous scheduling floor is lowered.
The new cohort's slowest replay is 261.82829702 seconds including the allowance.
All historical cohorts remain below 270 seconds under their existing models.
The first reserve-only draft passed those duration checks but failed the
existing dialogue-owner separation regression. The corrected plan gives each
split owner the same conservative 344.5-second family scheduling bound: the
sum of the three observed owners with the existing 65% reserve. That bound
keeps the owners on three distinct shards; it is not a measured child runtime.
The failed intermediate observation remains retained.

These are modeled durations, not a guarantee of future runner speed.

The deployed nine shards, four workers, 270-second warning, 360-second deadline
and 30-second modeled allowance remain unchanged. File coverage and the original
failed observations from earlier cohorts remain in the regression suite.
The capture and finite probe are reusable under `experiments/issue-1580/`.

## Verification boundaries

The corrected draft is checked by all eleven scheduling regressions and all
four dialogue ownership cases. All 143 local gate entries finish: 89 pass,
54 explicitly skip and none fail. The maintained admission entry also passes
all 184 cases. Exact-source hosted qualification remains pending.
This change does not complete broader template adoption, production delivery,
upstream dependency work or the original issue. PR #1581 remains draft and unmerged.
