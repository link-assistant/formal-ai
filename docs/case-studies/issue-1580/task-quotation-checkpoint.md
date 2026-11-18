# Task quotation boundaries and rejected context candidate

The input head `a97832237cbb365000dd7296050984e4bdd59ebb` finishes all 23
hosted workflows successfully. Layered CI run 44233930887 completes at
2026-11-18T04:06:29Z; the exact-head records are retained under
`experiments/issue-1580/evidence/ci/declared-language-policy-a9783223/`.
Scope-gated workflow successes do not establish native runtime acceptance.

## Reproduced quotation defect

`splitRequest('Count "salt and pepper" and report the size', 'en')` produces
three tasks before the repair: `Count "salt`, `pepper"`, and `report the size`.
The shared `task-decomposition` grammar declares curly quotes, CJK quotes and
backticks, but omits ASCII double and single quotes. Both runtime masks read
that incomplete list, so conjunctions inside those quotes become task edges.

Add both ASCII pairs to the shared grammar. The JavaScript and Rust masks
honor backslash escaping and distinguish word apostrophes from quotation
delimiters. An opening apostrophe touching the preceding non-CJK alphanumeric character
stays inside a word; a closing apostrophe touching the following non-CJK alphanumeric
character stays inside a word. This preserves unquoted contractions, plural
possessives and contractions inside quoted passages. Mixed-language quote
styles and the existing relation precedence continue to use the same grammar.

Ten new shared cases cover double-quoted aggregation and sequence,
single-quoted aggregation, an unquoted contraction, a quoted contraction,
a plural possessive, escaped double quotes, adjacent Chinese single and double
quotes, and a Russian double-quoted conjunction. Every original input,
expected result and trace in the 15-case fixture is unchanged. The native
fixture reader already iterates all cases; the fixture now contains 25.
The canonical fixture checker remains
`node experiments/issue-1550/parity-fixture.mjs --check`.

The focused regression fails before the repair with seven passing tests and
two failures: the new literal-boundary assertion and the deliberately stale
fixture inventory. The primary error shows the exact incorrect three-part
split. A direct command-driver attempt was refused as non-verbatim and earns
no verification credit; the retained before run executes through the supplied
Python orchestration with the original source and seed restored temporarily,
then restores the candidate bytes in a `finally` block.

Worker and TypeScript projections are regenerated through their maintained
writers. An initial bare `rustfmt --check` uses the wrong style edition and
also finds one long new expression; retain that failure. Format and check
using the repository's Rust 2024 edition. This is source formatting only.
The first focused post-repair run passes all nine executor tests, all 15 adjacent
factual-QA tests and both projection checks with 22 fixture cases. Adding three
multilingual controls initially fails on missing fixture entries, which does not
prove a parser defect. A corrected direct diagnostic, after installing the node
host, uses the actual Chinese connective `并且` inside the literal and reproduces
a three-part split for adjacent single quotes. Exclude CJK neighbors from the
word-apostrophe rule in both runtimes. The diagnostic's initial missing-host
failure and the fixture-only failure are retained separately.
The final 25-case fixture check passes. All nine executor tests and all
15 adjacent factual-QA tests pass in separate runs. The maintained worker and
717-file TypeScript projections, Rust 2024 formatting and whitespace checks pass.

Because this batch changes shared data, run the complete 138-entry local gate
inventory. The initial run records 83 passes, 52 explicit native/CI exclusions
and three failures: stale JavaScript-to-Rust projection, stale embedded seed
notation and the same stale seed in package self-containment. Read and retain
all three failed logs. Regenerate the embedded seed with its maintained writer
and the affected projection through the official single-job translator. The
translator still refuses 49 modules; this regeneration establishes currency,
not full JavaScript/Rust equivalence.

The final complete inventory records **86 passes, 52 explicit exclusions and
zero failures** in 620.949 seconds. Retain both full-run summaries and every
executed gate log. All 86 final compressed gate logs match their recorded raw
hashes. The repair worker finishes in 687.109 seconds, with a 256 MiB Node heap,
one translation job and peak child RSS of 1,041,380 KiB. No local Rust build or
native application execution occurs. The complete final aggregate is read;
archiving and hashing successful detailed logs do not imply literal review of
each detailed log.

Final aggregate SHA-256:
`b150c2758aa950b147a8a9d70371a209e12e1904993d853aa0146b72eac05d90`.
The full outcomes, per-command hashes and archived gate logs are under
`experiments/issue-1580/evidence/checks/task-quotation-boundaries/`.

## Rejected single-passage binding candidate

The finite disposable-VM experiment runs eight requests before and after
a supplied candidate, for 16 observations, with no actual or mocked transport
calls. It retains required child tests and leaves production sources unchanged.
The first harness attempt fails before any request because it assigns a
constant; its complete source and log are retained separately.

The corrected candidate restores context for the original meeting request and
changed concert facts. Withholding child `t0.3` still leaves the root unresolved.
However, only four of eight explicit qualification checks pass. The independent
passage-plus-arithmetic request changes from unresolved to resolved while its
answer remains only `2 + 2 = 4`, omitting the requested passage answer. The
uncovered-subject case also changes to resolved while returning a gap. The
two-passage request already reports resolved with an inadequate answer in both
variants; that pre-existing failure is separate from the candidate regression.
The quoted-conjunction control identifies the separate quotation defect above.

Reject the binding candidate. `runTaskTest` checks whether children resolved
and whether the root has nonempty, permitted-intent text; it does not establish
that a direct parent answer covers each required child result. Correct context
binding therefore also needs executable parent coverage obligations. Neither
the quotation repair nor a successful direct passage answer completes that
broader requirement. Do not make the request atomic or remove child checks.

The complete experiment, 16-line outcome log, initial harness failure and
explicit failed qualification checks are under
`experiments/issue-1580/evidence/checks/passage-routing-probe/`.
Selected outcome/binding/meta-execution review is separate from full literal
review of every nested solver trace in the 305,557-byte artifact.

## Remaining acceptance

No native executable runs locally. Hosted native quotation behavior, parent
coverage, multi-passage context binding and every earlier broad unchecked
criterion remain unfinished. The PR stays draft and unmerged. Supplied scripts,
patches and conclusions receive zero independent Formal AI authorship credit;
the independently accepted reasoning share remains 0/20.

## Publication checks

Code commit `13c2763ce9f7c31d9c36d9efd3e0f023899566e5` contains the
quotation repair and all maintained projections. Final release, naming and size
validation passes all five selected gates after the evidence archive is updated.
The TypeScript projection and whitespace checks pass again. The 05:02 UTC
feedback refresh finds unchanged issue and PR discussion, no inline comments or
reviews, and zero required sub-issues. Main 015a6831 remains merged. All 23
workflows on a9783223 are completed successfully, with empty in-progress and
queued lists. The quotation publication still needs its own hosted CI results.
