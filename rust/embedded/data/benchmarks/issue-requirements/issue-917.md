**Problem.** The vision requires truly solving translation between
languages — natural and formal (issue #914). Today `src/translation/`
round-trips four natural languages through the semantic meta language,
and #890 projects solved proofs into Rust and Python, but formal
languages are not yet first-class translation targets: there is no
general path from a natural statement to a logic formula, proof
obligation, or program specification and back.

**Approach.** Make formal languages concrete syntaxes of the existing
meta language: one abstract meaning layer, many projections, following
the abstract/concrete split proven by Grammatical Framework (whose
Informath line translates mathematical text to Lean and back) and the
unambiguous-entry design of Attempto Controlled English — as design
references, implemented natively on the link substrate. Grammar and
lexicon metadata live in the seed so adding a language stays a data
change (rule 5 of VISION.md's rule shapes). Round-trip survival (#526)
extends to natural-to-formal-to-natural pairs.

**Existing components.** `src/translation/` and its meta language;
`src/proof_program.rs` plus `data/seed/proof-program-templates.lino`
(#890); `src/intent_formalization.rs` P/Q anchoring; NSM primes in
`src/summarization/`; Grammatical Framework, ACE/APE, Universal
Dependencies, Open English WordNet, and FrameNet as external references
and license-safe metadata sources (see the issue #914 online research).

**Acceptance criteria.**
- A natural-language statement in any seed language translates to at
  least one formal target (logic statement, proof obligation, or program
  specification) and back, surviving the round trip.
- The formal targets are seed-defined projections of the meta language,
  not per-pair translators.
- The #526 round-trip suite extends to the new pairs and passes.
- Depends on E69 only where translation output is executed as code.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

