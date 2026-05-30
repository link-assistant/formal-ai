# Case study — Issue #349: "Unknown prompt: Сделай сортировку результатов в обратном порядке"

> Deep analysis of [issue #349](https://github.com/link-assistant/formal-ai/issues/349).
> Compiled per the issue's own request: *"compile that data to `./docs/case-studies/issue-{id}` folder, and
> use it to do deep case study analysis … reconstruct timeline/sequence of events, list of each and all
> requirements from the issue, find root causes of each problem, and propose possible solutions and solution
> plans for each requirement."*

| | |
|---|---|
| **Issue** | [#349](https://github.com/link-assistant/formal-ai/issues/349) — *Unknown prompt: Сделай сортировку результатов в обратном порядке* |
| **Author** | `konard` |
| **Labels** | `bug`, `documentation`, `enhancement` |
| **Opened** | 2026-05-30T09:48:59Z (last edited 10:01:07Z) |
| **Pull request** | [#350](https://github.com/link-assistant/formal-ai/pull/350) (draft, `issue-349-4243100887a0` → `main`, opened 2026-05-30T10:02:06Z) |
| **Product version at report time** | 0.152.0 (deployed at <https://link-assistant.github.io/formal-ai/>, wasm worker, manual mode, diagnostics off) |
| **Code version analysed** | 0.156.0 (`Cargo.toml`) |

This folder holds the raw evidence (`raw-data/`), the captured failure log of the *prior* solver session
(`logs/`), a runnable reproduction (`raw-data/repro_issue_349.rs`), the live reproduction transcript
(`raw-data/reproduction-output.txt`), and the GitHub-issue roadmap that operationalises the fix
([`ROADMAP.md`](./ROADMAP.md)).

---

## 1. Timeline / sequence of events

All timestamps UTC, taken from `raw-data/issue-349.json`, `raw-data/pr-350.json`,
`raw-data/pr-350-comments.json`, and the issue body.

| Time (2026-05-30) | Event | Source |
|---|---|---|
| 09:45:51.653Z | User reproduces the 5-turn Russian dialog in the browser build (v0.152.0, wasm worker, manual mode, diagnostics **off**). | issue body "Timestamp" |
| 09:48:59Z | Issue **#349** filed by `konard` (labels `bug`, `documentation`, `enhancement`). | `issue-349.json` |
| 10:01:07Z | Issue body edited (the meta/planning requirements added). | `issue-349.json` `updatedAt` |
| 10:02:06Z | Draft **PR #350** opened from `issue-349-4243100887a0`. | `pr-350.json` |
| 11:13:43Z | Automated **"Solution Draft Failed"** comment posted on PR #350, linking a 19 MB session log gist. | `pr-350-comments.json` |
| 11:32:55Z | `konard` recovery comment: *"We need to recover from fail at [gist] … That file should be included into analysis, and used. So we don't do the same work twice. Re-read … and continue."* | `pr-350-comments.json` |
| 11:33:37Z | Automated **"AI Work Session Started"** comment. | `pr-350-comments.json` |

### 1.1 The reproduction dialog (verbatim from the issue)

| # | Role | Message | System answer |
|---|---|---|---|
| 1 | user | `Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории` | Rust program listing files (`names.sort();`); "не запущено — браузерная песочница" |
| 2 | assistant | *(program)* | — |
| 3 | user | `Сделай так, чтобы программа принимала путь как аргумент` | updated Rust program with `env::args().nth(1)`; still `names.sort();` ✅ |
| 4 | assistant | *(updated program)* | — |
| 5 | user | `Сделай сортировку результатов в обратном порядке` | **`intent: unknown`** — *"Я не уверен, как на это ответить … use Report issue …"* ❌ |

Turn 3 ("accept a path as argument") is answered correctly. Turn 5 ("sort the results in reverse order")
— a *structurally identical* program-modification follow-up — falls through to the `unknown` intent. That
asymmetry is the bug, and §3 explains exactly why it happens.

### 1.2 Prior solver session (the failure we are recovering from)

The 11:13:43Z "Solution Draft Failed" comment links a gist with a 19 MB session log (preserved here,
gzipped, at `logs/solution-draft-log-pr-1780139614800.txt.gz`). Salvaged facts (see
`raw-data/prior-session-salvaged-summary.txt`):

- solve harness `v1.73.7`, model `opus`, ≈72 min wall-clock, **4290 turns**, 538,064 output tokens, ≈$16.03.
- Terminal reason: **`rapid_refill_breaker`** — the session entered a degenerate probe loop after repeated
  autocompaction (`pwd` ×993, `echo alive` ×1823), exited 1.
- **Net product work produced: zero** — no files written, no issues created, no commits.

**Lesson applied in this session** (documented here so the failure mode is not silently repeated): never read
multi-megabyte artifacts (the 19 MB log, the 10 MB `CHANGELOG`) into the model context; gzip and chunk
everything; delegate heavy reads to read-only sub-agents; capture a *live* reproduction early so progress is
durable. This case study and the roadmap are the durable product the prior session never produced.

---

## 2. Requirements — every ask in the issue

The issue is a **meta / planning + case-study** issue, not a single bug fix. Each row is a discrete,
testable requirement extracted from the issue body. The roadmap issue column points at the GitHub issue
that owns the work (see [`ROADMAP.md`](./ROADMAP.md)).

| # | Requirement (paraphrased; see issue for exact words) | Kind | Roadmap issue |
|---|---|---|---|
| R1 | The product must not answer `unknown` to "sort the results in reverse order" (and the class of bare program-modification follow-ups it represents). | bug | #4, #5 |
| R2 | Do **actual reasoning** on top of the semantic meta-language (links notation / binary links), not memorised-only rules; when no rule exists, *construct* one the way a human would. The current path-argument handling is "still fake". | architecture | #2, #4, #5 |
| R3 | Be **white-box**, not black-box — every reasoning step inspectable. | architecture | #2, #6 |
| R4 | Provide a **diagnostics / verbose mode** that shows every reasoning step (off by default). If data is insufficient to find a root cause, add debug output / verbose mode where missing. | feature | #6 |
| R5 | Apply fixes **across the entire codebase** — if a defect exists in multiple places (e.g. all the runtime mirrors), fix all of them. | quality | #4, #7 |
| R6 | **Bulk test suites** driven by large, legally-usable open datasets; download non-public-domain data only at test time. | testing | #8 |
| R7 | Reduce the pressure toward **"Report issue"**: reasoning first; report only as a last resort. | UX | #9 |
| R8 | **Self-improvement** — when the system can code itself, learn from accumulated unknown-traces (white-box). | research | #10 |
| R9 | **Plan the work as GitHub issues**, each *blocked-by* its dependencies via the GitHub dependencies API, each detailed enough that "even the weakest AI systems can implement them", each with reproducible examples, workarounds, and code-level fix suggestions. | process | #1–#11 + this PR |
| R10 | **Download all logs/data** about the issue into the repo and **compile a deep case study** in `docs/case-studies/issue-349/` (timeline, requirements, root causes, solution plans, existing-components review, online research). | process | this PR |
| R11 | If the issue relates to **other repositories/projects**, report issues there too. | process | n/a — see §6.4 |
| R12 | **Recover** from the prior failed session; reuse the salvaged log so work is not duplicated. | process | done — §1.2 |

---

## 3. Root-cause analysis

**Root-cause claim, reproduced live** (`raw-data/reproduction-output.txt`, generated by
`raw-data/repro_issue_349.rs` against the current library):

```
TURN 3  PROMPT: Сделай так, чтобы программа принимала путь как аргумент
        INTENT: write_program   CONF: 1.00   → full Rust program ✅

TURN 5  PROMPT: Сделай сортировку результатов в обратном порядке
        INTENT: unknown          CONF: 0.00   → "Я не смог определить … Report issue." ❌
```

Why turn 3 works and turn 5 does not decomposes into four independent defects, **A–D**. *All four* must be
addressed for R1/R2 to be truly satisfied; fixing only one leaves the class of bug alive.

### Root cause A — Intent routing is pure pattern matching, with no reasoning fallback

`route_for_prompt` (`src/intent_formalization.rs:340-355`) tries the hard-coded
`write_program_parameters` extractor, then scans `data/seed/intent-routing.lino` for a route whose literal
`keyword`/`phrase`/`token`/`combo` is contained in the prompt (`matches_route`,
`src/intent_formalization.rs:357-367`). `select_rule_for_intent` (`:296-307`) maps a matched route to a
rule and **everything else to `SelectedRule::Unknown`**:

```rust
match intent.route.as_deref() {
    Some("greeting") => ...,
    Some(WRITE_PROGRAM_INTENT) => write_program_rule_for_intent(intent),
    _ => SelectedRule::Unknown,
}
```

"Сделай сортировку результатов в обратном порядке" contains no routing literal in
`intent-routing.lino` (the only `сделай`-routes are `phrase "сделай запрос к"` / `"сделай запрос на"`), so it
routes to nothing and is classified `Unknown` **immediately** — there is no step that attempts to *reason* a
route when no literal matches. This is the concrete form of the user's R2 complaint ("memorized only rules").

### Root cause B — Follow-up recovery only fires for prompts that already contain a program-noun

Multi-turn recovery lives in `recover_write_program_rule` (`src/intent_formalization.rs:420-493`): it
reconstructs task + language from history and re-lowers via `program_plan::lower`. But it is only reached
when the *current* message already routes to `write_program`, which requires a program-noun
("программу"/"program"/…). Turn 3 keeps the routing alive implicitly through the parameter extractor; turn 5
("Сделай сортировку результатов…") has **no program-noun**, so recovery never runs. The
follow-up marker path (`software_project_followup.rs`, issue #341) is likewise a **narrow hard-coded list**
("test it", "run it", "покажи", "запусти") and does not include modification verbs like "сделай сортировку".

### Root cause C — Program modifications are a hard-coded single-entry allowlist

Even if turn 5 *did* route to `write_program`, there is no modifier to apply.
`PROGRAM_MODIFIERS` (`src/intent_formalization.rs:506-520`) contains **exactly one** entry, `path_argument`,
and its own comment states the design:

```rust
// Adding a new *modification* is data here plus a rule in the seed.
const PROGRAM_MODIFIERS: &[ProgramModifier] = &[ProgramModifier {
    slug: "path_argument",
    token_groups: &[ &["path","argument"], &["путь","аргумент"], … ],
}];
```

`detected_program_modifiers` (`:525-…`) matches only those token groups; the substitution that realises a
modifier is *also* hard-coded — `data/seed/program-plan-rules.lino` has a single rule
`path_argument_list_files` (`when "request:modifier -> path_argument"` → replace `list_files` with
`list_files_arg`). There is **no `reverse_sort` modifier and no substitution rule for it**. So "reverse the
sort" is unrepresentable regardless of routing. This is the literal "this is still fake" the issue calls out:
turn 3 only works because someone pre-wrote the `path_argument` modifier, the seed substitution rule, *and*
the `list_files_arg` catalog task.

### Root cause D — No coreference binds a bare imperative to the active program artifact

`data/seed/coreference.lino` resolves only the pronoun **"it"** to a "Rust" antecedent (14 lines, one
pronoun, one antecedent). Nothing binds "результатов" ("the results") in turn 5 to the file-list produced by
the program in turns 1–4. A human reads "sort the results in reverse" as "modify the program from the
previous turns"; the system has no mechanism to form that link, so even a future general-modification engine
needs a coreference step to know *what* to modify.

### Why the live answer differs from the issue's quoted answer (and why it matters for R5)

The issue quotes the legacy fallback *"Я не уверен, как на это ответить…"*; the live run produced *"Я не
смог определить … Report issue."* These are **different unknown exits**. The codebase has *several*
unknown-answer sites that must all be fixed for R1/R5:

- `src/solver.rs:591-617` — primary unknown path → `answer_unknown_prompt`
- `src/solver_unknown_reasoning.rs:124-140` — unresolved-unknown-with-focus variant (the one the live run hit)
- `src/unknown_opener.rs:54-63` — `language_aware_unknown_answer`
- `src/solver_handlers/user_intent.rs:96-114` — unbalanced-links unknown
- `src/engine.rs:38` — `FALLBACK_UNKNOWN_ANSWER` const (+ `russian_unknown_answer` at `:182-185`)
- `data/seed/multilingual-responses.lino` — en/ru/hi/zh fallback strings

…**plus the runtime mirrors** that ship to users and produced the reported answer:
`src/web/app.js:28`, `src/web/formal_ai_worker.js`, `src/web/wasm-worker/src/lib.rs`, and the seed mirror
`src/web/seed_loader.js`. R5 ("fix in all of them") is why the roadmap has a dedicated cross-runtime parity
issue (#7).

### Root-cause summary

| | Defect | Evidence | Fixed by |
|---|---|---|---|
| A | Routing is literal-match only; no reasoning fallback when nothing matches | `intent_formalization.rs:296-307,340-367`; `intent-routing.lino` | #5 |
| B | Follow-up recovery requires a program-noun; bare modification verbs never recover | `intent_formalization.rs:420-493`; `software_project_followup.rs` | #3, #4 |
| C | Modifications are a one-entry hard-coded allowlist + one seed substitution | `intent_formalization.rs:506-520`; `program-plan-rules.lino` | #4 |
| D | Coreference binds only "it"→Rust; nothing binds "the results" to the program | `coreference.lino` | #3 |

---

## 4. Solution plans (per requirement)

These are summaries; each maps to a fully-detailed GitHub issue in [`ROADMAP.md`](./ROADMAP.md) (with
reproducible examples, workarounds, and concrete file-level fix suggestions, per R9).

- **R1 (no `unknown` for the reverse-sort class)** — land the failing test + diagnostics fixture (#1), add
  the general modification model (#4), and the reasoned rule-construction fallback (#5). Verified done when
  `repro_issue_349.rs` turn 5 returns a non-`unknown` intent that reverses the sort, and the bulk suite (#8)
  shows no regressions.
- **R2 (reason, don't memorise)** — design doc (#2) defines rule synthesis *over links notation*: when no
  rule matches, decompose the imperative into (operation = reverse, target = sort order of the active
  program) and **construct a candidate substitution rule** from the operation vocabulary, rather than
  requiring a pre-written allowlist entry. #4 removes the single-entry `PROGRAM_MODIFIERS` allowlist in
  favour of a data-driven, composable model; #5 adds the construct-a-rule-when-none-exists step.
- **R3 / R4 (white-box + diagnostics)** — extend `SolverConfig::diagnostic_mode` (`solver.rs`) and
  `try_diagnostic` (`solver.rs:788-825`) so every reasoning step (route attempts, coreference binding,
  modifier detection, rule construction, verification) is emitted as inspectable links-notation trace,
  default off (#6). This also discharges "if there isn't enough data to find the root cause, add debug/verbose
  output."
- **R5 (fix everywhere)** — cross-runtime parity issue (#7): one behaviour spec, mirrored across the Rust
  core, `formal_ai_worker.js`, the wasm worker, and the seed; the six unknown-exit sites from §3 reconciled.
- **R6 (bulk datasets)** — #8 builds a multilingual multi-turn coding-modification benchmark with a
  ratchet, using **legally-usable open datasets**, downloaded only at test time (see §5 for candidates).
- **R7 (reduce "Report issue")** — #9 makes reasoning-first the default and demotes "Report issue" to a
  genuine last resort, pre-filled with the reasoning trace so a human triage is cheap.
- **R8 (self-improvement)** — #10 (research) closes the loop: learn rules from accumulated unknown-traces,
  white-box, gated behind the benchmark so learned rules cannot regress the suite.
- **R9 / R10 (issues + case study)** — this PR: the case study you are reading, plus the dependency-linked
  GitHub issues created from [`ROADMAP.md`](./ROADMAP.md).

---

## 5. Existing components / prior art reviewed

Per the issue: *"check known existing components/libraries"* and *"search online for additional facts and
data."* The point is to reuse, not reinvent — especially for R6 (datasets) and the R2/R5 modification model.

### 5.1 In-repo components to reuse (not rebuild)

| Component | Location | Reuse for |
|---|---|---|
| Universal 11-step solver | `src/solver.rs` (`UniversalSolver`, `solve` / `solve_with_history`) | host for the reasoning fallback (#5) and diagnostics (#6) |
| Program-plan lowering (substitution rules) | `src/program_plan.rs` + `data/seed/program-plan-rules.lino` | the general modification model (#4) extends this rather than replacing it |
| Operation vocabulary | `data/seed/operation-vocabulary.lino` (`sort_lines`, `reverse_words`, …) | source of operation primitives for constructing a `reverse_sort` rule |
| Follow-up recovery | `src/intent_formalization.rs:420-493`; `software_project_followup.rs` | generalised in #3/#4 to bare modification verbs |
| Diagnostics scaffold | `SolverConfig::diagnostic_mode`, `try_diagnostic` (`solver.rs:788-825`) | extended in #6 |
| Multilingual responses | `data/seed/multilingual-responses.lino` (en/ru/hi/zh) | parity surface for #7 |

### 5.2 Open datasets for the bulk modification benchmark (R6 / #8)

Code **editing-by-instruction** is exactly the task in turn 5, and there is an established open ecosystem:

- **CanItEdit** — 105 hand-crafted Python edit problems × {descriptive, lazy} instructions = 210 instances,
  with hidden tests, split across corrective/adaptive/perfective edits. Human-written instructions, hosted on
  HuggingFace. The closest public analogue to "make the program accept a path argument / sort in reverse".
  ([repo](https://github.com/nuprl/CanItEdit), [paper](https://arxiv.org/pdf/2312.12450))
- **CodeEditorBench** — broader code-editing eval; note its edit instructions are **LLM-generated**, which
  is a quality caveat for a ratchet. ([paper](https://arxiv.org/html/2404.03543v3))
- **HumanEvalFix** — bug-fix-by-instruction; useful as a "corrective edit" slice.
- **EDIT-Bench** — real-world instructed code edits; recent, worth tracking.
  ([paper](https://arxiv.org/pdf/2511.04486))
- Methodology caution: *"Edit, But Verify: An Empirical Audit of Instructed Code-Editing Benchmarks"* —
  read before trusting any of the above as a ratchet. ([paper](https://arxiv.org/html/2604.05100))

License/permissiveness and a Russian/Hindi/Chinese multilingual slice must be verified at integration time;
#8 specifies download-on-test so non-public-domain data never lands in the repo.

### 5.3 Multi-turn coreference / follow-up rewriting (R2, root causes B & D)

The "bind the bare follow-up to prior context" problem is **conversational query rewriting / ellipsis +
anaphora resolution**, a studied area:

- **CREAD — Combined Resolution of Ellipses and Anaphora in Dialogues** (Apple): jointly predicts
  coreference links to dialogue context and emits a *self-contained rewritten query* — precisely the
  "rewrite 'sort the results in reverse' into 'modify the file-listing program to sort in reverse'" step we
  need before lowering. Code + the **MuDoCo** dataset (7.5k task-oriented multi-turn dialogues, 6 domains)
  augmented with query-rewrite annotations are public. ([paper](https://arxiv.org/abs/2105.09914),
  [code](https://github.com/apple/ml-cread))
- **RiSAWOZ** — large multi-domain dialogue corpus with explicit ellipsis/coreference annotations; a source
  of supervised examples and an evaluation reference. ([paper](https://aclanthology.org/2020.emnlp-main.67.pdf))

Takeaway for #3: model the follow-up as *query rewriting against history* and bind referents
("the results", the program) before lowering — don't extend the ad-hoc marker list.

### 5.4 Reasoned rule construction over links notation (R2 / #5)

The "construct a rule when none exists" requirement is **rule induction / neuro-symbolic program synthesis**:
neural (or heuristic) search for candidate symbolic rules, with symbolic verification — interpretable and
verifiable by construction, which aligns with the white-box requirement (R3).

- **Neuro-Symbolic Program Synthesis** — synthesise interpretable programs from examples/specs; neural search
  + symbolic verification. ([paper](https://arxiv.org/pdf/1611.01855))
- **Learning Compositional Rules via Neural Program Synthesis** — learn novel rule *systems* from few
  examples with a symbolic rule representation; directly relevant to "construct a `reverse_sort` rule from one
  or two examples". ([paper](https://arxiv.org/pdf/2003.05562))
- **Proof of Thought** — neurosymbolic synthesis for robust, interpretable reasoning; a model for keeping the
  constructed rule auditable (R3/R4). ([paper](https://arxiv.org/html/2409.17270v2))

For this codebase the practical shape is: keep the symbolic substitution engine (`program_plan`), and add a
**white-box rule-construction step** that proposes a candidate substitution from the operation vocabulary
when no seed rule matches — verified against a TDD test before it is offered, never a black box.

---

## 6. Files & evidence index

### 6.1 This folder

| Path | What |
|---|---|
| `README.md` | this analysis |
| `ROADMAP.md` | the 11-issue dependency DAG operationalising the fix |
| `logs/solution-draft-log-pr-1780139614800.txt.gz` | prior failed session's 19 MB log, gzipped (R12) |
| `raw-data/issue-349.json` | issue metadata |
| `raw-data/issue-349-comments.json` | issue comments (empty — none posted) |
| `raw-data/pr-350.json`, `pr-350-comments.json`, `pr-350-review-comments.json` | PR metadata + comments |
| `raw-data/prior-session-salvaged-summary.txt` | salvaged analysis from the failed session |
| `raw-data/repro_issue_349.rs` | runnable reproduction (see §6.3) |
| `raw-data/reproduction-output.txt` | captured live transcript proving the bug |

### 6.2 Source files implicated (root-cause evidence)

`src/intent_formalization.rs` (routing, recovery, `PROGRAM_MODIFIERS`), `src/solver.rs` (solver + unknown
path + diagnostics), `src/solver_unknown_reasoning.rs`, `src/unknown_opener.rs`,
`src/solver_handlers/user_intent.rs`, `src/engine.rs`, `src/program_plan.rs`; seeds
`data/seed/intent-routing.lino`, `operation-vocabulary.lino`, `program-plan-rules.lino`, `coreference.lino`,
`multilingual-responses.lino`; runtime mirrors `src/web/app.js`, `src/web/formal_ai_worker.js`,
`src/web/wasm-worker/src/lib.rs`, `src/web/seed_loader.js`. Exact line numbers are cited inline in §3.

### 6.3 Reproducing the bug

`raw-data/repro_issue_349.rs` uses the public API `formal_ai::{solve_with_history, ConversationTurn}` to
replay the 5-turn dialog. It is intentionally kept under `raw-data/` (not `examples/`) so the PR stays
docs-only and triggers no release. To run it as a throwaway example:

```sh
cp docs/case-studies/issue-349/raw-data/repro_issue_349.rs examples/repro_issue_349.rs
cargo run --example repro_issue_349
rm examples/repro_issue_349.rs
```

Expected: turn 3 → `intent: write_program, conf 1.00`; turn 5 → `intent: unknown, conf 0.00` (the bug). The
captured output is in `raw-data/reproduction-output.txt`.

### 6.4 Other repositories (R11)

The defect is **entirely within this repository** (Rust core + bundled web runtimes). Root causes A–D are all
local; no third-party project is at fault, so no external issue is warranted. The reused open datasets/libraries
in §5 are integrated download-on-test and require no upstream change. If #4/#5 surface an upstream bug during
implementation, the owning issue will file it then.

---

## 7. Verification status

| Claim | How verified | Result |
|---|---|---|
| Turn 3 works, turn 5 returns `unknown` | live run of `repro_issue_349.rs` against current lib | ✅ `reproduction-output.txt` |
| `PROGRAM_MODIFIERS` has exactly one entry | read `intent_formalization.rs:506-520` | ✅ only `path_argument` |
| Only `path_argument` substitution rule exists | read `program-plan-rules.lino` | ✅ single rule |
| Coreference handles only "it"→Rust | read `coreference.lino` | ✅ 1 pronoun, 1 antecedent |
| Multiple distinct unknown-answer exits exist | read 6 sites in §3 | ✅ |
| GitHub dependencies API works on this repo | `GET .../issues/{n}/dependencies/blocked_by` | ✅ returned `[]` |
| Prior session produced zero product work | salvaged log summary | ✅ `prior-session-salvaged-summary.txt` |

**Bottom line.** The `unknown` answer in #349 is not a missing keyword — it is the visible symptom of a
**memorised-rule architecture** (root causes A–D). Turn 3 only works because the `path_argument` modifier, its
seed substitution, and the `list_files_arg` task were all pre-written by hand; turn 5 has no such pre-written
support and the system has no way to *reason* one into existence. The roadmap (#1–#11) replaces that with a
white-box, reasoned, data-driven, multilingually-tested modification pipeline — and fixes the defect in **all**
runtime surfaces, per R5.
