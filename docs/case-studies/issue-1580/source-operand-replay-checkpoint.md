# Source operand replay checkpoint

This checkpoint extends the existing hosted native regression target without a
local Rust build. The main-only synthesis failures remain acceptance failures
until the exact published tests execute successfully.

The real JavaScript worker recomputes the renumbered egg-sale problem as 36 for
24 total eggs and a price of 3, and as 72 for 30 total eggs and a price of 4.
Both cases retain the actual remainder and evaluated expression in their
evidence. The focused production-composition test now passes all four tests;
the related executor file passes all 25 tests. All four affected JavaScript
gates and whitespace checks pass. Complete payloads and hashes are retained in
`experiments/issue-1580/evidence/checks/source-operand-replays/validation/`.

Two new Rust tests replay these two source-operand cases and the three-instrument
membership count through the real offline `UniversalSolver`. They assert the
exact result and execution/composition evidence, including the membership-source
trace. They use the existing isolated native target, which hosted CI already
executes. Rust formatting passes; these new tests have not executed yet.

The initial probe also preserves two unresolved greeting-plus-definition
requests and the browser's explicit unverified count response. These are not
successful native replays or evidence that the wider synthesis criteria are
complete. The browser currently has no executor for its derived count program.

The preceding algebra correction is published at `acb9fa51`. All 23 workflows
have now passed, including the actual native algebra test. This checkpoint remains
a draft until the new source-operand tests execute on the next published head. The wider issue remains unfinished, with zero independent
authorship credit and 0/20 independently accepted reasoning units.


Java comparison also fully reads the 293-line historical original POM and the
230-line retained runner-image guide. These 523 supplementary source lines are
recorded separately from the continuous 111-page prefix to avoid double counts
when the forward page review reaches them. The original source identities,
whole payloads and per-file transfer decisions are preserved. A structured
inspection parses all 882 retained Node release records, including prereleases
and optional field absences; it does not claim a literal read of the remaining
catalog lines or current runtime compatibility. The runner-label check passes
across 74 files and 207 pinned labels. Full template applicability stays open.
