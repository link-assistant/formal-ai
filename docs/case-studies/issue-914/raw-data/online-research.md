# Issue 914 Online Research

Collected 2026-08-03 for the issue #914 planning pass. The question asked of
the external landscape: which existing components and libraries can help a
Rust system do general problem solving and coding via formal logical
reasoning, without neural networks in the reasoning itself, and which recent
developments confirm or contradict that direction.

## Symbolic Reasoning And Theorem Proving

- *Lean 4* (Apache-2.0)
  <https://lean-lang.org/>
  Interactive theorem prover and full programming language; Mathlib exceeds
  1.5M lines and ~232k theorems, with active funding through the Lean FRO.
  The de-facto target formal language for verified math and code; usable as a
  subprocess "ground truth" checker for statements and synthesized programs.
- *Rocq (formerly Coq)* (LGPL-2.1)
  <https://rocq-prover.org/>
  Dependent-type proof assistant, renamed with v9.0 in March 2025 and active
  since. Largest verified-software legacy (CompCert); extraction to
  OCaml/Haskell supports verified synthesis pipelines.
- *Isabelle* (BSD-3)
  <https://isabelle.in.tum.de/>
  Isabelle2025 improved Sledgehammer with cvc5 Alethe proof reconstruction.
  Sledgehammer is the best existing model of orchestrating many symbolic
  provers — an architectural template for solver dispatch.
- *Z3* (MIT) and *cvc5* (BSD-3)
  <https://github.com/Z3Prover/z3>
  <https://cvc5.github.io/>
  Industrial SMT solvers with official Rust bindings; the workhorses for
  decidable sub-problems in synthesis and verification.
- *Vampire* (BSD-3) and *E prover* (GPL-2+)
  <https://github.com/vprover>
  <https://github.com/eprover/eprover>
  Top first-order automated provers (CASC); Vampire adds arithmetic,
  induction, and higher-order support (CAV 2025 retrospective). Suited to
  heavy reasoning over large translated axiom sets; E's GPL is the licensing
  constraint to note.
- *Soufflé* (UPL-1.0) and *Ascent* (MIT/Apache-2.0)
  <https://souffle-lang.github.io/>
  <https://github.com/s-arash/ascent>
  Datalog engines: Soufflé compiles to parallel C++ and dominates program
  analysis; Ascent embeds Datalog in Rust macros (v0.8.0, 2026, semilattice
  support) — the best-fit in-process inference engine for a Rust core.
- *Scryer Prolog* (BSD-3)
  <https://github.com/mthom/scryer-prolog>
  ISO-conformant WAM Prolog in pure Rust, usable as a library crate; ideal
  for DCG grammars, controlled-language parsing, and classic logic
  programming inside the solver.
- *egg / egglog* (MIT)
  <https://github.com/egraphs-good/egg>
  <https://github.com/egraphs-good/egglog>
  Rust e-graph and equality-saturation libraries (egglog fuses Datalog with
  equality saturation, active into 2026). Natural core machinery for term
  rewriting, algebraic simplification, and rewrite-based synthesis.
- *miniKanren in Rust* (MIT-family)
  <https://github.com/ekzhang/ukanren-rs>
  <https://conf.researchr.org/home/icfp-splash-2025/minikanren-2025>
  Relational programming: relational interpreters enable synthesis-from-
  examples by "running programs backwards". Rust crates exist but are young.

## Natural Language To Formal Language Without Neural Networks

- *Grammatical Framework* (GPL compiler; LGPL/BSD runtime and grammars)
  <https://www.grammaticalframework.org/>
  Type-theoretic multilingual grammar formalism: one abstract syntax, many
  concrete languages, Resource Grammar Library near 40 languages. Ranta's
  Informath line translates mathematical text to and from Lean/Rocq/Agda via
  GF — the single strongest proven non-neural NL↔formal bridge. The C
  runtime (PGF) is callable from Rust.
- *Attempto Controlled English (ACE / APE)* (LGPL)
  <https://github.com/Attempto/APE>
  Controlled English with unambiguous translation to first-order logic and
  DRS, implemented in Prolog; dormant since 2013 but a proven design that
  could be re-hosted on Scryer Prolog; ACE-in-GF bridges it to GF.
- *CCG and Montague/DRT resources* (LGPL / Apache-2.0 / academic)
  <https://github.com/OpenCCG/openccg>
  <https://www.nltk.org/book/ch10.html>
  Combinatory Categorial Grammar and lambda-calculus compositional semantics
  are the textbook blueprints for syntax-transparent NL→logic; the symbolic
  implementations are dormant, so the composition layer would be rebuilt
  natively.
- *Universal Dependencies, Link Grammar, MaltParser* (CC BY-SA / LGPL)
  <https://universaldependencies.org/>
  <https://github.com/opencog/link-grammar>
  UD treebanks (150+ languages) are a symbolic grammar-metadata goldmine for
  the data seed even though state-of-the-art UD parsers are neural;
  Link Grammar remains a maintained rule-based parser.

## Program Synthesis Without Large Language Models

- *Rosette* (BSD-2)
  <https://emina.github.io/rosette/>
  Solver-aided language: symbolic evaluation compiles programs to SMT for
  verify/synthesize/repair; the reference architecture to reimplement over
  Rust Z3 bindings.
- *Sketch / CEGIS*
  <https://people.csail.mit.edu/asolar/>
  The pioneer of hole-based syntax-guided synthesis; the CEGIS loop
  (counterexample-guided inductive synthesis) is the algorithm to adopt.
- *Microsoft PROSE / FlashFill* (proprietary SDK, free use; samples MIT)
  <https://www.microsoft.com/en-us/research/group/prose/>
  Industrial programming-by-example via version-space algebras (ships in
  Excel); reimplementable from the papers for example-driven micro-synthesis.
