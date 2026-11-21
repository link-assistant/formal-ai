# Source transport and release receipt repairs

Layered CI run [44938456935](https://github.com/link-assistant/formal-ai/actions/runs/44938456935)
tested published source `9e945eae3f1f3a03a2b9e3302bdb81000f7820b1` and failed.
Its full-log SHA256 is `32679b18f6b87b086352a6d034d4db9a14c0cf49ec1730ab8925e0b11d07c9a4`.
The following repairs preserve the existing guards and point consumers at their
actual source owners and transport operands.

| Original log lines | Failure | Repair and verification |
| --- | --- | --- |
| 4712–4719, 4935–4957 | Receipt recognizer cannot see the writer hidden inside the extracted manifest shell script | Restore the same writer as the final inline workflow command; unchanged receipt and relocated-stage behavioral suites pass. |
| 7396–7405 | Structured-output VM omits the seed reader prerequisite | Load the actual prerequisite; a focused VM case consumes the actual suite's script list and parses a real record. Native HTTP replay remains CI-only. |
| 16486–16501 | Sparse checkout removes the archived seed reader test operand | Keep exact original bytes as a consumed test fixture. The conversation conservation operand uses the same fixture transport. A real Git sparse-checkout test uses the actual workflow patterns and excludes unrelated history. |
| 16505–16529 | Documentation lookup omits the moved Rust selection owner | Add the maintained owner to the existing source list; the procedure-documentation suite passes. |
| 27009 onward | Packaging assertion reads the former inline source owner | Read the captured manifest helper and retain the actual workflow invocation assertion; packaging and CLI release suites pass. |

All 309 maintained issue cases, the four focused source/transport suites, five
affected JavaScript gates and whitespace checks pass locally. Initial mistaken
test assertions and unsuccessful source lookups remain archived alongside the
corrected checks. No local Rust build or native execution is claimed. Supplied
edits, procedures and conclusions earn zero independent-authorship credit.

The separate failure at lines 26989–27005 remains unresolved: the committed WASM
input receipt predates the worker source extraction and requires an actual build.
Its guard is unchanged. Hosted final-head CI, complete template/practice review
and the wider issue remain unfinished. Evidence is retained under
`experiments/issue-1580/evidence/checks/seed-head-ci-repairs/`.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
