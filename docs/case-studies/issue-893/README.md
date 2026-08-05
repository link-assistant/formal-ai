# Issue 893 Case Study

Issue [#893](https://github.com/link-assistant/formal-ai/issues/893) (parent
[#710](https://github.com/link-assistant/formal-ai/issues/710)) records the
audit verdict *still-broken* for the part of issue
[#563](https://github.com/link-assistant/formal-ai/issues/563) that the file and
folder summarizers never covered. #563 did not only ask for
`summarize_repository_file`. It asked for a *protocol*: take two random
repository files, check their summaries, generalize what went wrong, take two
more, and keep going until the result is stable on files nobody optimized for —
at a quality bar of at least 80%. The pipeline had the recursion, the exact
captures and the determinism. Nothing sampled random files, nothing iterated,
and no metric existed to be 80% *of*.

## 1. Collected Data

- Issue body, labels and comments: [`raw-data/issue-893.json`](raw-data/issue-893.json).
- **Published criteria**, as the CLI prints them:
  [`raw-data/published-criteria.txt`](raw-data/published-criteria.txt)
  (`formal-ai summarization criteria`).
- **Protocol run** — the run the committed baseline records:
  [`raw-data/protocol-run.log`](raw-data/protocol-run.log). Seed 563, three
  iterations, six files, `53/53` criteria, stabilized before the bound, two
  Markdown embedded grammar blocks across two files.
- **Wide sweep** — [`raw-data/wide-sweep.log`](raw-data/wide-sweep.log). The
  stability loop stops early by design, so a separate sweep scored the first 600
  files of the same seeded permutation over a 10,560-file corpus:
  **4964/4968 criteria = 99%**, four failures, all `compression`.
- **Failing files**, re-run individually with their evidence:
  [`raw-data/failing-files.txt`](raw-data/failing-files.txt).

## 2. Requirements

Per-requirement rows live in [`requirements.md`](requirements.md) and in the
`Issue #893` section of [`REQUIREMENTS.md`](../../../REQUIREMENTS.md)
(R893-1 … R893-5).

## 3. Root Cause

The summarizer was not the gap; the *measurement* was. Reading
`src/summarization/` before writing anything, three things were missing:

1. **No sampling.** Every existing test named its own file. A summarizer that
   only ever sees files chosen by the person who wrote the summarizer has not
   been validated on "random repository files" in any sense.
2. **No iteration.** There was no loop, so there was no notion of the result
   *stabilizing* — and therefore no way to distinguish "we checked enough" from
   "we stopped".
3. **No metric.** "At least 80%" was unenforceable because nothing defined the
   denominator. A percentage needs a published list of criteria and an explicit
   rule for what happens when a criterion cannot apply to a given file.

The third is the one that quietly breaks quality gates. If an inapplicable
criterion is scored as a pass, a file the summarizer barely understands scores
*higher* than one it handles well, because more of its criteria fall away into
free points.

## 4. Implemented Design

- **Protocol** — [`src/summarization/validation.rs`](../../../src/summarization/validation.rs).
  `SamplingProtocol` sorts and de-duplicates the corpus paths, then permutes
  them with a seeded `splitmix64` Fisher-Yates shuffle. The draw depends on the
  seed and the set of paths only — hand the same files over in a different
  order and the same files come back — and it is a permutation, so a run never
  scores the same file twice. Defaults: `seed 563`, `files_per_iteration 2`,
  `max_iterations 24`, `stability_window 3`, `stability_tolerance_percent 5`.
- **Metric** — ten criteria, published by name and description, scored as an
  exact integer `passed/applicable` ratio. Percentages are floored, so 79.9%
  gates as 79%; an empty score is 0, never a vacuous 100. A criterion that
  cannot apply to a file is dropped from that file's denominator rather than
  counted as a pass. Scores are micro-averaged over criteria, never averaged
  over per-file averages.
- **Ratchet** — `QUALITY_RATCHET_PERCENT = 80`, `ratchet_violations`, and the
  committed [`data/summarization/quality-baseline.lino`](../../../data/summarization/quality-baseline.lino).
- **Operator surface** — [`src/cli_summarization.rs`](../../../src/cli_summarization.rs):
  `formal-ai summarization criteria | validate [--append] | ratchet`.
- **Independent oracle** — the `embedded_grammar_recursion` criterion counts
  fenced blocks with `fenced_block_languages`, a CommonMark fence scanner
  written against the spec rather than against the summarizer, so the
  implementation cannot grade itself. A run may not declare stability until at
  least one embedded grammar block has actually been exercised, and the ratchet
  rejects a run that recorded none.

### Published criteria

| Criterion | What it checks |
| --- | --- |
| `identity_names_path` | The summary names the file it summarizes. |
| `format_declared` | The summary names the detected file format. |
| `size_reported` | The summary reports the file's line and byte counts. |
| `content_retained` | A file with content yields retained content statements in the summary. |
| `content_grounded` | Every identifier-shaped token in the summary occurs in the file's path or content. |
| `compression` | The summary is shorter than the file it summarizes (files ≥ 400 bytes). |
| `embedded_grammar_recursion` | Every Markdown fenced block is recursively formalized and its language is named in the summary. |
| `meta_language_evidence` | A valid meta-language parse is reported with its label and syntax-link count. |
| `determinism` | Summarizing the same file twice returns byte-identical output. |
| `mode_ladder` | Short, Standard and Full summaries grow monotonically with the mode ladder. |

### Why the enforced floor is 80%, not the measured 100%

The corpus is every Git-tracked file, so it changes with every commit and the
seeded draw lands on different files — the same seed drew six different files
before and after this branch added its own. Pinning the enforced floor to a
lucky 100% run would turn an unlucky-but-honest draw into a red build: at four
failures per 600 files, a six-file sample hits one about 4% of the time. So the
baseline records two separate numbers. `percent` is what the committed run
measured (100). `ratchet_percent` is what the ratchet enforces (80, the
published minimum); it may only ever be raised, and raising it is a deliberate,
reviewed edit backed by a sweep — not an automatic consequence of one good
sample.

## 5. Prior Art And Existing Components

- `src/statement_audit/repository.rs` — `RepositoryCorpus::from_repository`
  already loads the Git-tracked corpus; the protocol samples from it rather
  than walking the filesystem itself.
- `src/summarization/file.rs` (issue #563) — `formalize_repository_file` and
  `RepositoryFileFormalization::summary` are the production path every sampled
  file goes through. No test double, no reimplementation.
- `src/cli_benchmark.rs` + `data/benchmarks/external-results.lino` — the
  committed-artifact-plus-CLI-subcommand shape reused for the baseline.
- `tests/unit/specification/equation_corpus.rs` (issue #891) — the
  floor-and-never-lower ratchet shape; the difference here is that the corpus is
  resampled rather than fixed, which is what forced the split between the
  recorded percent and the enforced floor.
- `crate::links_format::push_lino_node` — the baseline is rendered with the
  repository's own Links Notation writer, not hand-rolled quoting.

## 6. Verification

```sh
cargo test --test unit issue_893 -- --nocapture
formal-ai summarization criteria
formal-ai summarization ratchet
```

```text
seed 563 — 3 iteration(s), 53 of 53 criteria passed (100%)
stabilized before the iteration bound; 2 Markdown embedded grammar block(s) across 2 file(s)
summarization quality ratchet holds: 100% measured against a 80% minimum and a
committed ratchet of 80% (last recorded run: 100%)
```

### What the sweep found

Over the first 600 files of the seeded permutation: **4964/4968 = 99%**.

| Failing criterion | Count | What it is |
| --- | --- | --- |
| `compression` | 4 | A file just above the 400-byte floor whose structured summary (path, format, size, retained content) costs a few bytes more than the file itself — e.g. `summary_bytes=460 file_bytes=452`. |
| `content_grounded` | 0 in this sample | But it does bite: `docs/case-studies/issue-492/README.md` scores 87% because the summary emits `https://img.shields.io/badge/crates.io--orange` for a source that reads `https://img.shields.io/badge/crates.io-<version>-orange?logo=rust`. |

That second one is a real defect, not a checker artefact, and the metric is what
surfaced it. `strip_inline_code_and_html` in `src/summarization/markdown.rs`
treats any `<`…`>` as an HTML tag, including inside an inline code span, so a
placeholder such as `<version>` is deleted from text the summary then quotes as
if it were verbatim. Fixing the Markdown normalizer is a change to the
summarizer rather than to the validation protocol this issue asks for, so it is
recorded here with its root cause instead of being tuned out of the metric.

Grounding is checked against the file with its code-span backticks unwrapped:
a summary that renders `` `Topic`/`Short` `` as `Topic/Short` quoted the file
faithfully, and penalizing correct markup removal would have hidden the
`<version>` case among eight false positives. Dropped or invented text still
fails.

### Reproducing

```sh
cargo run --release --all-features --example issue_893_measure   # protocol + 600-file sweep
cp experiments/issue_893_failures.rs examples/ && \
  cargo run --all-features --example issue_893_failures -- <path>...   # per-file evidence
formal-ai summarization validate --append                         # rewrite the baseline
```

Raise `ratchet_percent` in the baseline whenever a sweep justifies it; never
lower it.
