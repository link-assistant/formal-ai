# Issue 890 Case Study

Issue [#890](https://github.com/link-assistant/formal-ai/issues/890) reported
that Formal AI could solve the Russian hidden-integer interval from issue #403,
but could not translate that formal proof into a requested programming
language. The implemented path now solves the prompt, preserves a
language-neutral proof meaning, renders complete Rust and Python programs, and
executes both programs to the same witness.

## 1. Collected Data

The `raw-data/github/` directory contains snapshots of the issue, issue
comments, prepared PR, all three GitHub PR feedback channels, and related issues
#403 and #526. `raw-data/online-research.md` records official LLVM, Rust, and
Python sources plus repository prior art. The issue and discussion contained no
screenshots or image attachments to download.

## 2. Requirements

The complete matrix is in `requirements.md` and mirrored in root
`REQUIREMENTS.md` as R890-1 through R890-7. The four issue acceptance criteria
each have a named unit/integration regression. Native and browser whole-task
tests, documentation traceability, release metadata, and real Agent CLI
authorship are recorded separately rather than hidden inside those four rows.

## 3. Reproduction And Root Cause

Before the implementation, this sequence failed:

```text
solve:     Я загадал число больше 1 но меньше 3. что это за число?
proof:     x > 1 and x < 3 is satisfiable
translate: Translate `x > 1 and x < 3 is satisfiable` to Rust
actual:    intent translate_en_to_en; returns the original statement
expected:  intent translate_proof_to_rust; returns an executable program
```

The number-constraint handler built a solver-specific statement and immediately
rendered localized prose. There was no production proof value to carry through
translation. Separately, `CodeMeaning` only represented a simple addition
function or unformalized source. Consequently the generic translation handler
treated the formal statement as ordinary English, and its command gate missed
head-final Hindi/Chinese forms even when a programming target was present.

## 4. Implemented Design

`src/proof_program.rs` introduces `FormalProof::IntegerInterval`, which owns the
semantic bounds, inclusivity flags, satisfiability decision, and witness. The
number solver publishes its canonical statement while retaining the existing
decision statement for the linear-proof engine.

`CodeMeaning::FormalProof` then makes proof translation use the same general
pipeline as other code: formalize one source meaning, render any supported
target. The Rust and Python projections in
`data/seed/proof-program-templates.lino` emit complete programs that assert the
bounds before printing `2`; the renderer only binds semantic proof fields and
never reparses localized prose. Target recognition uses the programming
catalog, and script-aware alias matching lets Chinese text adjoin `Rust`
without weakening ASCII substring boundaries.

The browser worker mirrors the proof formalizer, slug, slot expansion, and
routing while loading the same presentation seed. This is a behavioral feature
rather than a visual UI change, so the evidence is the Playwright interaction
test rather than before/after screenshots.

## 5. Verification

The focused native suite covers:

- proof representation independent of presentation;
- one solved proof through Rust and Python via the same meaning link;
- `rustc` compilation and execution plus `python3` execution;
- the entire registered natural-language set (`en`, `ru`, `hi`, `zh`); and
- the complete solve-to-translate-to-execute workflow.

`tests/e2e/tests/issue-890.spec.js` exercises the same behavior in a real local
browser. `experiments/issue_890_agent_cli.sh` boots the real
OpenAI-compatible server and drives the installed Agent CLI, then compares its
documentation leaf byte-for-byte. The captured session is
`ses_03b44e557ffeSQeAuCYzxfc3BR`; its raw evidence is under
`agent-cli-evidence/`.

The initial CI run correctly caught the red reproducer and also reported the
then-missing changelog fragment. The final implementation includes the passing
regressions and a minor release fragment.
