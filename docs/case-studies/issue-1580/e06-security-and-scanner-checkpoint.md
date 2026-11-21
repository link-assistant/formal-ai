# Successful CodeQL producer and scanner boundary repairs

Published `e06d1f18ef76745082a9c473831fa005ddc2f95c` has 27 terminal
workflows: 26 success, one skip and no failures. Pipeline, all eight native
families, registry-script checks and Layered CI pass. The ordinary PR Security
workflow skips the heavyweight producer; it is not credited as its execution.

The separate [manual Security producer 44879033589](https://github.com/link-assistant/formal-ai/actions/runs/44879033589)
passes on that exact source. Actual CodeQL 2.27.5 completes the included-string
query within the unchanged four-minute step: compilation 36.7 seconds and
evaluation 176 seconds. The inherited budget is capped at 8192 MiB; the CLI
resolves a 6245 MiB heap. The separate macro profile retains its 4096 MiB budget
and 2607 MiB heap. This qualifies the memory candidate on this producer run,
without increasing its deadline or dropping result rows and columns.

All 148 retained original producer files verify by archive and raw SHA256.
The included-string result has 2180 rows and all nine columns; the macro-context
result has 122579 rows and eight columns. Both original standard SARIF reports
have zero security results, and the strict unresolved-diagnostic gate passes.
The Rust report still records 631 unexpanded macro positions. Retained payloads
and successful extraction are not complete per-child semantic security review.

Local Secretlint 14 adoption removes the deprecated external preset while
retaining masked output, disabled profiling, exact engine identities, NUL paths,
bounded compressed scans and workflow-command suppression. Actual-engine
positive/negative fixtures and maintained regression tests pass. The budget
wrapper now emits its own annotations and completion marker on fresh lines
after unterminated child stdout or stderr. All three focused boundary cases
fail with the old wrapper and pass with the repair; the maintained template
entry point passes 116 cases. These local changes still need next-head CI.

Current JavaScript review covers all 30 changed files and two removals at
8e72538d, with 107 focused assertions. PHP review covers all 19 changed files
at b61e3347, including all 4774 lock lines, with 43 actual PHP tests and 651
assertions under a 128 MiB limit. Its current manifest validates with one
Composer advisory about the intentional version field; strict validation exits
1 and is retained as such. The PHP release repairs are applied upstream at
this pin. Full unchanged/historical template and practice acceptance remains
open; these focused comparisons do not close it.

The source warnings, dependency migrations, 631 macro-position semantics, live
Pages/runtime/model delivery, all original main-workflow acceptance and complete
PR review remain unfinished. No full local native build or local CodeQL
evaluation was performed. Supplied commands and code earn zero independent
authorship credit. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
remains draft and unmerged.

Final local publication checks initially rejected 113 abbreviated archive names
and a 4146-line producer manifest. Expanded names and a 150-line lossless
manifest pass both unchanged gates; all 148 payload hashes still verify. The
other five touched gates pass. Actual Secretlint 14 scans 568 changed files,
including 542 compressed archives, with empty stderr. Failed and passing
observations are retained separately. These are local checks, pending new-head CI.
