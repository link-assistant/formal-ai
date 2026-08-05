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
  [`raw-data/protocol-run.log`](raw-data/protocol-run.log). Seed 563, 12
  iterations, 24 files, `197/197` criteria = 100%, stabilized at the minimum
  sample and before the 24 iteration bound, one Markdown embedded grammar
  block drawn into iteration 0 by the stratified draw.
- **Wide sweep** — [`raw-data/wide-sweep.log`](raw-data/wide-sweep.log). The
  stability loop stops early by design, so a separate sweep scored the first 600
  files of the same seeded permutation over a 10,654-file corpus:
  **4983/4989 criteria = 99%**, six failures, all `compression`.
- **Failing files**, re-run individually with their evidence:
  [`raw-data/failing-files.txt`](raw-data/failing-files.txt) — the six sweep
  failures, plus the three `content_grounded` files below re-run by name.

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

- **Protocol** — [`src/summarization/validation/`](../../../src/summarization/validation):
  `mod.rs` (the loop, the scores and the reports), `sampling.rs` (the seeded,
  stratified draw), `criteria.rs` (the checks and the CommonMark oracle), and
  `baseline.rs` (the committed baseline and the ratchet). Four files rather than
  one because the repository fails any Rust file past a thousand lines — a gate
  this module hit at 1150.
  `SamplingProtocol` sorts and de-duplicates the corpus paths, then permutes
  them with a seeded `splitmix64` Fisher-Yates shuffle. The draw depends on the
  seed and the set of paths only — hand the same files over in a different
  order and the same files come back — and it is a permutation, so a run never
  scores the same file twice. Defaults: `seed 563`, `files_per_iteration 2`,
  `max_iterations 24`, `minimum_iterations 12`, `stability_window 3`,
  `stability_tolerance_percent 5`.
- **Minimum sample** — the stability window alone would stop a healthy run after
  three iterations, six files. Three perfect iterations over six files is not
  evidence about a corpus of ten thousand, and a CI gate that only ever looks at
  six files cannot notice a regression, so `minimum_iterations` holds a run to
  half the bound — twenty-four files — before the window is allowed to stop it.
  A corpus too small to supply that many is held to what it has, so a
  twelve-file fixture still terminates.
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
- **Stratified draw** — `SamplingProtocol::stratified_sampling_order` computes
  the seeded permutation and then promotes the first fence-carrying Markdown
  file to the front, so iteration 0 always reaches the recursive case. Every
  other file keeps its seeded position; the result is still a permutation and
  still a pure function of the seed and the file set. See
  "[Why the draw is stratified](#why-the-draw-is-stratified)".

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

The right-hand column is not written twice. R379 forbids hardcoded natural
language in `src/`, so every sentence this protocol prints — each criterion
description above, each ratchet-refusal sentence, each line of the report — lives
in [`data/seed/multilingual-responses-summarization-quality.lino`](../../../data/seed/multilingual-responses-summarization-quality.lino)
and is looked up by intent (`summarization_criterion_compression`, …) through
`quality_sentence`, the same `OnceLock`-indexed pattern `src/thinking_prose.rs`
uses. The criterion *names* stay language-neutral, so the committed baseline and
the report parser never read prose; adding a language means adding records to the
seed rather than editing Rust. `issue_893_summarization_validation` asserts the
lookup really resolved by rejecting a description that renders as its own intent.

### Why the enforced floor is 80%, not the measured percent

The corpus is every Git-tracked file, so it changes with every commit and the
seeded draw lands on different files. This is not hypothetical: the run recorded
before merging `main` into this branch drew six files and scored `53/53` = 100%;
the run recorded after the merge drew forty-four and scored `364/365` = 99%.
Same seed, same code, different corpus. Had the floor been pinned to that first
lucky 100%, merging `main` would have turned an honest draw into a red build. So
the baseline records two separate numbers. `percent` is what the committed run
measured (100, on the 24 files this draw reaches; the same protocol scored 99%
one merge earlier, and the 600-file sweep below scores 99% on the same
permutation — which is why no single run, perfect or not, is an argument for
moving the floor). `ratchet_percent` is what the ratchet
enforces (80, the
published minimum); it may only ever be raised, and raising it is a deliberate,
reviewed edit backed by a sweep — not an automatic consequence of one good
sample.

### Why the draw is stratified

