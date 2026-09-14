# Issue 923 Online Research

## Equality Saturation

- [egg 0.11 `Runner`](https://docs.rs/egg/0.11.0/egg/struct.Runner.html)
  documents the iteration, node, and time limits available for bounded e-graph
  execution.
- [Pinned egg math source](https://github.com/egraphs-good/egg/blob/2f31b28e3f9d78e02273b6c6d4201b5b0720b343/tests/math.rs)
  is the exact upstream declaration source mechanically adapted by the
  benchmark harness.
- [Pinned egg MIT license](https://raw.githubusercontent.com/egraphs-good/egg/2f31b28e3f9d78e02273b6c6d4201b5b0720b343/LICENSE)
  covers the dependency and benchmark source revision.

## Rule Inference

- [Pinned Ascent transitive-closure example](https://github.com/s-arash/ascent/blob/cf5e9a87525bb95268cf6680a59882264b0fe0de/ascent/examples/transitive_graph_closure.rs)
  supplies executable rules and five asserted consequences at the exact
  revision consumed by the harness.
- [Pinned Ascent MIT license](https://raw.githubusercontent.com/s-arash/ascent/cf5e9a87525bb95268cf6680a59882264b0fe0de/LICENSE)
  permits use of that example. The payload is downloaded on test, not vendored.
- [Ascent language guide](https://s-arash.github.io/ascent/) documents Ascent as
  a Datalog language embedded in Rust. The production evaluator uses the same
  positive least-fixed-point model but is independently implemented in-tree,
  avoiding a second runtime dependency.

## Repository Prior Art

- Issue #914 and PR #915 surveyed egg/egglog, Ascent/Scryer, and SMT bindings,
  and made E76 the breadth-and-measurement work item implemented here.
- Issue #698 and merged PR #816 established the external harness rule: fetch a
  pinned real upstream payload, grade by its actual criterion, and record an
  honest passed/total score rather than substitute a local proxy.
- `src/proof_engine/decision/{boolean,linear,sat}.rs` supplies the existing
  decision-boundary and structured-certificate conventions preserved by both
  new procedures.

No source is used for neural training or distillation, no model output enters
the repository, and no upstream payload is copied into a canonical data
directory. The source-review form for training artifacts is therefore not
applicable; immutable revisions and license evidence are recorded in
`data/benchmarks/LICENSES.md` instead.
