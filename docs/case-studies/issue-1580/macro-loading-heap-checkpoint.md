# Query-loading heap versus evaluator memory

The third original producer [run 44794225675](https://github.com/link-assistant/formal-ai/actions/runs/44794225675)
on published `1d7ba479` succeeds and preserves 124 identity-verified producer
payloads. Both original Rust and Actions SARIF reports contain zero findings
and zero warning/error execution notifications. The separate diagnostic query
compiles in 25.6 seconds and evaluates in 66 seconds with 122,534 rows, including
631 unexpanded source positions matching the original informational notes.
This remains positional observation, with per-child semantic coverage open.

The exact manifest proves `--ram=4096` reached CodeQL 2.27.5, but original
`query.stderr` still reports a maximum heap of only 1,800 MB and declines
stage caching. This follows observed heaps of 914 and 1,698 MiB for the earlier
2 and 3 GiB evaluator settings. The [official query-run reference](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/query-run)
documents evaluator RAM and JVM options separately. The next draft supplies
`-J-Xmx3072m` for loading while retaining the 4 GiB evaluator request, two
threads, 512 MiB disk cache, four-minute hosted deadline, compiler warnings as
errors and the original strict SARIF checks. Actual heap allocation and warning
removal require the next hosted producer; local mocks supply no such proof.

The newly added transport assertion fails against the original observer. All
19 security adapter groups and 18 production profile groups pass after the
change, as do the four archive/size/readability gates and whitespace check.
The initial nonexistent profile test-path invocation is retained separately
from the corrected passing command. The observer remains an explicit opt-in.

The same published head's actual [Actionlint job](https://github.com/link-assistant/formal-ai/actions/runs/44794245967/job/136489519709)
executes the general companion checker at original line 223: 60 workflows,
19 queue policies and zero unresolved dynamic values. Original lines 307–315
confirm all 13 controls pass with no skip or cancellation. These hosted results
qualify the companion's admission and execution, while actual publication,
remaining source warnings and all broader issue criteria remain unfinished.
All scripts, edits and assessments were supplied externally and earn zero
independent Formal AI authorship or reasoning credit.
