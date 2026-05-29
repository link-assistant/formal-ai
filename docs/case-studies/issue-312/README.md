# Case study — Issue #312: "Unknown prompt: список файлов на Rust"

> Reconstructed reasoning for solving not just this one prompt, but the whole
> **class** of "write a program that lists files in the current directory"
> requests across every supported UI language.

- **Issue:** [#312](https://github.com/link-assistant/formal-ai/issues/312) — *Unknown prompt: Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории*
- **Reported version:** 0.140.0 · WASM worker · manual mode
- **Pull request:** [#318](https://github.com/link-assistant/formal-ai/pull/318) (branch `issue-312-d1d44d5118bd`)
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json`, `pr-318.json`

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| 2026-05-27 08:04:12 | User submits, in the GitHub Pages WASM worker, the Russian prompt **"Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории"** ("Write me a Rust program that lists the files in the current directory"). |
| 2026-05-27 08:04:12 | The agent replies with **intent `unknown`**: *"Я тебя не понял…"* (I did not understand you), offering the behavior-rule / teach-a-fact fallbacks. |
| 2026-05-27 08:05:29 | Issue #312 is filed (label `bug`), quoting the failing dialog and pasting full working answers from Google Gemini and DeepSeek (both using `std::fs::read_dir`). |
| 2026-05-27 (this PR) | Root cause identified: **`list_files` was never an implemented task** in the `write_program` catalog. Implemented the task for all 10 catalog languages, made detection multilingual (en/ru/hi/zh) and CJK-aware, fixed the write_program-vs-specialized-handler precedence, and fixed a Rust↔JS normalization parity bug. |

The issue itself carries **no comments** (`issue-comments.json` is `[]`); all
requirements come from the issue body.

---

## 2. Requirements (every explicit and implicit ask)

### From the failing behavior
1. **R1 — Answer the prompt.** "list files in the current directory" in Rust must
   return a working program (the reference answers use `std::fs::read_dir`), not
   `unknown`.
2. **R2 — Solve the whole class, not one string.** The meta-instructions require
   reconstructing reasoning that resolves *the entire class* of code-generation
   requests. So the task must work in **every supported language** (English,
   Russian, Hindi, Chinese) and across the existing language catalog, not only
   Russian/Rust.
3. **R3 — Stay deterministic / honest.** formal-ai does no neural inference; the
   answer must come from the seed template catalog, and (in the browser sandbox)
   must stay honest about not actually running filesystem code.

### From the meta-instructions appended to the issue
4. **R4 — Case study.** Download all related data to `docs/case-studies/issue-312/`
   and perform a deep analysis: timeline, requirements, root causes, solution
   plans, and **online research** of existing components/libraries.
5. **R5 — Add tracing if data is insufficient.** If root cause cannot be found,
   add debug output / a verbose mode (off by default).
6. **R6 — Fix everywhere.** Apply the fix across the *entire* codebase — if the
   gap exists in multiple parallel implementations, fix all of them.
7. **R7 — Report upstream** to other repositories if the issue belongs there
   (with reproducible examples, workarounds, fixes).
8. **R8 — Single PR.** Plan and execute everything in PR #318.

---

## 3. Root-cause analysis

### Root cause A — `list_files` was an unimplemented task (causes R1)

At v0.141.0 the `write_program` catalog only knew two tasks:

```
# data/seed/hello-world-programs.lino @ main
  tasks "hello_world|count_to_three"
```

`grep -c list_files` returns **0** in all three source-of-truth implementations
at `main`:

- `src/engine_hello_world.rs` (Rust `PROGRAM_TASKS` / `PROGRAM_TEMPLATES`; since
  reorganized into `src/coding/catalog/` — see the issue #330 case study),
- `data/seed/hello-world-programs.lino` (Links Notation seed),
- `src/web/formal_ai_worker.js` (browser worker fallback).

The intent formalizer recognized "write a program in Rust" as a `write_program`
shape, but with **no `list_files` task** to bind to, the (language, task) pair
never resolved to a template, so the request fell through to `unknown`. This is
language-independent: an English or Hindi "list files" request failed for the
same reason. The Russian prompt simply happened to be the one a user reported.

### Root cause B — specialized handlers outrank concrete programs (latent, surfaces once A is fixed)

The solver runs `handle_specialized_pattern` (a 40-entry dispatch table that
includes `concept_lookup`) *before* selecting the `write_program` rule. A prompt
that names a language ("…на **Rust**…") matches `concept_lookup`, which would
answer the encyclopedia definition of *Rust* instead of returning the program.
Adding the `list_files` template exposes this ordering bug, so the fix is
incomplete without a precedence guard.

### Root cause C — tokenization gaps for non-space-delimited and combining scripts (causes part of R2/R6)

Two language-mechanics bugs block multilingual coverage:

- **Chinese (CJK).** `contains_token` / `contains_phrase` split on whitespace.
  Chinese writes "列出当前目录中的文件" with **no inter-word spaces**, so no
  whitespace token ever equals a Chinese alias. Detection must fall back to
  **substring** matching for CJK.
- **Hindi (Devanagari) — JS only.** The Rust `normalize_prompt` keeps the whole
  Devanagari block, but the JS-fallback `normalizePrompt` used
  `/[^\p{L}\p{N}]+/gu`, which strips Devanagari **combining marks** (matras /
  nukta / virama are Unicode category *M*, not *L*). That corrupted Hindi tokens
  on the fallback path and broke Rust↔JS parity.

### On R5 (tracing)
The root cause was fully determined from the source and reproduced with tests, so
no permanent debug/verbose scaffolding was required. The reproduction harness
lives in [`experiments/`](../../../experiments/) instead.

### On R7 (upstream)
The failure is entirely within this repository's seed catalog and detectors;
there is **no external dependency at fault**, so no upstream issue is warranted.
(`std::fs::read_dir` works exactly as documented — see §5.)

---

## 4. Solution plan and what was implemented

formal-ai keeps **four parallel implementations** of the program catalog that
must stay in sync, plus the solver routing. Per R6, every one was updated:

| # | Surface | Change |
| --- | --- | --- |
| 1 | `src/engine_hello_world.rs` | New `list_files` `ProgramTask` (en/ru/hi/zh aliases) + a `list_files` `ProgramTemplate` for all 10 languages; added `contains_cjk` and made `contains_token`/`contains_phrase` CJK-aware (substring fallback). |
| 2 | `src/intent_formalization.rs` | Hindi/Chinese `PROGRAM_NOUNS` / `PROGRAM_VERBS`; local `contains_token` made CJK-aware. |
| 3 | `data/seed/hello-world-programs.lino` | `tasks` list extended with `list_files`; `task_list_files` aliases (en/ru/hi/zh) + `template_list_files_*` for all 10 languages. |
| 4 | `src/web/formal_ai_worker.js` | Mirrored aliases/nouns/verbs; added `containsCjk` + CJK-aware token/phrase helpers; **fixed `normalizePrompt`** to keep Devanagari (`ऀ-ॿ`). |
| 5 | `src/solver.rs` | Precedence guard: a concrete `write_program` rule now runs **before** `handle_specialized_pattern`, so `concept_lookup` no longer hijacks "…на Rust…". |

Refactor note: to satisfy the 1000-line file cap, the self-contained
`is_inappropriate_content` predicate was relocated from `solver.rs` to
`solver_helpers.rs` alongside its sibling `is_*` policy predicates.

### Verification (R1–R3)
- Rust unit tests in `tests/unit/specification/code_generation.rs`:
  `russian_program_request…`, **`hindi_list_files_in_rust_returns_program`**,
  **`chinese_list_files_in_rust_returns_program`** — each asserts
  `write_program` + `rust`/`list_files` + a fenced ```` ```rust ```` block
  containing `read_dir`.
- Worker parity harness `experiments/issue-312-worker-parity.mjs` (17 checks:
  Russian/Hindi/Chinese routing, honest "not run" status in the browser sandbox,
  regression that JS hello-world still executes, bare language word not hijacked).
- Full suite: `cargo test --all-features` (644 passed), `cargo clippy
  -Dwarnings`, `cargo fmt --check`, the file-size check, and all four e2e checks
  (`check:i18n`, `check:intent-coverage`, `check:language-parity`,
  `check:language-test-coverage`) pass.

---

## 5. Online research — idiomatic file listing

The reference answers in the issue and the official docs converge on the same
approach, which the Rust template follows:

- **`std::fs::read_dir`** returns an iterator of `io::Result<DirEntry>`; entries
  are yielded in **arbitrary OS order**, so a deterministic answer must sort —
  the template collects names and `sort()`s them.
  (<https://doc.rust-lang.org/std/fs/fn.read_dir.html>)
- **Files vs. directories:** `entry.path().is_file()` (or `DirEntry::file_type`)
  distinguishes regular files; "list files" excludes directories, matching the
  DeepSeek/Gemini answers.
- **Non-UTF-8 names:** `OsStr::to_string_lossy()` is the documented way to print
  names safely (DeepSeek used exactly this).
- Equivalent idioms used by the other templates: Python `os.listdir` +
  `os.path.isfile`; Node `fs.readdirSync` + `fs.statSync().isFile()`; Go
  `os.ReadDir`; C `opendir`/`readdir` + `stat`/`S_ISREG`; C++
  `std::filesystem::directory_iterator`; Java `File.listFiles`; C#
  `Directory.GetFiles`; Ruby `Dir.entries` + `File.file?`.

No third-party crate/library is needed — the standard library covers it in every
target language, keeping the deterministic, dependency-free guarantee intact.

---

## 6. Reproduction

```bash
# Rust spec (fails on main, passes on this branch):
cargo test --all-features specification::code_generation

# Browser-worker parity (JS fallback path used by GitHub Pages):
node experiments/issue-312-worker-parity.mjs
```
