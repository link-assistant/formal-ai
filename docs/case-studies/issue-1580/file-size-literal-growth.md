# Literal Git filenames in size-warning ownership

The Rust template comparison requires separate size policies, directory-component
pruning and escaped annotation properties. Formal AI already preserves its own
1000-line Rust / 1500-line maintained-text policy, directory pruning, NUL Git file
inventory and fatal traversal/read errors. The template's whitespace changed-file
list and ignored traversal/read errors are weaker controls and are not adopted.

The comparison exposed a defect in Formal AI's separate warning-growth filter.
Its newline Git numstat output quotes tabs, newlines, backslashes and non-ASCII
filenames. Comparing that quoted text with literal traversed paths silently loses
their size warnings. A real two-commit Git fixture grows five files and a renamed
file from 900 to 901 lines: unchanged production prints only two of six expected
annotations. Three focused regressions fail before any production edit.

## Repair and checks

Both maintained implementations now request NUL numstat records and retain the
literal path after the first two tab-separated counts. A rename's separate old
and new names are consumed together; warning ownership follows the destination.
Binary, unchanged and shrinking entries retain their existing selection behavior.
Counts use the shared exact integer domain. Malformed or incomplete observations
return unknown and conservatively print every measured size warning.

FORMAL_AI_FILE_SIZE_DEBUG=1 explains that fallback; diagnostics are off by default.
The existing escaped annotation writer and log neutralization retain ownership,
including a filename containing a newline and a workflow-command-shaped suffix.

[The focused regression](../../../experiments/issue-1580/file-size-literal-growth.test.mjs)
passes four cases: literal records and growing renames, incomplete inventory
refusal, actual Git annotation ownership, and a partial Git observation with
default-off diagnostics. Its actual fixture preserves unchanged and shrinking
warning-band files without adding annotations for them. All 22 maintained twin
checks pass. All eleven affected JavaScript gates, Rust formatting and whitespace
checks pass. The maintained twin
suite imports these regressions, and the existing native workflow-contract
module includes the two new Rust parser cases.

An initial conversion of the old JavaScript fixture used a NUL escape immediately
before a digit, which JavaScript rejected as an octal escape. The fixture now uses
the unambiguous hexadecimal escape; that failed test draft remains preserved.
No native build runs locally. Hosted Rust execution and this repair's latest-head
qualification remain pending.

## Evidence and limits

The [sealed local receipt](../../../experiments/issue-1580/evidence/checks/file-size-literal-growth-20261123/manifest.json)
retains the unchanged before source, all focused checks, failed draft, source
snapshots and the pinned template comparison. Every archived member is read back
and hash checked. This fixes observed warning ownership; the repository's larger
strict-style debt and every other unmet issue #1580 criterion remain open.
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
Supplied edits and conclusions earn zero independent authorship credit.
