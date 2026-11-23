# Native semantic boundaries exposed by hosted CI

The current run [45085971679](https://github.com/link-assistant/formal-ai/actions/runs/45085971679)
builds and lints successfully on branch head `b459d0b0`. Its actual merge input is
`c0f4f72d7cbb1ef2cf848b43b6537df6023015af`; producer attempt 1 uploads native
artifact ID `14778573986`. Coding groups 1 and 2 and benchmarks pass. The three
failed groups observed so far contain four semantic assertion failures, rather
than build errors or exhausted budgets:

| Job | Preserved log lines | Failed contract |
| --- | --- | --- |
| [reasoning-1](https://github.com/link-assistant/formal-ai/actions/runs/45085971679/job/137406760782) | 862–871, 895–900 | Synthesis refusal loses consulted source outcomes; publisher ambiguity loses its clarification question. |
| [reasoning-2](https://github.com/link-assistant/formal-ai/actions/runs/45085971679/job/137406760817) | 883–888 | The Chinese documentation request receives an English citation. |
| [interfaces](https://github.com/link-assistant/formal-ai/actions/runs/45085971679/job/137406760874) | 880–889 | Named synthesis refusal loses the required research-trail prefix. |

The question scanner recognized bare URL query delimiters but missed structured
values such as `detail=https://…?…`, so its factual-question filter removed
observed research evidence. URL recognition now also reads assignment values and
Markdown link targets. A separate question on a later line is still classified
and refused. A two-case native regression covers structured URLs and genuine
referent clarification, and is registered in both the complete unit target and
the focused reasoning target.

The original agent-authored question seed stays byte-for-byte unchanged. A
composed extension classifies the general multilingual “do you mean” forms as
requirements; the same necessity trace and one-question budget still apply.
Factual unknowns remain factual. No answer assertion or test timeout is relaxed.

The documentation handler used whole-prompt language detection, allowing Latin
API identifiers to outweigh Chinese prose. Both native and worker handlers now
select the existing prose detector, preserve forced response languages, and keep
the existing fallback when there is no prose. Strengthening the maintained
JavaScript documentation route test reproduces the citation failure before the
fix and passes afterward, including English and Russian forced-language replays.
The original publisher capture, offline replay, ambiguity and routing controls
remain in that test. Its TypeScript twin is regenerated through the maintained
translator. Two added native interpreter lines and one worker line are measured
in the source census; all dispositions and outside-core ceilings stay binding.

Local checks pass: the documentation route, two original-authored-seed/extension
controls, thirteen native-module graph controls, projection currency, edition
2024 formatting, and exact handler/worker census. The actual native regression
execution remains pending the next combined push. The current matrix is still
running, and all remaining actionable failures must be collected before pushing.
Actual full and consumer-only native artifact reruns remain pending as well.

Complete logs, API coordinates, before/after JavaScript output and local check
streams are round-trip hash verified under
`experiments/issue-1580/evidence/checks/native-semantic-boundaries/`. This is a
finite semantic repair with zero supplied-code authorship credit; it does not
complete the broader issue.


The first complete candidate gate run is a clean-source diagnostic: 86 pass, three fail and 54 hosted-only exclusions. The failures identify a new underscore record name, a stale embedded seed copy and a generated status count. Renaming the record through the existing notation policy and regenerating both surfaces repairs all three focused gates; both original authored-seed controls still pass. A second complete clean-source run remains required before pushing.