The first version of the protocol drew uniformly and simply refused to certify a
run that never reached an embedded grammar. That is the right *rule* — a run
that never exercised the recursive case has not validated it — but paired with a
uniform draw it made requirement (d) a coin flip. Fenced Markdown is a small
minority of a 10,654-file repository, and the bound allows
`24 x 2 = 48` files. CI proved it: run
[30969709384](https://github.com/link-assistant/formal-ai/actions/runs/30969709384)
(commit `ca1412d0`, log in
[`raw-data/ci-fail-uniform-draw.log`](raw-data/ci-fail-uniform-draw.log)) walked
all 24 iterations at essentially 100% and then failed:

```text
Error: "summarization quality ratchet violated:
no Markdown embedded grammar block was exercised: the run never reached the recursive case the protocol exists to cover"
```

Nothing was wrong with the summaries. The draw simply never offered the checker
a file it could apply the criterion to, and the next commit passed only because
its draw happened to land on one at iteration 12.

There were three ways out and only one of them is honest. Raising
`max_iterations` until it "usually" works trades a red build for a slow one and
still leaves the guarantee probabilistic. Dropping the rule — certifying runs
that never reached the recursive case — deletes requirement (d) rather than
satisfying it. What the protocol does instead is *stratify*: the corpus is
partitioned into files that can be scored on recursion and files that cannot,
and one file from the first stratum is drawn into iteration 0. Stratified
sampling is the standard answer to exactly this problem — a stratum too rare for
a uniform draw of the affordable size to represent — and it costs the draw one
promoted file, not its randomness. The other 10,653 keep their seeded order, as
`issue_893_markdown_embedded_grammars_run_through_the_production_summarizer`
asserts by comparing the uniform and stratified orders element for element. That
test also runs four different seeds over a 413-file corpus with a single fenced
Markdown file in it; before the change, seeds that missed it failed the run.

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
seed 563 — 12 iteration(s), 197 of 197 criteria passed (100%)
stabilized before the iteration bound; 1 Markdown embedded grammar block(s) across 1 file(s)
summarization quality ratchet holds: 100% measured against a 80% minimum and a
committed ratchet of 80% (last recorded run: 100%)
```

This draw happens to score 100%, and that is exactly the situation the two-number
baseline exists for: the floor stays at 80 rather than following the sample up.
The 600-file sweep below, over the same seeded permutation, still finds six
failures — so a perfect 24-file draw is a fact about which files the seed
reached, not evidence that the summarizer is defect-free.

The run stops at 12 iterations rather than 3 because the window is not allowed
to end a run that has only seen six files, and it reaches an embedded grammar in
iteration 0 rather than by luck because the draw is stratified. Neither number
is a tuning knob found by trying values until the build went green: 12 is half
the bound, and the stratified promotion is one file.

### The ratchet in CI

`.github/workflows/summarization-ratchet.yml` re-measures on every pull request.
An early green run under the uniform draw
([30971344229](https://github.com/link-assistant/formal-ai/actions/runs/30971344229),
log kept in [`raw-data/ci-run.log`](raw-data/ci-run.log)) is itself evidence for
the two-number design:

```text
seed 563 — 13 iteration(s), 212 of 213 criteria passed (99%)
stabilized before the iteration bound; 2 Markdown embedded grammar block(s) across 1 file(s)
summarization quality ratchet holds: 99% measured against a 80% minimum and a committed ratchet of 80% (last recorded run: 99%)
```

Thirteen iterations and 213 criteria, where the local run at the same code saw
22 and 365 — same seed, two commits apart. Only a floor that does not chase the
last measurement survives that. The measured percent moves with the corpus; the
enforced floor does not.

Under the stratified draw, the same commit measured on CI
([30977384628](https://github.com/link-assistant/formal-ai/actions/runs/30977384628),
[`raw-data/ci-run-stratified.log`](raw-data/ci-run-stratified.log)) reproduces
the local run at that commit exactly — 12 iterations, `198/199`, the same
twenty-four files, the same five embedded grammar blocks, the same single
`content_grounded` failure. That is the determinism the protocol claims, now
visible across two machines rather than asserted. It does not make the enforced
floor safe to raise: the numbers agree there because the corpus is identical,
and the corpus is exactly what changes between commits — later merges of `main`
moved the same seed onto different twenty-four-file draws, scoring `191/191` and
then `197/197`.

### What the sweep found

Over the first 600 files of the seeded permutation: **4983/4989 = 99%**.

| Failing criterion | Count | What it is |
| --- | --- | --- |
| `compression` | 6 | A file just above the 400-byte floor whose structured summary (path, format, size, retained content) costs a few bytes more than the file itself — e.g. `summary_bytes=426 file_bytes=424` on `dev/log/issues/776/pulls/794/raw/test-docs-policy-final.log`. |

`content_grounded` failed nowhere in this sample, and that is a fact about which
files the seed reached rather than a fix: the three files below still fail it
today, re-run by name in
[`raw-data/failing-files.txt`](raw-data/failing-files.txt).
`docs/case-studies/issue-710/raw-data/report-closed-issues-351-plus.md` scores
87% with `ungrounded=/api//, data/cache//` — it was inside the sample one corpus
change ago; `experiments/agentic_cli_matrix/README.md` scores 88% with
`ungrounded=artifacts///, recorded//`; and `docs/case-studies/issue-492/README.md`
scores 87% because the summary emits
`https://img.shields.io/badge/crates.io--orange` for a source that reads
`https://img.shields.io/badge/crates.io-<version>-orange?logo=rust`.

All three are the same real defect, not a checker artefact, and the metric is
what surfaced it — on files chosen by the seed rather than by me.
`experiments/agentic_cli_matrix/README.md` writes `` `artifacts/<client>/<case>/` ``
and `` `recorded/<client>/<case>.jsonl` ``, which the summary quotes as
`artifacts///` and `recorded//`. `strip_inline_code_and_html` in `src/summarization/markdown.rs`
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
