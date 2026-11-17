# Calendar intent completion and second CI round

Pull request: https://github.com/link-assistant/formal-ai/pull/1596

The complete run for `ffd950a29bf2d4aa21992c7637b3baf8e1bcaabe`
finished with 20 successful workflows and one failing workflow. Layered CI
44109502804 reported stale generated TypeScript in four worker modules
(log lines 7566–7568). Its JavaScript suites passed; the pipeline native type
and lint check, source scanner and Security also passed.

Final review of the issue's prose outcome exposed another case beyond the
original four examples: `Experience: 2 years. Schedule: Monday at 9:00.`
and `Our meeting is tomorrow.` still selected event creation. A concrete
anchor alone was insufficient. Eight new shared document controls produced
six failing regressions before the repair (31 passed, 6 failed in 37 tests).

Both runtimes now require one sentence with a seed-declared scheduling act
and its own date/time anchor. Courtesy prefixes, imperative endings, date-led
Chinese/Hindi requests and existing Russian elliptical requests are retained.
Field labels such as `Schedule:` and event nouns in factual statements do not
supply the act. Native table dispatch rechecks this same request gate; event
field parsing reads only the accepted span, which is retained in evidence.
Twenty positive scheduling controls cover existing paraphrases and all five
languages. The reusable sentence splitter preserves decimal and clock dots.
Multiple independent scheduling requests decline composition instead of
arbitrarily choosing one.

Regenerate the TypeScript projections with `node scripts/translate-es.mjs --write`;
its independent `--check` verifies the result. This does not change CI policy.
The complete local gate inventory is required again because shared seed and
source-ledger data changed. Native compilation remains hosted, with no local
Cargo execution. See the retained `ci-round-2-evidence.txt.gz` for actual
commands, the original failing CI log, before/after tests and gate results.

These supplied changes and their procedures are externally authored by Codex,
executed through JavaScript Formal AI, with zero independent authorship or
reasoning credit. The original six-unit score is not expanded retrospectively.

A second negative-control draft also reproduced the factual sentence
`Schedule is Monday at 9:00.`. Seed-defined copula guards prevent its noun
from supplying an act. General creation/addition cues require an event as
their direct object, so `Create a poem about tomorrow` cannot supply a calendar
claim. Four added controls bring the shared inventory to 57 and the focused
suite to 62 tests. The earlier partial gate run was deliberately stopped
before these inputs changed; final verification starts from the finished draft.

Final local verification passes 173 tests across eight focused suites: the
expanded issue suite (62), conversions (10), date offsets (11), claim routing
(24), handler rules (27), routing corpus (3), fixture integrity (2) and Unicode
source witnesses (34). After the final object/copula refinement, all three
affected issue/routing/handler suites were rerun; the other five had already
passed in this batch. The full 138-entry gate run then finished with 86 passing,
52 explicitly CI-only/context-dependent skips and no failures. TypeScript
currency, embedded mirrors, Rust formatting and whitespace checks pass.
`after.jsonl` now contains all 57 actual full-worker probe responses.

Repository policy permits only the existing focused native type/lint check on
pull requests and reserves native tests for main. The registered shared Rust
fixture test has not been executed locally or on this PR. Do not interpret the
JavaScript checks or native compilation as native behavioral execution.