- *Popper* (MIT) and *ILASP* (proprietary, free academic)
  <https://github.com/logic-and-learning-lab/Popper>
  <https://ilasp.com/>
  Inductive logic programming. Popper ("learning from failures") is active,
  with 2025 results pruning rule search by orders of magnitude — the most
  credible non-neural "learn programs/rules from examples" engine today.
- *DreamCoder → Stitch* (MIT)
  <https://github.com/ellisk42/ec>
  <https://github.com/mlb2251/stitch>
  Wake-sleep library learning; Stitch reimplements the compression phase in
  Rust (`stitch_core` crate) at 1,000-10,000x DreamCoder's speed — a direct
  drop-in for discovering reusable abstractions from solved problems, which
  is exactly the "minimal core + growing method library" concept.

## Knowledge Bases And Data Seeds

- *Wikidata* (CC0)
  <https://www.wikidata.org/>
  ~115M items and over 1.5B statements, multilingual labels, typed
  properties, SPARQL endpoint and dumps; the primary license-safe structured
  seed, already the anchor of this repository's formalization layer.
- *ConceptNet 5* (CC BY-SA 4.0)
  <https://github.com/commonsense/conceptnet5>
  ~21M multilingual common-sense edges (UsedFor, CapableOf) that Wikidata
  lacks; maintenance mode, but the snapshot remains valuable seed data.
- *Cyc / OpenCyc → NextKB* (proprietary / open-license successor)
  <https://cyc.com/>
  <https://qrg.northwestern.edu/nextkb/index.html>
  Forty years of hand-built common sense; OpenCyc was withdrawn in 2017, and
  NextKB is the practical open substitute bundling FrameNet and
  OpenCyc-derived content.
- *WordNet / Open English WordNet* (WordNet license / CC BY 4.0)
  <https://wordnet.princeton.edu/>
  <https://github.com/globalwordnet/english-wordnet>
  Synset lexicon for word-sense grounding; Open English WordNet has active
  yearly releases.
- *FrameNet* (CC research licensing)
  <https://framenet.icsi.berkeley.edu/>
  ~1,200 semantic frames with roles — the "rich metadata" shape needed to
  map verbs to predicate-argument structure; multilingual FrameNets exist,
  including a GF FrameNet grammar.
- *Abstract Meaning Representation* (LDC corpus license)
  <https://amr.isi.edu/>
  Sentence-level meaning graphs; a design reference for meaning
  representations, secondary to GF/controlled-language routes because the
  corpus is fee-licensed and current parsers are neural.

## Rust Crates Relevant To This Repository

- SMT: `z3` / `z3-sys`, official `cvc5-rs`, solver-agnostic `smtlib` and
  `rsmt2` (SMT-LIB over any solver binary).
- Datalog and logic: `ascent`, `crepe`, `datafrog` (the engine behind
  rustc's Polonius), `differential-dataflow` for incremental computation.
- Rewriting: `egg`, `egglog`.
- Prolog and relational: `scryer-prolog` as a library crate; `ukanren`,
  `proto-vulcan`.
- Library learning: `stitch_core`.
- Verifying code the system writes: Verus (<https://github.com/verus-lang/verus>),
  Kani (<https://github.com/model-checking/kani>, used for Rust stdlib
  verification in CI), Creusot, Prusti — a fully symbolic "did the
  synthesized Rust meet its spec?" oracle.
- Parsing infrastructure: `tree-sitter` for code-side parsing; `nlprule`
  (rule-based LanguageTool port) as a rare non-neural Rust NLP crate.

## 2024-2026 Developments In Coding Via Formal Reasoning

- *AlphaProof* (DeepMind): reinforcement learning plus Lean reached IMO 2024
  silver; the Nature paper landed November 2025, and 2025 follow-ups
  resolved several Erdős open problems — every output machine-checked by
  Lean's kernel.
  <https://www.nature.com/articles/s41586-025-09833-y>
- *Formally verified IMO 2025 gold*: Harmonic's Aristotle produced
  Lean-verified gold-medal solutions; open neural provers
  (DeepSeek-Prover-V2, Goedel-Prover-V2, Kimina-Prover) share one
  denominator — a symbolic checker provides all the trust.
- *Verified code generation*: AlphaVerus bootstraps formally verified Rust
  through Verus (<https://arxiv.org/pdf/2412.06176>); AWS runs Kani over the
  Rust standard library
  (<https://rust-lang.github.io/rust-project-goals/2024h2/std-verification.html>).
  The 2025-2026 pattern is a "verification convergence": generation may be
  neural elsewhere, but checking is symbolic everywhere.
- *Symbolic-side advances*: Rocq 9.x modernization, Isabelle2025 prover
  portfolios, Vampire's induction and higher-order support, egglog maturing
  as egg's successor, Popper's 2025 search-pruning results, and
  Stitch/LILO making library learning practical.

**Strategic takeaway.** The ecosystem trend validates this repository's
architecture: every headline system uses a symbolic kernel (Lean, Verus,
SMT) as the arbiter of correctness. A Rust-native stack of Scryer Prolog +
Ascent + egglog + z3/cvc5 bindings + Stitch covers reasoning, rewriting, and
library learning in-process; GF with an ACE-style controlled language is the
strongest proven non-neural NL↔formal bridge; Wikidata plus Open English
WordNet plus FrameNet form the most license-safe data seed; Lean 4 and Verus
serve as external verification oracles for synthesized code.

All external sources above are official project sites, primary
documentation, or peer-reviewed publications; no external source code or
prose was copied into this repository.
